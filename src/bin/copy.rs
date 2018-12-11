extern crate a03;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use a03::*;
use std::net::{TcpStream, Shutdown};
use std::io::Write;
use std::fs;

fn main() {
    let args = get_cli_args();
    let mut stream = TcpStream::connect(args.endpoint).unwrap();
    if args.is_copy_to_dfs {
        serde_json::to_writer(
            &mut stream,
            &Packet {
                p_type: PacketType::GetFile,
                json: Some(serde_json::to_string(
                    &PutFile { name: args.filepath, size: 32 }).unwrap()),
            })
            .unwrap();
    } else {
        serde_json::to_writer(
            &mut stream,
            &Packet {
                p_type: PacketType::PutFile,
                json: Some(serde_json::to_string(
                    &PutFile { name: args.filepath, size: 32 }).unwrap()),
            })
            .unwrap();
    }
    println!("Sent file");
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();

    let files: Vec<AvailableNodes> = serde_json::from_reader(&mut stream).unwrap();
    for f in files {
        println!("Chunk ID: {}", f.chunk_id);
    }

    let mut file = fs::read(args.filename).expect("File not found!");
    println!("{} bytes", file.len());
//    let mut stream = TcpStream::connect("localhost:6771").unwrap();
//    stream.write(&file).unwrap();
//    stream.flush().unwrap();
//    stream.shutdown(Shutdown::Write).unwrap();
}

#[derive(Debug)]
pub struct CliArgs {
    pub endpoint: String,
    pub filepath: String,
    pub filename: String,
    pub is_copy_to_dfs: bool,
}

pub fn get_cli_args() -> CliArgs {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        panic!("Requires 2 arguments; IP:PORT:FILEPATH and a Local filename/filepath")
    }
    let endpoint_arg: String = args.get(0).unwrap().clone();

    let endpoint;
    let filepath;
    let filename;
    let is_copy_to_dfs;

    if endpoint_arg.contains(":") {
        let splits: Vec<&str> = endpoint_arg.split(':').collect();
        if splits.len() < 3 {
            panic!("Incorrect endpoint argument format! Please provide IP:PORT:FILE");
        }
        endpoint = format!("{}:{}", splits[0], splits[1]);
        filepath = String::from(splits[2]);
        filename = args.get(1).unwrap().clone();
        is_copy_to_dfs = true;
    } else {
        let endpoint_arg: String = args.get(1).unwrap().clone();
        let splits: Vec<&str> = endpoint_arg.split(':').collect();
        if splits.len() < 3 {
            panic!("Incorrect endpoint argument format! Please provide IP:PORT:FILE");
        }
        endpoint = format!("{}:{}", splits[0], splits[1]);
        filepath = String::from(splits[2]);
        filename = args.get(0).unwrap().clone();
        is_copy_to_dfs = false;
    }

    CliArgs { endpoint, filepath, filename, is_copy_to_dfs }
}

