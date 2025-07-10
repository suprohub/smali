use std::borrow::Cow;

use winnow::{
    ModalParser, Parser,
    combinator::{delimited, opt, preceded, repeat},
    error::InputError,
    token::{literal, one_of, take_while},
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

pub fn parse_field<'a>() -> impl ModalParser<&'a str, Field<'a>, InputError<&'a str>> {
    delimited(
        ws(literal(".field")),
        (
            parse_modifiers(),
            parse_type_parameter(),
            opt(preceded(
                ws(one_of('=')),
                // TODO: This can be any type, needed fixes
                take_while(0.., |c| c != '\n').map(Cow::Borrowed),
            )),
            repeat(0.., parse_annotation()),
        ),
        opt(ws(literal(".end field"))),
    )
    .map(|(modifiers, param, i, annotations)| Field {
        modifiers,
        param,
        initial_value: i,
        annotations,
    })
}

mod tests {
    #[test]
    fn test_parse_field() {
        use crate::field::parse_field;
        use winnow::Parser;
        let _f = parse_field()
            .parse_next(&mut ".field private volatile synthetic workerCtl$volatile:I")
            .unwrap();

        let f = parse_field()
            .parse_next(&mut ".field private final body:Lokhttp3/ResponseBody;")
            .unwrap();
        assert_eq!(f.param.ident, "body".to_string());
        assert_eq!(f.modifiers.len(), 2);
        assert_eq!(f.param.ts.to_jni(), "Lokhttp3/ResponseBody;");
    }
}
