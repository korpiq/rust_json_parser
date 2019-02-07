#[macro_use]
extern crate nom;

use std::io::{stdin, Read};
use circular::Buffer;

mod json;
use self::json::JsonNode;

fn main() {
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Buffer::with_capacity(1000);

    loop {
        let read_result = stdin.read(buffer.space());
        match read_result {
            Ok(read_length) =>  if read_length > 0 {
                buffer.fill(read_length);
                println!("{}", JsonNode::from_bytes(buffer.data()));
            } else {
                println!("Completed.");
                break;
            },
            Err(reason) => panic!("Reading input failed: {}", reason)
        }
    }
}
