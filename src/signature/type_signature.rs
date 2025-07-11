use std::{borrow::Cow, fmt};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalParser, Parser,
    combinator::{alt, delimited, preceded, terminated},
    error::InputError,
    token::{one_of, take_while},
};

use crate::{
    object_identifier::{ObjectIdentifier, parse_object_identifier},
    signature::parse_type_parameters,
    ws,
};

/// Represents a Java type: array, object or primitive type
///
/// # Examples
///
/// ```
///  use smali::types::TypeSignature;
///
///  let t = TypeSignature::Bool;
///  assert_eq!(t.to_jni(), "Z");
/// ```
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum TypeSignature<'a> {
    Array(Box<TypeSignature<'a>>),
    Object(Box<ObjectIdentifier<'a>>),
    Int,
    Bool,
    Byte,
    Char,
    Short,
    Long,
    Float,
    Double,
    Void,
    TypeParameters(Vec<TypeSignature<'a>>, Box<TypeSignature<'a>>),
    TypeParameter(Box<TypeParameter<'a>>),
    TypeVariableSignature(Cow<'a, str>),
    WildcardPlus,
    WildcardMinus,
    WildcardStar,
}

impl fmt::Display for TypeSignature<'_> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.to_jni())
    }
}

impl TypeSignature<'_> {
    pub fn from_jni(mut s: &str) -> TypeSignature {
        parse_typesignature()
            .parse_next(&mut s)
            .unwrap_or_else(|_| panic!("Could not parse TypeSignature: {s}"))
    }

    pub fn to_jni(&self) -> String {
        match self {
            TypeSignature::Array(a) => "[".to_string() + &a.to_jni(),
            TypeSignature::Bool => "Z".to_string(),
            TypeSignature::Byte => "B".to_string(),
            TypeSignature::Char => "C".to_string(),
            TypeSignature::Short => "S".to_string(),
            TypeSignature::Int => "I".to_string(),
            TypeSignature::Long => "J".to_string(),
            TypeSignature::Float => "F".to_string(),
            TypeSignature::Double => "D".to_string(),
            TypeSignature::Object(o) => o.as_jni_type(),
            TypeSignature::Void => "V".to_string(),
            TypeSignature::TypeVariableSignature(i) => format!("T{i};"),
            TypeSignature::TypeParameters(params, rest) => {
                let mut s = "<".to_string();
                for p in params {
                    s.push_str(&p.to_jni())
                }
                s.push('>');
                s.push_str(&rest.to_jni());
                s
            }
            TypeSignature::TypeParameter(t) => {
                format!("{}:{}", t.ident, t.ts)
            }
            TypeSignature::WildcardPlus => "+".to_string(),
            TypeSignature::WildcardMinus => "-".to_string(),
            TypeSignature::WildcardStar => "*".to_string(),
        }
    }

    pub fn to_java(&self) -> String {
        match self {
            TypeSignature::Array(a) => format!("{}[]", a.to_java()),
            TypeSignature::Bool => "boolean".to_string(),
            TypeSignature::Byte => "byte".to_string(),
            TypeSignature::Char => "char".to_string(),
            TypeSignature::Short => "short".to_string(),
            TypeSignature::Int => "int".to_string(),
            TypeSignature::Long => "long".to_string(),
            TypeSignature::Float => "float".to_string(),
            TypeSignature::Double => "double".to_string(),
            TypeSignature::Object(o) => o.as_java_type(),
            TypeSignature::Void => "void".to_string(),
            _ => "".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TypeParameter<'a> {
    pub ident: Cow<'a, str>,
    pub ts: TypeSignature<'a>,
}

pub fn parse_type_parameter<'a>()
-> impl ModalParser<&'a str, TypeParameter<'a>, InputError<&'a str>> {
    (
        terminated(
            take_while(0.., |c: char| {
                c.is_alphanumeric() || ['_', '$', '-'].contains(&c)
            }),
            one_of(':'),
        ),
        |input: &mut &'a str| {
            //println!("test2");
            parse_typesignature().parse_next(input)
        },
    )
        .map(|(ident, ts)| TypeParameter {
            ident: ident.into(),
            ts,
        })
}

