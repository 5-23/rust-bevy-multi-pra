// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use bevy_game::{UdpEvent, User};
use lazy_static::lazy_static;
use renet::transport::{ClientAuthentication, NetcodeClientTransport, NETCODE_USER_DATA_BYTES};
use renet::{ClientId, ConnectionConfig, DefaultChannel, RenetClient};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{self, SystemTime};
use winit::window::Icon;

lazy_static! {
    static ref USERS: Arc<Mutex<HashMap<ClientId, User>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref CLIENT: Arc<Mutex<RenetClient>> =
        Arc::new(Mutex::new(RenetClient::new(ConnectionConfig::default())));
}

static mut CLIENT_ID: ClientId = ClientId::from_raw(1);

fn main() {
    App::new()
        // .insert_resource(Msaa::Off)
        // .insert_resource(AssetMetaCheck::Never)
        // .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy game".to_string(), // ToDo
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // The canvas size is constrained in index.html and build/web/styles.css
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        // .add_plugins(DefaultPlugins)
        .add_systems(Startup, set_window_icon)
        .add_systems(Startup, startup)
        .add_systems(Startup, client_system)
        .add_systems(Update, user_management)
        .add_systems(Update, user_movement)
        .run();
}
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let Some(primary) = windows.get_window(primary_entity) else {
        return;
    };
    let icon_buf = Cursor::new(include_bytes!("../build/icon_1024x1024.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn client_system() {
    let ip: SocketAddr = "127.0.0.1:4001".parse().unwrap();
    let mut client = RenetClient::new(ConnectionConfig::default());
    let soket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let name = {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        buffer.trim().to_string()
    };

    let client_id = {
        let a = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap();
        ClientId::from_raw(a.as_millis() as u64)
    };
    unsafe { CLIENT_ID = client_id }

    {
        let mut users = USERS.lock().unwrap();
        users.insert(
            client_id,
            User {
                id: client_id,
                name: name.clone(),
                pos: Vec2::default(),
            },
        );
    }

    let auth = ClientAuthentication::Unsecure {
        protocol_id: 7,
        client_id: client_id.raw(),
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
    thread::spawn(move || loop {
        let new_time = time::Instant::now();
        let ping = old_time - new_time;
        old_time = new_time;
        // transport.update(ping, &mut client).unwrap();
        let client = CLIENT.lock();
        if client.is_ok() {
            let mut client = client.unwrap();
            transport.update(ping, &mut client).unwrap();
            if client.is_connected() {
                while let Some(msg) = client.receive_message(DefaultChannel::ReliableOrdered) {
                    match bincode::deserialize::<UdpEvent>(&msg).unwrap() {
                        UdpEvent::Move(client_id, pos) => {
                            let users = USERS.lock();
                            if users.is_ok() {
                                let mut users = users.unwrap();
                                if let Some(user) = users.get_mut(&client_id) {
                                    user.pos = pos;
                                }
                            }
                        }
                        UdpEvent::Connect(client_id, name) => {
                            println!("{name}({client_id}) CONNECTED");
                            let mut users = USERS.lock().unwrap();
                            users.insert(
                                client_id,
                                User {
                                    id: client_id,
                                    name: name,
                                    pos: Vec2::default(),
                                },
                            );
                        }
                        UdpEvent::Disconnect(client_id, name) => {
                            println!("{name}({client_id}) DISCONNECTED");
                            let mut users = USERS.lock().unwrap();
                            users.remove(&client_id);
                        }
                        UdpEvent::UserInfo(info) => {
                            let users = USERS.lock();
                            if users.is_ok() {
                                let mut users = users.unwrap();
                                for (k, v) in info {
                                    users.insert(k, v);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        let a = CLIENT.lock();
        if a.is_ok() {
            let mut a = a.unwrap();
            transport.send_packets(&mut a).unwrap();
        }
        // transport.send_packets(&mut client).unwrap();
        thread::sleep(time::Duration::from_micros(50))
    });
    println!("CLIENT SYSTEM SPAWN")
}

fn user_management(
    mut commands: Commands,
    mut user: Query<(&mut Transform, &mut User), With<User>>,
    asset_server: Res<AssetServer>,
) {
    let users = USERS.lock();
    if users.is_ok() {
        let users = users.unwrap();
        for (id, data) in users.iter() {
            let mut dont_work = true;
            for (mut transform, mut user) in &mut user {
                if id == &user.id {
                    user.pos = data.pos;
                    transform.translation = Vec3::new(user.pos.x, user.pos.y, 0.);
                    dont_work = false;
                }
            }

            if dont_work {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(100., 100.)),
                            color: Color::Rgba {
                                red: 0.5,
                                green: 0.5,
                                blue: 0.5,
                                alpha: 0.3,
                            },
                            ..Default::default()
                        },

                        ..Default::default()
                    })
                    .insert(User {
                        id: id.clone(),
                        name: data.name.to_string(),
                        pos: Vec2::default(),
                    });
                commands
                    .spawn(
                        // Create a TextBundle that has a Text with a single section.
                        Text2dBundle {
                            text: Text::from_section(
                                data.name.clone(),
                                TextStyle {
                                    font: asset_server.load("fonts/IntroDemoCond-BlackCAPS.ttf"),
                                    font_size: 35.,
                                    ..Default::default()
                                },
                            )
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        },
                    )
                    .insert(User {
                        id: id.clone(),
                        name: data.name.to_string(),
                        pos: Vec2::default(),
                    });
            }
        }
    }
}

fn user_movement(input: Res<Input<KeyCode>>) {
    let client = CLIENT.lock();
    if client.is_ok() {
        let mut client = client.unwrap();
        let mut m = Vec2::default();
        let speed = 1.5;
        if input.pressed(KeyCode::Up) {
            m.y += speed
        }
        if input.pressed(KeyCode::Down) {
            m.y -= speed
        }

        if input.pressed(KeyCode::Left) {
            m.x -= speed
        }
        if input.pressed(KeyCode::Right) {
            m.x += speed
        }
        if m == Vec2::default() {
            return;
        }
        client.send_message(
            DefaultChannel::ReliableOrdered,
            bincode::serialize(&UdpEvent::Move(unsafe { CLIENT_ID }, m)).unwrap(),
        );
    }
}
