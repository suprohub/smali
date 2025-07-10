use crate::{
    annotation::{Annotation, parse_annotation, write_annotation},
    modifier::{Modifier, parse_modifiers, write_modifiers},
    op::{Op, parse_op},
    param::{Param, parse_param, write_param},
    parse_int_lit,
    signature::method_signature::{MethodParameter, parse_method_parameter},
    ws,
};
use winnow::{
    ModalParser, Parser,
    combinator::{delimited, opt, preceded, repeat},
    error::InputError,
    token::literal,
};

/// Struct representing a Java method
///
#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    /// Method modifiers
    pub modifiers: Vec<Modifier>,

    pub param: MethodParameter<'a>,
    /// Number of local variables required by the operations
    pub locals: u32,
    /// Method params
    pub params: Vec<Param<'a>>,
    /// Any method level annotations
    pub annotations: Vec<Annotation<'a>>,
    /// Method operations
    pub ops: Vec<Op<'a>>,
}

pub fn parse_method<'a>() -> impl ModalParser<&'a str, Method<'a>, InputError<&'a str>> {
    delimited(
        ws(literal(".method")),
        (
            parse_modifiers(),
            parse_method_parameter(),
            preceded(ws(literal(".locals")), ws(parse_int_lit::<u32>())),
            repeat(0.., parse_param()),
            repeat(0.., parse_annotation()),
            opt(ws(literal(".prologue"))),
            repeat(0.., parse_op()),
        ),
        ws(literal(".end method")),
    )
    .map(
        |(modifiers, param, locals, params, annotations, _, ops)| Method {
            modifiers,
            param,
            locals,
            params,
            annotations,
            ops,
        },
    )
}

pub fn write_method(method: &Method) -> String {
    let mut out = format!(".method {}", write_modifiers(&method.modifiers));
    out.push_str(&format!(
        "{}{}\n",
        method.param.ident,
        method.param.ms.to_jni()
    ));
    if !method.ops.is_empty() {
        out.push_str(&format!("    .locals {:}\n", method.locals));
    }

    for param in &method.params {
        out.push_str("    ");
        out.push_str(&write_param(param));
        out.push('\n');
    }

    for a in &method.annotations {
        out.push_str(&write_annotation(a, false, true));
    }

    for i in &method.ops {
        match i {
            Op::Line(l) => {
                out.push_str(&format!("    .line {l:}\n"));
            }
            Op::Label(l) => {
                out.push_str(&format!("    {l}\n"));
            }
            Op::Op(s) => {
                out.push_str(&format!("    {s}\n"));
            }
            Op::Catch(c) => {
                out.push_str(&format!("    {c}\n"));
            }
            Op::ArrayData(ad) => {
                out.push_str(&format!("    {ad}\n"));
            }
            Op::PackedSwitch(ps) => {
                out.push_str(&format!("    {ps}\n"));
            }
            Op::SparseSwitch(ss) => {
                out.push_str(&format!("    {ss}\n"));
            }
        }
    }

    out.push_str(".end method\n\n");
    out
}

mod tests {
    #[test]
    fn test_method() {
        use super::*;
        use winnow::Parser;
        let mut smali = r#".method public static values()[La0h;
    .locals 1

    sget-object v0, La0h;->v:[La0h;

    invoke-virtual {v0}, [La0h;->clone()Ljava/lang/Object;

    move-result-object v0

    check-cast v0, [La0h;

    return-object v0
.end method"#;

        let _ = parse_method().parse_next(&mut smali).unwrap();
    }

    #[test]
    fn test_method_with_param_annotation() {
        use super::*;
        use winnow::Parser;
        let mut smali = r#".method private static final isInitialized(Lkotlin/reflect/KProperty0;)Z
    .locals 1
    .param p0    # Lkotlin/reflect/KProperty0;
        .annotation build Lkotlin/internal/AccessibleLateinitPropertyLiteral;
        .end annotation
    .end param
    .annotation system Ldalvik/annotation/Signature;
        value = {
            "(",
            "Lkotlin/reflect/KProperty0<",
            "*>;)Z"
        }
    .end annotation

    new-instance p0, Lkotlin/NotImplementedError;

    const-string v0, "Implementation is intrinsic"

    invoke-direct {p0, v0}, Lkotlin/NotImplementedError;-><init>(Ljava/lang/String;)V

    throw p0
.end method
    "#;

        let method = parse_method().parse_next(&mut smali).unwrap();
        println!("{method:?}");

        assert_eq!(method.param.ident, "isInitialized");
        assert_eq!(method.annotations.len(), 1); // Signature annotation
        assert_eq!(method.ops.len(), 4);
        assert_eq!(method.locals, 1);
        assert_eq!(method.modifiers.len(), 3); // private, static, final
    }
}
