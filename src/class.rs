use std::{
    borrow::Cow,
    fs,
    hash::{Hash, Hasher},
    path::Path,
};

use anyhow::Result;

use winnow::{
    ModalParser, Parser,
    combinator::{opt, preceded, repeat},
    error::InputError,
    token::literal,
};

use crate::{
    SmaliError,
    annotation::{Annotation, parse_annotation, write_annotation},
    field::{Field, parse_field},
    method::{Method, parse_method, write_method},
    modifier::{Modifier, parse_modifiers, write_modifiers},
    object_identifier::{ObjectIdentifier, parse_object_identifier},
    parse_string_lit, ws,
};

/// Represents a smali class i.e. the whole .smali file
///
/// # Examples
///
/// ```no_run
///  use std::path::Path;
///  use smali::types::SmaliClass;
///
///  let c = SmaliClass::read_from_file(Path::new("smali/com/cool/Class.smali")).expect("Uh oh, does the file exist?");
///  println!("Java class: {}", c.name.as_java_type());
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    /// The name of this class
    pub name: ObjectIdentifier<'a>,
    /// Class modifiers
    pub modifiers: Vec<Modifier>,
    /// The source filename if included in the smali doc
    pub source: Option<Cow<'a, str>>,
    /// The class' superclass (every Java class has one)
    pub super_class: ObjectIdentifier<'a>,
    /// List of all the interfaces the class implements
    pub implements: Vec<ObjectIdentifier<'a>>,
    /// Class level annotations
    pub annotations: Vec<Annotation<'a>>,
    /// All the fields defined by the class
    pub fields: Vec<Field<'a>>,
    /// All the methods defined by the class
    pub methods: Vec<Method<'a>>,
}

pub fn parse_class<'a>() -> impl ModalParser<&'a str, Class<'a>, InputError<&'a str>> {
    (
        preceded(
            ws(literal(".class")),
            (parse_modifiers(), ws(parse_object_identifier())),
        ),
        preceded(ws(literal(".super")), ws(parse_object_identifier())),
        opt(preceded(ws(literal(".source")), ws(parse_string_lit())).map(Cow::Borrowed)),
        repeat(
            0..,
            preceded(ws(literal(".implements")), ws(parse_object_identifier())),
        ),
        repeat(0.., parse_annotation()),
        repeat(0.., parse_field()),
        repeat(0.., parse_method()),
    )
        .map(
            |((modifiers, name), super_class, source, implements, annotations, fields, methods)| {
                Class {
                    name,
                    modifiers,
                    source,
                    super_class,
                    implements,
                    annotations,
                    fields,
                    methods,
                }
            },
        )
}

impl Hash for Class<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<'a> Class<'a> {
    /// Creates a smali document string from the current class
    ///
    /// # Examples
    ///
    /// ```no_run
    ///  use std::path::Path;
    ///  use smali::types::SmaliClass;
    ///
    ///  let c = SmaliClass::read_from_file(Path::new("smali/com/cool/Class.smali")).expect("Uh oh, does the file exist?");
    ///  println!("{}", c.to_smali());
    ///
    /// ```
    pub fn to_smali(&self) -> String {
        write_class(self)
    }

    /// Writes the current SmaliClass to the specified file path as a smali document
    ///
    /// # Examples
    ///
    /// ```no_run
    ///  use std::path::Path;
    ///  use smali::types::SmaliClass;
    ///
    ///  let c = SmaliClass::read_from_file(Path::new("smali/com/cool/Class.smali")).expect("Uh oh, does the file exist?");
    ///  c.write_to_file(Path::new("smali_classes2/com/cool/Class.smali")).unwrap();
    ///
    /// ```
    pub fn write_to_file(&self, path: &Path) -> Result<(), SmaliError> {
        let smali = self.to_smali();
        if let Err(e) = fs::write(path, smali) {
            Err(SmaliError {
                details: e.to_string(),
            })
        } else {
            Ok(())
        }
    }
}

pub fn write_class(dex: &Class) -> String {
    let mut out = format!(
        ".class {}{}\n",
        write_modifiers(&dex.modifiers),
        dex.name.as_jni_type()
    );
    out.push_str(&format!(".super {}\n", dex.super_class.as_jni_type()));
    if let Some(s) = &dex.source {
        out.push_str(&format!(".source \"{s}\"\n"));
    }

    if !dex.implements.is_empty() {
        out.push_str("\n# interfaces\n");
        for i in &dex.implements {
            out.push_str(".implements ");
            out.push_str(&i.as_jni_type());
            out.push('\n');
        }
    }

    if !dex.annotations.is_empty() {
        out.push_str("\n# annotations\n");
        for a in &dex.annotations {
            out.push_str(&write_annotation(a, false, false));
            out.push('\n');
        }
    }

    if !dex.fields.is_empty() {
        out.push_str("\n# fields\n");
        for f in &dex.fields {
            out.push_str(&format!(
                ".field {}{}:{}",
                write_modifiers(&f.modifiers),
                f.param.ident,
                f.param.ts.to_jni()
            ));
            if let Some(iv) = &f.initial_value {
                out.push_str(&format!(" = {iv}"));
            }
            out.push('\n');
            if !f.annotations.is_empty() {
                for a in &f.annotations {
                    out.push_str(&write_annotation(a, false, true));
                }
                out.push_str(".end field\n");
            }
            out.push('\n');
        }
    }

    if !dex.methods.is_empty() {
        out.push_str("\n# methods\n");
        for m in &dex.methods {
            out.push_str(&write_method(m));
        }
    }

    out
}

mod tests {
    #[test]
    fn test_parse_class() {
        use super::*;
        use winnow::Parser;

        for dir in fs::read_dir("tests").unwrap() {
            let dir = dir.unwrap();
            println!("{:?}", dir.file_name());
            let smali = fs::read_to_string(dir.path()).unwrap();
            let _ = parse_class().parse_next(&mut smali.as_str()).unwrap();
        }
    }

    #[test]
    fn test_read_write_class() {
        use super::*;
        use winnow::Parser;

        for dir in fs::read_dir("tests").unwrap() {
            let dir = dir.unwrap();
            println!("{:?}", dir.file_name());
            let smali = fs::read_to_string(dir.path()).unwrap();

            let c = parse_class().parse_next(&mut smali.as_str()).unwrap();

            assert!(!c.annotations.is_empty());
            assert!(!c.fields.is_empty());
            assert!(!c.methods.is_empty());

            let second_smali = c.to_smali();

            let c2 = parse_class()
                .parse_next(&mut second_smali.as_str())
                .unwrap();

            assert!(!c2.annotations.is_empty());
            assert!(!c2.fields.is_empty());
            assert!(!c2.methods.is_empty());

            assert_eq!(c, c2);
        }
    }
}
