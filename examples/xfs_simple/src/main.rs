use io_uring::{opcode, types, IoUring};
use std::ffi::CString;
use std::io;
use std::os::unix::io::AsRawFd;

fn main() -> io::Result<()> {
    let mut ring = IoUring::new(8)?;

    let path = "/home/mamluk/Projects/uring_examples/README.md";
    let mut buf = vec![0; 1024];

    let dirfd = types::Fd(libc::AT_FDCWD);
    let flags = libc::O_DIRECT as u64 | libc::O_SYNC as u64;
    let openhow = types::OpenHow::new().flags(flags);
    let path = CString::new(path.as_bytes())?;

    let open_e = opcode::OpenAt2::new(dirfd, path.as_ptr(), &openhow)
        .build()
        .user_data(0x41);

    unsafe {
        ring.submission()
            .push(&open_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    let fd = cqe.result();

    let read_e = opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
        .build()
        .user_data(0x42);

    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    let content = std::str::from_utf8(&buf).unwrap();
    println!("bytes read: {:?}", content);

    Ok(())
}
