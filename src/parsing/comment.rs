use nom::{
    bytes::complete::{tag, take_until},
    combinator::value,
    error::ParseError,
    sequence::tuple,
    IResult,
};

pub fn multiline_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value((), tuple((tag("/*"), take_until("*/"), tag("*/"))))(i)
}

pub fn eol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value((), tuple((tag("//"), take_until("\n"), tag("\n"))))(i)
}
