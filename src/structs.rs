use std::error::Error;
use num::{Float, Integer, NumCast, ToPrimitive};
use serde::de::MapAccess;
use serde::de::SeqAccess;
use std::collections::HashMap;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

#[cfg(test)]
use crate::de::from_str;

#[derive(Debug)]
enum Any <'a> {
    Number(&'a Number),
    Literal(&'a Literal),
    Value(&'a Value)
}

impl fmt::Display for Any <'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Any::Number(n) => write!(f, "{}", n),
            Any::Literal(l) => write!(f, "{}", l),
            Any::Value(v) => write!(f, "{}", v)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    Int(i64),
    Flt(f64),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number::Int(int) => write!(f, "Int({})", int),
            Number::Flt(flt) => write!(f, "Flt({})", flt)
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

#[derive(Debug)]
pub struct InvalidAsErr <'a> {
    original : Any<'a>,
    attempted_type_convert : String
}

impl  <'a> InvalidAsErr <'a> {
    fn new (original : Any<'a>, attempted_type_convert : String) -> Self {
        Self {
            original,
            attempted_type_convert
        }
    }
}

impl Error for InvalidAsErr <'_> {}

impl fmt::Display for InvalidAsErr <'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "attempted to convert {} to {}", self.original, self.attempted_type_convert)
    }
}

impl Number {
    pub fn from_int <T: Integer + ToPrimitive> (i : T) -> Result<Self, NumCastErr> {
        Ok(Self::Int(NumCast::from(i).ok_or(NumCastErr)?))
    }

    pub fn from_float <T: Float> (f : T) -> Result<Self, NumCastErr> {
        Ok(Self::Flt(NumCast::from(f).ok_or(NumCastErr)?))
    }

    fn get_err(&self, convert_type : &str) -> InvalidAsErr {
        InvalidAsErr::new(Any::Number(self), convert_type.to_string())
    }

    pub fn as_int(&self) -> Result<&i64, InvalidAsErr> {
        if let Number::Int(i) = self {
            Ok(i)
        } else {
            Err(self.get_err("i64"))
        }
    }

    pub fn as_flt(&self) -> Result<&f64, InvalidAsErr> {
        if let Number::Flt(f) = self {
            Ok(f)
        } else {
            Err(self.get_err("f64"))
        }
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
    Bool(bool)
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Number(num) => write!(f, "Number({})", num),
            Literal::String(s) => write!(f, "String({})", s),
            Literal::Bool(b) => write!(f, "Bool({})", b)
        }
    }
}

impl Literal {
    pub fn from_string<S: Into<String>>(s : S) -> Self {
        Self::String(s.into())
    }

    pub fn from_bool(b : bool) -> Self {
        Self::Bool(b)
    }

    pub fn from_int<T : Integer + ToPrimitive>(i : T) -> Result<Self, NumCastErr> {
        Ok(Self::Number(Number::from_int(i)?))
    }

    pub fn from_flt<T : Float>(f : T) -> Result<Self, NumCastErr> {
        Ok(Self::Number(Number::from_float(f)?))
    }

    fn get_err(&self, convert_type : &str) -> InvalidAsErr {
        InvalidAsErr::new(Any::Literal(self), convert_type.to_string())
    }

    pub fn as_string(&self) -> Result<&String, InvalidAsErr> {
        if let Literal::String(s) = self {
            Ok(s)
        } else {
            Err(self.get_err("String"))
        }
    }

    pub fn as_bool(&self) -> Result<&bool, InvalidAsErr> {
        if let Literal::Bool(b) = self {
            Ok(b)
        } else {
            Err(self.get_err("bool"))
        }
    }

    pub fn as_int(&self) -> Result<&i64, InvalidAsErr> {
        if let Literal::Number(num) = self {
            Ok(num.as_int()?)
        } else {
            Err(self.get_err("Number"))
        }
    }

