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
    annotation::{Annotation, parse_annotation, write_annotation},
    op::dex_op::{Register, parse_register},
    parse_string_lit, ws,
};

/// Struct representing a method parameter
#[derive(Debug, PartialEq)]
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
                parse_register(),
                opt(preceded(ws(char(',')), ws(parse_string_lit()))),
                opt(terminated(many0(parse_annotation()), ws(tag(".end param")))),
            ),
        ),
        |(register, n, a)| Param {
            register,
            name: n.map(|s| s.into()),
            annotations: a.unwrap_or_default(),
        },
    )
}

pub fn write_param(param: &Param) -> String {
    let mut out = String::new();
    out.push_str(".param ");
    out.push_str(&param.register.to_string());

    if let Some(name) = &param.name {
        out.push_str(", ");
        out.push('\"');
        out.push_str(name);
        out.push('\"');
    }

    if !param.annotations.is_empty() {
        out.push('\n');
        for ann in &param.annotations {
            out.push_str(&write_annotation(ann, false, true));
        }
        out.push_str(".end param");
    }

    out
}

mod tests {
    #[test]
    fn test_simple() {
        use super::*;
        use nom::Parser;
        let input = ".param p0";
        let (rem, _) = parse_param().parse_complete(input).unwrap();
        assert!(rem.is_empty());
    }

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
