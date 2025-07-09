use std::borrow::Cow;

use nom::{
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    error::Error,
    multi::many0,
    sequence::preceded,
};

use crate::{
    annotation::{Annotation, parse_annotation},
    modifier::{Modifier, parse_modifiers},
    parse_string_lit,
    signature::type_signature::{TypeParameter, parse_type_parameter},
    ws,
};

/// Struct representing a Java field
///
#[derive(Debug)]
pub struct Field<'a> {
    /// Any modifiers
    pub modifiers: Vec<Modifier>,
    /// Type signature of the field
    pub param: TypeParameter<'a>,
    /// If an initialiser is included
    pub initial_value: Option<Cow<'a, str>>,
    /// Field level annotations
    pub annotations: Vec<Annotation<'a>>,
}

pub fn parse_field<'a>() -> impl Parser<&'a str, Output = Field<'a>, Error = Error<&'a str>> {
    map(
        preceded(
            tag(".field"),
            (
                parse_modifiers(),
                parse_type_parameter(),
                opt(preceded(
                    ws(char('=')),
                    alt((parse_string_lit(), tag("null"))),
                )),
                many0(parse_annotation()),
            ),
        ),
        |(modifiers, param, i, annotations)| Field {
            modifiers,
            param,
            initial_value: i.map(|i| i.into()),
            annotations,
        },
    )
}

mod tests {
    #[test]
    fn test_parse_field() {
        use crate::field::parse_field;
        use nom::Parser;
        let (_, f) = parse_field()
            .parse_complete(".field private final callTimeoutMillis:I\n")
            .unwrap();
        assert_eq!(f.param.ident, "callTimeoutMillis".to_string());
        assert_eq!(f.modifiers.len(), 2);
        assert_eq!(f.param.ts.to_jni(), "I");
    }
}
