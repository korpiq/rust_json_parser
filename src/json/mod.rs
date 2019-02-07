use std::fmt;
#[warn(unused_imports)]
use std::collections::HashMap;
mod parser;
use self::parser::parse_json;

#[derive(PartialEq, Debug)]
pub enum JsonNode {
    Number(f64),
    String(String),
    Array(Vec<JsonNode>),
    Object(HashMap<String, JsonNode>),
    Null
}

impl fmt::Display for JsonNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonNode::Number(n) => f64::fmt(n, f),
            JsonNode::String(s) => write!(f, "\"{}\"", s),
            JsonNode::Array(a) => JsonNode::fmt_array(a, f),
            JsonNode::Object(o) => JsonNode::fmt_object(o, f),
            JsonNode::Null => write!(f, "null")
        }
    }
}

impl JsonNode {
    pub fn from_str(json : &str) -> JsonNode {
        JsonNode::from_bytes(json.as_bytes())
    }

    pub fn from_bytes(buffer : &[u8]) -> JsonNode {
        let result = parse_json(&buffer);
        match result {
            Ok(rest_and_json) => rest_and_json.1,
            Err(reason) => panic!("JSON parsing failed: {}", reason.to_string())
        }
    }

    fn fmt_array(a : &Vec<JsonNode>, f: &mut fmt::Formatter) -> fmt::Result {
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

    fn fmt_object(o : &HashMap<String, JsonNode>, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma = false;
        let mut elements = o.iter();

        let r = write!(f, "{{");
        match r { Err(_) => return r, Ok(_) => () }

        while let Some((k, v)) = elements.next() {
            if comma {
                let r = write!(f, ",");
                match r { Err(_) => return r, Ok(_) => () }
            }
            let r = write!(f, "\"{}\":{}", k, v);
            match r { Err(_) => return r, Ok(_) => () }
            comma = true
        }

        write!(f, "}}")
    }
}
