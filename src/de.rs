use std::fmt;
use std::{error, fmt::{Debug, Display}, str::FromStr};

use crate::parsing;
use nom::{branch::alt, multi::many0};
use num::{Float, Integer};
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor, DeserializeOwned};


#[derive(Eq, PartialEq)]
enum DataType {
    STRUCT,
    HASHMAP,
    SEQ,
}

pub struct JaclDeError {
    col: usize,
    line: usize,
    line_str: String,
}

impl JaclDeError {
    pub fn new(d: &Deserializer) -> Self {
        let index = d
            .begin
            .rfind(d.input)
            .expect("There's a bug in the parser!");

        let mut curr = 0;
        let mut col = 0;
        let mut line = 1;

        for c in d.begin.chars() {
            if curr == index {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            curr += 1
        }

        let mut curr = 0;
        let mut line_str = "".to_string();
        for c in d.begin.chars() {
            if curr >= index - col {
                line_str = format!("{}{}", line_str, c);
                if c == '\n' {
                    break;
                }
            }
            curr += 1
        }

        JaclDeError {
            col,
            line,
            line_str,
        }
    }
}

impl Debug for JaclDeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for JaclDeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let marker_str = format!("{}^\n", "-".repeat(self.col));
        write!(
            f,
            "error at line: {} col: {}\n{}\n{}",
            self.line, self.col, self.line_str, marker_str
        )?;
        Ok(())
    }
}

impl error::Error for JaclDeError {}

impl de::Error for JaclDeError {
    fn custom<T>(_: T) -> Self
    where
        T: std::fmt::Display,
    {
        unreachable!();
    }
}

pub struct Deserializer<'de> {
    begin: &'de str,
    pre: Option<char>,
    input: &'de str,
    post: Option<char>,
}

impl<'de> Deserializer<'de> {
    pub fn new(pre: Option<char>, data: &'de str, post: Option<char>) -> Self {
        Deserializer {
            pre,
            post,
            begin: data,
            input: data,
        }
    }
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        // handling implicit cases for []: arrays {}: maps (): structs
        let mut d = Deserializer::new(None, input, None);
        if d.try_parse_literal() {
            if d.try_parse_literal() {
                return Deserializer::new(Some('['), input, Some(']'));
            } else if let Ok(delim) = d.parse_delim() {
                if delim == ':' {
                    return Deserializer::new(Some('{'), input, Some('}'));
                }
            }
        } else if let Ok(_) = d.parse_identifier() {
            return Deserializer::new(Some('('), input, Some(')'));
        }
        return Deserializer::new(None, input, None);
    }

    fn try_parse_literal(&mut self) -> bool {
        if let Ok(_) = self.parse_bool() {
            return true;
        }
        if let Ok(_) = self.parse_float::<f32>() {
            return true;
        }
        if let Ok(_) = self.parse_int::<i64>() {
            return true;
        }
        if let Ok(_) = self.parse_string() {
            return true;
        }
        if let Ok(_) = self.parse_null() {
            return true;
        }
        return false;
    }
}

pub fn from_str<T>(s: impl Into<String>) -> Result<T, JaclDeError>
where
    T: DeserializeOwned,
{
    let s : String = s.into();
    let mut deserializer = Deserializer::from_str(&s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(JaclDeError::new(&deserializer))
    }
}

impl<'de> Deserializer<'de> {
    fn skip_non_tokens(&mut self) -> Result<(), JaclDeError> {
        if self.pre.is_some() {
            unreachable!("There's a bug in the parser! This should never happen! If pre hasn't been consumed, we shouldn't be skipping tokens... pre = {:?}", self.pre);
        }
        self.input = many0(alt((
            parsing::comment::multiline_comment,
            parsing::comment::eol_comment,
            parsing::whitespace,
        )))(self.input)
        .unwrap_or((self.input, vec![]))
        .0;
        return Ok(());
    }

