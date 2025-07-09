use std::borrow::Cow;

use nom::{
    Parser,
    bytes::complete::take_until,
    character::complete::char,
    combinator::{map, opt},
    error::Error,
    multi::many0,
    sequence::{delimited, preceded},
};
use serde::{Deserialize, Serialize};

use crate::signature::{
    parse_type_parameters,
    type_signature::{TypeSignature, parse_typesignature},
};

/// Represents a Java method signature consisting of arguments and a return type
///
/// # Examples
///
/// ```
///  use smali::types::{MethodSignature, TypeSignature};
///
///  let m = MethodSignature::from_jni("([I)V");
///  assert_eq!(m.result, TypeSignature::Void);
/// ```
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct MethodSignature<'a> {
    pub(crate) type_parameters: Option<Vec<TypeSignature<'a>>>,
    pub args: Vec<TypeSignature<'a>>,
    pub result: TypeSignature<'a>,
    pub throws: Option<TypeSignature<'a>>,
}

impl MethodSignature<'_> {
    pub fn from_jni(s: &str) -> MethodSignature {
        let (_, m) = parse_methodsignature()
            .parse_complete(s)
            .expect("Can't parse MethodSignature");
        m
    }

    pub fn to_jni(&self) -> String {
        let mut s = String::new();
        if let Some(v) = &self.type_parameters {
            s.push('<');
            for t in v {
                s.push_str(&t.to_jni());
            }
            s.push('>');
        }
        s.push('(');
        for t in &self.args {
            let ts = t.to_jni();
            s.push_str(&ts);
        }
        s.push(')');
        s.push_str(&self.result.to_jni());
        if let Some(t) = &self.throws {
            s.push('^');
            s.push_str(&t.to_jni());
        }
        s
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct MethodParameter<'a> {
    pub ident: Cow<'a, str>,
    pub ms: MethodSignature<'a>,
}

pub fn parse_method_parameter<'a>()
-> impl Parser<&'a str, Output = MethodParameter<'a>, Error = Error<&'a str>> {
    map((take_until("("), parse_methodsignature()), |(ident, ms)| {
        MethodParameter {
            ident: ident.into(),
            ms,
        }
    })
}

fn parse_arguments<'a>()
-> impl Parser<&'a str, Output = Vec<TypeSignature<'a>>, Error = Error<&'a str>> {
    delimited(char('('), many0(parse_typesignature()), char(')'))
}

pub(crate) fn parse_methodsignature<'a>()
-> impl Parser<&'a str, Output = MethodSignature<'a>, Error = Error<&'a str>> {
    map(
        (
            opt(parse_type_parameters()),
            parse_arguments(),
            parse_typesignature(),
            opt(preceded(char('^'), parse_typesignature())),
        ),
        |(type_parameters, args, result, throws)| MethodSignature {
            type_parameters,
            args,
            result,
            throws,
        },
    )
}

#[cfg(test)]
mod tests {
    use nom::Parser;

    use crate::signature::method_signature::{MethodSignature, parse_methodsignature};

    #[test]
    fn test_methodsignature() {
        let (_, t) = parse_methodsignature().parse_complete("([B)V").unwrap();
        println!("{t:?}");
    }

    #[test]
    fn test_method_signature1() {
        let ts = "(TTSource;TTAccumulate;Lcom/strobel/core/Accumulator<TTSource;TTAccumulate;>;Lcom/strobel/core/Selector<TTAccumulate;TTResult;>;)TTResult;";
        let m = MethodSignature::from_jni(ts);
        println!("{m:?}");
        assert_eq!(m.to_jni(), ts);
    }

    #[test]
    fn test_method_signature2() {
        let ts = "<R2:Ljava/lang/Object;>(Lcom/strobel/core/Selector<-TR;+TR2;>;)Ljava/lang/Iterable<TR2;>;^Ljava/lang/Exception;";
        let m = MethodSignature::from_jni(ts);
        println!("{m:?}");
        assert_eq!(m.to_jni(), ts);
    }

    #[test]
    fn test_method_signature3() {
        let ts = "<U:TT;>(TU;)I";
        let m = MethodSignature::from_jni(ts);
        println!("{m:?}");
        assert_eq!(m.to_jni(), ts);
    }

    #[test]
    fn test_method_signature4() {
        let ts = "<R2:Ljava/lang/Object;>(Lcom/strobel/core/Selector<-TR;+TR2;>;)Ljava/lang/Iterable<TR2;>;";
        let m = MethodSignature::from_jni(ts);
        println!("{m:?}");
        assert_eq!(m.to_jni(), ts);
    }

    #[test]
    fn test_method_signature5() {
        let ts = "<T:Landroidx/lifecycle/ViewModel;>(Ljava/lang/Class<TT;>;)TT;";
        let m = MethodSignature::from_jni(ts);
        println!("{m:?}");
        assert_eq!(m.to_jni(), ts);
    }
}
