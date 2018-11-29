extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::borrow::Cow;
use std::net::Ipv4Addr;

#[derive(Serialize, Deserialize, Debug)]
pub enum PacketType {
    RegisterNode,
    ListFiles,
    PutFiles,
    GetFiles,
    AddDataBlocks,
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
pub struct RegisterNode {
    pub ip: String,
    pub port: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PutFiles {
    pub files: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFiles {}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddDataBlocks {}

#[derive(Debug)]
pub struct DataNode {
    pub id: u32,
    pub address: String,
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
    pub c_id: String,
}

