use std::io::prelude::*;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    println!("Starting simple UDP echo example!");

    let listener = UdpSocket::bind(("0.0.0.0", 3456))?;
    listener.connect("0.0.0.0:12345")?;
    println!("Connected to server!");

    listener.send("test data \n".as_ref()).expect("Failed to send packet");
    println!("Sent data to server!");

    let mut buffer = [0; 4096];
    let count = listener.recv(&mut buffer)?;
    println!("Received data from server! {} bytes", count);
    println!("Response: {:?}", String::from_utf8_lossy(&buffer[..count]));

    Ok(())
}