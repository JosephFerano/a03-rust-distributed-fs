extern crate a03;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use a03::*;
use std::net::{TcpStream, Shutdown};
use std::io::{Write, Read};
use std::net::TcpListener;
use serde_json::from_str;
use std::fs::File;

fn main() {
    let endpoint = parse_endpoint_from_cli(0);
    let listener = TcpListener::bind(&endpoint).unwrap();
    register_with_meta_server(&endpoint);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        match serde_json::from_reader(&mut stream) {
            Ok(Packet { p_type: PacketType::GetFile, json, data, }) =>
                get(&mut stream, &json.unwrap(), &data.unwrap()),
            Ok(Packet { p_type: PacketType::PutFile, json, data,  }) =>
                put(&mut stream, &json.unwrap(), &data.unwrap()),
            Ok(Packet { p_type: PacketType::ShutdownDataNode, .. }) =>
                shutdown(&mut stream, &endpoint),
            Ok(_) => eprintln!("We don't handle this PacketType"),
            Err(e) => eprintln!("Error parsing json: {}", e.to_string()),
        };
    }
}

fn register_with_meta_server(endpoint: &String) {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    let split: Vec<&str> = endpoint.split(":").collect();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::NodeRegistration,
            json: Some(serde_json::to_string(
                &NodeRegistration {
                    register: true,
                    ip: String::from(split[0]),
                    port: from_str(split[1]).unwrap() })
                .unwrap()),
            data: None,
        })
        .unwrap();
    println!("Registered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
}

fn put(stream: &mut TcpStream, json: &String, data: &Vec<u8>) {
    let chunk_id: Chunk = serde_json::from_str(json).unwrap();
    println!("CId: {:?}", chunk_id);
    println!("Data Amount: {:?}", data.len());
    let mut copy = File::create(format!("{}_{}", chunk_id.filename, chunk_id.id)).unwrap();
    copy.write_all(&data[..]).unwrap();
}

fn get(stream: &mut TcpStream, json: &String, data: &Vec<u8>) {
//    let files: String = serde_json::from_str(json).unwrap();
}

fn shutdown(stream: &mut TcpStream, endpoint: &String) {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    let split: Vec<&str> = endpoint.split(":").collect();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::NodeRegistration,
            json: Some(serde_json::to_string(
                &NodeRegistration {
                    register: false,
                    ip: String::from(split[0]),
                    port: from_str(split[1]).unwrap() })
                .unwrap()),
            data: None,
        })
        .unwrap();
    println!("Unregistered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
    std::process::exit(0);
}
