use io_uring::{opcode, types, IoUring};
use std::os::unix::net::UnixListener;
use std::{io, ptr};
use std::io::prelude::*;
use std::os::fd::{AsRawFd, RawFd};

enum State {
    Accept,
    Recv,
    Send,
    Closed,
}

struct Connection {
    id: usize,
    state: State,
    fd_conn: RawFd,
}

fn main() -> io::Result<()> {
    println!("Starting Unix Domain Socket example!");
    use std::io::prelude::*;

    let mut ring = IoUring::new(128)?;
    let mut listener = UnixListener::bind("/tmp/uds.sock")?;
    let uds_socket = listener.as_raw_fd();
    let mut buffs = vec![vec![0u8; 4096]; 128];

    let mut connections: Vec<Connection> = Vec::with_capacity(8);
    submit_accept(&mut ring, uds_socket, &mut connections);

    'outer: loop {
        ring.submit_and_wait(1)?;

        let cqe = ring.completion().next().expect("completion queue is empty");
        println!("cqe: {:?}", cqe);

        let id = cqe.user_data() as usize;
        let connection = connections.get_mut(id).unwrap();
        match connection.state {
            State::Accept => {
                let fd_conn = cqe.result();
                if fd_conn < 0 {
                    break 'outer;
                }
                connections[id].fd_conn = fd_conn;

                println!("accepted fd: {}", fd_conn);
                submit_accept(&mut ring, uds_socket, &mut connections);
                submit_recv(&mut ring, &mut connections[id], &mut buffs[id]);
            }
            State::Recv => {
                let byte_read = cqe.result();
                if byte_read == 0 {
                    continue 'outer;
                }
                if byte_read < 0 {
                    break 'outer;
                }
                let byte_read = byte_read as usize;
                let msg = String::from_utf8(buffs[id].clone()).unwrap();
                println!("received: {}", msg);
                prepend_string(&mut buffs[id], byte_read);
                submit_send(&mut ring, &mut connections[id], &mut buffs[id]);
            }
            State::Send => {
                let bytes_written = cqe.result();
                if bytes_written == -32 {
                    connections[id].state = State::Closed;
                }
                if bytes_written < 0 {
                    break 'outer;
                }
                println!("write fd: {}", bytes_written);
                submit_recv(&mut ring, &mut connections[id], &mut buffs[id]);
            }
            State::Closed => unsafe {
                libc::close(connection.fd_conn);
            },
        }
    }


    Ok(())
}

fn submit_accept(ring: &mut IoUring, socket: RawFd, connections: &mut Vec<Connection>) {
    let id = connections.len();
    connections.push(Connection {
        id,
        state: State::Accept,
        fd_conn: 0,
    });

    let accept = opcode::Accept::new(types::Fd(socket), ptr::null_mut(), ptr::null_mut())
        .build()
        .user_data(id as u64);

    unsafe {
        ring.submission()
            .push(&accept)
            .expect("submission queue is full");
    }
}

fn submit_recv(ring: &mut IoUring, request: &mut Connection, buf: &mut Vec<u8>) {
    let read_e = opcode::Recv::new(
        types::Fd(request.fd_conn),
        buf.as_mut_ptr(),
        buf.len() as u32,
    )
        .build()
        .user_data(request.id as u64);

    request.state = State::Recv;

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }
}

fn submit_send(ring: &mut IoUring, connection: &mut Connection, buf: &mut Vec<u8>) {
    let read_e = opcode::Send::new(
        types::Fd(connection.fd_conn),
        buf.as_mut_ptr(),
        buf.len() as u32,
    )
        .build()
        .user_data(connection.id as u64);

    connection.state = State::Send;

    unsafe {
        ring.submission()
            .push(&read_e)
            .expect("submission queue is full");
    }
}

fn prepend_string(vec: &mut Vec<u8>, len: usize) {
    let msg = "Hello \"".as_bytes();
    let msg_len = msg.len();
    for i in (0..len).rev() {
        vec[i + msg_len] = vec[i];
    }
    for i in 0..msg_len {
        vec[i] = msg[i];
    }

    vec[len + msg_len - 1] = '\"' as u8;
    vec[len + msg_len] = '!' as u8;
    vec[len + msg_len + 1] = '\n' as u8;
}