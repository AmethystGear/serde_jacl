use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete,
    character::complete::one_of,
    combinator::{map_res, opt, recognize, value},
    multi::{many0, many1},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use num::{Float, Integer};
use std::str::FromStr;

pub fn integer<'a, T: Integer + FromStr>(input: &'a str) -> IResult<&'a str, T> {
    map_res(
        recognize(pair(
            opt(complete::char('-')),
            many1(terminated(one_of("0123456789"), many0(complete::char('_')))),
        )),
        |out: &str| T::from_str(&str::replace(&out, "_", "")),
    )(input)
}

pub fn float<'a, T: Float + FromStr>(input: &'a str) -> IResult<&'a str, T> {
    map_res(
        alt((
            // Case one: .42
            recognize(tuple((
                complete::char('.'),
                integer::<i64>,
                opt(tuple((one_of("eE"), opt(one_of("+-")), integer::<i64>))),
            ))), // Case two: 42e42 and 42.42e42
            recognize(tuple((
                integer::<i64>,
                opt(preceded(complete::char('.'), integer::<i64>)),
                one_of("eE"),
                opt(one_of("+-")),
                integer::<i64>,
            ))), // Case three: 42. and 42.42
            recognize(tuple((
                integer::<i64>,
                complete::char('.'),
                opt(integer::<i64>),
            ))), // Case four: 42
            recognize(integer::<i64>)
        )),
        |out: &str| T::from_str(out),
    )(input)
}

pub fn boolean<'a>(input: &'a str) -> IResult<&'a str, bool> {
    alt((value(false, tag("false")), value(true, tag("true"))))(input)
}

pub fn null<'a>(input: &'a str) -> IResult<&'a str, ()> {
    value((), tag("null"))(input)
}
