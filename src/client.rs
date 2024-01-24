use std::{
    net::{SocketAddr, UdpSocket},
    time,
};

use bevy::prelude::*;
use renet::{
    transport::{self, ClientAuthentication, NetcodeClientTransport, NETCODE_USER_DATA_BYTES},
    ConnectionConfig, RenetClient,
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        let ip: SocketAddr = "127.0.0.1:4001".parse().unwrap();
        let client = RenetClient::new(ConnectionConfig::default());
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
        let transport =
            NetcodeClientTransport::new(time::Duration::from_micros(50), auth, soket).unwrap();
    }
}
