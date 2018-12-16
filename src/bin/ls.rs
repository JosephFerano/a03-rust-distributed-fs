extern crate a03;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use a03::*;
use std::net::{TcpListener, TcpStream, Shutdown, SocketAddrV4, Ipv4Addr};
use std::borrow::Cow;
use std::thread;
use std::io::Read;
use std::io::Write;

fn main() {
    let endpoint = parse_endpoint_from_cli(0);
    let mut stream = TcpStream::connect(endpoint).unwrap();
    serde_json::to_writer(
        &mut stream,
        &Packet {
            p_type: PacketType::ListFiles,
            json: None,
            data: None,
        })
        .unwrap();
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    let files: FilePaths = serde_json::from_reader(&mut stream).unwrap();
    for path in files.paths.iter() {
        println!("/home/{}", path);
    }
}

