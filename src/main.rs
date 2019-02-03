#[macro_use]
extern crate nom;

use std::str;
use std::io::{stdin, Read};
use std::collections::HashMap;
use circular::Buffer;

#[derive(PartialEq, Debug)]
pub enum JsonNode<'a> {
    Number(f64),
    String(&'a str),
    Array(Vec<JsonNode<'a>>),
    Object(&'a HashMap<&'a str, &'a JsonNode<'a>>),
    Null
}

impl JsonNode<'_> {
    pub fn from_str(json : &str) -> JsonNode {
        JsonNode::from_bytes(json.as_bytes())
    }

    pub fn from_bytes(buffer : &[u8]) -> JsonNode {
        let result = parse_json_element(&buffer);
        match result {
            Ok(rest_and_json) => rest_and_json.1,
            Err(reason) => panic!("JSON parsing failed: {}", reason.to_string())
        }
    }
}

named!(parse_json_element<&[u8], JsonNode>,
    alt!(
        parse_json_null | parse_json_number | parse_json_array
    )
);

named!(parse_json_null<&[u8], JsonNode>,
    do_parse!(
        tag_s!("null") >>
        (JsonNode::Null)
    )
);

named!(parse_json_number<&[u8], JsonNode>,
    do_parse!(
        tag_s!("0") >>
        (JsonNode::Number("0.0".parse::<f64>().unwrap()))
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

fn main() {
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buffer = Buffer::with_capacity(1000);

    loop {
        let read_result = stdin.read(buffer.space());
        match read_result {
            Ok(read_length) =>  if read_length > 0 {
                buffer.fill(read_length);
                JsonNode::from_bytes(buffer.data());
            } else {
                println!("Completed.");
                break;
            },
            Err(reason) => panic!("Reading input failed: {}", reason)
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_null_ok() {
        assert_eq!(JsonNode::from_str("null"), JsonNode::Null);
    }

    #[test]
    fn test_zero_ok() {
        assert_eq!(JsonNode::from_str("0"), JsonNode::Number(0.0));
    }

    #[test]
    fn test_empty_list_ok() {
        let expected = Vec::new();
        assert_eq!(JsonNode::from_str("[]"), JsonNode::Array(expected));
    }
    
    #[test]
    fn test_lists_within_lists_ok() {
        let mut expected = Vec::new();
        expected.push(JsonNode::Array(Vec::new()));
        assert_eq!(JsonNode::from_str("[[]]"), JsonNode::Array(expected));

        let mut expected = Vec::new();
        expected.push(JsonNode::Array(Vec::new()));
        expected.push(JsonNode::Array(Vec::new()));
        assert_eq!(JsonNode::from_str("[[],[]]"), JsonNode::Array(expected));

        let mut expected = Vec::new();
        let mut inner = Vec::new();
        inner.push(JsonNode::Array(Vec::new()));
        expected.push(JsonNode::Array(inner));
        assert_eq!(JsonNode::from_str("[[[]]]"), JsonNode::Array(expected));
    }

    #[test]
    #[should_panic(expected = "JSON parsing failed: Error(")]
    fn test_list_with_a_comma_only_fails() {
        JsonNode::from_str("[,]");
    }

    #[test]
    #[should_panic(expected = "JSON parsing failed: Error(")]
    fn test_list_with_extra_comma_fails() {
        JsonNode::from_str("[[],]");
    }

    #[test]
    #[should_panic(expected = "JSON parsing failed: Error(")]
    fn test_list_starting_with_comma_fails() {
        JsonNode::from_str("[,[]]");
    }

    #[test]
    #[should_panic(expected = "JSON parsing failed: Incomplete(Size(")]
    fn test_empty_input_fails() {
        JsonNode::from_str("");
    }

    #[test]
    #[should_panic(expected = "JSON parsing failed: Error(")]
    fn test_bad_syntax_input_fails() {
        JsonNode::from_str("x");
    }
}
