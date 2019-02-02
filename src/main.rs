#[macro_use]
extern crate nom;

use std::str;
use std::io::{stdin, Read};
use std::collections::HashMap;
use circular::Buffer;

pub enum JsonNode<'a> {
    Number(f64),
    String(&'a str),
    Array(Vec<JsonNode<'a>>),
    Object(&'a HashMap<&'a str, &'a JsonNode<'a>>),
    Null
}

named!(parse_json<&[u8], JsonNode>,
    do_parse!(
        element: parse_json_element >>
        (element)
    )
);

named!(parse_json_element<&[u8], JsonNode>,
    alt!(
        parse_json_array | parse_json_null
    )
);

named!(parse_json_null<&[u8], JsonNode>,
    do_parse!(
        tag_s!("null") >>
        (JsonNode::Null)
    )
);

named!(parse_json_array<&[u8], JsonNode>,
    do_parse!(
        tag_s!("[") >>
        content: opt!(separated_list_complete!(tag_s!(","), parse_json_element)) >>
        tag_s!("]") >>
        (
            match content {
                Some(elements) => (
                    JsonNode::Array(elements)
                ),
                None => (
                    JsonNode::Array(Vec::<JsonNode>::new())
                )
            }
        )
    )
);

fn parse_some_json(buffer : &[u8]) -> JsonNode {
    let result = parse_json(&buffer);
    match result {
        Ok(rest_and_json) => rest_and_json.1,
        Err(reason) => panic!("JSON parsing failed: {}", reason.to_string())
    }
}

fn main() {
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Buffer::with_capacity(1000);

    loop {
        let read_result = stdin.read(buffer.space());
        match read_result {
            Ok(read_length) =>  if read_length > 0 {
                buffer.fill(read_length);
                parse_some_json(buffer.data());
            } else {
                println!("Completed.");
                break;
            },
            Err(reason) => panic!("Reading input failed: {}", reason)
        }
    }
}
