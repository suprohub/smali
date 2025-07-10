use winnow::{
    ModalParser, Parser,
    combinator::{delimited, repeat},
    error::InputError,
    token::one_of,
};

use crate::signature::type_signature::{TypeSignature, parse_typesignature};

pub mod method_signature;
pub mod type_signature;

pub fn parse_type_parameters<'a>()
-> impl ModalParser<&'a str, Vec<TypeSignature<'a>>, InputError<&'a str>> {
    delimited(
        one_of('<'),
        repeat(0.., |input: &mut &'a str| {
            parse_typesignature().parse_next(input)
        }),
        one_of('>'),
    )
}
