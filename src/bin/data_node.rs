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
use std::net::TcpListener;
use std::fs::File;


fn main() {
    register_with_meta_server();
    let listener = TcpListener::bind("localhost:6771").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
//        let mut buf = Vec::new();
//        match stream.read_to_end(&mut buf) {
//            Ok(size) => {
//                println!("Total bytes: {}", size);
//                let mut copy = File::create("new_version").unwrap();
//                copy.write_all(&buf[..]).unwrap();
//            },
//            Err(e) => println!("{}", e),
//        }
        match serde_json::from_reader(&mut stream) {
            Ok(packet @ Packet { .. }) => match packet.p_type {
//                PacketType::GetFiles => shutdown(&mut stream),
//                PacketType::PutFile => put(&mut stream, &packet.json.unwrap(), &mut Vec::new()),
                PacketType::ShutdownDataNode => shutdown(&mut stream),
                _ => (),
            },
            Err(e) => println!("Error parsing json: {}", e.to_string()),
        };
    }
}

fn register_with_meta_server() {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::NodeRegistration,
            json: Some(serde_json::to_string(
                &NodeRegistration { register: true, ip: String::from("localhost"), port: 6771 }).unwrap()),
        })
        .unwrap();
    println!("Registered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
}

//fn put(stream: &mut TcpStream, json: &String, files: &mut Vec<String>) {
//    let files: PutFiles = serde_json::from_str(json).unwrap();
//}

fn shutdown(stream: &mut TcpStream) {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::NodeRegistration,
            json: Some(serde_json::to_string(
                &NodeRegistration { register: false, ip: String::from("localhost"), port: 6771 }).unwrap()),
        })
        .unwrap();
    println!("Unregistered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
    std::process::exit(0);
}
