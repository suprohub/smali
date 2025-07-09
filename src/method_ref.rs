use std::fmt;

use nom::{Parser, bytes::complete::tag, combinator::map, error::Error, sequence::terminated};

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
pub fn parse_method_ref<'a>() -> impl Parser<&'a str, Output = MethodRef<'a>, Error = Error<&'a str>>
{
    map(
        (
            terminated(parse_object_identifier(), tag("->")),
            parse_method_parameter(),
        ),
        |(class, param)| MethodRef { class, param },
    )
}
