use ball::BallPlugin;
use bevy::prelude::*;

use std::collections::HashSet;
// use std::net::TcpStream;
use std::{net::UdpSocket,
          collections::VecDeque
};

use serde::{Deserialize, Serialize};
// use serde_json;

mod ball;
mod components;

struct WinSize {
    h: f32,
    w: f32,
}

#[derive(Clone,Serialize, Deserialize, Debug, PartialEq, Default)]
struct Data {
    x: f64,
    y: f64,
    packet_index: i16
}

struct GameTextures {
    ball: Handle<Image>,
}

struct NConnection {
    socket: UdpSocket,
    //stream: TcpStream,
}

struct PacketStatus {
    index: i16,
    queue: VecDeque<ball::Data>
}


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Ball collision".to_string(),
            width: 400.0,
            height: 400.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BallPlugin)
        .add_startup_system(setup)
        .insert_resource(HashSet::<Entity>::new())
        .run();
}

fn  setup(mut commands: Commands, asset_server: Res<AssetServer>, mut windows: ResMut<Windows>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let window = windows.get_primary_mut().unwrap();
    let (WinW, WinH) = (window.width(), window.height());

    let win_size = WinSize { h: WinH, w: WinW };
    commands.insert_resource(win_size);

    let game_textures = GameTextures {
        ball: asset_server.load("ball.png"),
    };

    commands.insert_resource(game_textures);

    let connection = NConnection {
        socket: UdpSocket::bind("127.0.0.1:8000").expect("Could not bind client socket")
    // stream: TcpStream::connect("127.0.0.1:6379").expect("Could not connect to server"),
    };
    connection.socket.connect("127.0.0.1:8888").expect("Could not connect to server");

    commands.insert_resource(connection);

    let packetstatus = PacketStatus {
        index: 0,
        queue: VecDeque::new(),
    };
    commands.insert_resource(packetstatus);
}
