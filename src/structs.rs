use serde::de::DeserializeOwned;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use crate::de::from_str;
use crate::de::JaclDeError;
use crate::ser::to_string;

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Int(i64),
    Flt(f64),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number::Int(int) => write!(f, "Int({})", int),
            Number::Flt(flt) => write!(f, "Flt({})", flt),
        }
    }
}

#[derive(Debug)]
pub struct NumCastErr;

impl Error for NumCastErr {}

impl fmt::Display for NumCastErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to cast number")
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NumberVisitor)
    }
}

struct NumberVisitor;

impl<'de> Visitor<'de> for NumberVisitor {
    type Value = Number;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a float")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Int(value))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Flt(v))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
    Null,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Number(num) => write!(f, "Number({})", num),
            Literal::String(s) => write!(f, "String({})", s),
            Literal::Bool(b) => write!(f, "Bool({})", b),
            Literal::Null => write!(f, "Null"),
        }
    }
}

impl Literal {
    pub fn from_string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
    }

    pub fn from_bool(b: bool) -> Self {
        Self::Bool(b)
    }

    pub fn from_int(i: i64) -> Self {
        Self::Number(Number::Int(i))
    }

    pub fn from_flt(f: f64) -> Self {
        Self::Number(Number::Flt(f))
    }
}

impl<'de> Deserialize<'de> for Literal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(LiteralVisitor)
    }
}

struct LiteralVisitor;

impl<'de> Visitor<'de> for LiteralVisitor {
    type Value = Literal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a number, a string, a bool, or null")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Literal::Number(Number::Int(value)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Literal::Number(Number::Flt(v)))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Literal::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Literal::String(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Literal::Bool(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Literal::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Literal(Literal),
    Map(HashMap<String, Value>),
    Struct(HashMap<String, Value>),
    Seq(Vec<Value>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Literal(lit) => write!(f, "Literal({})", lit),
            Value::Map(map) => write!(f, "Map({:?})", map),
            Value::Struct(map) => write!(f, "Struct({:?})", map),
            Value::Seq(seq) => write!(f, "Seq({:?})", seq),
        }
    }
}

impl Value {
    pub fn string<S: Into<String>>(s: S) -> Self {
        Self::Literal(Literal::String(s.into()))
    }

    pub fn bool(b: bool) -> Self {
        Self::Literal(Literal::Bool(b))
    }

    pub fn int(i: i64) -> Self {
        Self::Literal(Literal::Number(Number::Int(i)))
    }

    pub fn flt(f: f64) -> Self {
        Self::Literal(Literal::Number(Number::Flt(f)))
    }

    pub fn null() -> Self {
        Self::Literal(Literal::Null)
    }

    pub fn convert<T>(&self) -> Result<T, JaclDeError>
    where
        T: DeserializeOwned + Serialize,
    {
        from_str(to_string(&self).expect("bug! could not serialize Value!"))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Literal(l) => match l {
                Literal::Number(n) => match n {
                    Number::Int(v) => serializer.serialize_i64(*v),
                    Number::Flt(v) => serializer.serialize_f64(*v),
                },
                Literal::String(v) => serializer.serialize_str(v),
                Literal::Bool(v) => serializer.serialize_bool(*v),
                Literal::Null => serializer.serialize_none(),
            },
            Value::Map(m) => {
                let mut map = serializer.serialize_map(None)?;
                for (key, value) in m {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Value::Struct(s) => {
                let mut map = serializer.serialize_map(Some(0))?;
                for (key, value) in s {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Value::Seq(s) => {
                let mut seq = serializer.serialize_seq(None)?;
                for value in s {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
        }
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a literal, a map, or a sequence")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Literal(Literal::Number(Number::Int(value))))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Literal(Literal::Number(Number::Flt(v))))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Literal(Literal::String(v.to_string())))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Literal(Literal::String(v)))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Literal(Literal::Bool(v)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(elem) = seq.next_element()? {
            vec.push(elem);
        }
        Ok(Value::Seq(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut m = HashMap::new();
        while let Some((key, value)) = map.next_entry()? {
            m.insert(key, value);
        }
        if map.size_hint().is_none() {
            Ok(Value::Map(m))
        } else {
            Ok(Value::Struct(m))
        }
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Literal(Literal::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let int = Number::Int(1);
        let flt = Number::Flt(1.75);
        // check parsing
        assert_eq!(int, from_str::<Number>("1").unwrap());
        assert_eq!(flt, from_str::<Number>("1.75").unwrap());
    }

    #[test]
    fn test_literal() {
        let int = Literal::from_int(1);
        let flt = Literal::from_flt(1.75);
        let b = Literal::from_bool(true);
        let s = Literal::from_string("hello world");
        let null = Literal::Null;

        assert_eq!(b, from_str("true").unwrap());
        assert_eq!(s, from_str(r#"  "hello world"   "#).unwrap());
        assert_eq!(int, from_str("1").unwrap());
        assert_eq!(flt, from_str("1.75").unwrap());
        assert_eq!(null, from_str("null").unwrap());
    }

    #[test]
    fn test_value() {
        let val = Value::Seq(vec![
            Value::bool(true),
            Value::flt(1.0),
            Value::string("hello world"),
            Value::null(),
            Value::Map(HashMap::from([
                ("key_0".into(), Value::null()),
                ("key_1".into(), Value::bool(false)),
            ])),
        ]);

        assert_eq!(
            val,
            from_str(
                r#"
            true
            1.0
            "hello world"
            null
            {
                "key_0" : null
                "key_1" : false
            }
        "#
            )
            .unwrap()
        );

        let map = Value::Map(HashMap::from([
            ("key_0".into(), Value::null()),
            ("key_1".into(), Value::bool(false)),
        ]));

        assert_eq!(
            map,
            from_str(
                r#"
            "key_0" : null
            "key_1" : false
        "#
            )
            .unwrap()
        );
    }

    #[test]
    fn test_value_struct() {
        let val = Value::Struct(HashMap::from([
            ("a".into(), Value::int(0)),
        ]));
        assert_eq!(val, from_str("\"a\" : 0").unwrap());
    }
}
