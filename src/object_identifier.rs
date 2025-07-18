use core::fmt;
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};
use winnow::{
    ModalParser, Parser,
    combinator::{delimited, opt, preceded},
    error::InputError,
    token::{one_of, take_while},
};

use crate::signature::{parse_type_parameters, type_signature::TypeSignature};

/// Represents a Java object identifier
///
/// # Examples
///
/// ```
///
///
/// use smali::types::ObjectIdentifier;
///
/// let o = ObjectIdentifier::from_java_type("com.basic.Test");
///  assert_eq!(o.as_java_type(), "com.basic.Test");
///  assert_eq!(o.as_jni_type(), "Lcom/basic/Test;");
/// ```
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct ObjectIdentifier<'a> {
    pub class_name: Cow<'a, str>,
    pub type_arguments: Option<Vec<TypeSignature<'a>>>,
    pub suffix: Option<Cow<'a, str>>,
}

impl Hash for ObjectIdentifier<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
    }
}

impl fmt::Display for ObjectIdentifier<'_> {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.as_jni_type())
    }
}

impl ObjectIdentifier<'_> {
    pub fn as_jni_type(&self) -> String {
        let mut s = "L".to_string();
        s.push_str(&self.class_name);
        if let Some(v) = &self.type_arguments {
            s.push('<');
            for t in v {
                s.push_str(&t.to_jni());
            }
            s.push('>');
        }
        if let Some(suffix) = &self.suffix {
            s.push('.');
            s.push_str(suffix);
        }
        s.push(';');
        s
    }

    pub fn as_java_type(&self) -> String {
        self.class_name.replace('/', ".")
    }
}

pub fn parse_object_identifier<'a>()
-> impl ModalParser<&'a str, ObjectIdentifier<'a>, InputError<&'a str>> {
    delimited(
        one_of('L'),
        (
            take_while(0.., |x| (x != ';') && (x != '<')),
            opt(parse_type_parameters()),
            opt(preceded(one_of('.'), take_while(0.., |x| x != ';'))),
        ),
        one_of(';'),
    )
    .map(|(name, type_arguments, suf)| ObjectIdentifier {
        class_name: Cow::Borrowed(name),
        type_arguments,
        suffix: suf.map(Cow::Borrowed),
    })
}
