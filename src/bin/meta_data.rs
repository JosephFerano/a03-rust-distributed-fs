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
    // TODO: We need to check if the DB exists, if not, ask the user to run the python script
    // Maybe we can even run it for them
    let conn = Connection::open("dfs.db").unwrap();
    let mut file_list: Vec<String> = Vec::new();

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
        "DELETE FROM dnode WHERE address=?1 AND port=?2)",
        &[&address as &ToSql, &port],
    ) {
        Ok(n) => println!("{} rows updated", n),
        Err(e) => println!("DELETE error: {}", e),
    };
}

fn get_data_node(conn: &Connection, address: &str, port: i32) -> DataNode {
    let mut stmt = conn
        .prepare("SELECT nid, address, port FROM dnode WHERE address=?1 AND port=?2")
        .unwrap();
    stmt.query_row(&[&address as &ToSql, &port], |row| DataNode {
        id: row.get(0),
        ip: row.get(1),
        port: row.get(2),
    }).unwrap()
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
     let iter = stmt.query_map(NO_PARAMS, |row| INode {
         id: 0,
         name: row.get(0),
         size: row.get(1),
     }).unwrap();
     let mut files = Vec::new();
     for f in iter {
         let f = f.unwrap();
         files.push(format!("{} {} bytes", f.name, f.size));
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
         "SELECT address, port, cid FROM dnode, block WHERE dnode.nid = block.nid AND block.fid = ?1")
         .unwrap();
     let iter = stmt.query_map(
         &[&file.id],
         |row| BlockQuery {
             data_node: DataNode { id: 0, ip: row.get(0), port: row.get(1) },
             chunk_id: row.get(2),
             id: 0,
         }).unwrap();
     let mut blocks: Vec<BlockQuery> = Vec::new();
     for b in iter {
         blocks.push(b.unwrap());
     }
     (file, blocks)
 }

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tables(conn: &Connection) {
        conn.execute(
"CREATE TABLE inode (
fid INTEGER PRIMARY KEY ASC AUTOINCREMENT,
fname TEXT UNIQUE NOT NULL DEFAULT \" \",
fsize INTEGER NOT NULL default \"0\")",
            NO_PARAMS
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
            NO_PARAMS
        ).unwrap();

        // Create UNIQUE tuple for block
        conn.execute("CREATE UNIQUE INDEX dnodeA ON dnode(address, port)", NO_PARAMS)
            .unwrap();

        // Create UNIQUE tuple for block
        conn.execute("CREATE UNIQUE INDEX blocknc ON block(nid, cid)", NO_PARAMS)
            .unwrap();
    }

    fn get_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn);
        conn
    }

    #[test]
    fn add_dnode_with_correct_ip_and_port() {
        let conn = get_test_db();
        let ip = "127.0.0.1";
        let port = 65533;
        add_data_node(&conn, &ip, port as i32);
        let dnode = get_data_node(&conn, &ip, port as i32);
        assert_eq!(dnode.id, 1);
        assert_eq!(dnode.ip, ip);
        assert_eq!(dnode.port, port);
    }

    #[test]
    fn removes_dnode_with_correct_ip_and_port() {
        // TODO: I don't know how to test a delete
        let conn = get_test_db();
        let ip = "127.0.0.1";
        let port = 65533;
        add_data_node(&conn, &ip, port as i32);
        let dnode = get_data_node(&conn, &ip, port as i32);
        assert_eq!(dnode.id, 1);
        assert_eq!(dnode.ip, ip);
        assert_eq!(dnode.port, port);
    }

    #[test]
    fn gets_all_data_nodes() {
        let conn = get_test_db();
        let ip1 = "127.0.0.1";
        let port1 = 65533;
        let ip2 = "127.0.0.2";
        let port2 = port1 + 1;
        let ip3 = "127.0.0.3";
        let port3 = port2 + 1;
        add_data_node(&conn, &ip1, port1 as i32);
        add_data_node(&conn, &ip2, port2 as i32);
        add_data_node(&conn, &ip3, port3 as i32);
        let ds = get_data_nodes(&conn);
        for i in 0..ds.len() {
            let d = &ds[i];
            assert_eq!(d.ip, format!("127.0.0.{}", i + 1));
            assert_eq!(d.port, 65533 + i as u32);
        }
    }

    #[test]
    fn adds_file() {
        let conn = get_test_db();
        add_file(&conn, &String::from("my_1337_virus"), 32);
        let files = get_files(&conn);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "my_1337_virus 32 bytes");
    }

    #[test]
    fn deletes_file() {
        let conn = get_test_db();
        add_file(&conn, &String::from("my_1337_virus"), 32);
        let files = get_files(&conn);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], "my_1337_virus 32 bytes");
    }

    #[test]
    fn gets_all_file() {
        let conn = get_test_db();
        add_file(&conn, &String::from("file1"), 32);
        add_file(&conn, &String::from("file2"), 64);
        add_file(&conn, &String::from("file3"), 128);
        let files = get_files(&conn);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0], "file1 32 bytes");
        assert_eq!(files[1], "file2 64 bytes");
        assert_eq!(files[2], "file3 128 bytes");
    }

    #[test]
    fn adds_blocks_to_inode() {
        let conn = get_test_db();
        let filename = String::from("main_file");
        add_file(&conn, &filename, 128);
        add_data_node(&conn, "127.0.0.1", 1337);
        add_data_node(&conn, "127.0.0.2", 1338);
        let inode = get_file_info(&conn, &filename);
        let blocks = vec!(
            Block {
                file_id: inode.id,
                id: 0,
                node_id: 1,
                chunk_id: String::from("c1"),
            },
            Block {
                file_id: inode.id,
                id: 0,
                node_id: 2,
                chunk_id: String::from("c2"),
            },
        );
        add_blocks_to_inode(&conn, &filename, &blocks);
        let (inode, blocks) = get_file_inode(&conn, &filename);
        assert_eq!(inode.name, "main_file");
        assert_eq!(inode.size, 128);
        assert_eq!(blocks.len(), 2);
        let dn1 = get_data_node(&conn, "127.0.0.1", 1337);
        let dn2 = get_data_node(&conn, "127.0.0.2", 1338);
        assert_eq!(dn1.id, 1);
        assert_eq!(dn2.id, 2);
    }

}
