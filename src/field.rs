use std::borrow::Cow;

use nom::{
    Parser,
    bytes::complete::{tag, take_while},
    character::complete::char,
    combinator::{map, opt},
    error::Error,
    multi::many0,
    sequence::{delimited, preceded},
};

use crate::{
    annotation::{Annotation, parse_annotation},
    modifier::{Modifier, parse_modifiers},
    signature::type_signature::{TypeParameter, parse_type_parameter},
    ws,
};

/// Struct representing a Java field
///
#[derive(Debug, PartialEq, Clone)]
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
        delimited(
            ws(tag(".field")),
            (
                parse_modifiers(),
                parse_type_parameter(),
                opt(preceded(
                    ws(char('=')),
                    // TODO: This can be any type, needed fixes
                    take_while(|c| c != '\n').map(Cow::Borrowed),
                )),
                many0(parse_annotation()),
            ),
            opt(ws(tag(".end field"))),
        ),
        |(modifiers, param, i, annotations)| Field {
            modifiers,
            param,
            initial_value: i,
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
            .parse_complete(".field private final sentRequestAtMillis:J\n\n#aa")
            .unwrap();
        assert_eq!(f.param.ident, "sentRequestAtMillis".to_string());
        assert_eq!(f.modifiers.len(), 2);
        assert_eq!(f.param.ts.to_jni(), "J");

        let (_, f) = parse_field()
            .parse_complete(".field private final body:Lokhttp3/ResponseBody;")
            .unwrap();
        assert_eq!(f.param.ident, "body".to_string());
        assert_eq!(f.modifiers.len(), 2);
        assert_eq!(f.param.ts.to_jni(), "Lokhttp3/ResponseBody;");
    }
}
