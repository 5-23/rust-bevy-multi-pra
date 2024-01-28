use bevy::math::Vec2;
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct User {
    pub name: String,
    pub pos: Vec2,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum UdpEvent {
    Move(Vec2),
}
