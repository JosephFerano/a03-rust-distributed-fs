extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::borrow::Cow;
use std::net::TcpStream;
use std::fs::File;
use std::io::{Write, Read, BufWriter};

pub const DEFAULT_PORT: &str = "8000";

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketType {
    NodeRegistration,
    ListFiles,
    PutFile,
    GetFile,
    RequestRead,
    RequestWrite,
    AddDataBlocks,
    ShutdownDataNode,
    Success,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    pub p_type: PacketType,
    pub json: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FilePaths<'a> {
    pub paths: Cow<'a, [String]>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeRegistration {
    pub register: bool,
    pub ip: String,
    pub port: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddFile {
    pub name: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableNodes {
    pub ip: String,
    pub port: u32,
    pub chunk_index: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Chunk {
    pub index: u32,
    pub filename: String,
    pub file_size: i64,
}

#[derive(Debug)]
pub struct DataNode {
    pub id: u32,
    pub ip: String,
    pub port: u32,
}

#[derive(Debug)]
pub struct INode {
    pub id: u32,
    pub name: String,
    pub size: u32,
}

#[derive(Debug)]
pub struct Block {
    pub id: u32,
    pub file_id: u32,
    pub node_id: u32,
    pub chunk_index: u32,
}

#[derive(Debug)]
pub struct BlockQuery {
    pub data_node: DataNode,
    pub chunk_index: u32
}

pub fn parse_endpoint_from_cli(arg_index : usize) -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let endpoint_arg: String = args.get(arg_index).expect("No IP provided").clone();

    if endpoint_arg.contains(":") {
        endpoint_arg
    } else {
        format!("{}:{}", endpoint_arg, DEFAULT_PORT)
    }
}

pub fn receive_chunk(stream: &mut TcpStream, chunk: &Chunk, chunk_buf: &mut BufWriter<File>) {
    let mut buf = [0u8; 256];
    let mut bytes_read = 0;
    while bytes_read < chunk.file_size as usize {
        let bytes = stream.read(&mut buf).unwrap();
        chunk_buf.write_all(&buf[0..bytes]).unwrap();
        bytes_read += bytes;
    }
    chunk_buf.flush().unwrap();
}



