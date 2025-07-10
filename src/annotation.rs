use std::{borrow::Cow, str::FromStr};

use winnow::{
    ModalParser, Parser,
    ascii::alphanumeric1,
    combinator::{alt, delimited, opt, preceded, repeat, separated, terminated},
    error::InputError,
    token::{literal, one_of, take_till},
};

use crate::{
    SmaliError,
    field_ref::{FieldRef, parse_field_ref},
    parse_string_lit,
    signature::type_signature::{TypeSignature, parse_typesignature},
    ws,
};

/// Simple enum to represent annotation visibility: build, runtime, system.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnnotationVisibility {
    Build,
    Runtime,
    System,
}

impl AnnotationVisibility {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Build => "build",
            Self::Runtime => "runtime",
            Self::System => "system",
        }
    }
}

pub fn parse_visibility<'a>() -> impl ModalParser<&'a str, AnnotationVisibility, InputError<&'a str>>
{
    ws(alt((
        literal("build").value(AnnotationVisibility::Build),
        literal("runtime").value(AnnotationVisibility::Runtime),
        literal("system").value(AnnotationVisibility::System),
    )))
}

/// Annotation values can be a Single value, Array, Enum or another Annotation
///
#[derive(Debug, PartialEq, Clone)]
pub enum AnnotationValue<'a> {
    String(Cow<'a, str>),
    Array(Vec<AnnotationValue<'a>>),
    SubAnnotation(Annotation<'a>),
    Enum(FieldRef<'a>),

    Any(Cow<'a, str>),
}

impl FromStr for AnnotationVisibility {
    type Err = SmaliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "build" => Self::Build,
            "runtime" => Self::Runtime,
            "system" => Self::System,
            _ => {
                return Err(SmaliError {
                    details: "Unknown Annotation visibility".to_string(),
                });
            }
        })
    }
}

/// Name, value pair for annotation elements. There can be several of these per annotation.
///
#[derive(Debug, PartialEq, Clone)]
pub struct AnnotationElement<'a> {
    pub name: Cow<'a, str>,
    pub value: AnnotationValue<'a>,
}

/// Struct representing a Java annotation, these can occur at class level, method level, within a field or within another annotation.
///
#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'a> {
    pub visibility: AnnotationVisibility,
    pub annotation_type: TypeSignature<'a>,
    pub elements: Vec<AnnotationElement<'a>>,
}

pub fn parse_annotation<'a>() -> impl ModalParser<&'a str, Annotation<'a>, InputError<&'a str>> {
    delimited(
        ws(alt((literal(".annotation"), literal(".subannotation")))),
        (
            opt(parse_visibility()),
            parse_typesignature(),
            repeat(0.., parse_annotation_element()),
        ),
        ws(alt((
            literal(".end annotation"),
            literal(".end subannotation"),
        ))),
    )
    .map(|(v, annotation_type, elements)| Annotation {
        visibility: v.unwrap_or(AnnotationVisibility::System),
        annotation_type,
        elements,
    })
}

pub fn parse_annotation_element<'a>()
-> impl ModalParser<&'a str, AnnotationElement<'a>, InputError<&'a str>> {
    (
        terminated(ws(alphanumeric1), ws(one_of('='))),
        parse_annotation_value(),
    )
        .map(|(name, value)| AnnotationElement {
            name: name.into(),
            value,
        })
}

pub fn parse_annotation_value<'a>()
-> impl ModalParser<&'a str, AnnotationValue<'a>, InputError<&'a str>> {
    alt((
        (|input: &mut &'a str| parse_annotation().parse_next(input))
            .map(AnnotationValue::SubAnnotation),
        delimited(
            ws(one_of('{')),
            separated(
                0..,
                |input: &mut &'a str| parse_annotation_value().parse_next(input),
                ws(one_of(',')),
            ),
            ws(one_of('}')),
        )
        .map(AnnotationValue::Array),
        parse_string_lit().map(|s: &'a str| AnnotationValue::String(s.into())),
        preceded(ws(literal(".enum")), parse_field_ref()).map(AnnotationValue::Enum),
        // TODO: This can be any type, needed fixes
        take_till(0.., |c| c == ',' || c == '}' || c == '\n')
            .map(|s: &'a str| AnnotationValue::Any(s.into())),
    ))
}

pub fn write_annotation(ann: &Annotation, subannotation: bool, indented: bool) -> String {
    let end_literal;
    let mut indent = "";
    let inset = "    ";
    if indented && subannotation {
        indent = "        ";
    } else if indented || subannotation {
        indent = "    ";
    }

    let mut out = if subannotation {
        end_literal = ".end subannotation";
        ".subannotation ".to_string()
    } else {
        end_literal = ".end annotation";
        format!("{}.annotation {} ", indent, ann.visibility.to_str())
    };
    out.push_str(&ann.annotation_type.to_jni());
    out.push('\n');

    for i in &ann.elements {
        out.push_str(&format!("{}{}{} = ", indent, inset, i.name));
        write_annotation_value(&mut out, &i.value, indented, indent, inset);
    }

    out.push_str(indent);
    out.push_str(end_literal);
    out.push('\n');

    out
}

pub fn write_annotation_value(
    out: &mut String,
    i: &AnnotationValue,
    indented: bool,
    indent: &str,
    inset: &str,
) {
    match &i {
        AnnotationValue::Array(a) => {
            out.push_str("{\n");
            let mut c = 0;
            for v in a {
                out.push_str(indent);
                out.push_str(inset);
                out.push_str(inset);
                write_annotation_value(out, v, indented, indent, inset);
                c += 1;
                if c < a.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&format!("{indent}{inset}}}\n"));
        }
        AnnotationValue::SubAnnotation(s) => {
            out.push_str(&write_annotation(s, true, indented));
        }
        AnnotationValue::Enum(f) => {
            out.push_str(&format!(
                ".enum {}->{}:{}\n",
                f.class.as_jni_type(),
                f.param.ident,
                f.param.ts
            ));
        }
        AnnotationValue::String(s) => {
            out.push_str(&format!("\"{s}\"\n"));
        }
        AnnotationValue::Any(s) => {
            out.push_str(&format!("{s}\n"));
        }
    }
}

mod tests {
    #[test]
    fn test_parse_annotation_element_array() {
        use super::*;
        use winnow::Parser;
        let a = parse_annotation_element()
            .parse_next(&mut " key = {\n  \"a,\", \n \"b\\\"\", \"c\" \n }\n")
            .unwrap();
        assert_eq!(a.name, "key");
        match a.value {
            AnnotationValue::Array(a) => {
                assert_eq!(a[0], AnnotationValue::String(Cow::Borrowed("a,")));
            }
            _ => {
                println!("{a:?}");
            }
        }

        let a = parse_annotation_element().parse_next(&mut " value = .enum Ljava/lang/annotation/RetentionPolicy;->SOURCE:Ljava/lang/annotation/RetentionPolicy;\n").unwrap();
        match a.value {
            AnnotationValue::Enum(f) => {
                assert_eq!(f.param.ident, "SOURCE");
                assert_eq!(
                    f.class.as_jni_type(),
                    "Ljava/lang/annotation/RetentionPolicy;"
                );
            }
            _ => {
                println!("{a:?}");
            }
        }
    }

    #[test]
    fn annotation_test1() {
        use super::*;

        let input = ".annotation system Ldalvik/annotation/MemberClasses;
    value = {
        Lokhttp3/OkHttpClient$Builder;,
        Lokhttp3/OkHttpClient$Companion;
    }
.end annotation";
        println!("{:?}", parse_annotation().parse(input).unwrap());
    }
}
