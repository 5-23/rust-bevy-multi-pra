use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::{self, SystemTime},
};

use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport, NETCODE_USER_DATA_BYTES},
    ConnectionConfig, DefaultChannel, RenetClient,
};

#[derive(serde::Serialize)]
struct User {
    name: String,
}

fn main() {
    let ip: SocketAddr = "127.0.0.1:4001".parse().unwrap();
    let mut client = RenetClient::new(ConnectionConfig::default());
    let soket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let auth = ClientAuthentication::Unsecure {
        protocol_id: 7,
        client_id: 0,
        server_addr: ip,
        user_data: Some({
            let mut a = [0 as u8; NETCODE_USER_DATA_BYTES];
            let text = "sus";
            a[0..text.len()].copy_from_slice(text.as_bytes());
            a
        }),
    };
    let mut transport = NetcodeClientTransport::new(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        auth,
        soket,
    )
    .unwrap();
    let mut old_time = time::Instant::now();
    loop {
        let new_time = time::Instant::now();
        let ping = old_time - new_time;
        old_time = new_time;
        transport.update(ping, &mut client).unwrap();
        if client.is_connected() {
            let test = bincode::serialize(&User {
                name: "5-23".to_string(),
            });
            client.send_message(DefaultChannel::ReliableOrdered, test.unwrap())
        }

        transport.send_packets(&mut client).unwrap();
        thread::sleep(time::Duration::from_micros(50))
    }
}
