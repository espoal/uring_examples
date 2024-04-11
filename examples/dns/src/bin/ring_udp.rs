use io_uring::{opcode, IoUring};
use std::ffi::CString;
use std::io;
use std::net::{UdpSocket};
use std::os::unix::io::AsRawFd;
use simple_dns::{CLASS, Name, Packet, Question, ResourceRecord, TYPE};
use simple_dns::rdata::{A, RData};

fn main() -> io::Result<()> {
    let mut ring = IoUring::new(8)?;

    let listener = UdpSocket::bind(("0.0.0.0", 3456))?;
    let sender = listener.connect("8.8.8.8:53")?;
    let listener_socket = listener.as_raw_fd();
    let sender_socket = sender.as_raw_fd();
    let mut buffs = vec![vec![0u8; 4096]; 128];

    let send_e = opcode::Send::
    new(sender_socket, CString::new("Hello, world!")?.as_ptr(), 13, 0)
        .build()
        .user_data(0x41);

    unsafe {
        ring.submission()
            .push(&send_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    let fd = cqe.result();

    let rcv_e = opcode::Recv::new(listener_socket, buffs[0].as_mut_ptr(), buffs[0].len() as _)
        .build()
        .user_data(0x42);

    // Note that the developer needs to ensure
    // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    unsafe {
        ring.submission()
            .push(&rcv_e)
            .expect("submission queue is full");
    }

    ring.submit_and_wait(1)?;

    let cqe = ring.completion().next().expect("completion queue is empty");

    assert_eq!(cqe.user_data(), 0x42);
    // Find out why it's not working
    // assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    let content = std::str::from_utf8(&buf).unwrap();
    println!("bytes read: {:?}", content);

    Ok(())
}

fn dns_packet_factory () -> Vec<u8> {

    let domain_queried: &str = "corriere.it";

    let mut packet = Packet::new_query(1);

    let question = Question::new(
        Name::new(domain_queried).unwrap(),
        TYPE::A.into(),
        CLASS::IN.into(),
        false);
    packet.questions.push(question);

    let resource = ResourceRecord::new(
        Name::new_unchecked(domain_queried), CLASS::IN, 10, RData::A(A { address: 10 }));
    packet.additional_records.push(resource);


    packet.build_bytes_vec()?

}
