extern crate a03;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use a03::*;
use std::net::{TcpStream, Shutdown};
use std::io::{Write, BufWriter};
use std::net::TcpListener;
use serde_json::from_str;
use std::fs::File;
use std::fs;
use std::error::Error;
use std::time::Instant;

fn main() {
    let node_endpoint = parse_endpoint_from_cli(0);
    let metadata_endpoint = parse_endpoint_from_cli(1);
    let data_path = std::env::args().skip(3).next()
        .unwrap_or(String::from("."));
    let listener = TcpListener::bind(&node_endpoint).unwrap();
    register_with_meta_server(&metadata_endpoint, &node_endpoint);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        match serde_json::Deserializer::from_reader(&mut stream).into_iter().next().unwrap() {
            Ok(Packet { p_type: PacketType::GetFile, json }) => {
                send_chunk(&data_path, &mut stream, &json.unwrap());
                stream.flush().unwrap();
                stream.shutdown(Shutdown::Write).unwrap();
            }
            Ok(Packet { p_type: PacketType::PutFile, json }) => {
                println!("Receiving chunk");
                receive_chunk_from_copy(&mut stream, &data_path, &json.unwrap());
            },
            Ok(Packet { p_type: PacketType::ShutdownDataNode, .. }) =>
                shutdown(&metadata_endpoint, &node_endpoint),
            Ok(_) => eprintln!("We don't handle this PacketType"),
            Err(e) => eprintln!("Error parsing json: {}", e.to_string()),
        };
    }
}

fn receive_chunk_from_copy(stream: &mut TcpStream, base_path: &String, json: &String) {
    let chunk: Chunk = serde_json::from_str(json).unwrap();
    let filepath = format!("{}/{}_{}", base_path, chunk.filename, chunk.index);
    let mut file = BufWriter::new(File::create(filepath).unwrap());
    let start = Instant::now();
    receive_chunk(&mut *stream, &chunk, &mut file);
    let elapsed = start.elapsed();
    println!("Elapsed: {} ms", (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64);
    file.flush().unwrap();
}

fn send_chunk(base_path: &String, stream: &mut TcpStream, json: &String) {
    let chunk: Chunk = serde_json::from_str(json).unwrap();
    println!("Sending {}", chunk.filename);
    match fs::read(format!("{}/{}_{}", base_path, &chunk.filename, &chunk.index)) {
        Ok(file) => {
            serde_json::to_writer(
                &mut *stream,
                &Packet {
                    p_type: PacketType::GetFile,
                    json: Some(serde_json::to_string(
                        &Chunk { file_size: file.len() as i64, ..chunk}).unwrap()),
                }).unwrap();
            stream.flush().unwrap();
            stream.write(&file).unwrap();
            stream.flush().unwrap();
            stream.shutdown(Shutdown::Write).unwrap();
        }
        Err(e) => {
            match serde_json::to_writer(
                stream,
                &Packet {
                    p_type: PacketType::Error,
                    json: Some(String::from(e.description())),
                }) {
                Ok(_) => println!("{}", "Copy client attempted to read non-existing file"),
                Err(e) => println!("{}", e),
            }
        }
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
        })
        .unwrap();
    println!("Registered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
}

fn shutdown(metadata_endpoint: &String, node_endpoint: &String) {
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
        })
        .unwrap();
    println!("Unregistered myself");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let result: Packet = serde_json::from_reader(&mut stream).unwrap();
    println!("{:?}", result);
    std::process::exit(0);
}
