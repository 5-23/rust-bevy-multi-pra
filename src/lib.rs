use std::collections::HashMap;

use bevy::{ecs::component::Component, math::Vec2};
use renet::ClientId;
#[derive(serde::Serialize, serde::Deserialize, Debug, Component, Clone)]
pub struct User {
    pub id: ClientId,
    pub name: String,
    pub pos: Vec2,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum UdpEvent {
    // id, position
    Move(ClientId, Vec2),
    // id, name
    Connect(ClientId, String),
    // id, name
    Disconnect(ClientId, String),
    UserInfo(HashMap<ClientId, User>),
}
