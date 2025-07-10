use std::fmt;
use winnow::{ModalParser, Parser, combinator::terminated, error::InputError, token::literal};

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

pub fn parse_field_ref<'a>() -> impl ModalParser<&'a str, FieldRef<'a>, InputError<&'a str>> {
    (
        terminated(parse_object_identifier(), literal("->")),
        parse_type_parameter(),
    )
        .map(|(class, param)| FieldRef { class, param })
}
