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


    let udp_socket = UdpSocket::bind(("0.0.0.0", 12345))?;
    let udp_fd = udp_socket.as_raw_fd();


    let mut msg_hdr: libc::msghdr = unsafe { std::mem::zeroed() };
    // I copied this from tokio/io-uring test code, I don't know why it works
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
                // let msg = read_bufs(&mut bufs, byte_read, cqe.flags());
                // println!("msg: {}", msg);
                println!("namelen: {:?}", msg_hdr.msg_namelen);
                print_msg(&mut bufs, byte_read, cqe.flags(), msg_hdr);


                // Broken code, can't understand where to send the message
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

fn print_msg(vec: &mut Vec<u8>, len: usize, flags: u32, mut msg_hdr: libc::msghdr) {
    println!("msghdr: ");

    let buf_id = io_uring::cqueue::buffer_select(flags).unwrap();
    let buf_start = 1024 * buf_id as usize;
    let buf_end = buf_start + len;

    // Why it doesn't change if I use the original msg_hdr, or if I create a new one?
    // let mut msg_hdr: libc::msghdr = unsafe { std::mem::zeroed() };

    let msg_out = types::RecvMsgOut::parse(&vec[buf_start..buf_end], &msg_hdr).unwrap();

    println!("msg_out: {:?}", msg_out);

    // I need to parse twice, once to get the name length, and once to parse again
    // with the correct name length, otherwise the payload will be wrong
    // msg_hdr.msg_namelen = msg_out.incoming_name_len();

    // Or I can read the first byte from the buffer, which stores the length of the name
    msg_hdr.msg_namelen = vec[buf_start] as _;
    let msg_out = types::RecvMsgOut::parse(&vec[buf_start..buf_end], &msg_hdr).unwrap();
    println!("msg_out: {:?}", msg_out);

    let payload = String::from_utf8_lossy(msg_out.payload_data());
    println!("payload: {:?}", payload);

    println!("buffer: {:?}", vec[buf_start..buf_end].to_vec());

    /*println!("name len: {:?}", msg_out.incoming_name_len());
    println!("name: {:?}", msg_out.name_data());
    println!("control len: {:?}", msg_out.incoming_control_len());
    println!("control: {:?}", msg_out.control_data());
    println!("payload len: {:?}", msg_out.incoming_payload_len());
    println!("payload: {:?}", msg_out.payload_data());
    println!("flags: {:?}", msg_out.flags());*/
}