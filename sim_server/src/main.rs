use std::{net::UdpSocket,
    time::{Duration, Instant},
    str,
    io::{ErrorKind},
    collections::VecDeque
  };
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Clone,Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
    packet_index: i16
}

#[derive(Deserialize,Debug)]
struct ClientResponse {
    packet_index: i16
}

trait Send {
    fn send_data(&self, packet: &Point, addr: &str);
}

impl Send for UdpSocket {
    fn send_data(&self, packet: &Point, addr: &str) {

        let json_str = serde_json::to_string(&packet).unwrap();
        self.send_to(&json_str.as_bytes(), &addr).expect("Unable to send data!");
        
    }   
}

fn main() {

    let socket = UdpSocket::bind("0.0.0.0:8888").expect("Could not bind socket");
    socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();

    let mut ref_point = Point { x: 1., y: 1., packet_index:1 };
    let mut origin_point = Point {x:0., y:0., packet_index:0};
    
    let mut data_queue:VecDeque<Point> = VecDeque::new();
    let mut queue_copy:VecDeque<Point> = VecDeque::new();

    let mut start = Instant::now();


    loop {

        let value = update_movement(&mut origin_point,  &mut ref_point);
        data_queue.push_back(value.clone());

        handle_client(&socket, &data_queue, &value);

        if queue_management(start,&mut data_queue) {
            println!("Queue Copy: {:?}", queue_copy.iter());
            queue_copy = data_queue.clone();
            start = Instant::now();
        }
    }

}



fn update_movement( point: &mut Point,  v:&mut Point) -> Point {

    let win_size_w = 400.;
    let win_size_h = 400.;

    let ball_radius = 20.;
    let col_padding = 20.;

    point.x += v.x;
    point.y += v.y;
    point.packet_index += v.packet_index;

    let win_w_half = win_size_w / 2.;
    let win_h_half = win_size_h / 2.;

    if point.x <= (-win_w_half + ball_radius) {
        v.x *= -1.;
        point.x += col_padding;
    } else if point.x >= (win_w_half - ball_radius) {
        v.x *= -1.;
        point.x -= col_padding;
    }

    if point.y <= (-win_h_half + ball_radius) {
        v.y *= -1.;
        point.y += col_padding;
    } else if point.y >= (win_h_half - ball_radius) {
        v.y *= -1.;
        point.y -= col_padding;
    }
        
        
    let  w_point = Point {
        x: point.x,
        y: point.y,
        packet_index: point.packet_index
    };
    return w_point;    
}


fn queue_management(start:Instant, dataqueue:&mut VecDeque<Point>) -> bool {
    if dataqueue.len() >= 500 {
        // dataqueue.pop_front();
    }

    if start.elapsed() >= Duration::from_millis(500) {
        return true;
    }else {
        return false;
    }
}

fn handle_client(socket:&UdpSocket, queue:&VecDeque<Point>, packet: &Point) {
    
    let mut buf = [0;1500];
    let remote_adrr = "127.0.0.1:8000";

    loop{
        /* Starting to send data pakcets to client
        This if-else is for mimicing the packet loss
        Currently we are simulating packet 5 loss
         */
        if packet.packet_index % 3 == 0 {
            break;
        }

        socket.send_data(packet, remote_adrr);

        /* Recieving the responses from Client
         */
        let result = socket.recv_from(&mut buf);
        let (bytes ,_)= match result {
                                Ok(res) => res,
                                Err(e) => match e.kind(){
                                                    ErrorKind::TimedOut => {break;},
                                                    _ =>{ println!("{:?}", e); break;}
                                                }
                                            };   

        let msg_frm_client = str::from_utf8(&buf[..bytes]).expect("No message from client").to_string();
        let response_from_client: ClientResponse = serde_json::from_str(&msg_frm_client.as_str()).expect("Unable to parse");
        
        /* Serving the Client response after each data packet transmission 
         */
        if response_from_client.packet_index != 0 {
            let res = binary_search(&response_from_client, &queue);
            
            let index = match res {
                Some(_) => res.unwrap(),
                None    => break,
            };
            socket.send_data(&queue[index], remote_adrr);
            break;
        }
        break;
    } 
}

fn binary_search(k: &ClientResponse, items: &VecDeque<Point>) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
 
    let mut low: usize = 0;
    let mut high: usize = items.len() - 1;
 
    while low <= high {
        let middle = (high + low) / 2;
        if let Some(current) = items.get(middle) {
            if current.packet_index == k.packet_index  {
                return Some(middle);
            }
            if current.packet_index > k.packet_index {
                if middle == 0 {
                    return None;    
                }
                high = middle - 1
            }
            if current.packet_index < k.packet_index {
                low = middle + 1
            }
        }
    }
    None
}