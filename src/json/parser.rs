use nom;
use nom::double;
use crate::json::JsonNode;
use std::collections::HashMap;
#[allow(unused_imports)]
use std::io::Write;

pub fn parse_json(input: &[u8]) -> Result<(&[u8], JsonNode), nom::Err<&[u8], u32>> {
    parse_json_element(input)
}

named!(parse_json_element<&[u8], JsonNode>,
    alt!(
        parse_json_null | parse_json_number | parse_json_string | parse_json_array | parse_json_object
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
        value: parse_json_escaped_string >>
        (JsonNode::String(value))
    )
);

named!(parse_json_escaped_string<&[u8], String>,
    do_parse!(
        tag_s!("\"") >>
        result: many0!(
            alt!(
                map!( parse_json_escaped_ascii, Vec::from )
                | parse_json_unicode_escape
            )
        ) >>
        tag_s!("\"") >>
        (String::from_utf8(flatten(result)).unwrap())
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

named!(parse_json_object<&[u8], JsonNode>,
    do_parse!(
        tag_s!("{") >>
        content: opt!(separated_list_complete!(tag_s!(","), parse_json_pair)) >>
        tag_s!("}") >>
        (
            {
                let mut container = HashMap::<String, JsonNode>::new();
                match content {
                    Some(mut elements) => {
                        while let Some((k, v)) = elements.pop() {
                            match container.insert(k, v) { _ => () }
                        }
                    },
                    None => ()
                }
                JsonNode::Object(container)
            }
        )
    )
);

named!(parse_json_pair<&[u8], (String, JsonNode)>,
    do_parse!(
        name: parse_json_escaped_string >>
        tag_s!(":") >>
        value: parse_json_element >>
        ( (name, value) )
    )
);

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
    fn test_empty_object_ok() {
        let expected = HashMap::<String, JsonNode>::new();
        assert_eq!(JsonNode::from_str("{}"), JsonNode::Object(expected));
    }

    #[test]
    fn test_object_ok() {
        let mut expected = HashMap::<String, JsonNode>::new();
        expected.insert("foo".to_string(), JsonNode::Null);
        assert_eq!(JsonNode::from_str("{\"foo\":null}"), JsonNode::Object(expected));
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
