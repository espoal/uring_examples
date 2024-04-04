use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    println!("Starting simple TCP echo example!");

    let echo_servers = [
        "tcpbin.com:4242",
        "localhost:8080"
    ];

    let mut stream = TcpStream::connect(echo_servers[0])
        .expect("Failed to connect to server");
    println!("Connected to server!");

    stream.write("test data".as_ref()).expect("Failed to send packet");
    println!("Sent data to server!");

    let mut buffer = [0; 4096];
    let count = stream.read(&mut buffer)?;
    println!("Received data from server! {} bytes", count);
    println!("Response: {:?}", String::from_utf8_lossy(&buffer[..count]));

    println!("End of simple TCP echo example!");
    Ok(())
}