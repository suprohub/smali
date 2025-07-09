use nom::{Parser, bytes::complete::tag, combinator::map, error::Error, sequence::terminated};
use std::fmt;

use crate::{
    object_identifier::{ObjectIdentifier, parse_object_identifier},
    signature::type_signature::{TypeParameter, parse_type_parameter},
};

/// A symbolic reference to a field.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldRef<'a> {
    /// The fully qualified class name, e.g. "Lcom/example/MyClass;".
    pub class: ObjectIdentifier<'a>,
    pub param: TypeParameter<'a>,
}

impl fmt::Display for FieldRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Example: Lcom/example/MyClass;->myField:I
        write!(
            f,
            "{}->{}:{}",
            self.class,
            self.param.ident,
            self.param.ts.to_jni()
        )
    }
}

pub fn parse_field_ref<'a>() -> impl Parser<&'a str, Output = FieldRef<'a>, Error = Error<&'a str>>
{
    map(
        (
            terminated(parse_object_identifier(), tag("->")),
            parse_type_parameter(),
        ),
        |(class, param)| FieldRef { class, param },
    )
}
