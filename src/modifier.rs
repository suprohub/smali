use std::str::FromStr;

use nom::{
    Parser, branch::alt, bytes::complete::tag, combinator::value, error::Error, multi::many0,
};

use crate::{SmaliError, ws};

/// Simple enum to represent Java method, field and class modifiers
///

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modifier {
    Public,
    Private,
    Protected,
    Static,
    Final,
    Synchronized,
    Volatile,
    Bridge,
    Transient,
    Varargs,
    Native,
    Interface,
    Abstract,
    Strict,
    Synthetic,
    Annotation,
    Enum,
    Constructor,
    DeclaredSynchronized,
}

impl FromStr for Modifier {
    type Err = SmaliError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "public" => Self::Public,
            "protected" => Self::Protected,
            "private" => Self::Private,
            "static" => Self::Static,
            "final" => Self::Final,
            "abstract" => Self::Abstract,
            "interface" => Self::Interface,
            "synthetic" => Self::Synthetic,
            "transient" => Self::Transient,
            "volatile" => Self::Volatile,
            "synchronized" => Self::Synchronized,
            "native" => Self::Native,
            "varargs" => Self::Varargs,
            "annotation" => Self::Annotation,
            "enum" => Self::Enum,
            "strict" => Self::Static,
            "bridge" => Self::Bridge,
            "constructor" => Self::Constructor,
            _ => {
                return Err(SmaliError {
                    details: "Unknown modifier".to_string(),
                });
            }
        })
    }
}

impl Modifier {
    pub fn to_str(&self) -> &str {
        match self {
            Self::Public => "public",
            Self::Protected => "protected",
            Self::Private => "private",
            Self::Static => "static",
            Self::Final => "final",
            Self::Abstract => "abstract",
            Self::Interface => "interface",
            Self::Synthetic => "synthetic",
            Self::Transient => "transient",
            Self::Volatile => "volatile",
            Self::Synchronized => "synchronized",
            Self::Native => "native",
            Self::Varargs => "varargs",
            Self::Annotation => "annotation",
            Self::Enum => "enum",
            Self::Strict => "strict",
            Self::Bridge => "bridge",
            Self::Constructor => "constructor",
            Self::DeclaredSynchronized => "synchronized",
        }
    }
}

pub fn parse_modifiers<'a>() -> impl Parser<&'a str, Output = Vec<Modifier>, Error = Error<&'a str>>
{
    many0(ws(alt((
        value(Modifier::Public, tag("public")),
        value(Modifier::Protected, tag("protected")),
        value(Modifier::Private, tag("private")),
        value(Modifier::Static, tag("static")),
        value(Modifier::Final, tag("final")),
        value(Modifier::Abstract, tag("abstract")),
        value(Modifier::Interface, tag("interface")),
        value(Modifier::Synthetic, tag("synthetic")),
        value(Modifier::Transient, tag("transient")),
        value(Modifier::Volatile, tag("volatile")),
        value(Modifier::Synchronized, tag("synchronized")),
        value(Modifier::Native, tag("native")),
        value(Modifier::Varargs, tag("varargs")),
        value(Modifier::Annotation, tag("annotation")),
        value(Modifier::Enum, tag("enum")),
        value(Modifier::Strict, tag("strict")),
        value(Modifier::Bridge, tag("bridge")),
        value(Modifier::Constructor, tag("constructor")),
    ))))
}

pub fn write_modifiers(mods: &Vec<Modifier>) -> String {
    let mut out = "".to_string();

    for m in mods {
        out.push_str(&format!("{} ", m.to_str()));
    }

    out
}
