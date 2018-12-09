extern crate a03;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use a03::*;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use std::borrow::Cow;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn main() {
    let conn = Connection::open("dfs.db")
        .expect("Error opening 'dfs.db', consider running 'python createdb.py'");

    let listener = TcpListener::bind("localhost:6770").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        match serde_json::from_reader(&mut stream) {
            Ok(packet @ Packet { .. }) => match packet.p_type {
                PacketType::ListFiles => list(&mut stream, &conn),
                PacketType::NodeRegistration =>
                    node_registration(&mut stream, &packet.json.unwrap(), &conn),
//                PacketType::AddFile =>
                _ => (),
            },
            Err(e) => println!("Error parsing json {}", e.to_string()),
        };
        stream.flush().unwrap();
    }
}

fn list(stream: &mut TcpStream, conn: &Connection) {
    match serde_json::to_writer(
        stream,
        &FilePaths {
            paths: Cow::from(get_files(&conn)),
        },
    ) {
        Ok(_) => println!("{}", "Sent file paths"),
        Err(e) => println!("{}", e),
    };
}

fn node_registration(stream: &mut TcpStream, json: &String, conn: &Connection) {
    let endpoint : NodeRegistration = serde_json::from_str(json).unwrap();
    let message = if endpoint.register {
        // TODO: We probably should check if the endpoint already exists!
        add_data_node(&conn, &endpoint.ip, endpoint.port as i32);
        "You were successfully registered"
    }
    else {
        // TODO: We should check if the endpoint exists!
        remove_data_node(&conn, &endpoint.ip, endpoint.port as i32);
        "You were successfully unregistered"
    };
    report_success(stream, message);
}

fn report_success(stream: &mut TcpStream, message: &str) {
    match serde_json::to_writer(
        stream,
        &Packet {
            p_type: PacketType::Success,
            json: None,
        },
    ) {
        Ok(_) => println!("{}", message),
        Err(e) => println!("{}", e),
    };
}

fn add_data_node(conn: &Connection, address: &str, port: i32) {
    match conn.execute(
        "INSERT INTO dnode (address, port) VALUES (?1, ?2)",
        &[&address as &ToSql, &port],
    ) {
        Ok(n) => println!("{} rows updated", n),
        Err(e) => println!("INSERT error: {}", e),
    };
}

fn remove_data_node(conn: &Connection, address: &str, port: i32) {
    match conn.execute(
        "DELETE FROM dnode WHERE address=?1 AND port=?2",
        &[&address as &ToSql, &port],
    ) {
        Ok(n) => println!("{} rows updated", n),
        Err(e) => println!("DELETE error: {}", e),
    };
}

fn get_data_node(conn: &Connection, address: &str, port: i32) -> Option<DataNode> {
    let mut stmt = conn
        .prepare("SELECT nid, address, port FROM dnode WHERE address=?1 AND port=?2")
        .unwrap();
    stmt.query_row(&[&address as &ToSql, &port], |row| DataNode {
        id: row.get(0),
        ip: row.get(1),
        port: row.get(2),
    }).ok()
}

fn get_data_nodes(conn: &Connection) -> Vec<DataNode> {
    let mut stmt = conn.prepare("SELECT nid, address, port FROM dnode").unwrap();
    let iter = stmt.query_map(NO_PARAMS, |row| DataNode {
        id: row.get(0),
        ip: row.get(1),
        port: row.get(2),
    }).unwrap();
    let mut nodes = Vec::new();
    for n in iter {
        let n = n.unwrap();
        nodes.push(n);
    }
    nodes
}

fn add_file(conn: &Connection, fname: &String, fsize: i32) {
    conn.execute(
        "INSERT INTO inode (fname, fsize) VALUES (?1, ?2)",
        &[&fname as &ToSql, &fsize])
        .unwrap();
}

fn remove_file(conn: &Connection, fname: String) {
    conn.execute(
        "DELETE FROM inode WHERE fname=?1",
        &[&fname as &ToSql])
        .unwrap();
}

 fn get_file_info(conn: &Connection, fname: &String) -> INode {
     conn.query_row(
         "SELECT fid, fsize FROM inode where fname=?1",
         &[&fname as &ToSql],
         |row| INode { id: row.get(0), size: row.get(1), name: fname.clone() }
     ).unwrap()
 }

 fn get_files(conn: &Connection) -> Vec<String> {
     let mut stmt = conn
         .prepare("SELECT fname, fsize FROM inode")
         .unwrap();
     let mut files = Vec::new();
     match stmt.query_map(NO_PARAMS, |row| INode {
         id: 0,
         name: row.get(0),
         size: row.get(1),
     }) {
         Ok(iter) => {
             for f in iter {
                 let f = f.unwrap();
                 files.push(format!("{} {} bytes", f.name, f.size));
             }
         },
         Err(e) => println!("Error! {}", e),
     }
     files
 }

 fn add_blocks_to_inode(conn: &Connection, fname: &String, blocks: &Vec<Block>) {
     let fid : u32 = get_file_info(&conn, fname).id;
     for block in blocks {
         conn.execute(
             "INSERT INTO block (nid, fid, cid) VALUES (?1, ?2, ?3)",
             &[&block.node_id as &ToSql, &fid, &block.chunk_id]).unwrap();
     }
 }

 fn get_file_inode(conn: &Connection, fname: &String) -> (INode, Vec<BlockQuery>) {
     let file = get_file_info(&conn, &fname);
     let mut stmt = conn.prepare(
         "SELECT dnode.nid, address, port, cid FROM dnode, block WHERE dnode.nid = block.nid AND block.fid = ?1")
         .unwrap();
     let iter = stmt.query_map(
         &[&file.id],
         |row| BlockQuery {
             data_node: DataNode { id: row.get(0), ip: row.get(1), port: row.get(2) },
             chunk_id: row.get(3),
         }).unwrap();
     let mut blocks: Vec<BlockQuery> = Vec::new();
     for b in iter {
         blocks.push(b.unwrap());
     }
     (file, blocks)
 }