    pub fn as_flt(&self) -> Result<&f64, InvalidAsErr> {
        if let Literal::Number(num) = self {
            Ok(num.as_flt()?)
        } else {
            Err(self.get_err("Number"))
        }
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
        formatter.write_str("a number, a string, or a bool")
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
}

#[derive(Debug, Clone)]
pub enum Value {
    Literal(Literal),
    Map(HashMap<String, Value>),
    Seq(Vec<Value>)
}


impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Literal(lit) => write!(f, "Literal({})", lit),
            Value::Map(map) => write!(f, "Map({:?})", map),
            Value::Seq(seq) => write!(f, "Seq({:?})", seq)
        }
    }
}


impl Value {
    pub fn from_string<S: Into<String>>(s : S) -> Self {
        Self::Literal(Literal::String(s.into()))
    }

    pub fn from_bool(b : bool) -> Self {
        Self::Literal(Literal::Bool(b))
    }

    pub fn from_int<T : Integer + ToPrimitive>(i : T) -> Result<Self, NumCastErr> {
        Ok(Self::Literal(Literal::Number(Number::from_int(i)?)))
    }

    pub fn from_flt<T : Float>(f : T) -> Result<Self, NumCastErr> {
        Ok(Self::Literal(Literal::Number(Number::from_float(f)?)))
    }

    pub fn from_map(map: HashMap<String, Value>) -> Self {
        Self::Map(map)
    }

    pub fn from_seq(seq: Vec<Value>) -> Self {
        Self::Seq(seq)
    }

    fn get_err(&self, convert_type : &str) -> InvalidAsErr {
        InvalidAsErr::new(Any::Value(&self), convert_type.to_string())
    }

    pub fn as_string(&self) -> Result<&String, InvalidAsErr> {
        if let Value::Literal(l) = self {
            Ok(l.as_string()?)
        } else {
            Err(self.get_err("Literal"))
        }
    }

    pub fn as_bool(&self) -> Result<&bool, InvalidAsErr> {
        if let Value::Literal(l) = self {
            Ok(l.as_bool()?)
        } else {
            Err(self.get_err("Literal"))
        }
    }

    pub fn as_int(&self) -> Result<&i64, InvalidAsErr> {
        if let Value::Literal(l) = self {
            Ok(l.as_int()?)
        } else {
            Err(self.get_err("Literal"))
        }
    }

    pub fn as_flt(&self) -> Result<&f64, InvalidAsErr> {
        if let Value::Literal(l) = self {
            Ok(l.as_flt()?)
        } else {
            Err(self.get_err("Literal"))
        }
    }

    pub fn as_map(&self) -> Result<&HashMap<String, Value>, InvalidAsErr> {
        if let Value::Map(m) = self {
            Ok(m)
        } else {
            Err(self.get_err("Map"))
        }
    }

    pub fn as_vec(&self) -> Result<&Vec<Value>, InvalidAsErr> {
        if let Value::Seq(s) = self {
            Ok(s)
        } else {
            Err(self.get_err("Vec"))
        }
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

struct ValueVisitor;

impl<'de> Visitor <'de> for ValueVisitor {
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
        Ok(Value::Map(m))
    }
}

#[test]
fn test_number() {
    let int = Number::Int(1);
    let flt = Number::Flt(1.75);
    // check parsing
    assert_eq!(int, from_str::<Number>("1").unwrap());
    assert_eq!(flt, from_str::<Number>("1.75").unwrap());

    // check 'as' functions
    assert_eq!(*int.as_int().unwrap(), 1i64);
    assert!(int.as_flt().is_err());
    assert_eq!(*flt.as_flt().unwrap(), 1.75f64);
    assert!(flt.as_int().is_err());
}

#[test]
fn test_literal() {
    let int = Literal::from_int(1).unwrap();
    let flt = Literal::from_flt(1.75).unwrap();
    let b = Literal::from_bool(true);
    let s = Literal::from_string("hello world");

    assert_eq!(b, from_str::<Literal>("true").unwrap());
    assert_eq!(s, from_str::<Literal>(r#"  "hello world"   "#).unwrap());
    assert_eq!(int, from_str::<Literal>("1").unwrap());
    assert_eq!(flt, from_str::<Literal>("1.75").unwrap());

    assert_eq!(*s.as_string().unwrap(), "hello world");
    assert_eq!(*b.as_bool().unwrap(), true);
}

