extern crate a03;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use a03::*;
use std::net::{TcpStream, Shutdown};
use std::io::Write;
use std::fs::File;
use std::fs;
use std::io::Read;

fn main() {
    let args = get_cli_args();
    let mut stream = TcpStream::connect(&args.endpoint).unwrap();
    let packet_type;
    let json;
    if args.is_copy_to_dfs {
        packet_type = PacketType::RequestWrite;
        let file = fs::read(&args.filename).unwrap();
        let size = file.len();
        println!("Sending file of size {}", size);
        json = Some(serde_json::to_string(
            &AddFile { name: args.filepath.clone(), size: size as u64 }).unwrap())
    } else {
        packet_type = PacketType::RequestRead;
        println!("Requesting Read of {}", args.filepath);
        json = Some(serde_json::to_string::<String>(&args.filepath).unwrap())
    }
    serde_json::to_writer(&mut stream, &Packet { p_type: packet_type, json, })
        .unwrap();
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();

    let mut nodes: Option<Vec<AvailableNodes>> = None;
    match serde_json::from_reader(&mut stream) {
        Ok(Packet { p_type: PacketType::Success, json, .. }) =>
            nodes = Some(serde_json::from_str::<Vec<AvailableNodes>>(&json.unwrap())
                .unwrap()),
        Ok(Packet { p_type: PacketType::Error, json, .. }) => {
            eprintln!("Meta Data Server Error: {}", &json.unwrap());
        }
        Ok(_) => {}
        Err(e) => eprintln!("Error parsing json {}", e.to_string()),
    };
    let filename = &args.filepath;
    let destination = &args.filename;
    if args.is_copy_to_dfs {
        let file = fs::read(&args.filename).unwrap();
        nodes.map(|ns| send_file_to_data_nodes(&filename, &ns, &file));
    } else {
        nodes.map(|ns| get_file_from_data_nodes(&destination, &filename, &ns));
    }
}

fn send_file_to_data_nodes(
    filename: &String,
    nodes: &Vec<AvailableNodes>,
    file: &Vec<u8>)
{
    let div: usize = ((file.len() as f32) / (nodes.len() as f32)).ceil() as usize;
    let chunks: Vec<_> = file.chunks(div).collect();
    for node in nodes {
        let endpoint = format!("{}:{}", node.ip, node.port);
        println!("{}", endpoint);
        let mut stream = TcpStream::connect(&endpoint).unwrap();
        let file_size = chunks[node.chunk_index as usize].len() as i64;
        let chunk = Chunk {
            index: node.chunk_index,
            filename: filename.clone(),
            file_size,
        };
        serde_json::to_writer(
            &mut stream,
            &Packet {
                p_type: PacketType::PutFile,
                json: Some(serde_json::to_string(&chunk).unwrap()),
            }).unwrap();
        stream.flush().unwrap();
        stream.write_all(chunks[node.chunk_index as usize]).unwrap();
        stream.flush().unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
    }
}

fn get_file_from_data_nodes(
    destination_path: &String,
    filename: &String,
    nodes: &Vec<AvailableNodes>)
{
    let mut chunks: Vec<Vec<u8>> = Vec::with_capacity(nodes.len());
    let mut successful = 0;
    for node in nodes {
        let chunk = Chunk {
            index: node.chunk_index,
            filename: filename.clone(),
            file_size: 128,
        };
        let endpoint = format!("{}:{}", node.ip, node.port);
        let mut stream = TcpStream::connect(endpoint).unwrap();
        serde_json::to_writer(
            &stream,
            &Packet {
                p_type: PacketType::GetFile,
                json: Some(serde_json::to_string(&chunk).unwrap()),
            }).unwrap();
        stream.flush().unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        match serde_json::from_reader(stream) {
            Ok(Packet { p_type: PacketType::GetFile, json, }) => {
//                let data = data.unwrap();
                let chunk: Chunk = serde_json::from_str(&json.unwrap()).unwrap();
//                chunks.insert(chunk.index as usize, data);
                successful += 1;
            }
            Ok(Packet { p_type: PacketType::Error, json, .. }) => {
                eprintln!("Data Node Server Error: {}", &json.unwrap());
            }
            Ok(_) => {}
            Err(e) => eprintln!("Error parsing json {}", e.to_string()),
        };
    }
    if successful == nodes.len() {
        let mut copy = File::create(destination_path).unwrap();
        for chunk in chunks {
            copy.write(&chunk[..]).unwrap();
        }
    }
}

#[derive(Debug)]
pub struct CliArgs {
    pub endpoint: String,
    pub filepath: String,
    pub filename: String,
    pub is_copy_to_dfs: bool,
}

pub fn get_cli_args() -> CliArgs {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        panic!("Requires 2 arguments; IP:PORT:FILEPATH and a Local filename/filepath")
    }
    let mut endpoint_arg: String = args.get(0).unwrap().clone();

    let endpoint;
    let filepath;
    let filename;
    let splits: Vec<&str>;

    let is_copy_to_dfs = !endpoint_arg.contains(":");
    if is_copy_to_dfs {
        endpoint_arg = args.get(1).unwrap().clone();
        splits = endpoint_arg.split(':').collect();
        if splits.len() < 3 {
            panic!("Incorrect endpoint argument format! Please provide IP:PORT:FILE");
        }
        filename = args.get(0).unwrap().clone();
    } else {
        splits = endpoint_arg.split(':').collect();
        if splits.len() < 3 {
            panic!("Incorrect endpoint argument format! Please provide IP:PORT:FILE");
        }
        filename = args.get(1).unwrap().clone();
    }
    endpoint = format!("{}:{}", splits[0], splits[1]);
    filepath = String::from(splits[2]);

    CliArgs { endpoint, filepath, filename, is_copy_to_dfs }
}

