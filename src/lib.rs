/* Struct to represent a java object type identifer e.g. java.lang.Object */
/* They are stored in the smali native (also JNI) format e.g. Ljava/lang/Object; */

use std::fmt::{self, Debug};

use winnow::{
    ModalParser, Parser,
    ascii::{multispace0, take_escaped},
    combinator::{alt, delimited, opt, preceded},
    error::InputError,
    token::{literal, none_of, one_of, take_while},
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

pub fn ws<'a, O, F>(inner: F) -> impl ModalParser<&'a str, O, InputError<&'a str>>
where
    F: ModalParser<&'a str, O, InputError<&'a str>>,
{
    delimited(
        multispace0,
        inner,
        (
            multispace0,
            opt((comment(), |i: &mut &'a str| {
                //println!("test1 {:?}", i.chars().take(50).collect::<String>());
                multispace0(i)
            })),
        ),
    )
}

pub fn comment<'a>() -> impl ModalParser<&'a str, &'a str, InputError<&'a str>> {
    preceded(one_of('#'), take_while(0.., |c| c != '\n'))
}

/// Parses a string literal that may be empty.
/// For example, it can parse `""` as well as `"builder"`.
pub fn parse_string_lit<'a>() -> impl ModalParser<&'a str, &'a str, InputError<&'a str>> {
    delimited(
        (multispace0, one_of('"')),
        alt((
            take_escaped(
                none_of(['\\', '\"']),
                '\\',
                one_of(['\'', '\"', 't', 'b', 'n', 'r', 'f', 'u', '\\']),
            ),
            literal(""),
        )),
        one_of('"'),
    )
}

pub fn parse_int_lit<'a, T>() -> impl ModalParser<&'a str, T, InputError<&'a str>>
where
    T: num_traits::Num + std::str::FromStr + TryFrom<i64>,
    <T as TryFrom<i64>>::Error: Debug,
{
    (
        opt(one_of('-')),
        alt((
            preceded(
                alt((literal("0x"), literal("0X"))),
                take_while(0.., |c: char| c.is_ascii_hexdigit()),
            )
            .map(|s| (16, s)),
            take_while(0.., |c: char| c.is_ascii_digit()).map(|s| (10, s)),
        )),
        opt(one_of('L')),
    )
        .try_map(|(sign, (base, digits), _)| match sign {
            Some(_) => T::from_str_radix(&format!("-{digits}"), base),
            None => T::from_str_radix(digits, base),
        })
}
