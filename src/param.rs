use std::borrow::Cow;

use nom::{
    Parser,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    error::Error,
    multi::many0,
    sequence::{preceded, terminated},
};

use crate::{
    annotation::{Annotation, parse_annotation},
    comment,
    op::dex_op::{Register, parse_register},
    parse_string_lit, ws,
};

/// Struct representing a method parameter
#[derive(Debug)]
pub struct Param<'a> {
    /// Register used for the parameter
    pub register: Register,
    /// Parameter name
    pub name: Option<Cow<'a, str>>,
    /// Parameter annotations
    pub annotations: Vec<Annotation<'a>>,
}

pub fn parse_param<'a>() -> impl Parser<&'a str, Output = Param<'a>, Error = Error<&'a str>> {
    map(
        preceded(
            ws(tag(".param")),
            (
                ws(parse_register()),
                terminated(
                    opt(preceded(ws(char(',')), parse_string_lit())),
                    opt(comment()),
                ),
                opt(terminated(many0(parse_annotation()), tag(".end param"))),
            ),
        ),
        |(register, n, a)| Param {
            register,
            name: n.map(|s| s.into()),
            annotations: a.unwrap_or_default(),
        },
    )
}

mod tests {

    #[test]
    fn test_parse_param_block_with_annotation() {
        use super::*;
        use nom::Parser;
        let input = r#".param p0    # Lkotlin/reflect/KProperty0;
            .annotation build Lkotlin/internal/AccessibleLateinitPropertyLiteral;
            .end annotation
        .end param"#;
        let (rem, _) = parse_param().parse_complete(input).unwrap();
        assert!(rem.is_empty());
    }

    #[test]
    fn test_parse_param_with_string() {
        use super::*;
        use nom::Parser;
        let input = r#".param p0, "_this"    # Landroidx/core/internal/view/SupportMenuItem;"#;
        let (_rem, param) = parse_param().parse_complete(input).unwrap();
        assert_eq!(param.register, Register::Parameter(0));
        assert_eq!(param.name, Some("_this".into()));
        assert!(param.annotations.is_empty());
    }
}
