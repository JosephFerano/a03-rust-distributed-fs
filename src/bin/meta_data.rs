extern crate a03;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

 use a03::*;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};
use std::net::{TcpListener,TcpStream};
use std::thread;
use std::io::Read;

fn main() {
    let listener = TcpListener::bind("localhost:6770").unwrap();
    println!("Binding!");
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        println!("Got here!");
        let test_obj : TestObj = serde_json::from_reader(&mut stream).unwrap();
        match serde_json::to_writer(
            &mut stream,
            &TestObj { message : String::from("I got it dude") })
        {
            Ok(_) => println!("{}", test_obj.message),
            Err(e) => println!("{}", e),
        };
    }
}

#[derive(Debug)]
struct DataNode {
    id: i32,
    address: String,
    port: i32,
}

#[derive(Debug)]
struct INode {
    id: i32,
    name: String,
    size: i32,
}

#[derive(Debug)]
struct Block {
    id : i32,
    file_id : i32,
    node_id : i32,
    c_id : String,
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
    stmt.query_row(
        &[&address as &ToSql, &port],
        |row| DataNode {
            id: row.get(0),
            address: row.get(1),
            port: row.get(2),
        })
        .unwrap()
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
