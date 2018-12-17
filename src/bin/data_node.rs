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
use std::fs;
use std::error::Error;

fn main() {
    let node_endpoint = parse_endpoint_from_cli(0);
    let metadata_endpoint = parse_endpoint_from_cli(1);
    let data_path = std::env::args().skip(3).next()
        .expect("Missing data path");
    let listener = TcpListener::bind(&node_endpoint).unwrap();
    register_with_meta_server(&metadata_endpoint, &node_endpoint);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        match serde_json::from_reader(&mut stream) {
            Ok(Packet { p_type: PacketType::GetFile, json, .. }) => {
                send_file(&data_path, &mut stream, &json.unwrap());
                stream.flush().unwrap();
                stream.shutdown(Shutdown::Write).unwrap();
            }
            Ok(Packet { p_type: PacketType::PutFile, json, data, }) =>
                receive_file(&data_path, &json.unwrap(), &data.unwrap()),
            Ok(Packet { p_type: PacketType::ShutdownDataNode, .. }) =>
                shutdown(&mut stream, &metadata_endpoint, &node_endpoint),
            Ok(_) => eprintln!("We don't handle this PacketType"),
            Err(e) => eprintln!("Error parsing json: {}", e.to_string()),
        };
    }
}

fn receive_file(base_path: &String, json: &String, data: &Vec<u8>) {
    let chunk: Chunk = serde_json::from_str(json).unwrap();
    let filepath = format!("{}/{}_{}", base_path, chunk.filename, chunk.index);
    println!("{}", filepath);
    let mut copy = File::create(filepath).unwrap();
    copy.write_all(&data[..]).unwrap();
}

fn send_file(base_path: &String, stream: &mut TcpStream, json: &String) {
    let chunk: Chunk = serde_json::from_str(json).unwrap();
    println!("{}", chunk.filename);
    match fs::read(format!("{}/{}_{}", base_path, &chunk.filename, &chunk.index)) {
        Ok(f) => {
            serde_json::to_writer(
                stream,
                &Packet {
                    p_type: PacketType::GetFile,
                    json: Some(json.clone()),
                    data: Some(Vec::from(f)),
                }).unwrap();
        },
        Err(e) => {
            match serde_json::to_writer(
                stream,
                &Packet {
                    p_type: PacketType::Error,
                    json: Some(String::from(e.description())),
                    data: None,
                }) {
                Ok(_) => println!("{}", "Copy client attempted to read non-existing file"),
                Err(e) => println!("{}", e),
            }
        },
    };
}

fn register_with_meta_server(metadata_endpoint: &String, node_endpoint: &String) {
    println!("{}", metadata_endpoint);
    let mut stream = TcpStream::connect(&metadata_endpoint).unwrap();
    let split: Vec<&str> = node_endpoint.split(":").collect();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::NodeRegistration,
            json: Some(serde_json::to_string(
                &NodeRegistration {
                    register: true,
                    ip: String::from(split[0]),
                    port: from_str(split[1]).unwrap(),
                })
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

fn shutdown(stream: &mut TcpStream, metadata_endpoint: &String, node_endpoint: &String) {
    let mut stream = TcpStream::connect(&metadata_endpoint).unwrap();
    let split: Vec<&str> = node_endpoint.split(":").collect();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::NodeRegistration,
            json: Some(serde_json::to_string(
                &NodeRegistration {
                    register: false,
                    ip: String::from(split[0]),
                    port: from_str(split[1]).unwrap(),
                })
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
