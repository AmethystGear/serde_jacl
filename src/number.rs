use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

#[cfg(test)]
use crate::de::from_str;

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64),
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

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(i64::from(value)))
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(i64::from(value)))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(i64::from(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(value))
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(i64::from(value)))
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(i64::from(value)))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Integer(i64::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value <= i64::MAX as u64 {
            Ok(Number::Integer(value as i64))
        } else {
            Err(E::custom(format!("i64 out of range: {}", value)))
        }
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Float(v as f64))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Number::Float(v))
    }
}

#[test]
fn test_number() {
    assert_eq!(Number::Integer(1), from_str::<Number>("1").unwrap());
    assert_eq!(Number::Float(1.75), from_str::<Number>("1.75").unwrap());
}
