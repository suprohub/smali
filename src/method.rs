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
    pub locals: Option<u32>,
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
            opt(preceded(ws(literal(".locals")), ws(parse_int_lit::<u32>()))),
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
        if let Some(locals) = method.locals {
            out.push_str(&format!("    .locals {locals}\n"));
        }
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
        let mut smali = r#".method public final a(JIIILxpf;)V
    .locals 13

    move-object v0, p0

    move/from16 v1, p5

    iget-object v2, v0, La27;->d:Lea6;

    invoke-virtual {v2}, Ljava/lang/Object;->getClass()Ljava/lang/Class;

    iget v2, v0, La27;->f:I

    sub-int/2addr v2, v1

    sub-int v3, v2, p4

    iget-object v4, v0, La27;->e:[B

    invoke-static {v4, v3, v2}, Ljava/util/Arrays;->copyOfRange([BII)[B

    move-result-object v3

    new-instance v4, Ll8b;

    invoke-direct {v4, v3}, Ll8b;-><init>([B)V

    iget-object v3, v0, La27;->e:[B

    const/4 v5, 0x0

    invoke-static {v3, v2, v3, v5, v1}, Ljava/lang/System;->arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V

    iput v1, v0, La27;->f:I

    iget-object v1, v0, La27;->d:Lea6;

    iget-object v1, v1, Lea6;->n:Ljava/lang/String;

    iget-object v2, v0, La27;->c:Lea6;

    iget-object v3, v2, Lea6;->n:Ljava/lang/String;

    invoke-static {v1, v3}, Lv0g;->a(Ljava/lang/Object;Ljava/lang/Object;)Z

    move-result v1

    if-eqz v1, :cond_0

    goto :goto_0

    :cond_0
    iget-object v1, v0, La27;->d:Lea6;

    iget-object v1, v1, Lea6;->n:Ljava/lang/String;

    const-string v3, "application/x-emsg"

    invoke-virtual {v3, v1}, Ljava/lang/String;->equals(Ljava/lang/Object;)Z

    move-result v1

    if-eqz v1, :cond_2

    iget-object v1, v0, La27;->a:Lvp;

    invoke-virtual {v1}, Ljava/lang/Object;->getClass()Ljava/lang/Class;

    invoke-static {v4}, Lvp;->N(Ll8b;)Lc95;

    move-result-object v1

    invoke-virtual {v1}, Lc95;->h()Lea6;

    move-result-object v3

    iget-object v2, v2, Lea6;->n:Ljava/lang/String;

    if-eqz v3, :cond_1

    iget-object v3, v3, Lea6;->n:Ljava/lang/String;

    invoke-static {v2, v3}, Lv0g;->a(Ljava/lang/Object;Ljava/lang/Object;)Z

    move-result v3

    if-eqz v3, :cond_1

    new-instance v4, Ll8b;

    invoke-virtual {v1}, Lc95;->n()[B

    move-result-object v1

    invoke-virtual {v1}, Ljava/lang/Object;->getClass()Ljava/lang/Class;

    invoke-direct {v4, v1}, Ll8b;-><init>([B)V

    :goto_0
    invoke-virtual {v4}, Ll8b;->a()I

    move-result v10

    iget-object v1, v0, La27;->b:Lzpf;

    invoke-interface {v1, v4, v10, v5}, Lzpf;->b(Ll8b;II)V

    iget-object v6, v0, La27;->b:Lzpf;

    const/4 v11, 0x0

    move-wide v7, p1

    move/from16 v9, p3

    move-object/from16 v12, p6

    invoke-interface/range {v6 .. v12}, Lzpf;->a(JIIILxpf;)V

    return-void

    :cond_1
    invoke-virtual {v1}, Lc95;->h()Lea6;

    move-result-object v0

    new-instance v1, Ljava/lang/StringBuilder;

    const-string v3, "Ignoring EMSG. Expected it to contain wrapped "

    invoke-direct {v1, v3}, Ljava/lang/StringBuilder;-><init>(Ljava/lang/String;)V

    invoke-virtual {v1, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    const-string v2, " but actual wrapped format: "

    invoke-virtual {v1, v2}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    invoke-virtual {v1, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/Object;)Ljava/lang/StringBuilder;

    invoke-virtual {v1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-static {v0}, Li8b;->V(Ljava/lang/String;)V

    return-void

    :cond_2
    new-instance v1, Ljava/lang/StringBuilder;

    const-string v2, "Ignoring sample for unsupported format: "

    invoke-direct {v1, v2}, Ljava/lang/StringBuilder;-><init>(Ljava/lang/String;)V

    iget-object v0, v0, La27;->d:Lea6;

    iget-object v0, v0, Lea6;->n:Ljava/lang/String;

    invoke-virtual {v1, v0}, Ljava/lang/StringBuilder;->append(Ljava/lang/String;)Ljava/lang/StringBuilder;

    invoke-virtual {v1}, Ljava/lang/StringBuilder;->toString()Ljava/lang/String;

    move-result-object v0

    invoke-static {v0}, Li8b;->V(Ljava/lang/String;)V

    return-void
.end method"#;

        let m = parse_method().parse_next(&mut smali).unwrap();
        println!("{}", write_method(&m))
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
        assert_eq!(method.locals, Some(1));
        assert_eq!(method.modifiers.len(), 3); // private, static, final
    }
}
