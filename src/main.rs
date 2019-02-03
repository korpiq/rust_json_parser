#[macro_use]
extern crate nom;

use nom::double;
use std::io::{stdin, Read};
use std::collections::HashMap;
use circular::Buffer;

#[derive(PartialEq, Debug)]
pub enum JsonNode<'a> {
    Number(f64),
    String(String),
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
        parse_json_null | parse_json_number | parse_json_string | parse_json_array
    )
);

named!(parse_json_null<&[u8], JsonNode>,
    do_parse!(
        tag_s!("null") >>
        (JsonNode::Null)
    )
);

named!(parse_json_number<&[u8], JsonNode>,
    do_parse!(value: double >> (JsonNode::Number(value)))
);

named!(parse_json_string<&[u8], JsonNode>,
    do_parse!(
        tag_s!("\"") >>
        value: parse_json_escaped_string >>
        tag_s!("\"") >>
        (JsonNode::String(String::from_utf8(value).unwrap()))
    )
);

named!(parse_json_escaped_string<&[u8], Vec<u8>>,
    do_parse!(
        result: many0!(
            alt!(
                is_not!("\\\"")
                | value!("\\".as_bytes(), tag_s!("\\\\"))
                | value!("\"".as_bytes(), tag_s!("\\\""))
            )
        )
        >>
        (collect_many(result))
    )
);

// TODO: there must be a train wreck to replace this.
fn collect_many(many : Vec<&[u8]>) -> Vec<u8> {
    let mut flat : Vec<u8> = Vec::new();
    let mut arrays_iterator = many.iter();
    while let Some(current_array) = arrays_iterator.next() {
        let mut character_iterator = current_array.iter();
        while let Some(character) = character_iterator.next() {
            flat.push(*character);
        }
    }

    flat
}

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
    fn test_number_ok() {
        // we provide an extra character to make parser realize the number is complete.

        assert_eq!(JsonNode::from_str("0 "), JsonNode::Number(0.0));
        assert_eq!(JsonNode::from_str("+0 "), JsonNode::Number(0.0));
        assert_eq!(JsonNode::from_str("-0 "), JsonNode::Number(0.0));

        assert_eq!(JsonNode::from_str(".0 "), JsonNode::Number(0.0));
        assert_eq!(JsonNode::from_str("0.0 "), JsonNode::Number(0.0));
        assert_eq!(JsonNode::from_str("00.000 "), JsonNode::Number(0.0));

        assert_eq!(JsonNode::from_str("1 "), JsonNode::Number(1.0));
        assert_eq!(JsonNode::from_str("00012345 "), JsonNode::Number(12345.0));
        assert_eq!(JsonNode::from_str("12.345000 "), JsonNode::Number(12.345));
        assert_eq!(JsonNode::from_str("67e89 "), JsonNode::Number(67e89));
        assert_eq!(JsonNode::from_str("-67e89 "), JsonNode::Number(-67e89));
        assert_eq!(JsonNode::from_str("5.67e-89 "), JsonNode::Number(5.67e-89));
    }

    #[test]
    fn test_empty_string_ok() {
        assert_eq!(JsonNode::from_str("\"\""), JsonNode::String("".to_string()));
    }

   #[test]
    fn test_strings_ok() {
        assert_eq!(JsonNode::from_str("\" \""), JsonNode::String(" ".to_string()));
        assert_eq!(JsonNode::from_str("\"#€%&/()=\""), JsonNode::String("#€%&/()=".to_string()));
    }

   #[test]
    fn test_escaped_strings_ok() {
        assert_eq!(JsonNode::from_str("\"\\\"\""), JsonNode::String("\"".to_string()));
        assert_eq!(JsonNode::from_str("\"\\\\\""), JsonNode::String("\\".to_string()));
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
