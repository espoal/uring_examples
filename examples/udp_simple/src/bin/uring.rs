use std::ffi::CString;
use std::net::UdpSocket;
use std::os::fd::AsRawFd;
use io_uring::{opcode, types, IoUring};

fn main() -> std::io::Result<()> {
    println!("Starting simple UDP uring echo example!");

    let mut ring = IoUring::new(8)?;

    let udp_socket = UdpSocket::bind(("0.0.0.0", 3456))?;
    udp_socket.connect("0.0.0.0:12345")?;
    let udp_fd = udp_socket.as_raw_fd();


    let message = CString::new("test data \n").unwrap();

    let send_e = opcode::Send::
    new(types::Fd(udp_fd), message.as_ptr() as *const u8, message.as_bytes().len() as u32)
        .build()
        .user_data(0x41);

    unsafe {
        ring.submission()
            .push(&send_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");
    println!("cqe: {:?}", cqe);

    let mut buffer = [0u8; 4096];
    let recv_e = opcode::Recv::new(types::Fd(udp_fd), buffer.as_mut_ptr(), 4096)
        .build()
        .user_data(0x42);

    unsafe {
        ring.submission()
            .push(&recv_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");
    println!("cqe: {:?}", cqe);
    println!("Response: {:?}", String::from_utf8_lossy(&buffer[..cqe.result() as usize]));

    Ok(())
}