extern crate a03;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use a03::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::Read;
use std::io::Write;
use std::fs::File;
use std::fs;

fn main() {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::PutFile,
            json: Some(serde_json::to_string(
                &PutFile { name: String::from("Somefile"), size: 32 }).unwrap()),
        })
        .unwrap();
    println!("Sent file");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();

    let files: Vec<AvailableNodes> = serde_json::from_reader(&mut stream).unwrap();
    for f in files {
        println!("Chunk ID: {}", f.chunk_id);
    }

//    std::process::exit(0);
//    let mut file = fs::read("/home/joe/Downloads/ideaIU-2018.3.tar.gz").unwrap();
//    let mut stream = TcpStream::connect("localhost:6771").unwrap();
//    stream.write(&file).unwrap();
//    stream.flush().unwrap();
//    stream.shutdown(Shutdown::Write).unwrap();
}
