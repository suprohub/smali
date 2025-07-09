use nom::{Parser, character::char, error::Error, multi::many0, sequence::delimited};

use crate::signature::type_signature::{TypeSignature, parse_typesignature};

pub mod method_signature;
pub mod type_signature;

pub fn parse_type_parameters<'a>()
-> impl Parser<&'a str, Output = Vec<TypeSignature<'a>>, Error = Error<&'a str>> {
    delimited(
        char('<'),
        many0(|input| parse_typesignature().parse_complete(input)),
        char('>'),
    )
}
