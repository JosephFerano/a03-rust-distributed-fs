extern crate a03;
extern crate rusqlite;

use a03::*;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

fn main() {
	let conn = Connection::open("dfs.db").unwrap();
}

#[derive(Debug)]
struct Dnode {
    id : i32,
    address : String,
    port : i32,
}

fn add_data_node(conn : &Connection, address : &str, port : i32) {
    let dn = Dnode {
        id : 0,
        address : address.to_string(),
        port,
    };
    match conn.execute(
        "INSERT INTO dnode (address, port) VALUES (?1, ?2)",
        &[&dn.address as &ToSql, &dn.port]) {
        Ok(n) => println!("{} rows updated", n),
        Err(e) => println!("INSERT error: {}", e)
    }
}

fn check_node(conn : Connection, address : String, port : usize) {
}

fn get_data_nodes(conn : Connection) {
}

fn insert_file(conn : Connection, fname : String, fsize : usize) {
}

fn get_file_info(conn : Connection, fname : String) {
}

fn get_files(conn : Connection) {
}

fn add_block_to_inode(conn : Connection, fname : String, blocks : usize) {
}

fn get_file_inode(conn : Connection, fname : String) {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_dnode() {
    }

}
