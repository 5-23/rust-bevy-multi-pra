use std::{
    net::{SocketAddr, UdpSocket},
    thread,
    time::{self, SystemTime},
};

use bevy::math::Vec2;
use bevy_game::*;
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport, NETCODE_USER_DATA_BYTES},
    ConnectionConfig, DefaultChannel, RenetClient,
};

fn main() {
    let ip: SocketAddr = "127.0.0.1:4001".parse().unwrap();
    let mut client = RenetClient::new(ConnectionConfig::default());
    let soket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let name = {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        buffer.trim().to_string()
    };
    let auth = ClientAuthentication::Unsecure {
        protocol_id: 7,
        client_id: {
            let a = time::SystemTime::now()
                .duration_since(time::SystemTime::UNIX_EPOCH)
                .unwrap();
            a.as_millis() as u64
        },
        server_addr: ip,
        user_data: Some({
            let mut a = [0 as u8; NETCODE_USER_DATA_BYTES];
            a[0..name.len()].copy_from_slice(name.as_bytes());
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
            let m = bincode::serialize(&UdpEvent::Move(Vec2::new(1., 0.))).unwrap();
            client.send_message(DefaultChannel::ReliableOrdered, m)
        }

        transport.send_packets(&mut client).unwrap();
        thread::sleep(time::Duration::from_micros(50))
    }
}
