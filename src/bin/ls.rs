extern crate distributed_fs;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use distributed_fs::*;
use std::net::{TcpStream, Shutdown };
use std::io::Write;

fn main() {
    let endpoint = parse_endpoint_from_cli(0);
    let mut stream = TcpStream::connect(endpoint).unwrap();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::ListFiles,
            json: None,
        })
        .unwrap();
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let files: FilePaths = serde_json::from_reader(&mut stream).unwrap();
    for path in files.paths.iter() {
        println!("/home/{}", path);
    }
}

