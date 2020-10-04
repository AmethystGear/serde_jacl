pub mod comment;
pub mod literal;
pub mod string;

use nom::{
    character::complete::one_of,
    combinator::{recognize, value},
    multi::{many0, many1},
    sequence::pair,
    IResult,
};

const ALPHA: &str = "qwertyuiopasdfghjklzxcvbnm_";
const ALPHANUM: &str = "qwertyuiopasdfghjklzxcvbnm_1234567890";

pub fn delimiter<'a>(input: &'a str) -> IResult<&'a str, char> {
    return one_of(":(){}[]")(input);
}

pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    recognize(pair(many1(one_of(ALPHA)), many0(one_of(ALPHANUM))))(input)
}

pub fn whitespace<'a>(input: &'a str) -> IResult<&'a str, ()> {
    return value((), many1(one_of(" ,\r\n\t")))(input);
}
