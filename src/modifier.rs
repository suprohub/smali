use std::str::FromStr;

use winnow::{
    ModalParser, Parser,
    combinator::{alt, repeat},
    error::InputError,
    token::literal,
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

pub fn parse_modifiers<'a>() -> impl ModalParser<&'a str, Vec<Modifier>, InputError<&'a str>> {
    repeat(
        0..,
        ws(alt((
            literal("public").value(Modifier::Public),
            literal("protected").value(Modifier::Protected),
            literal("private").value(Modifier::Private),
            literal("static").value(Modifier::Static),
            literal("final").value(Modifier::Final),
            literal("abstract").value(Modifier::Abstract),
            literal("interface").value(Modifier::Interface),
            literal("synthetic").value(Modifier::Synthetic),
            literal("transient").value(Modifier::Transient),
            literal("volatile").value(Modifier::Volatile),
            literal("synchronized").value(Modifier::Synchronized),
            literal("native").value(Modifier::Native),
            literal("varargs").value(Modifier::Varargs),
            literal("annotation").value(Modifier::Annotation),
            literal("enum").value(Modifier::Enum),
            literal("strict").value(Modifier::Strict),
            literal("bridge").value(Modifier::Bridge),
            literal("constructor").value(Modifier::Constructor),
        ))),
    )
}

pub fn write_modifiers(mods: &Vec<Modifier>) -> String {
    let mut out = "".to_string();

    for m in mods {
        out.push_str(&format!("{} ", m.to_str()));
    }

    out
}
