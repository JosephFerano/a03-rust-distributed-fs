extern crate a03;
extern crate rusqlite;

use a03::*;
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

fn main() {
	let conn = Connection::open("dfs.db").unwrap();
}


fn connect(conn : Connection) {
}

fn close(conn : Connection) {
}

fn add_data_node(conn : Connection, address : String, port : usize) {
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

