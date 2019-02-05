#[macro_use]
extern crate nom;

use nom::double;
use std::fmt;
#[allow(unused_imports)]
use std::io::{stdin, Read, Write};
#[warn(unused_imports)]
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

    fn fmt_array(a : &Vec<JsonNode<'_>>, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma = false;
        let mut elements = a.iter();

        let r = write!(f, "[");
        match r { Err(_) => return r, Ok(_) => () }

        while let Some(it) = elements.next() {
            if comma {
                let r = write!(f, ",");
                match r { Err(_) => return r, Ok(_) => () }
            }
            let r = write!(f, "{}", it);
            match r { Err(_) => return r, Ok(_) => () }
            comma = true
        }

        write!(f, "]")
    }
}

impl fmt::Display for JsonNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonNode::Number(n) => f64::fmt(n, f),
            JsonNode::String(s) => write!(f, "\"{}\"", s),
            JsonNode::Array(a) => JsonNode::fmt_array(a, f),
            JsonNode::Object(o) => write!(f, "{{{:?}}}", o),
            JsonNode::Null => write!(f, "null")
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
                map!( parse_json_escaped_ascii, Vec::from )
                | parse_json_unicode_escape
            )
        )
        >>
        (flatten(result))
    )
);

fn flatten(deep : Vec<Vec<u8>>) -> Vec<u8> {
    let mut flat : Vec<u8> = Vec::new();
    deep.iter().for_each(| inner | flat.extend(inner.iter()));
    flat
}

named!(parse_json_escaped_ascii<&[u8], &[u8]>,
    alt!(
        is_not!("\\\"")
        | value!("\\".as_bytes(), tag_s!("\\\\"))
        | value!("\"".as_bytes(), tag_s!("\\\""))
        | value!("/".as_bytes(), tag_s!("\\/"))
        | value!("\08".as_bytes(), tag_s!("\\b"))
        | value!("\n".as_bytes(), tag_s!("\\n"))
        | value!("\r".as_bytes(), tag_s!("\\r"))
        | value!("\t".as_bytes(), tag_s!("\\t"))
    )
);

named!(parse_json_unicode_escape<&[u8], Vec<u8>>,
    do_parse!(
        tag_s!("\\u") >>
        result: map!( take!(4), codepoint_from_hex ) >>
        (result)
    )
);

#[allow(dead_code)] // used in parse_json_unicode_escape
fn codepoint_from_hex(input: &[u8]) -> Vec<u8> {
  let hex = String::from_utf8(input.to_vec()).unwrap();
  let value = u32::from_str_radix(&hex, 16).unwrap();
  let mut buffer : [u8; 4] = [0; 4];

  std::char::from_u32(value).unwrap().encode_utf8(&mut buffer).as_bytes().to_vec()
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
                println!("{}", JsonNode::from_bytes(buffer.data()));
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
        assert_eq!(JsonNode::from_str("\"\\/\""), JsonNode::String("/".to_string()));
        assert_eq!(JsonNode::from_str("\"\\b\""), JsonNode::String("\08".to_string()));
        assert_eq!(JsonNode::from_str("\"\\n\""), JsonNode::String("\n".to_string()));
        assert_eq!(JsonNode::from_str("\"\\r\""), JsonNode::String("\r".to_string()));
        assert_eq!(JsonNode::from_str("\"\\t\""), JsonNode::String("\t".to_string()));
        assert_eq!(JsonNode::from_str("\"\\u211D\""), JsonNode::String("\u{211D}".to_string()));
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

    #[test]
    fn test_output_format_ok() {
        let mut output = Vec::new();
        let json_string = "[1.23,null,\"foo\"]";
        let json = JsonNode::from_str(json_string);
        write!(output, "{}", json).expect("write never fails");
        assert_eq!(std::str::from_utf8(&output).unwrap(), json_string);
    }
}
