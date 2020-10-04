use escape8259::{unescape, UnescapeError};
use std::error::Error;

fn parse_string<'a>(input: &'a str) -> Result<(&'a str, String), Box<dyn Error>> {
    let mut escp = false;
    let mut first = true;
    let mut s = "".to_string();
    for (i, c) in input.char_indices() {
        if c != '"' && first {
            println!("not a string ^{}^", input);
            return Err("not a string".into());
        } else if c == '\\' && !escp {
            escp = true;
        } else if c == '"' && !escp && !first {
            return Ok((&input[(i+1)..input.len()], s));
        } else if !c.is_whitespace() {
            escp = false;
        }
        if !first {
            if c == '\n' {
                s += "\\n";
            } else if c == '\r' {
                s += "\\r";
            } else {
                s += &format!("{}", c);
            }            
        }
        first = false;
    }
    println!("unclosed string ^{}^", input);
    return Err("unclosed string".into());
}

pub fn string<'a>(input: &'a str) -> Result<(&str, Result<String, UnescapeError>), Box<dyn Error>> {
    return parse_string(input).map(|out: (&str, String)| (out.0, unescape(&out.1)));
}