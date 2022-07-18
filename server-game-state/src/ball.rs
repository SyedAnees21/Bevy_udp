use crate::{
    components::{Position, Velocity},
    GameTextures, NConnection, WinSize, PacketStatus,
};
use bevy::{
    prelude::*,
};
use rand::Rng;
use std::{str, net::UdpSocket};

use std::collections::HashSet;

const BALL_SPRITE_SCALE: f32 = 0.05;
const BALL_RADIUS: f32 = 17.;
const MAX_NUM_BALLS: u16 = 1;
const _COL_PADDING: f32 = 0.;

use serde::{Deserialize, Serialize};    

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone,Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Data {
    x: f64,
    y: f64,
    packet_index: i16
}

#[derive(Serialize,Debug)]
struct ClientResponse {
    packet_index: i16
}

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, setup_system)
            .add_system(ball_movement_system);
    }
}

fn setup_system(mut commands: Commands, win_size: Res<WinSize>, game_textures: Res<GameTextures>) {
    let win_w_half = win_size.w / 2.;
    let win_h_half = win_size.h / 2.;

    let mut ball_count = 1;
    while ball_count <= MAX_NUM_BALLS {
        let mut rng = rand::thread_rng();
        let p_x = rng.gen_range((-win_w_half + BALL_RADIUS)..(win_w_half - BALL_RADIUS));
        let p_y = rng.gen_range((-win_h_half + BALL_RADIUS)..(win_h_half - BALL_RADIUS));

        let v_x = rng.gen_range(-2.0..2.0);
        let v_y = rng.gen_range(-2.0..2.0);

        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.ball.clone(),
                transform: Transform {
                    translation: Vec3::new(p_x, p_y, 10.),
                    scale: Vec3::new(BALL_SPRITE_SCALE, BALL_SPRITE_SCALE, 1.0),
                    ..Default::default()
                },
                ..default()
            })
            .insert(Velocity { x: v_x, y: v_y })
            .insert(Position { x: p_x, y: p_y });
        ball_count += 1;
    }
}

fn ball_movement_system(
    mut _commands: Commands,
    _win_size: Res<WinSize>,
    mut _balls_map: ResMut<HashSet<Entity>>,
    connection: Res<NConnection>,
    mut pstatus: ResMut<PacketStatus>,
    mut query: Query<(Entity, &mut Velocity, &mut Transform)>,
) {
    let mut buf= [0; 1500];

    let len = connection.socket.recv(&mut buf).expect("Could not get the datagram");
    let json_str = (str::from_utf8(&buf[..len]).expect("unable to parse")).to_string();

    let mut recieved_data:Data = serde_json::from_str(json_str.as_str()).unwrap();
    // println!("{:?}", recieved_data);
    pstatus.queue.push_back(recieved_data.clone());

    pstatus.index = packet_validation( &mut pstatus, &mut recieved_data, &connection.socket);

    for (mut _e, mut _v, mut t) in query.iter_mut() {
        t.translation.x = pstatus.queue[(pstatus.index-1) as usize].x as f32;
        t.translation.y = pstatus.queue[(pstatus.index-1) as usize].y as f32;
    }

    if pstatus.queue.len() >= 500 {
        println!("{:?}", pstatus.queue.iter());
        panic!();
    }
}

fn packet_validation(index: &mut PacketStatus, recv_packet:&mut Data, sock:&UdpSocket) -> i16 { 
    
    match  recv_packet.packet_index -index.index  {

        1 => {                                                           //if all OKAY!
            /*
            Acknowleding the server packets are intact!
            */ 
            index.index = recv_packet.packet_index;
                
            let response = ClientResponse{packet_index:0};
            let res_json = serde_json::to_string(&response).expect("Could not parse response");
            sock.send(res_json.as_bytes()).unwrap();
        },
        0 => {                                                           //if the same packet is recieved twice
            index.queue.pop_back();

            println!("Packet {}  recieved twice", recv_packet.packet_index);
            let response = ClientResponse{packet_index:0};
            let res_json = serde_json::to_string(&response).expect("Could not parse response");
            sock.send(res_json.as_bytes()).unwrap();

        },
        _ => {                                                           //if the certain packet is missing
            /*
            Requesting the server for the missing packet
            */
            let mut buf= [0; 1500];
    
            let response = ClientResponse{packet_index:recv_packet.packet_index-1};
            let res_json = serde_json::to_string(&response).expect("Could not parse response");
            sock.send(res_json.as_bytes()).unwrap();

            let len =sock.recv(&mut buf).expect("Could not get the datagram");
            let json_str = (str::from_utf8(&buf[..len]).expect("unable to parse")).to_string();

            let data:Data = serde_json::from_str(json_str.as_str()).unwrap();

            index.index = data.packet_index+1;

            index.queue.push_back(data);
            index.queue.swap((recv_packet.packet_index-1)as usize, (recv_packet.packet_index-2) as usize);

        }
    }
    index.index
}