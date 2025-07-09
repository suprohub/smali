/* Struct to represent a java object type identifer e.g. java.lang.Object */
/* They are stored in the smali native (also JNI) format e.g. Ljava/lang/Object; */

use std::fmt::{self, Debug};

use nom::{
    Parser,
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_while1},
    character::complete::{char, multispace0, none_of, one_of},
    combinator::{map, map_res, opt},
    error::Error,
    sequence::{delimited, preceded},
};

pub mod annotation;
pub mod class;
pub mod field;
pub mod field_ref;
pub mod method;
pub mod method_ref;
pub mod modifier;
pub mod object_identifier;
pub mod op;
pub mod param;
pub mod signature;

/* Custom error for our command helper */
#[derive(Debug)]
pub struct SmaliError {
    pub details: String,
}

impl SmaliError {
    pub fn new(msg: &str) -> SmaliError {
        SmaliError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for SmaliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

pub fn ws<'a, O, F>(inner: F) -> impl Parser<&'a str, Output = O, Error = Error<&'a str>>
where
    F: Parser<&'a str, Output = O, Error = Error<&'a str>>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn comment<'a>() -> impl Parser<&'a str, Output = &'a str, Error = Error<&'a str>> {
    preceded(char('#'), is_not("\n\r"))
}

/// Parses a string literal that may be empty.
/// For example, it can parse `""` as well as `"builder"`.
pub fn parse_string_lit<'a>() -> impl Parser<&'a str, Output = &'a str, Error = Error<&'a str>> {
    delimited(
        (multispace0, char('"')),
        alt((
            escaped(none_of("\\\""), '\\', one_of("'\"tbnrfu\\")),
            tag(""),
        )),
        char('"'),
    )
}

pub(crate) fn parse_int_lit<'a, T>() -> impl Parser<&'a str, Output = T, Error = Error<&'a str>>
where
    T: num_traits::Num + std::str::FromStr + TryFrom<i64>,
    <T as TryFrom<i64>>::Error: Debug,
{
    map_res(
        (
            opt(char::<&str, Error<&str>>('-')),
            alt((
                map(
                    preceded(
                        alt((tag("0x"), tag("0X"))),
                        take_while1(|c: char| c.is_ascii_hexdigit()),
                    ),
                    |s| (16, s),
                ),
                map(take_while1(|c: char| c.is_ascii_digit()), |s| (10, s)),
            )),
            opt(char('L')),
        ),
        |(sign, (base, digits), _)| match sign {
            Some(_) => T::from_str_radix(&format!("-{digits}"), base),
            None => T::from_str_radix(digits, base),
        },
    )
}
