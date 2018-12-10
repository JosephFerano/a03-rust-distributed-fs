extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::borrow::Cow;
use std::net::Ipv4Addr;

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketType {
    NodeRegistration,
    ListFiles,
    PutFile,
    GetFiles,
    AddDataBlocks,
    ShutdownDataNode,
    Success,
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
pub struct PutFile {
    pub name: String,
    pub size: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableNodes {
    pub ip: String,
    pub port: u32,
    pub chunk_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFiles {}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddDataBlocks {}

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
    pub chunk_id: String,
}

#[derive(Debug)]
pub struct BlockQuery {
    pub data_node: DataNode,
    pub chunk_id: String
}