    fn parse_null(&mut self) -> Result<(), JaclDeError> {
        self.skip_non_tokens()?;
        let v = match parsing::literal::null(self.input) {
            Ok((inp, b)) => {
                self.input = inp;
                Ok(b)
            }
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn parse_bool(&mut self) -> Result<bool, JaclDeError> {
        self.skip_non_tokens()?;
        let v = match parsing::literal::boolean(self.input) {
            Ok((inp, b)) => {
                self.input = inp;
                Ok(b)
            }
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn parse_int<T: Integer + FromStr>(&mut self) -> Result<T, JaclDeError> {
        self.skip_non_tokens()?;
        let v = match parsing::literal::integer(self.input) {
            Ok((inp, i)) => {
                self.input = inp;
                Ok(i)
            }
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn parse_float<T: Float + FromStr>(&mut self) -> Result<T, JaclDeError> {
        self.skip_non_tokens()?;
        let v = match parsing::literal::float(self.input) {
            Ok((inp, f)) => {
                self.input = inp;
                Ok(f)
            }
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn parse_string(&mut self) -> Result<String, JaclDeError> {
        self.skip_non_tokens()?;
        let v = match parsing::string::string(self.input) {
            Ok((inp, st)) => match st {
                Ok(s) => {
                    self.input = inp;
                    Ok(s)
                }
                Err(_) => Err(JaclDeError::new(self)),
            },
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn parse_delim(&mut self) -> Result<char, JaclDeError> {
        if let Some(c) = self.pre {
            self.pre = None;
            return Ok(c);
        }
        self.skip_non_tokens()?;
        if self.input.len() == 0 {
            if let Some(c) = self.post {
                self.post = None;
                return Ok(c);
            } else {
                return Err(JaclDeError::new(self));
            }
        }
        let v = match parsing::delimiter(self.input) {
            Ok((inp, c)) => {
                self.input = inp;
                Ok(c)
            }
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn parse_identifier(&mut self) -> Result<&str, JaclDeError> {
        self.skip_non_tokens()?;
        let v = match parsing::identifier(self.input) {
            Ok((inp, s)) => {
                self.input = inp;
                Ok(s)
            }
            Err(_) => Err(JaclDeError::new(self)),
        };
        self.skip_non_tokens()?;
        return v;
    }

    fn next_char(&self) -> Result<char, JaclDeError> {
        if let Some(pre) = self.pre {
            return Ok(pre);
        }
        if let Some(next) = self.input.chars().next() {
            return Ok(next);
        }
        if let Some(post) = self.post {
            return Ok(post);
        }
        return Err(JaclDeError::new(self));
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = JaclDeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        if self.pre.is_none() {
            self.skip_non_tokens().unwrap_or(());
        }
        match self.next_char()? {
            'n' => self.deserialize_option(visitor),
            't' | 'f' => self.deserialize_bool(visitor),
            '"' => self.deserialize_str(visitor),
            '-' | '0'..='9' => match parsing::literal::integer::<i64>(self.input) {
                Ok(res) => match res.0.chars().next() {
                    Some('.') => self.deserialize_f64(visitor),
                    _ => self.deserialize_i64(visitor),
                },
                Err(_) => Err(JaclDeError::new(self)),
            },
            '[' => self.deserialize_seq(visitor),
            '{' => self.deserialize_map(visitor),
            '(' => self.deserialize_struct("", &[""], visitor),
            _ => Err(JaclDeError::new(self)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        let s = self.parse_string()?;
        visitor.visit_str(&s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        self.skip_non_tokens()?;
        if self.input.starts_with("null") {
            self.input = &self.input["null".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        Err(JaclDeError::new(self))
        
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        Err(JaclDeError::new(self))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        Err(JaclDeError::new(self))
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        if self.parse_delim()? == '[' {
            return visitor.visit_seq(Separated::new(&mut self, DataType::SEQ));
        } else {
            Err(JaclDeError::new(self))
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        if self.parse_delim()? == '{' {
            return visitor.visit_map(Separated::new(&mut self, DataType::HASHMAP));
        } else {
            Err(JaclDeError::new(self))
        }
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        if self.parse_delim()? == '(' {
            return visitor.visit_map(Separated::new(&mut self, DataType::STRUCT));
        } else {
            Err(JaclDeError::new(self))
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(self.parse_identifier()?)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, JaclDeError>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}


// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct Separated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    datatype: DataType,
}

impl<'a, 'de> Separated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, datatype: DataType) -> Self {
        Separated { de, datatype }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for Separated<'a, 'de> {
    type Error = JaclDeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, JaclDeError>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.next_char()? == ']' {
            self.de.parse_delim()?;
            Ok(None)
        } else {
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for Separated<'a, 'de> {
    type Error = JaclDeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, JaclDeError>
    where
        K: DeserializeSeed<'de>,
    {
        if let Ok(val) = self.de.parse_delim() {
            if (val == '}' && self.datatype == DataType::HASHMAP)
                || (val == ')' && self.datatype == DataType::STRUCT)
            {
                return Ok(None);
            } else {
                return Err(JaclDeError::new(self.de));
            }
        }
        let z = seed.deserialize(&mut *self.de);
        match z {
            Ok(z) => Ok(Some(z)),
            Err(e) => Err(e),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, JaclDeError>
    where
        V: DeserializeSeed<'de>,
    {
        if let Ok(val) = self.de.parse_delim() {
            if val == ':' {
                return seed.deserialize(&mut *self.de);
            } else {
                return Err(JaclDeError::new(self.de));
            }
        } else {
            return Err(JaclDeError::new(self.de));
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.datatype {
            DataType::STRUCT => Some(0),
            DataType::HASHMAP => None,
            DataType::SEQ => None,
        }
    }
}



////////////////////////////////////////////////////////////////////////////////

mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            vec: Vec<String>,
            map: HashMap<String, Test>,
            underscore_test: u8,
        }

        let j = r#"
            (
                int : 1
                vec : ["hello","world",]
                map : {
                    "hello" : (
                        int : 17
                        vec : ["hello",,,,,   
                                    "world",,, ]
                        map: {}
                        underscore_test: 1
                    )
                }
                underscore_test: 1
            )"#;
        let vec = vec!["hello".to_string(), "world".to_string()];
        let inner = Test {
            int: 17,
            vec: vec.clone(),
            map: HashMap::new(),
            underscore_test: 1,
        };
        let mut map = HashMap::new();
        map.insert("hello".to_string(), inner);
        let expected = Test {
            int: 1,
            vec: vec.clone(),
            map,
            underscore_test: 1,
        };
        assert_eq!(expected, from_str(j).unwrap());
    }

    #[test]
    fn test_vec() {
        let v: Vec<u8> = vec![1, 2, 3, 4];
        assert_eq!(v, from_str::<Vec<u8>>("1   2 3 4,").unwrap());
        let v: Vec<String> = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(
            v,
            from_str::<Vec<String>>(r#" "hello"  "world"     "#).unwrap()
        );

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
        }

        let v: Vec<Test> = vec![Test { int: 1 }, Test { int: 2 }];
        assert_eq!(
            v,
            from_str::<Vec<Test>>(r#" [(int : 1), (int : 2)]     "#).unwrap()
        );
    }

    #[test]
    fn test_comments() {
        let v: Vec<u8> = vec![1, 2, 3, 4, 5];
        assert_eq!(
            v,
            from_str::<Vec<u8>>(
                "
        // single line comment
        1
        2
        /* multiline
           comment */
        3
        /* multiline
        comment */ // comment    
        // comment //comment //     comment
        /* comment*/ /*comment */ 4 /*comment*/
        5
        "
            )
            .unwrap()
        );
    }

    #[test]
    fn test_literals() {
        // no whitespace
        assert_eq!(true, from_str::<bool>("true").unwrap());
        assert_eq!(1u8, from_str::<u8>("1").unwrap());
        assert_eq!(1.0, from_str::<f64>("1.0").unwrap());
        assert_eq!("test", from_str::<String>(r#""test""#).unwrap());

        // with whitespace on back
        assert_eq!(true, from_str::<bool>("true      ").unwrap());
        assert_eq!(1u8, from_str::<u8>("1      ").unwrap());
        assert_eq!(1.0, from_str::<f64>("1.0      ").unwrap());
        assert_eq!("test", from_str::<String>(r#""test"   "#).unwrap());

        // whitespace front and back
        assert_eq!(true, from_str::<bool>("     true   ").unwrap());
        assert_eq!(1.0, from_str::<f64>("   1.0  ").unwrap());
        assert_eq!(1u8, from_str::<u8>("   \n1   ").unwrap());
        assert_eq!(
            "test",
            from_str::<String>(r#"   "test"   ,,,,,,,,,,,"#).unwrap()
        );

        // multiline string literal
        assert_eq!(
            "test\ntest\n",
            from_str::<String>(
                r#"
"test
test
""#
            )
            .unwrap()
        );
    }

    #[test]
    fn test_err() {
        let val: JaclDeError = from_str::<Vec<usize>>("[1 2 3]      abc")
            .expect_err("invalid jacl didn't return error?");
        assert_eq!(1, val.line);
        assert_eq!(13, val.col);
    }

    #[test]
    fn test_option() {
        assert_eq!(Some(0), from_str::<Option<u32>>("0").unwrap());
        assert_eq!(None, from_str::<Option<u32>>(" null").unwrap());
    }
}
