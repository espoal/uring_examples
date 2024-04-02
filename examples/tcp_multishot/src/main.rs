use io_uring::{opcode, types, IoUring};
use std::net::TcpListener;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io};

enum State {
    Accept,
    Recv,
}

struct Connection {
    state: State,
    fd_conn: RawFd,
}

fn main() -> io::Result<()> {
    println!("Starting TCP socket example!");

    let mut ring = IoUring::new(128)?;
    let listener = TcpListener::bind(("127.0.0.1", 3456))?;
    let tcp_socket = listener.as_raw_fd();
    // let mut buffs = vec![vec![0u8; 4096]; 128];
    let mut bufs = vec![0; 4 * 1024];

    let provide_bufs_e = opcode::ProvideBuffers::new(bufs.as_mut_ptr(), 1024, 4, 0xdeed, 0)
        .build()
        .user_data(0x21)
        .into();

    unsafe {
        ring.submission()
            .push(&provide_bufs_e)
            .expect("submission queue is full");
    }


    let mut connections: Vec<Connection> = Vec::with_capacity(16);
    submit_multishot_accept(&mut ring, tcp_socket);

    'outer: loop {
        ring.submit_and_wait(1)?;

        let cqe = ring.completion().next().expect("completion queue is empty");
        println!("cqe: {:?}", cqe);

        let result = cqe.result();
        println!("result: {:?}", result);

        match cqe.user_data() {
            0xdead => {
                let fd_conn = cqe.result();
                if fd_conn < 0 {
                    println!("fd_conn < 0");
                    break 'outer;
                }
                let conn_id = connections.len();
                connections.push(Connection {
                    state: State::Accept,
                    fd_conn,
                });

                submit_multishot_recv(&mut ring, connections.get_mut(conn_id).unwrap());

                println!("accepted fd: {}", fd_conn);
            }
            0xbeef => {
                let byte_read = cqe.result();
                if byte_read == 0 {
                    println!("byte_read == 0");
                    continue 'outer;
                }
                if byte_read < 0 {
                    println!("byte_read < 0");
                    break 'outer;
                }
                let byte_read = byte_read as usize;
                println!("byte_read: {}", byte_read);
                let resp = prepend_string(&mut bufs, byte_read, cqe.flags());
                println!("resp: {}", resp);
                // println!("bufs: {:?}", bufs);
                let buf_id = io_uring::cqueue::buffer_select(cqe.flags()).unwrap();
                if buf_id == 3 {
                    let provide_bufs_e = opcode::ProvideBuffers::new(bufs.as_mut_ptr(), 1024, 4, 0xdeed, 0)
                        .build()
                        .user_data(0x21)
                        .into();

                    unsafe {
                        ring.submission()
                            .push(&provide_bufs_e)
                            .expect("submission queue is full");
                    }
                }
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

fn submit_multishot_accept(ring: &mut IoUring, tcp_socket: RawFd) {
    let id: u64 = 0xdead;

    let multishot_accept = opcode::AcceptMulti::new(types::Fd(tcp_socket))
        .build()
        .user_data(id as u64);

    unsafe {
        ring.submission()
            .push(&multishot_accept)
            .expect("submission queue is full");
    }
}

fn submit_multishot_recv(ring: &mut IoUring, connection: &mut Connection) {
    let read_e = opcode::RecvMulti::new(
        types::Fd(connection.fd_conn),
        0xdeed,
    )
        .build()
        .user_data(0xbeef as u64)
        .into();

    connection.state = State::Recv;

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }
}

fn prepend_string(vec: &mut Vec<u8>, len: usize, flags: u32) -> String {
    let buf_id = io_uring::cqueue::buffer_select(flags).unwrap();
    let buf_start = 1024 * buf_id as usize;
    let buf_end = buf_start + len - 1;
    println!("Buffer: {}", buf_id);
    let resp = format!("Hello {}!", String::from_utf8(vec[buf_start..buf_end].to_vec()).unwrap());
    return resp;
}

