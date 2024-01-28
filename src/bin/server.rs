use std::fmt::format;
use std::time::{self, SystemTime};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    thread,
    time::{Duration, Instant},
};

use renet::{
    transport::{self, NetcodeServerTransport, ServerConfig},
    ConnectionConfig, RenetServer,
};
use renet::{DefaultChannel, ServerEvent};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct User {
    name: String,
}

fn main() {
    let ip: SocketAddr = "127.0.0.1:4001".parse().unwrap();
    let mut server = RenetServer::new(ConnectionConfig::default());
    let socket = UdpSocket::bind(ip).unwrap();
    let server_config = ServerConfig {
        authentication: transport::ServerAuthentication::Unsecure,
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 100,
        protocol_id: 7,
        public_addresses: vec![ip],
    };
    let mut transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let mut old_time = time::Instant::now();
    loop {
        let new_time = time::Instant::now();
        let ping = new_time - old_time;
        old_time = new_time;
        transport.update(ping, &mut server).unwrap();

        while let Some(event) = server.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    log(LogType::Connect, &client_id.to_string())
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    log(LogType::Disconnect, &format!("{client_id} | {reason:?}"))
                }
            }
        }
        for client_id in server.clients_id() {
            while let Some(msg) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
            {
                println!("{:?}", bincode::deserialize::<User>(&msg));
            }
        }

        transport.send_packets(&mut server);
        thread::sleep(Duration::from_micros(50))
    }
}

#[derive(Debug)]
enum LogType {
    Connect,
    Disconnect,
}

fn log(log_type: LogType, message: &str) {
    println!("[{}] {message}", format!("{log_type:?}").to_uppercase())
}
