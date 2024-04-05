use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:34567")?;
    println!("Starting UDP blocking echo server\n");
    let mut buf = [0; 512];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        let msg = std::str::from_utf8(&buf[..amt]).unwrap();
        println!("Received msg: {}\nFrom: {:?}", msg, src);

        socket.send_to(&buf, src)?;
    }

    Ok(())
}