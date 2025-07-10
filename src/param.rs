use std::borrow::Cow;

use winnow::{
    ModalParser, Parser,
    combinator::{opt, preceded, repeat, terminated},
    error::InputError,
    token::{literal, one_of},
};

use crate::{
    annotation::{Annotation, parse_annotation, write_annotation},
    op::dex_op::{Register, parse_register},
    parse_string_lit, ws,
};

/// Struct representing a method parameter
#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    /// Register used for the parameter
    pub register: Register,
    /// Parameter name
    pub name: Option<Cow<'a, str>>,
    /// Parameter annotations
    pub annotations: Vec<Annotation<'a>>,
}

pub fn parse_param<'a>() -> impl ModalParser<&'a str, Param<'a>, InputError<&'a str>> {
    preceded(
        ws(literal(".param")),
        (
            parse_register(),
            opt(preceded(ws(one_of(',')), ws(parse_string_lit()))),
            opt(terminated(
                repeat(0.., parse_annotation()),
                ws(literal(".end param")),
            )),
        ),
    )
    .map(|(register, n, a)| Param {
        register,
        name: n.map(|s| s.into()),
        annotations: a.unwrap_or_default(),
    })
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
        use winnow::Parser;
        let mut input = ".param p0";
        let _ = parse_param().parse_next(&mut input).unwrap();
    }

    #[test]
    fn test_parse_param_block_with_annotation() {
        use super::*;
        use winnow::Parser;
        let mut input = r#".param p0    # Lkotlin/reflect/KProperty0;
            .annotation build Lkotlin/internal/AccessibleLateinitPropertyLiteral;
            .end annotation
        .end param"#;
        let _ = parse_param().parse_next(&mut input).unwrap();
    }

    #[test]
    fn test_parse_param_with_string() {
        use super::*;
        use winnow::Parser;
        let mut input = r#".param p0, "_this"    # Landroidx/core/internal/view/SupportMenuItem;"#;
        let param = parse_param().parse_next(&mut input).unwrap();
        assert_eq!(param.register, Register::Parameter(0));
        assert_eq!(param.name, Some("_this".into()));
        assert!(param.annotations.is_empty());
    }
}