// Its needed to be recursive, sadly ;(
pub fn parse_typesignature<'a>() -> impl ModalParser<&'a str, TypeSignature<'a>, InputError<&'a str>>
{
    ws(alt((
        alt((
            one_of('Z').value(TypeSignature::Bool),
            one_of('B').value(TypeSignature::Byte),
            one_of('C').value(TypeSignature::Char),
            one_of('S').value(TypeSignature::Short),
            one_of('I').value(TypeSignature::Int),
            one_of('J').value(TypeSignature::Long),
            one_of('F').value(TypeSignature::Float),
            one_of('D').value(TypeSignature::Double),
            one_of('V').value(TypeSignature::Void),
            one_of('*').value(TypeSignature::WildcardStar),
            one_of('+').value(TypeSignature::WildcardPlus),
            one_of('-').value(TypeSignature::WildcardMinus),
        )),
        (parse_type_parameters(), |input: &mut &'a str| {
            parse_typesignature().parse_next(input)
        })
            .map(|(ts, ts_rest)| TypeSignature::TypeParameters(ts, Box::new(ts_rest))),
        parse_object_identifier().map(|o| TypeSignature::Object(Box::new(o))),
        delimited(one_of('T'), take_while(0.., |x| x != ';'), one_of(';'))
            .map(|name: &str| TypeSignature::TypeVariableSignature(Cow::Borrowed(name))),
        preceded(one_of('['), |input: &mut &'a str| {
            parse_typesignature().parse_next(input)
        })
        .map(|arr| TypeSignature::Array(Box::new(arr))),
        parse_type_parameter().map(|t| TypeSignature::TypeParameter(Box::new(t))),
    )))
}

mod tests {

    #[allow(unused_imports)]
    use crate::signature::TypeSignature;

    #[test]
    fn test_typesignature() {
        use super::*;
        use winnow::Parser;
        let t = parse_typesignature().parse_next(&mut "[B").unwrap();
        println!("{t:?}");
        let t = parse_typesignature().parse_next(&mut "V").unwrap();
        println!("{t:?}");
        let t = parse_typesignature()
            .parse_next(&mut "Lcom/none/Class;")
            .unwrap();
        println!("{t:?}");
    }

    #[test]
    fn test_signature() {
        let ts = "Ljava/util/HashMap<Ljava/lang/Class<+Lorg/antlr/v4/runtime/atn/Transition;>;Ljava/lang/Integer;>;";
        let o = TypeSignature::from_jni(ts);
        assert_eq!(o.to_jni(), ts);
    }

    #[test]
    fn test_signature2() {
        let ts = "Lorg/jf/dexlib2/writer/DexWriter<Lorg/jf/dexlib2/writer/builder/BuilderStringReference;Lorg/jf/dexlib2/writer/builder/BuilderStringReference;Lorg/jf/dexlib2/writer/builder/BuilderTypeReference;Lorg/jf/dexlib2/writer/builder/BuilderTypeReference;Lorg/jf/dexlib2/writer/builder/BuilderMethodProtoReference;Lorg/jf/dexlib2/writer/builder/BuilderFieldReference;Lorg/jf/dexlib2/writer/builder/BuilderMethodReference;Lorg/jf/dexlib2/writer/builder/BuilderClassDef;Lorg/jf/dexlib2/writer/builder/BuilderCallSiteReference;Lorg/jf/dexlib2/writer/builder/BuilderMethodHandleReference;Lorg/jf/dexlib2/writer/builder/BuilderAnnotation;Lorg/jf/dexlib2/writer/builder/BuilderAnnotationSet;Lorg/jf/dexlib2/writer/builder/BuilderTypeList;Lorg/jf/dexlib2/writer/builder/BuilderField;Lorg/jf/dexlib2/writer/builder/BuilderMethod;Lorg/jf/dexlib2/writer/builder/BuilderEncodedValues$BuilderArrayEncodedValue;Lorg/jf/dexlib2/writer/builder/BuilderEncodedValues$BuilderEncodedValue;Lorg/jf/dexlib2/writer/builder/BuilderAnnotationElement;Lorg/jf/dexlib2/writer/builder/BuilderStringPool;Lorg/jf/dexlib2/writer/builder/BuilderTypePool;Lorg/jf/dexlib2/writer/builder/BuilderProtoPool;Lorg/jf/dexlib2/writer/builder/BuilderFieldPool;Lorg/jf/dexlib2/writer/builder/BuilderMethodPool;Lorg/jf/dexlib2/writer/builder/BuilderClassPool;Lorg/jf/dexlib2/writer/builder/BuilderCallSitePool;Lorg/jf/dexlib2/writer/builder/BuilderMethodHandlePool;Lorg/jf/dexlib2/writer/builder/BuilderTypeListPool;Lorg/jf/dexlib2/writer/builder/BuilderAnnotationPool;Lorg/jf/dexlib2/writer/builder/BuilderAnnotationSetPool;Lorg/jf/dexlib2/writer/builder/BuilderEncodedArrayPool;>.SectionProvider;";
        let o = TypeSignature::from_jni(ts);
        println!("{o:?}");
        assert_eq!(o.to_jni(), ts);
    }

    #[test]
    fn test_signature3() {
        let ts = "<TSource:Ljava/lang/Object;TAccumulate:Ljava/lang/Object;TResult:Ljava/lang/Object;>Ljava/lang/Object;";
        let o = TypeSignature::from_jni(ts);
        println!("{o:?}");
        assert_eq!(o.to_jni(), ts);
    }

    #[test]
    fn test_signature4() {
        use super::*;
        use winnow::Parser;

        let mut ts = "CONSTANT_FIELD:I";
        let o = parse_type_parameter().parse_next(&mut ts).unwrap();
        println!("{o:?}");
    }
}
