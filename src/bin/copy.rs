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
    let mut file = fs::read("/home/joe/Downloads/ideaIU-2018.3.tar.gz").unwrap();
    let mut stream = TcpStream::connect("localhost:6771").unwrap();
    stream.write(&file).unwrap();
    stream.flush().unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
}
