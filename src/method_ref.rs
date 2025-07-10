use std::fmt;

use winnow::{ModalParser, Parser, combinator::terminated, error::InputError, token::literal};

use crate::{
    object_identifier::{ObjectIdentifier, parse_object_identifier},
    signature::method_signature::{MethodParameter, parse_method_parameter},
};

/// A symbolic reference to a method.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodRef<'a> {
    /// The fully qualified class name, e.g. "Lcom/example/MyClass;".
    pub class: ObjectIdentifier<'a>,
    pub param: MethodParameter<'a>,
}

impl fmt::Display for MethodRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Example: Lkotlin/jvm/internal/Intrinsics;->checkNotNullParameter(Ljava/lang/Object;Ljava/lang/String;)V
        write!(
            f,
            "{}->{}{}",
            self.class,
            self.param.ident,
            self.param.ms.to_jni()
        )
    }
}

/// Parse a method reference of the form:
///    L<class>;-><method>(<args>)<ret>
/// For example:
///    Lkotlin/jvm/internal/Intrinsics;->checkNotNullParameter(Ljava/lang/Object;Ljava/lang/String;)V
pub fn parse_method_ref<'a>() -> impl ModalParser<&'a str, MethodRef<'a>, InputError<&'a str>> {
    (
        terminated(parse_object_identifier(), literal("->")),
        parse_method_parameter(),
    )
        .map(|(class, param)| MethodRef { class, param })
}
