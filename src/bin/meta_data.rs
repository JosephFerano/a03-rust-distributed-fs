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
    let mut data_nodes: Vec<DataNode> = Vec::new();
    let mut file_list: Vec<String> = Vec::new();

    file_list.push(String::from("/"));
    file_list.push(String::from("/home"));
    file_list.push(String::from("/home/joe"));
    file_list.push(String::from("/bin"));
    let listener = TcpListener::bind("localhost:6770").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        match serde_json::from_reader(&mut stream) {
            Ok(packet @ Packet { .. }) => match packet.p_type {
                PacketType::ListFiles => list(&mut stream, &file_list[..]),
                PacketType::PutFiles => put(&mut stream, &packet.json.unwrap(), &mut file_list),
                PacketType::NodeRegistration =>
                    node_registration(&mut stream, &packet.json.unwrap(), &mut data_nodes),
                _ => (),
            },
            Err(e) => println!("Error parsing json {}", e.to_string()),
        };
        stream.flush().unwrap();
    }
}

fn list(stream: &mut TcpStream, files: &[String]) {
    match serde_json::to_writer(
        stream,
        &FilePaths {
            paths: Cow::from(files),
        },
    ) {
        Ok(_) => println!("{}", "Sent file paths"),
        Err(e) => println!("{}", e),
    };
}

fn put(stream: &mut TcpStream, json: &String, files: &mut Vec<String>) {
    let files: PutFiles = serde_json::from_str(json).unwrap();
    report_success(stream, "Successfully Put Files");
}

fn node_registration(stream: &mut TcpStream, json: &String, data_nodes: &mut Vec<DataNode>) {
    let endpoint : NodeRegistration = serde_json::from_str(json).unwrap();
    let message = if endpoint.register {
        data_nodes.push(DataNode { ip: endpoint.ip, port: endpoint.port, id: 1 });
        "You were successfully registering"
    }
    else {
        match data_nodes.iter()
            .position(|dn| dn.ip == endpoint.ip && dn.port == endpoint.port) {
            Some(index) => {
                data_nodes.remove(index);
                "You were successfully unregistering"
            },
            None => {
                println!("Data Node at {}:{} does not exit", endpoint.ip, endpoint.port);
                "You weren't found"
            }
        }
    };
    for dn in data_nodes {
        println!("{:?}", dn);
    }
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

fn create_tables(conn: &Connection) {
    conn.execute(
        "CREATE TABLE inode (
fid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
fname TEXT UNIQUE NOT NULL DEFAULT \" \",
fsize INTEGER NOT NULL default \"0\")",
        NO_PARAMS,
    ).unwrap();

    conn.execute(
        "CREATE TABLE dnode (
nid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
address TEXT NOT NULL DEFAULT \" \",
port INTEGER NOT NULL DEFAULT \"0\")",
        NO_PARAMS,
    ).unwrap();

    conn.execute(
        "CREATE TABLE block (
bid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
fid INTEGER NOT NULL DEFAULT \"0\",
nid INTEGER NOT NULL DEFAULT \"0\",
cid TEXT NOT NULL DEFAULT \"0\")",
        NO_PARAMS,
    ).unwrap();

    // Create UNIQUE tuple for block
    conn.execute("CREATE UNIQUE INDEX blocknc ON block(nid, cid)", NO_PARAMS)
        .unwrap();
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

fn check_node(conn: &Connection, address: &str, port: i32) -> DataNode {
    let mut stmt = conn
        .prepare("SELECT nid, address, port FROM dnode WHERE address=?1 AND port=?2")
        .unwrap();
    stmt.query_row(&[&address as &ToSql, &port], |row| DataNode {
        id: row.get(0),
        ip: row.get(1),
        port: row.get(2),
    }).unwrap()
}

fn get_data_nodes(conn: &Connection) {
    let mut stmt = conn.prepare("SELECT address, port FROM dnode WHERE 1");
}

fn insert_file(conn: &Connection, fname: String, fsize: usize) {
    let mut stmt = conn.prepare("INSERT INTO inode (fname, fsize) VALUES (\"?1\", ?2)");
}

// fn get_file_info(conn: &Connection, fname: String) {}

// fn get_files(conn: &Connection) {}

// fn add_block_to_inode(conn: &Connection, fname: String, blocks: usize) {}

// fn get_file_inode(conn: &Connection, fname: String) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn);
        conn
    }

    #[test]
    fn inserts_dnode_with_correct_ip_and_port() {
        let conn = get_test_db();
        let ip = "127.0.0.1";
        add_data_node(&conn, &ip, 65533);
        let dnode = check_node(&conn, &ip, 65533);
        assert_eq!(dnode.id, 1);
        assert_eq!(dnode.address, ip);
        assert_eq!(dnode.port, 65533);
    }
}
