use std::fmt::format;
use std::time::{self, SystemTime};
use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    thread,
    time::{Duration, Instant},
};

use bevy::math::Vec2;
use bevy_game::*;

use renet::{
    transport::{self, NetcodeServerTransport, ServerConfig},
    ConnectionConfig, RenetServer,
};
use renet::{ClientId, DefaultChannel, ServerEvent};

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

    let mut users: HashMap<ClientId, User> = HashMap::new();

    loop {
        let new_time = time::Instant::now();
        let ping = new_time - old_time;
        old_time = new_time;
        transport.update(ping, &mut server).unwrap();

        let event = server.get_event();

        if event.is_some() {
            let mut event = event.unwrap();
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    users.insert(
                        client_id,
                        User {
                            id: client_id,
                            name: String::from_utf8(
                                transport.user_data(client_id).unwrap().to_vec(),
                            )
                            .unwrap(),
                            pos: Vec2::new(0., 0.),
                        },
                    );
                    server.send_message(
                        client_id,
                        DefaultChannel::ReliableOrdered,
                        bincode::serialize(&UdpEvent::UserInfo({
                            let mut users_copy = HashMap::new();
                            for (k, v) in &users {
                                users_copy.insert(*k, (*v).clone());
                            }
                            users_copy
                        }))
                        .unwrap(),
                    );

                    let bin = bincode::serialize(&UdpEvent::Connect(
                        client_id,
                        String::from_utf8(transport.user_data(client_id).unwrap().to_vec())
                            .unwrap(),
                    ))
                    .unwrap();
                    server.broadcast_message(DefaultChannel::ReliableOrdered, bin);
                    log(LogType::Connect, &client_id.to_string())
                }
                ServerEvent::ClientDisconnected { client_id, reason } => {
                    let bin = bincode::serialize(&UdpEvent::Disconnect(
                        client_id,
                        users.remove(&client_id).unwrap().name,
                    ))
                    .unwrap();
                    server.broadcast_message_except(
                        client_id,
                        DefaultChannel::ReliableOrdered,
                        bin,
                    );
                    log(LogType::Disconnect, &format!("{client_id} | {reason:?}"))
                }
            }
        }
        for client_id in server.clients_id() {
            let msg = server.receive_message(client_id, DefaultChannel::ReliableOrdered);
            if msg.is_some() {
                let msg = msg.unwrap();
                match bincode::deserialize::<UdpEvent>(&msg).unwrap() {
                    UdpEvent::Move(id, pos) => {
                        (*users.get_mut(&client_id).unwrap()).pos += pos;
                        // println!("{id:?} {pos:?}");
                        let bin = bincode::serialize(&UdpEvent::Move(
                            id,
                            users.get(&client_id).unwrap().pos,
                        ))
                        .unwrap();
                        server.broadcast_message(DefaultChannel::ReliableOrdered, bin)
                    }
                    _ => {}
                }
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
