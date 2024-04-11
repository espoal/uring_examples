use simple_dns::*;
use simple_dns::rdata::*;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    println!("Starting simple DNS example!");

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


    let bytes = packet.build_bytes_vec();
    assert!(bytes.is_ok());

    let unwrapped = bytes.unwrap();

    println!("Packet bytes: {:?}", unwrapped);

    let mut socket = UdpSocket::bind("0.0.0.0:6789")
        .expect("Failed to bind socket");

    socket.send_to(&unwrapped, "8.8.8.8:53").expect("Failed to send packet");


    let mut buffer = [0; 512];
    let bytes = socket.recv(&mut buffer)?;

    println!("Received data from server! {} bytes", bytes);
    println!("Response: {:?}", &buffer[..bytes]);

    let response = Packet::parse(&buffer).expect("Failed to parse packet");

    println!("Response: {:?}", response);


    println!("End of simple DNS example!");

    Ok(())
}