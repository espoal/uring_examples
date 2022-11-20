mod nvme;

use std::os::unix::fs;
use std::os::unix::io::{AsRawFd};
use std::os::unix::io;

use io_uring::squeue::Entry;
use io_uring::types::Fd;
use io_uring::{opcode, types, IoUring};

use nvme::nvme_uring_cmd;

fn main() -> io::Result<()> {

    // Params
    let path = "/dev/nvme0";
    let lba: u64 = 1000;
    let num_blocks: u32 = 1;

    // TODO: find nsid
    let nsid = 1;


    let mut builder = IoUring::builder();
    let mut ring = builder.build(128)?;

    let fd = fs::File::open(path)?;

    let mut buff = [0u8; 4096];
    let mut buf: *mut u8 = &mut buff[0];
    let tfd = Fd(fd.as_raw_fd());


    // TODO: check correct cmd_opcode
    let cmd_op: u32 = 0x80;
    let opcode: u8 = 0x2;
    let data_addr = buf as u64;
    let data_len = 1 as u32;
    let cdw10 = (lba & 0xffffffff) as u32;
    let cdw11 = (lba >> 32) as u32;
    let cdw12 = num_blocks - 1;


    let cmd = nvme_uring_cmd {
        opcode,
        // TODO: find nsid
        nsid,
        addr: data_addr,
        data_len,
        cdw10,
        cdw11,
        cdw12,
        ..Default::default()
    };

    let mut cmd_bytes = [0u8; 80];
    unsafe {
        cmd_bytes
            .as_mut_ptr()
            .cast::<nvme_uring_cmd>()
            .write_unaligned(cmd);
    }

    let nvme_read = opcode::UringCmd80::new(tfd, cmd_op)
        .cmd(cmd_bytes)
        .build()
        .user_data(0x22);

    unsafe {
        ring.submission()
            .push(&nvme_read)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    let content = std::str::from_utf8(&buff).unwrap();
    println!("bytes read: {:?}", content);

    Ok(())
}
