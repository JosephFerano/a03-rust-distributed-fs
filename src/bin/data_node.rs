extern crate a03;

use a03::*;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::net::{TcpListener,TcpStream,Shutdown};
use std::thread;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut stream = TcpStream::connect("localhost:6770").unwrap();
    let writer = serde_json::to_writer(
        &mut stream,
        &TestObj { message : String::from("I can't think of something clever")})
        .unwrap();
    println!("Message sent!");
    stream.flush().unwrap();
//    stream.shutdown(Shutdown::Write);
    let test_obj : TestObj = serde_json::from_reader(&mut stream).unwrap();
    println!("Received message back! {}", test_obj.message);
}
