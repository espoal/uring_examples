use std::net::UdpSocket;
use std::io::{Result};
use ring::rand::*;

const MAX_DATAGRAM_SIZE: usize = 1350;

fn main() -> Result<()> {
    println!("Quiche example!");

    let mut out = [0; MAX_DATAGRAM_SIZE];

    let listener = UdpSocket::bind(("0.0.0.0", 0))?;
    let local_addr = listener.local_addr()?;

    let url = url::Url::parse("https://cloudflare-quic.com/").unwrap();
    let peer_addr = url.socket_addrs(|| None).unwrap()[0];

    println!("Connecting to {}", peer_addr);

    // Generate a random source connection ID for the connection.
    let mut scid = [0; quiche::MAX_CONN_ID_LEN];
    SystemRandom::new().fill(&mut scid[..]).unwrap();

    let scid = quiche::ConnectionId::from_ref(&scid);

    let mut config = quiche::Config::new(quiche::PROTOCOL_VERSION).unwrap();

    // *CAUTION*: this should not be set to `false` in production!!!
    config.verify_peer(false);

    config
        .set_application_protos(&[
            b"hq-interop",
            b"hq-29",
            b"hq-28",
            b"hq-27",
            b"http/0.9",
        ])
        .unwrap();

    config.set_max_idle_timeout(5000);
    config.set_max_recv_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_max_send_udp_payload_size(MAX_DATAGRAM_SIZE);
    config.set_initial_max_data(10_000_000);
    config.set_initial_max_stream_data_bidi_local(1_000_000);
    config.set_initial_max_stream_data_bidi_remote(1_000_000);
    config.set_initial_max_streams_bidi(100);
    config.set_initial_max_streams_uni(100);
    config.set_disable_active_migration(true);

    let mut conn =
        quiche::connect(url.domain(), &scid, local_addr, peer_addr, &mut config).unwrap();

    let (write, send_info) = conn.send(&mut out).expect("initial send failed");

    println!("Sent {} bytes", write);
    println!("Send info: {:?}", send_info);
    println!("Out buffer: {:?}", &out[0..write]);

    println!("Done!");
    Ok(())
}
