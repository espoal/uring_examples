mod nvme;

use std::os::unix::io::AsRawFd;
use std::{fs, io};

use io_uring::types::Fd;
use io_uring::{cqueue, opcode, squeue, IoUring};

use crate::nvme::nvme_uring_cmd_io;
use nvme::nvme_uring_cmd;

// Double check output with:
// sudo nvme read /dev/nvme0n1 -s 1000000 -c 1 -z 4096

fn main() -> io::Result<()> {
    // Params
    let path = "/dev/nvme0n1";
    let lba: u64 = 1000000;
    let num_blocks: u32 = 1;

    // sudo nvme id-ns /dev/nvme0n1
    let nsid = 1;

    let builder = IoUring::<squeue::Entry128, cqueue::Entry32>::generic_builder();
    let mut ring = builder.build(128)?;

    let fd = fs::File::open(path)?;

    let mut buff = [0u8; 4096];
    let buf: *mut u8 = &mut buff[0];
    let tfd = Fd(fd.as_raw_fd());

    // TODO: check correct cmd_opcode
    let cmd_op = nvme_uring_cmd_io();
    let opcode = 0x02 as u8;
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

    println!("cmd bytes: {:?}", cmd_bytes);
    println!("cqe: {:?}", cqe);
    //println!("big cqe: {:?}", cqe.big_cqe());

    println!("cmd: {:?}", nvme_read);

    assert_eq!(cqe.user_data(), 0x22);
    //assert!(cqe.result() >= 0, "read error: {}", cqe.result());
    println!("read {} bytes", cqe.result());

    let content = std::str::from_utf8(&buff).unwrap();
    //println!("bytes read: {:?}", content);
    println!("bytes read: {:?}", buff);

    Ok(())
}
