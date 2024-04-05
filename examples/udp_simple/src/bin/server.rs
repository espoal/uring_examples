use std::ffi::CString;
use std::net::UdpSocket;
use std::mem;
use std::os::fd::{AsRawFd, RawFd};
use io_uring::{opcode, types, IoUring, cqueue};
use libc;

fn main() -> std::io::Result<()> {
    println!("Starting simple UDP uring echo example!");

    let mut ring = IoUring::new(64)?;

    let mut bufs = vec![0; 16 * 1024];

    let provide_bufs_e = opcode::ProvideBuffers::new(bufs.as_mut_ptr(), 1024, 16, 0xdeed, 0)
        .build()
        .user_data(0x21)
        .into();

    unsafe {
        ring.submission()
            .push(&provide_bufs_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;


    let udp_socket = UdpSocket::bind(("0.0.0.0", 12345))?;
    let udp_fd = udp_socket.as_raw_fd();


    let mut msg: libc::msghdr = unsafe { mem::zeroed() };

    // submit_multishot_accept(&mut ring, udp_fd);
    submit_multishot_recvmsg(&mut ring, udp_fd, &mut msg);

    'outer: loop {
        ring.submit_and_wait(1)?;

        let cqe = ring.completion().next().expect("completion queue is empty");
        println!("cqe: {:?}", cqe);

        let result = cqe.result();
        println!("result: {:?}", result);

        match cqe.user_data() {
            0xbeef => {
                let byte_read = cqe.result();
                if byte_read == 0 {
                    println!("byte_read == 0");
                    continue 'outer;
                }
                if byte_read == -libc::ENOBUFS {
                    println!("error: provided buffers are full!");
                    break 'outer;
                }
                if byte_read < 0 {
                    println!("error: byte_read < 0");
                    break 'outer;
                }

                if !cqueue::more(cqe.flags()) {
                    println!("error: there is no more data in the socket, case unhandled");
                    break 'outer;
                }

                let byte_read = byte_read as usize;
                println!("byte_read: {}", byte_read);
                let msg = read_bufs(&mut bufs, byte_read, cqe.flags());
                println!("msg: {}", msg);
                println!("msghdr: {:?}", msg);
                let resp = format!("echo: {}", msg);
                println!("resp: {}", resp);
                let message = match CString::new(resp) {
                    Ok(cstr) => { cstr }
                    Err(_) => { CString::new("error converting string!").unwrap() }
                };

                println!("message: {:?}", message.as_bytes());

                let send_e = opcode::Send::
                new(types::Fd(5), message.as_ptr() as *const u8, message.as_bytes().len() as u32)
                    .build()
                    .user_data(0xdead);

                unsafe {
                    ring.submission()
                        .push(&send_e)
                        .expect("submission queue is full");
                }
            }
            0xdead => {
                println!("send operation");
                println!("cqe: {:?}", cqe);
            }

            33 => {
                println!("bufs provided");
            }
            _ => {
                println!("unexpected user_data: {:?}", cqe.user_data());
            }
        }
    }


    Ok(())
}

fn submit_multishot_accept(ring: &mut IoUring, socket: RawFd) {
    let id: u64 = 0xdead;

    let multishot_accept = opcode::AcceptMulti::new(types::Fd(socket))
        .build()
        .user_data(id as u64);

    unsafe {
        ring.submission()
            .push(&multishot_accept)
            .expect("submission queue is full");
    }
}

fn submit_multishot_recv(ring: &mut IoUring, socket: RawFd) {
    let read_e = opcode::RecvMulti::new(
        types::Fd(socket),
        0xdeed,
    )
        .build()
        .user_data(0xbeef as u64)
        .into();

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }
}

fn submit_multishot_recvmsg(ring: &mut IoUring, socket: RawFd, msg: &mut libc::msghdr) {
    let read_e = opcode::RecvMsgMulti::new(
        types::Fd(socket),
        msg,
        0xdeed,
    )
        .build()
        .user_data(0xbeef as u64)
        .into();

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }
}

fn read_bufs(vec: &mut Vec<u8>, len: usize, flags: u32) -> String {
    let buf_id = io_uring::cqueue::buffer_select(flags).unwrap();
    let buf_start = 1024 * buf_id as usize;
    let buf_end = buf_start + len - 1;
    let resp = String::from_utf8(vec[buf_start..buf_end].to_vec()).unwrap();
    resp
}

