pub mod comment;
pub mod literal;
pub mod string;

use nom::{
    character::complete::{alpha1, alphanumeric0, one_of},
    combinator::{recognize, value},
    sequence::pair,
    IResult, multi::many1,
};

pub fn delimiter<'a>(input: &'a str) -> IResult<&'a str, char> {
    return one_of(":(){}[]")(input);
}

pub fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    recognize(pair(alpha1, alphanumeric0))(input)
}

pub fn whitespace<'a>(input: &'a str) -> IResult<&'a str, ()> {
    return value((), many1(one_of(" ,\r\n\t")))(input);
}
