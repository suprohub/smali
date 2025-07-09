use std::{borrow::Cow, str::FromStr};

use nom::{
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char},
    combinator::{map, opt, value},
    error::Error,
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, terminated},
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

pub fn parse_visibility<'a>()
-> impl Parser<&'a str, Output = AnnotationVisibility, Error = Error<&'a str>> {
    ws(alt((
        value(AnnotationVisibility::Build, tag("build")),
        value(AnnotationVisibility::Runtime, tag("runtime")),
        value(AnnotationVisibility::System, tag("system")),
    )))
}

/// Annotation values can be a Single value, Array, Enum or another Annotation
///
#[derive(Debug, PartialEq, Clone)]
pub enum AnnotationValue<'a> {
    Single(Cow<'a, str>),
    Array(Vec<AnnotationValue<'a>>),
    SubAnnotation(Annotation<'a>),
    Enum(FieldRef<'a>),
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

pub fn parse_annotation<'a>()
-> impl Parser<&'a str, Output = Annotation<'a>, Error = Error<&'a str>> {
    map(
        delimited(
            ws(alt((tag(".annotation"), tag(".subannotation")))),
            (
                opt(parse_visibility()),
                parse_typesignature(),
                many0(parse_annotation_element()),
            ),
            ws(alt((tag(".end annotation"), tag(".end subannotation")))),
        ),
        |(v, annotation_type, elements)| Annotation {
            visibility: v.unwrap_or(AnnotationVisibility::System),
            annotation_type,
            elements,
        },
    )
}

pub fn parse_annotation_element<'a>()
-> impl Parser<&'a str, Output = AnnotationElement<'a>, Error = Error<&'a str>> {
    map(
        (
            terminated(ws(alphanumeric1), ws(char('='))),
            parse_annotation_value(),
        ),
        |(name, value)| AnnotationElement {
            name: name.into(),
            value,
        },
    )
}

pub fn parse_annotation_value<'a>()
-> impl Parser<&'a str, Output = AnnotationValue<'a>, Error = Error<&'a str>> {
    alt((
        map(
            |input| parse_annotation().parse_complete(input),
            AnnotationValue::SubAnnotation,
        ),
        map(preceded(ws(tag(".enum")), parse_field_ref()), |f| {
            AnnotationValue::Enum(f)
        }),
        map(
            delimited(
                ws(char('{')),
                separated_list0(ws(char(',')), |input| {
                    parse_annotation_value().parse_complete(input)
                }),
                ws(char('}')),
            ),
            AnnotationValue::Array,
        ),
        map(parse_string_lit(), |s| AnnotationValue::Single(s.into())),
    ))
}

pub fn write_annotation(ann: &Annotation, subannotation: bool, indented: bool) -> String {
    let end_tag;
    let mut indent = "";
    let inset = "    ";
    if indented && subannotation {
        indent = "        ";
    } else if indented || subannotation {
        indent = "    ";
    }

    let mut out = if subannotation {
        end_tag = ".end subannotation";
        ".subannotation ".to_string()
    } else {
        end_tag = ".end annotation";
        format!("{}.annotation {} ", indent, ann.visibility.to_str())
    };
    out.push_str(&ann.annotation_type.to_jni());
    out.push('\n');

    for i in &ann.elements {
        out.push_str(&format!("{}{}{} = ", indent, inset, i.name));
        write_annotation_value(&mut out, &i.value, indented, indent, inset);
    }

    out.push_str(indent);
    out.push_str(end_tag);
    out.push('\n');

    out
}

pub fn write_annotation_value(out: &mut String, i: &AnnotationValue, indented: bool, indent: &str, inset: &str) {
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
        AnnotationValue::Single(s) => {
            out.push_str(&format!("{s}\n"));
        }
    }
}

mod tests {
    #[test]
    fn test_parse_annotation_element_array() {
        use super::*;
        use nom::Parser;
        let (_, a) = parse_annotation_element()
            .parse_complete(" key = {\n  \"a,\", \n \"b\\\"\", \"c\" \n }\n")
            .unwrap();
        assert_eq!(a.name, "key");
        match a.value {
            AnnotationValue::Array(a) => {
                assert_eq!(a[0], AnnotationValue::Single(Cow::Borrowed("a,")));
            }
            _ => {
                println!("{a:?}");
            }
        }

        let (_, a) = parse_annotation_element().parse_complete(" value = .enum Ljava/lang/annotation/RetentionPolicy;->SOURCE:Ljava/lang/annotation/RetentionPolicy;\n").unwrap();
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
}
