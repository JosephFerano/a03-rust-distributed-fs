extern crate a03;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use a03::*;
use std::net::{TcpStream, Shutdown};
use std::thread;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::RegisterNode,
            json: Some(serde_json::to_string(
                &RegisterNode { ip: String::from("localhost"), port: 6770 }).unwrap()),
        })
        .unwrap();
    println!("Registered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
}
