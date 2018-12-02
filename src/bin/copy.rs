extern crate a03;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use a03::*;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::Read;
use std::io::Write;
use std::fs::File;
use std::fs;

fn main() {
    let mut file = fs::read("dfs.db").unwrap();
    let mut copy = File::create("copy").unwrap();
    copy.write_all(&file[..]).unwrap();
}
