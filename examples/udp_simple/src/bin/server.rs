use std::ffi::CString;
use std::net::UdpSocket;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, RawFd};
use io_uring::{opcode, types, IoUring, cqueue};
use libc;
use libc::msghdr;

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


    // Maybe I should set the IP_PTKINFO flag
    let udp_socket = UdpSocket::bind(("0.0.0.0", 12345))?;
    let udp_fd = udp_socket.as_raw_fd();


    let mut msg_hdr: libc::msghdr = unsafe { std::mem::zeroed() };
    // I copied this from tokio/io-uring test code, I don't know why it works
    // https://github.com/tokio-rs/io-uring/blob/master/io-uring-test/src/tests/net.rs
    msg_hdr.msg_namelen = 16;


    // submit_multishot_accept(&mut ring, udp_fd);
    submit_multishot_recvmsg(&mut ring, udp_fd, &msg_hdr as *const _);

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

                println!("namelen: {:?}", msg_hdr.msg_namelen);
                let (msg, port) = print_msg(&mut bufs, byte_read, cqe.flags(), msg_hdr);


                udp_socket.send_to(msg.as_ref(), ("127.0.0.1", port))
                    .expect("Failed to send packet");

                // Broken code, can't understand how to send the message
                /*let message = match CString::new(resp) {
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
                }*/
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

fn submit_multishot_recvmsg(ring: &mut IoUring, socket: RawFd, msg_hdr: *const msghdr) {
    let read_e = opcode::RecvMsgMulti::new(
        types::Fd(socket),
        msg_hdr,
        0xdeed,
    )
        // .flags(libc::MSG_TRUNC as u32)
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

/*

msghdr: 
msg_out: RecvMsgOut { header: io_uring_recvmsg_out { namelen: 16, controllen: 0, payloadlen: 5, flags: 0 }, msghdr_name_len: 16, name_data: [2, 0, 129, 54, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0], control_data: [], payload_data: [107, 107, 107, 107, 10] }
payload: "kkkk\n"
buffer: [16, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 2, 0, 129, 54, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 107, 107, 107, 107, 10]
name: [2, 0, 129, 54, 127, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]


 */
fn print_msg(vec: &mut Vec<u8>, len: usize, flags: u32, mut msg_hdr: libc::msghdr) -> (String, u16) {
    println!("msghdr: ");

    let buf_id = io_uring::cqueue::buffer_select(flags).unwrap();
    let buf_start = 1024 * buf_id as usize;
    let buf_end = buf_start + len;

    // I need to set the name length to the correct value, otherwise the parse will fail
    // let mut msg_hdr: libc::msghdr = unsafe { std::mem::zeroed() };
    // msg_hdr.msg_namelen = 16;

    let msg_out = types::RecvMsgOut::parse(&vec[buf_start..buf_end], &msg_hdr).unwrap();

    println!("msg_out: {:?}", msg_out);

    let payload = String::from_utf8_lossy(msg_out.payload_data());
    println!("payload: {:?}", payload);

    println!("buffer: {:?}", vec[buf_start..buf_end].to_vec());

    let name = msg_out.name_data();
    println!("name: {:?}", name);

    let port = u16::from_be_bytes([name[2], name[3]]);
    println!("port: {:?}", port);


    return (payload.parse().unwrap(), port)

}