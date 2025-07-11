//! TODO:
//! What about multispace0 and space0? Where i can set this?
//! Is there any .smali grammar docs? Like c++

use std::{
    borrow::Cow,
    fmt::{self, Debug},
    str::FromStr,
};

use winnow::{
    ModalParser, ModalResult, Parser,
    ascii::{alphanumeric1, digit1, space1},
    combinator::{alt, delimited, preceded, separated, terminated},
    error::{ErrMode, InputError},
    token::{literal, one_of, take_while},
};

use crate::{
    field_ref::{FieldRef, parse_field_ref},
    method_ref::{MethodRef, parse_method_ref},
    op::{Label, parse_label},
    parse_int_lit, parse_string_lit,
    signature::type_signature::{TypeSignature, parse_typesignature},
    ws,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    Parameter(u16),
    Local(u16),
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Here we don't know the method context so we just print the raw value.
        // In a full implementation you would convert using the method context.
        match self {
            Register::Parameter(n) => write!(f, "p{n}"),
            Register::Local(n) => write!(f, "v{n}"),
        }
    }
}

/// A symbolic range of registers as written in smali, e.g. "{v0 .. v6}"
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterRange {
    pub start: Register,
    pub end: Register,
}

impl fmt::Display for RegisterRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print in smali style: "{v0 .. v6}"
        write!(f, "{{ {} .. {} }}", self.start, self.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvokeType {
    Virtual,
    Super,
    Interface,
    Direct,
    Static,
    VirtualRange,
    SuperRange,
    DirectRange,
    StaticRange,
    InterfaceRange,
    Polymorphic,
    PolymorphicRange,
    Custom,
    CustomRange,
}

impl InvokeType {
    pub fn is_range(&self) -> bool {
        matches!(
            self,
            Self::VirtualRange
                | Self::SuperRange
                | Self::DirectRange
                | Self::StaticRange
                | Self::InterfaceRange
                | Self::PolymorphicRange
                | Self::CustomRange
        )
    }
}

impl FromStr for InvokeType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "invoke-virtual" => Ok(InvokeType::Virtual),
            "invoke-super" => Ok(InvokeType::Super),
            "invoke-interface" => Ok(InvokeType::Interface),
            "invoke-direct" => Ok(InvokeType::Direct),
            "invoke-static" => Ok(InvokeType::Static),
            "invoke-virtual/range" => Ok(InvokeType::VirtualRange),
            "invoke-super/range" => Ok(InvokeType::SuperRange),
            "invoke-direct/range" => Ok(InvokeType::DirectRange),
            "invoke-static/range" => Ok(InvokeType::StaticRange),
            "invoke-interface/range" => Ok(InvokeType::InterfaceRange),
            "invoke-polymorphic" => Ok(InvokeType::Polymorphic),
            "invoke-polymorphic/range" => Ok(InvokeType::PolymorphicRange),
            "invoke-custom" => Ok(InvokeType::Custom),
            "invoke-custom/range" => Ok(InvokeType::CustomRange),
            _ => Err(()),
        }
    }
}

impl fmt::Display for InvokeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvokeType::Virtual => write!(f, "invoke-virtual"),
            InvokeType::Super => write!(f, "invoke-super"),
            InvokeType::Interface => write!(f, "invoke-interface"),
            InvokeType::Direct => write!(f, "invoke-direct"),
            InvokeType::Static => write!(f, "invoke-static"),
            InvokeType::VirtualRange => write!(f, "invoke-virtual/range"),
            InvokeType::SuperRange => write!(f, "invoke-super/range"),
            InvokeType::DirectRange => write!(f, "invoke-direct/range"),
            InvokeType::StaticRange => write!(f, "invoke-static/range"),
            InvokeType::InterfaceRange => write!(f, "invoke-interface/range"),
            InvokeType::Polymorphic => write!(f, "invoke-polymorphic"),
            InvokeType::PolymorphicRange => write!(f, "invoke-polymorphic/range"),
            InvokeType::Custom => write!(f, "invoke-custom"),
            InvokeType::CustomRange => write!(f, "invoke-custom/range"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstType {
    String,
    StringJumbo,
    Class,
    MethodHandle,
    MethodType,
}

impl FromStr for ConstType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "const-string" => Ok(ConstType::String),
            "const-string/jumbo" => Ok(ConstType::StringJumbo),
            "const-class" => Ok(ConstType::Class),
            "const-method-handle" => Ok(ConstType::MethodHandle),
            "const-method-type" => Ok(ConstType::MethodType),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ConstType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstType::String => write!(f, "const-string"),
            ConstType::StringJumbo => write!(f, "const-string/jumbo"),
            ConstType::Class => write!(f, "const-class"),
            ConstType::MethodHandle => write!(f, "const-method-handle"),
            ConstType::MethodType => write!(f, "const-method-type"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TwoRegMoveType {
    Normal,
    From16,
    Wide,
    WideFrom16,
    Wide16,
    Object,
    ObjectFrom16,
    Object16,
}

impl FromStr for TwoRegMoveType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "move" => Ok(TwoRegMoveType::Normal),
            "move/from16" => Ok(TwoRegMoveType::From16),
            "move-wide" => Ok(TwoRegMoveType::Wide),
            "move-wide/from16" => Ok(TwoRegMoveType::WideFrom16),
            "move-wide/16" => Ok(TwoRegMoveType::Wide16),
            "move-object" => Ok(TwoRegMoveType::Object),
            "move-object/from16" => Ok(TwoRegMoveType::ObjectFrom16),
            "move-object/16" => Ok(TwoRegMoveType::Object16),
            _ => Err(()),
        }
    }
}

impl fmt::Display for TwoRegMoveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TwoRegMoveType::Normal => write!(f, "move"),
            TwoRegMoveType::From16 => write!(f, "move/from16"),
            TwoRegMoveType::Wide => write!(f, "move-wide"),
            TwoRegMoveType::WideFrom16 => write!(f, "move-wide/from16"),
            TwoRegMoveType::Wide16 => write!(f, "move-wide/16"),
            TwoRegMoveType::Object => write!(f, "move-object"),
            TwoRegMoveType::ObjectFrom16 => write!(f, "move-object/from16"),
            TwoRegMoveType::Object16 => write!(f, "move-object/16"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OneRegMoveType {
    Result,
    ResultWide,
    ResultObject,
    Exception,
}

impl FromStr for OneRegMoveType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "move-result" => Ok(OneRegMoveType::Result),
            "move-result-wide" => Ok(OneRegMoveType::ResultWide),
            "move-result-object" => Ok(OneRegMoveType::ResultObject),
            "move-exception" => Ok(OneRegMoveType::Exception),
            _ => Err(()),
        }
    }
}

impl fmt::Display for OneRegMoveType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OneRegMoveType::Result => write!(f, "move-result"),
            OneRegMoveType::ResultWide => write!(f, "move-result-wide"),
            OneRegMoveType::ResultObject => write!(f, "move-result-object"),
            OneRegMoveType::Exception => write!(f, "move-exception"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReturnType {
    Void,
    Normal,
    Wide,
    Object,
}

impl FromStr for ReturnType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "return-void" => Ok(ReturnType::Void),
            "return" => Ok(ReturnType::Normal),
            "return-wide" => Ok(ReturnType::Wide),
            "return-object" => Ok(ReturnType::Object),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ReturnType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReturnType::Void => write!(f, "return-void"),
            ReturnType::Normal => write!(f, "return"),
            ReturnType::Wide => write!(f, "return-wide"),
            ReturnType::Object => write!(f, "return-object"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StaticFieldAccessType {
    Get,
    Put,
}

impl FromStr for StaticFieldAccessType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sget" => Ok(StaticFieldAccessType::Get),
            "sput" => Ok(StaticFieldAccessType::Put),
            _ => Err(()),
        }
    }
}

impl fmt::Display for StaticFieldAccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StaticFieldAccessType::Get => write!(f, "sget"),
            StaticFieldAccessType::Put => write!(f, "sput"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DynamicFieldAccessType {
    Get,
    Put,
}

impl FromStr for DynamicFieldAccessType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iget" => Ok(DynamicFieldAccessType::Get),
            "iput" => Ok(DynamicFieldAccessType::Put),
            _ => Err(()),
        }
    }
}

impl fmt::Display for DynamicFieldAccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DynamicFieldAccessType::Get => write!(f, "iget"),
            DynamicFieldAccessType::Put => write!(f, "iput"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldValueType {
    Normal,
    Wide,
    Object,
    Boolean,
    Byte,
    Char,
    Short,
}

impl FromStr for FieldValueType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(FieldValueType::Normal),
            "wide" => Ok(FieldValueType::Wide),
            "object" => Ok(FieldValueType::Object),
            "boolean" => Ok(FieldValueType::Boolean),
            "byte" => Ok(FieldValueType::Byte),
            "char" => Ok(FieldValueType::Char),
            "short" => Ok(FieldValueType::Short),
            _ => Err(()),
        }
    }
}

impl fmt::Display for FieldValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FieldValueType::Normal => Ok(()),
            FieldValueType::Wide => write!(f, "wide"),
            FieldValueType::Object => write!(f, "object"),
            FieldValueType::Boolean => write!(f, "boolean"),
            FieldValueType::Byte => write!(f, "byte"),
            FieldValueType::Char => write!(f, "char"),
            FieldValueType::Short => write!(f, "short"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithType {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Ushr,
}

impl FromStr for ArithType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(ArithType::Add),
            "sub" => Ok(ArithType::Sub),
            "mul" => Ok(ArithType::Mul),
            "div" => Ok(ArithType::Div),
            "rem" => Ok(ArithType::Rem),
            "and" => Ok(ArithType::And),
            "or" => Ok(ArithType::Or),
            "xor" => Ok(ArithType::Xor),
            "shl" => Ok(ArithType::Shl),
            "shr" => Ok(ArithType::Shr),
            "ushr" => Ok(ArithType::Ushr),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ArithType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithType::Add => write!(f, "add"),
            ArithType::Sub => write!(f, "sub"),
            ArithType::Mul => write!(f, "mul"),
            ArithType::Div => write!(f, "div"),
            ArithType::Rem => write!(f, "rem"),
            ArithType::And => write!(f, "and"),
            ArithType::Or => write!(f, "or"),
            ArithType::Xor => write!(f, "xor"),
            ArithType::Shl => write!(f, "shl"),
            ArithType::Shr => write!(f, "shr"),
            ArithType::Ushr => write!(f, "ushr"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithUnaryType {
    Neg,
    Not,
}

impl FromStr for ArithUnaryType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "neg" => Ok(ArithUnaryType::Neg),
            "not" => Ok(ArithUnaryType::Not),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ArithUnaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithUnaryType::Neg => write!(f, "neg"),
            ArithUnaryType::Not => write!(f, "not"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithOperandType {
    Int,
    Long,
    Float,
    Double,
}

impl FromStr for ArithOperandType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(ArithOperandType::Int),
            "long" => Ok(ArithOperandType::Long),
            "float" => Ok(ArithOperandType::Float),
            "double" => Ok(ArithOperandType::Double),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ArithOperandType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithOperandType::Int => write!(f, "int"),
            ArithOperandType::Long => write!(f, "long"),
            ArithOperandType::Float => write!(f, "float"),
            ArithOperandType::Double => write!(f, "double"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArithOperand2AddrType {
    Int,
    Long,
    Float,
    Double,
}

impl FromStr for ArithOperand2AddrType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int/2addr" => Ok(ArithOperand2AddrType::Int),
            "long/2addr" => Ok(ArithOperand2AddrType::Long),
            "float/2addr" => Ok(ArithOperand2AddrType::Float),
            "double/2addr" => Ok(ArithOperand2AddrType::Double),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ArithOperand2AddrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArithOperand2AddrType::Int => write!(f, "int/2addr"),
            ArithOperand2AddrType::Long => write!(f, "long/2addr"),
            ArithOperand2AddrType::Float => write!(f, "float/2addr"),
            ArithOperand2AddrType::Double => write!(f, "double/2addr"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConditionType {
    Eqz,
    Nez,
    Ltz,
    Gez,
    Gtz,
    Lez,
}

impl FromStr for ConditionType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "if-eqz" => Ok(ConditionType::Eqz),
            "if-nez" => Ok(ConditionType::Nez),
            "if-ltz" => Ok(ConditionType::Ltz),
            "if-gez" => Ok(ConditionType::Gez),
            "if-gtz" => Ok(ConditionType::Gtz),
            "if-lez" => Ok(ConditionType::Lez),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ConditionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConditionType::Eqz => write!(f, "if-eqz"),
            ConditionType::Nez => write!(f, "if-nez"),
            ConditionType::Ltz => write!(f, "if-ltz"),
            ConditionType::Gez => write!(f, "if-gez"),
            ConditionType::Gtz => write!(f, "if-gtz"),
            ConditionType::Lez => write!(f, "if-lez"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TwoRegConditionType {
    Eq,
    Ne,
    Lt,
    Ge,
    Gt,
    Le,
}

impl FromStr for TwoRegConditionType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "if-eq" => Ok(TwoRegConditionType::Eq),
            "if-ne" => Ok(TwoRegConditionType::Ne),
            "if-lt" => Ok(TwoRegConditionType::Lt),
            "if-ge" => Ok(TwoRegConditionType::Ge),
            "if-gt" => Ok(TwoRegConditionType::Gt),
            "if-le" => Ok(TwoRegConditionType::Le),
            _ => Err(()),
        }
    }
}

impl fmt::Display for TwoRegConditionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TwoRegConditionType::Eq => write!(f, "if-eq"),
            TwoRegConditionType::Ne => write!(f, "if-ne"),
            TwoRegConditionType::Lt => write!(f, "if-lt"),
            TwoRegConditionType::Ge => write!(f, "if-ge"),
            TwoRegConditionType::Gt => write!(f, "if-gt"),
            TwoRegConditionType::Le => write!(f, "if-le"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GotoType {
    Normal,
    Size16,
    Size32,
}

impl FromStr for GotoType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "goto" => Ok(GotoType::Normal),
            "goto/16" => Ok(GotoType::Size16),
            "goto/32" => Ok(GotoType::Size32),
            _ => Err(()),
        }
    }
}

impl fmt::Display for GotoType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GotoType::Normal => write!(f, "goto"),
            GotoType::Size16 => write!(f, "goto/16"),
            GotoType::Size32 => write!(f, "goto/32"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstLiteralType {
    Const4,
    Const16,
    Const,
    ConstHigh16,
    ConstWide16,
    ConstWide32,
    ConstWide,
    ConstWideHigh16,
}

impl FromStr for ConstLiteralType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "const/4" => Ok(ConstLiteralType::Const4),
            "const/16" => Ok(ConstLiteralType::Const16),
            "const" => Ok(ConstLiteralType::Const),
            "const/high16" => Ok(ConstLiteralType::ConstHigh16),
            "const-wide/16" => Ok(ConstLiteralType::ConstWide16),
            "const-wide/32" => Ok(ConstLiteralType::ConstWide32),
            "const-wide" => Ok(ConstLiteralType::ConstWide),
            "const-wide/high16" => Ok(ConstLiteralType::ConstWideHigh16),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ConstLiteralType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstLiteralType::Const4 => write!(f, "const/4"),
            ConstLiteralType::Const16 => write!(f, "const/16"),
            ConstLiteralType::Const => write!(f, "const"),
            ConstLiteralType::ConstHigh16 => write!(f, "const/high16"),
            ConstLiteralType::ConstWide16 => write!(f, "const-wide/16"),
            ConstLiteralType::ConstWide32 => write!(f, "const-wide/32"),
            ConstLiteralType::ConstWide => write!(f, "const-wide"),
            ConstLiteralType::ConstWideHigh16 => write!(f, "const-wide/high16"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstLiteralValue {
    Const4(i8),
    Const16(i16),
    Const(i32),
    ConstHigh16(i64),
    ConstWide16(i16),
    ConstWide32(i32),
    ConstWide(i64),
    ConstWideHigh16(i64),
}

impl fmt::Display for ConstLiteralValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConstLiteralValue::Const4(v) => write!(f, "{v}"),
            ConstLiteralValue::Const16(v) => write!(f, "{v}"),
            ConstLiteralValue::Const(v) => write!(f, "{v}"),
            ConstLiteralValue::ConstHigh16(v) => write!(f, "0x{:04x}0000", *v as u16),
            ConstLiteralValue::ConstWide16(v) => write!(f, "{v}"),
            ConstLiteralValue::ConstWide32(v) => write!(f, "{v}"),
            ConstLiteralValue::ConstWide(v) => write!(f, "0x{v:x}L"),
            ConstLiteralValue::ConstWideHigh16(v) => {
                write!(f, "0x{:04x}000000000000", *v as u16)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LitArithType8 {
    AddIntLit8,
    RSubIntLit8,
    MulIntLit8,
    DivIntLit8,
    RemIntLit8,
    AndIntLit8,
    OrIntLit8,
    XorIntLit8,
    ShlIntLit8,
    ShrIntLit8,
    UshrIntLit8,
}

impl FromStr for LitArithType8 {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add-int/lit8" => Ok(LitArithType8::AddIntLit8),
            "rsub-int/lit8" => Ok(LitArithType8::RSubIntLit8),
            "mul-int/lit8" => Ok(LitArithType8::MulIntLit8),
            "div-int/lit8" => Ok(LitArithType8::DivIntLit8),
            "rem-int/lit8" => Ok(LitArithType8::RemIntLit8),
            "and-int/lit8" => Ok(LitArithType8::AndIntLit8),
            "or-int/lit8" => Ok(LitArithType8::OrIntLit8),
            "xor-int/lit8" => Ok(LitArithType8::XorIntLit8),
            "shl-int/lit8" => Ok(LitArithType8::ShlIntLit8),
            "shr-int/lit8" => Ok(LitArithType8::ShrIntLit8),
            "ushr-int/lit8" => Ok(LitArithType8::UshrIntLit8),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LitArithType8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LitArithType8::AddIntLit8 => write!(f, "add-int/lit8"),
            LitArithType8::RSubIntLit8 => write!(f, "rsub-int/lit8"),
            LitArithType8::MulIntLit8 => write!(f, "mul-int/lit8"),
            LitArithType8::DivIntLit8 => write!(f, "div-int/lit8"),
            LitArithType8::RemIntLit8 => write!(f, "rem-int/lit8"),
            LitArithType8::AndIntLit8 => write!(f, "and-int/lit8"),
            LitArithType8::OrIntLit8 => write!(f, "or-int/lit8"),
            LitArithType8::XorIntLit8 => write!(f, "xor-int/lit8"),
            LitArithType8::ShlIntLit8 => write!(f, "shl-int/lit8"),
            LitArithType8::ShrIntLit8 => write!(f, "shr-int/lit8"),
            LitArithType8::UshrIntLit8 => write!(f, "ushr-int/lit8"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LitArithType16 {
    AddIntLit16,
    RSubIntLit16,
    MulIntLit16,
    DivIntLit16,
    RemIntLit16,
    AndIntLit16,
    OrIntLit16,
    XorIntLit16,
}

impl FromStr for LitArithType16 {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add-int/lit16" => Ok(LitArithType16::AddIntLit16),
            "rsub-int" => Ok(LitArithType16::RSubIntLit16),
            "mul-int/lit16" => Ok(LitArithType16::MulIntLit16),
            "div-int/lit16" => Ok(LitArithType16::DivIntLit16),
            "rem-int/lit16" => Ok(LitArithType16::RemIntLit16),
            "and-int/lit16" => Ok(LitArithType16::AndIntLit16),
            "or-int/lit16" => Ok(LitArithType16::OrIntLit16),
            "xor-int/lit16" => Ok(LitArithType16::XorIntLit16),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LitArithType16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LitArithType16::AddIntLit16 => write!(f, "add-int/lit16"),
            LitArithType16::RSubIntLit16 => write!(f, "rsub-int"),
            LitArithType16::MulIntLit16 => write!(f, "mul-int/lit16"),
            LitArithType16::DivIntLit16 => write!(f, "div-int/lit16"),
            LitArithType16::RemIntLit16 => write!(f, "rem-int/lit16"),
            LitArithType16::AndIntLit16 => write!(f, "and-int/lit16"),
            LitArithType16::OrIntLit16 => write!(f, "or-int/lit16"),
            LitArithType16::XorIntLit16 => write!(f, "xor-int/lit16"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConvertType {
    // Existing conversion operations
    IntToByte,
    IntToChar,
    IntToShort,
    IntToLong,
    IntToFloat,
    IntToDouble,
    LongToInt,
    LongToFloat,
    LongToDouble,
    FloatToInt,
    FloatToLong,
    FloatToDouble,
    DoubleToInt,
    DoubleToLong,
    DoubleToFloat,
}

impl FromStr for ConvertType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Existing conversion operations
            "int-to-byte" => Ok(ConvertType::IntToByte),
            "int-to-char" => Ok(ConvertType::IntToChar),
            "int-to-short" => Ok(ConvertType::IntToShort),
            "int-to-long" => Ok(ConvertType::IntToLong),
            "int-to-float" => Ok(ConvertType::IntToFloat),
            "int-to-double" => Ok(ConvertType::IntToDouble),
            "long-to-int" => Ok(ConvertType::LongToInt),
            "long-to-float" => Ok(ConvertType::LongToFloat),
            "long-to-double" => Ok(ConvertType::LongToDouble),
            "float-to-int" => Ok(ConvertType::FloatToInt),
            "float-to-long" => Ok(ConvertType::FloatToLong),
            "float-to-double" => Ok(ConvertType::FloatToDouble),
            "double-to-int" => Ok(ConvertType::DoubleToInt),
            "double-to-long" => Ok(ConvertType::DoubleToLong),
            "double-to-float" => Ok(ConvertType::DoubleToFloat),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ConvertType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConvertType::IntToByte => write!(f, "int-to-byte"),
            ConvertType::IntToChar => write!(f, "int-to-char"),
            ConvertType::IntToShort => write!(f, "int-to-short"),
            ConvertType::IntToLong => write!(f, "int-to-long"),
            ConvertType::IntToFloat => write!(f, "int-to-float"),
            ConvertType::IntToDouble => write!(f, "int-to-double"),
            ConvertType::LongToInt => write!(f, "long-to-int"),
            ConvertType::LongToFloat => write!(f, "long-to-float"),
            ConvertType::LongToDouble => write!(f, "long-to-double"),
            ConvertType::FloatToInt => write!(f, "float-to-int"),
            ConvertType::FloatToLong => write!(f, "float-to-long"),
            ConvertType::FloatToDouble => write!(f, "float-to-double"),
            ConvertType::DoubleToInt => write!(f, "double-to-int"),
            ConvertType::DoubleToLong => write!(f, "double-to-long"),
            ConvertType::DoubleToFloat => write!(f, "double-to-float"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArrayAccessType {
    Get,
    Put,
}

impl FromStr for ArrayAccessType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "aget" => Ok(ArrayAccessType::Get),
            "aput" => Ok(ArrayAccessType::Put),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ArrayAccessType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArrayAccessType::Get => write!(f, "aget"),
            ArrayAccessType::Put => write!(f, "aput"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArrayValueType {
    Normal,
    Wide,
    Object,
    Boolean,
    Byte,
    Char,
    Short,
}

impl FromStr for ArrayValueType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(ArrayValueType::Normal),
            "wide" => Ok(ArrayValueType::Wide),
            "object" => Ok(ArrayValueType::Object),
            "boolean" => Ok(ArrayValueType::Boolean),
            "byte" => Ok(ArrayValueType::Byte),
            "char" => Ok(ArrayValueType::Char),
            "short" => Ok(ArrayValueType::Short),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ArrayValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArrayValueType::Normal => Ok(()),
            ArrayValueType::Wide => write!(f, "wide"),
            ArrayValueType::Object => write!(f, "object"),
            ArrayValueType::Boolean => write!(f, "boolean"),
            ArrayValueType::Byte => write!(f, "byte"),
            ArrayValueType::Char => write!(f, "char"),
            ArrayValueType::Short => write!(f, "short"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmpType {
    CmplFloat,
    CmpgFloat,
    CmplDouble,
    CmpgDouble,
    CmpLong,
}

impl FromStr for CmpType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmpl-float" => Ok(CmpType::CmplFloat),
            "cmpg-float" => Ok(CmpType::CmpgFloat),
            "cmpl-double" => Ok(CmpType::CmplDouble),
            "cmpg-double" => Ok(CmpType::CmpgDouble),
            "cmp-long" => Ok(CmpType::CmpLong),
            _ => Err(()),
        }
    }
}

impl fmt::Display for CmpType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CmpType::CmplFloat => write!(f, "cmpl-float"),
            CmpType::CmpgFloat => write!(f, "cmpg-float"),
            CmpType::CmplDouble => write!(f, "cmpl-double"),
            CmpType::CmpgDouble => write!(f, "cmpg-double"),
            CmpType::CmpLong => write!(f, "cmp-long"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwitchType {
    PackedSwitch,
    SparseSwitch,
}

impl FromStr for SwitchType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "packed-switch" => Ok(SwitchType::PackedSwitch),
            "sparse-switch" => Ok(SwitchType::SparseSwitch),
            _ => Err(()),
        }
    }
}

impl fmt::Display for SwitchType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SwitchType::PackedSwitch => write!(f, "packed-switch"),
            SwitchType::SparseSwitch => write!(f, "sparse-switch"),
        }
    }
}

/// A high-level representation of a DEX operation.
///
/// This enum “lifts” many opcodes so that literal values and symbolic references
/// (e.g. for strings, classes, methods, fields, call sites, prototypes) are stored
/// directly rather than as indices.
#[derive(Debug, Clone, PartialEq)]
pub enum DexOp<'a> {
    Invoke {
        invoke_type: InvokeType,
        registers: Vec<Register>,
        range: Option<RegisterRange>,
        method: Option<Box<MethodRef<'a>>>,
        call_site: Option<Cow<'a, str>>,
        proto: Option<Cow<'a, str>>,
    },
    Const {
        const_type: ConstType,
        dest: Register,
        value: StringOrTypeSig<'a>,
    },
    MoveTwoReg {
        move_type: TwoRegMoveType,
        dest: Register,
        src: Register,
    },
    MoveOneReg {
        move_type: OneRegMoveType,
        dest: Register,
    },
    Return {
        return_type: ReturnType,
        src: Option<Register>,
    },
    Arith {
        arith_type: ArithType,
        operand_type: ArithOperandType,
        dest: Register,
        src1: Register,
        src2: Register,
    },
    ArithUnary {
        arith_type: ArithUnaryType,
        operand_type: ArithOperandType,
        dest: Register,
        src: Register,
    },
    Arith2Addr {
        arith_type: ArithType,
        operand_type: ArithOperand2AddrType,
        dest: Register,
        src: Register,
    },
    Condition {
        cond_type: ConditionType,
        reg1: Register,
        offset: Label<'a>,
    },
    TwoRegCondition {
        cond_type: TwoRegConditionType,
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    Goto {
        goto_type: GotoType,
        offset: Label<'a>,
    },
    ConstLiteral {
        const_type: ConstLiteralType,
        dest: Register,
        value: ConstLiteralValue,
    },
    LitArith8 {
        arith_type: LitArithType8,
        dest: Register,
        src: Register,
        literal: i8,
    },
    LitArith16 {
        arith_type: LitArithType16,
        dest: Register,
        src: Register,
        literal: i16,
    },
    Convert {
        convert_type: ConvertType,
        dest: Register,
        src: Register,
    },
    Cmp {
        cmp_type: CmpType,
        dest: Register,
        src1: Register,
        src2: Register,
    },
    ArrayAccess {
        access_type: ArrayAccessType,
        value_type: ArrayValueType,
        reg: Register,
        arr: Register,
        idx: Register,
    },
    DynamicFieldAccess {
        access_type: DynamicFieldAccessType,
        value_type: FieldValueType,
        reg: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    StaticFieldAccess {
        access_type: StaticFieldAccessType,
        value_type: FieldValueType,
        reg: Register,
        field: FieldRef<'a>,
    },

    Nop,
    MonitorEnter {
        src: Register,
    },
    MonitorExit {
        src: Register,
    },
    CheckCast {
        dest: Register,
        class: StringOrTypeSig<'a>,
    },
    InstanceOf {
        dest: Register,
        src: Register,
        class: StringOrTypeSig<'a>,
    },
    ArrayLength {
        dest: Register,
        array: Register,
    },
    NewInstance {
        dest: Register,
        class: StringOrTypeSig<'a>,
    },
    NewArray {
        dest: Register,
        size_reg: Register,
        class: StringOrTypeSig<'a>,
    },
    FilledNewArray {
        registers: Vec<Register>,
        class: StringOrTypeSig<'a>,
    },
    FilledNewArrayRange {
        registers: RegisterRange,
        class: StringOrTypeSig<'a>,
    },
    FillArrayData {
        reg: Register,
        offset: Label<'a>,
    },
    Throw {
        src: Register,
    },
    Switch {
        switch_type: SwitchType,
        reg: Register,
        offset: Label<'a>,
    },
    Unused {
        opcode: u8,
    },
}

impl fmt::Display for DexOp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DexOp::Invoke {
                invoke_type,
                registers,
                range,
                method,
                call_site,
                proto,
            } => {
                let regs_str = if let Some(range) = range {
                    format!("{range}")
                } else {
                    let regs: Vec<String> = registers.iter().map(|r| r.to_string()).collect();
                    format!("{{{}}}", regs.join(", "))
                };

                match invoke_type {
                    InvokeType::Polymorphic | InvokeType::PolymorphicRange => {
                        write!(
                            f,
                            "{} {}, {}, {}",
                            invoke_type,
                            regs_str,
                            method.as_ref().unwrap(),
                            proto.as_ref().unwrap()
                        )
                    }
                    InvokeType::Custom | InvokeType::CustomRange => {
                        write!(
                            f,
                            "{} {}, {}",
                            invoke_type,
                            regs_str,
                            call_site.as_ref().unwrap()
                        )
                    }
                    _ => {
                        write!(
                            f,
                            "{} {}, {}",
                            invoke_type,
                            regs_str,
                            method.as_ref().unwrap()
                        )
                    }
                }
            }
            DexOp::Const {
                const_type,
                dest,
                value,
            } => {
                write!(f, "{const_type} {dest}, {value}")
            }
            DexOp::MoveTwoReg {
                move_type,
                dest,
                src,
            } => write!(f, "{move_type} {dest}, {src}"),
            DexOp::MoveOneReg { move_type, dest } => write!(f, "{move_type} {dest}"),
            DexOp::Return { return_type, src } => {
                if let Some(src_reg) = src {
                    write!(f, "{return_type} {src_reg}")
                } else {
                    write!(f, "{return_type}")
                }
            }
            DexOp::DynamicFieldAccess {
                access_type,
                value_type,
                reg,
                object,
                field,
            } => {
                write!(f, "{access_type}-{value_type} {reg}, {object}, {field}")
            }
            DexOp::StaticFieldAccess {
                access_type,
                value_type,
                reg,
                field,
            } => {
                write!(f, "{access_type}-{value_type} {reg}, {field}")
            }
            DexOp::Arith {
                arith_type,
                operand_type,
                dest,
                src1,
                src2,
            } => {
                write!(f, "{arith_type}-{operand_type} {dest}, {src1}, {src2}")
            }
            DexOp::ArithUnary {
                arith_type,
                operand_type,
                dest,
                src,
            } => {
                write!(f, "{arith_type}-{operand_type} {dest}, {src}")
            }
            DexOp::Arith2Addr {
                arith_type,
                operand_type,
                dest,
                src,
            } => {
                write!(f, "{arith_type}-{operand_type} {dest}, {src}")
            }
            DexOp::Condition {
                cond_type,
                reg1,
                offset,
            } => {
                write!(f, "{cond_type} {reg1}, {offset}")
            }
            DexOp::TwoRegCondition {
                cond_type,
                reg1,
                reg2,
                offset,
            } => {
                write!(f, "{cond_type} {reg1}, {reg2}, {offset}")
            }
            DexOp::Goto { goto_type, offset } => {
                write!(f, "{goto_type} {offset}")
            }
            DexOp::ConstLiteral {
                const_type,
                dest,
                value,
            } => {
                write!(f, "{const_type} {dest}, {value}")
            }
            DexOp::LitArith8 {
                arith_type,
                dest,
                src,
                literal,
            } => {
                write!(f, "{arith_type} {dest}, {src}, {literal}")
            }
            DexOp::LitArith16 {
                arith_type,
                dest,
                src,
                literal,
            } => {
                write!(f, "{arith_type} {dest}, {src}, {literal}")
            }
            DexOp::Convert {
                convert_type,
                dest,
                src,
            } => {
                write!(f, "{convert_type} {dest}, {src}")
            }
            DexOp::ArrayAccess {
                access_type,
                value_type,
                reg,
                arr,
                idx,
            } => {
                if let ArrayValueType::Normal = *value_type {
                    write!(f, "{access_type} {reg}, {arr}, {idx}")
                } else {
                    write!(f, "{access_type}-{value_type} {reg}, {arr}, {idx}")
                }
            }
            DexOp::Cmp {
                cmp_type,
                dest,
                src1,
                src2,
            } => {
                write!(f, "{cmp_type} {dest}, {src1}, {src2}")
            }
            DexOp::Switch {
                switch_type,
                reg,
                offset,
            } => {
                write!(f, "{switch_type} {reg}, {offset}")
            }
            DexOp::Nop => write!(f, "nop"),
            DexOp::MonitorEnter { src } => write!(f, "monitor-enter {src}"),
            DexOp::MonitorExit { src } => write!(f, "monitor-exit {src}"),
            DexOp::CheckCast { dest, class } => write!(f, "check-cast {dest}, {class}"),
            DexOp::InstanceOf { dest, src, class } => {
                write!(f, "instance-of {dest}, {src}, {class}")
            }
            DexOp::ArrayLength { dest, array } => {
                write!(f, "array-length {dest}, {array}")
            }
            DexOp::NewInstance { dest, class } => {
                write!(f, "new-instance {dest}, {class}")
            }
            DexOp::NewArray {
                dest,
                size_reg,
                class,
            } => write!(f, "new-array {dest}, {size_reg}, {class}"),
            DexOp::FilledNewArray { registers, class } => {
                let regs: Vec<String> = registers.iter().map(|r| r.to_string()).collect();
                write!(f, "filled-new-array {{{}}}, {}", regs.join(", "), class)
            }
            DexOp::FilledNewArrayRange { registers, class } => {
                write!(f, "filled-new-array/range {registers}, {class}")
            }
            DexOp::FillArrayData { reg, offset } => {
                write!(f, "fill-array-data {reg}, {offset}")
            }
            DexOp::Throw { src } => write!(f, "throw {src}"),
            DexOp::Unused { opcode } => write!(f, "unused {opcode}"),
        }
    }
}

/// Parse a register reference like "v0" or "p1", returning its number.
pub fn parse_register<'a>() -> impl ModalParser<&'a str, Register, InputError<&'a str>> {
    ws(
        (alt((one_of('v'), one_of('p'))), digit1).try_map(|(t, o): (char, &str)| {
            let num = o.parse::<u16>().map_err(|_| InputError::at(o))?;
            Ok::<Register, InputError<&str>>(match t {
                'v' => Register::Local(num),
                'p' => Register::Parameter(num),
                _ => unreachable!(),
            })
        }),
    )
}

/// Parse a comma-separated list of registers inside curly braces.
fn parse_register_list<'a>() -> impl ModalParser<&'a str, Vec<Register>, InputError<&'a str>> {
    delimited(
        ws(one_of('{')),
        separated(0.., parse_register(), ws(one_of(','))),
        ws(one_of('}')),
    )
}

fn parse_const_high16<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (parse_register(), ws(one_of(',')), parse_int_lit::<i32>()).map(|(dest, _, value32)| {
            let value = (value32 >> 16) as i64;
            DexOp::ConstLiteral {
                const_type: ConstLiteralType::ConstHigh16,
                dest,
                value: ConstLiteralValue::ConstHigh16(value),
            }
        }),
    )
}

fn parse_const_wide_high16<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (parse_register(), ws(one_of(',')), parse_int_lit::<i64>()).map(|(dest, _, value64)| {
            let value = value64 >> 48;
            DexOp::ConstLiteral {
                const_type: ConstLiteralType::ConstWideHigh16,
                dest,
                value: ConstLiteralValue::ConstWideHigh16(value),
            }
        }),
    )
}

/// Parses a register range enclosed in braces, e.g. "{v0 .. v6}".
/// Returns a tuple (first_reg, last_reg)
fn parse_register_range<'a>() -> impl ModalParser<&'a str, RegisterRange, InputError<&'a str>> {
    delimited(
        ws(one_of('{')),
        (
            terminated(parse_register(), literal("..")),
            parse_register(),
        )
            .map(|(start, end)| RegisterRange { start, end }),
        ws(one_of('}')),
    )
}

fn parse_invoke_polymorphic<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (
            parse_register_list(),
            delimited(ws(one_of(',')), parse_method_ref(), ws(one_of(','))),
            alphanumeric1,
        )
            .map(|(registers, method, proto)| DexOp::Invoke {
                invoke_type: InvokeType::Polymorphic,
                registers,
                range: None,
                method: Some(Box::new(method)),
                call_site: None,
                proto: Some(Cow::Borrowed(proto)),
            }),
    )
}

fn parse_invoke_polymorphic_range<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
{
    preceded(
        space1,
        (
            parse_register_range(),
            delimited(ws(one_of(',')), parse_method_ref(), ws(one_of(','))),
            alphanumeric1,
        )
            .map(|(range, method, proto)| DexOp::Invoke {
                invoke_type: InvokeType::PolymorphicRange,
                registers: Vec::new(),
                range: Some(range),
                method: Some(Box::new(method)),
                call_site: None,
                proto: Some(Cow::Borrowed(proto)),
            }),
    )
}

fn parse_invoke_custom<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (parse_register_list(), ws(one_of(',')), alphanumeric1).map(|(registers, _, call_site)| {
            DexOp::Invoke {
                invoke_type: InvokeType::Custom,
                registers,
                range: None,
                method: None,
                call_site: Some(Cow::Borrowed(call_site)),
                proto: None,
            }
        }),
    )
}

fn parse_invoke_custom_range<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (parse_register_range(), ws(one_of(',')), alphanumeric1).map(|(range, _, call_site)| {
            DexOp::Invoke {
                invoke_type: InvokeType::CustomRange,
                registers: Vec::new(),
                range: Some(range),
                method: None,
                call_site: Some(Cow::Borrowed(call_site)),
                proto: None,
            }
        }),
    )
}

fn parse_invoke<'a>(
    invoke_type: InvokeType,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    (
        terminated(parse_register_list(), ws(one_of(','))),
        parse_method_ref(),
    )
        .map(move |(registers, method)| DexOp::Invoke {
            invoke_type,
            registers,
            range: None,
            method: Some(Box::new(method)),
            call_site: None,
            proto: None,
        })
}

fn parse_invoke_range<'a>(
    invoke_type: InvokeType,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    (
        terminated(parse_register_range(), ws(one_of(','))),
        parse_method_ref(),
    )
        .map(move |(range, method)| DexOp::Invoke {
            invoke_type,
            registers: Vec::new(),
            range: Some(range),
            method: Some(Box::new(method)),
            call_site: None,
            proto: None,
        })
}

fn parse_one_reg_op<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register) -> DexOp<'a>,
{
    parse_register().map(constructor)
}

/// Helper function: it consumes a space, then a register, then a comma (with optional spaces), then another register.
fn parse_two_reg_op<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register) -> DexOp<'a>,
{
    (
        terminated(parse_register(), ws(one_of(','))),
        parse_register(),
    )
        .map(move |(r1, r2)| constructor(r1, r2))
}

/// Helper function: parses three registers from the input.
/// It expects at least one space, then a register, a comma, another register,
/// a comma, and a third register.
fn parse_three_reg_op<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register, Register) -> DexOp<'a>,
{
    (
        parse_register(),
        delimited(ws(one_of(',')), parse_register(), ws(one_of(','))),
        parse_register(),
    )
        .map(move |(r1, r2, r3)| constructor(r1, r2, r3))
}

/// Helper for one-reg + literal operations.
/// It assumes the opcode has already been consumed.
fn parse_one_reg_and_literal<'a, T, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    T: num_traits::Num + std::ops::Neg<Output = T> + std::str::FromStr + TryFrom<i64> + 'a,
    F: Fn(Register, T) -> DexOp<'a>,
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
{
    (
        terminated(parse_register(), ws(one_of(','))),
        parse_int_lit::<T>(),
    )
        .map(move |(reg, literal)| constructor(reg, literal))
}

/// Helper for two-reg + literal operations.
fn parse_two_reg_and_literal<'a, T, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    T: num_traits::Num + std::ops::Neg<Output = T> + std::str::FromStr + TryFrom<i64> + 'a,
    F: Fn(Register, Register, T) -> DexOp<'a>,
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
{
    (
        parse_register(),
        delimited(ws(one_of(',')), parse_register(), ws(one_of(','))),
        parse_int_lit::<T>(),
    )
        .map(move |(r1, r2, literal)| constructor(r1, r2, literal))
}

fn parse_one_reg_and_fieldref<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, FieldRef) -> DexOp + 'a,
{
    (
        terminated(parse_register(), ws(one_of(','))),
        parse_field_ref(),
    )
        .map(move |(dest, field)| constructor(dest, field))
}

fn parse_two_reg_and_fieldref<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register, FieldRef) -> DexOp + 'a,
{
    (
        parse_register(),
        delimited(ws(one_of(',')), parse_register(), ws(one_of(','))),
        parse_field_ref(),
    )
        .map(move |(reg1, reg2, field)| constructor(reg1, reg2, field))
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringOrTypeSig<'a> {
    String(Cow<'a, str>),
    TypeSig(TypeSignature<'a>),
}

impl fmt::Display for StringOrTypeSig<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Prepend a colon when printing
        match &self {
            Self::String(s) => {
                write!(f, "\"{s}\"")
            }
            Self::TypeSig(ts) => {
                write!(f, "{ts}")
            }
        }
    }
}

/// Helper for one-reg + literal operations.
/// It assumes the opcode has already been consumed.
fn parse_one_reg_and_string<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, StringOrTypeSig<'a>) -> DexOp<'a>,
{
    (
        terminated(parse_register(), ws(one_of(','))),
        alt((
            parse_string_lit().map(|s| StringOrTypeSig::String(Cow::Borrowed(s))),
            parse_typesignature().map(StringOrTypeSig::TypeSig),
        )),
    )
        .map(move |(reg, literal)| constructor(reg, literal))
}

fn parse_two_reg_and_string<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register, StringOrTypeSig<'a>) -> DexOp<'a>,
{
    (
        parse_register(),
        delimited(ws(one_of(',')), parse_register(), ws(one_of(','))),
        alt((
            parse_string_lit().map(|s| StringOrTypeSig::String(Cow::Borrowed(s))),
            parse_typesignature().map(StringOrTypeSig::TypeSig),
        )),
    )
        .map(move |(reg1, reg2, literal)| constructor(reg1, reg2, literal))
}

fn parse_one_reg_and_label<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Label<'a>) -> DexOp<'a>,
{
    (terminated(parse_register(), ws(one_of(','))), parse_label())
        .map(move |(reg, label)| constructor(reg, label))
}

fn parse_two_reg_and_label<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register, Label<'a>) -> DexOp<'a>,
{
    preceded(
        space1,
        (
            parse_register(),
            delimited(ws(one_of(',')), parse_register(), ws(one_of(','))),
            parse_label(),
        )
            .map(move |(reg1, reg2, label)| constructor(reg1, reg2, label)),
    )
}

// Higher level parser for all operations
// Higher level parser for all operations
pub fn parse_dex_op<'a>(input: &mut &'a str) -> ModalResult<DexOp<'a>, InputError<&'a str>> {
    let op =
        take_while(1.., |c: char| c.is_alphanumeric() || c == '-' || c == '/').parse_next(input)?;

    // Handle ungrouped operations first
    let op_result = match op {
        "nop" => return Ok(DexOp::Nop),
        "monitor-enter" => parse_one_reg_op(|src| DexOp::MonitorEnter { src }).parse_next(input)?,
        "monitor-exit" => parse_one_reg_op(|src| DexOp::MonitorExit { src }).parse_next(input)?,
        "check-cast" => parse_one_reg_and_string(|dest, class| DexOp::CheckCast { dest, class })
            .parse_next(input)?,
        "instance-of" => {
            parse_two_reg_and_string(|dest, src, class| DexOp::InstanceOf { dest, src, class })
                .parse_next(input)?
        }
        "array-length" => {
            parse_two_reg_op(|dest, array| DexOp::ArrayLength { dest, array }).parse_next(input)?
        }
        "new-instance" => {
            parse_one_reg_and_string(|dest, class| DexOp::NewInstance { dest, class })
                .parse_next(input)?
        }
        "new-array" => parse_two_reg_and_string(|dest, size_reg, class| DexOp::NewArray {
            dest,
            size_reg,
            class,
        })
        .parse_next(input)?,
        "filled-new-array" => preceded(
            space1,
            (
                parse_register_list(),
                ws(one_of(',')),
                parse_typesignature(),
            )
                .map(|(registers, _, class)| DexOp::FilledNewArray {
                    registers,
                    class: StringOrTypeSig::TypeSig(class),
                }),
        )
        .parse_next(input)?,
        "filled-new-array/range" => preceded(
            space1,
            (
                parse_register_range(),
                ws(one_of(',')),
                parse_typesignature(),
            )
                .map(|(registers, _, class)| DexOp::FilledNewArrayRange {
                    registers,
                    class: StringOrTypeSig::TypeSig(class),
                }),
        )
        .parse_next(input)?,
        "fill-array-data" => {
            parse_one_reg_and_label(|reg, offset| DexOp::FillArrayData { reg, offset })
                .parse_next(input)?
        }
        "throw" => parse_one_reg_op(|src| DexOp::Throw { src }).parse_next(input)?,
        _ => {
            if let Ok(invoke_type) = InvokeType::from_str(op) {
                match invoke_type {
                    InvokeType::Polymorphic => parse_invoke_polymorphic().parse_next(input)?,
                    InvokeType::PolymorphicRange => {
                        parse_invoke_polymorphic_range().parse_next(input)?
                    }
                    InvokeType::Custom => parse_invoke_custom().parse_next(input)?,
                    InvokeType::CustomRange => parse_invoke_custom_range().parse_next(input)?,
                    _ => {
                        if invoke_type.is_range() {
                            parse_invoke_range(invoke_type).parse_next(input)?
                        } else {
                            parse_invoke(invoke_type).parse_next(input)?
                        }
                    }
                }
            } else if let Ok(const_type) = ConstType::from_str(op) {
                parse_one_reg_and_string(|dest, value| DexOp::Const {
                    const_type,
                    dest,
                    value,
                })
                .parse_next(input)?
            } else if let Ok(move_type) = TwoRegMoveType::from_str(op) {
                parse_two_reg_op(|dest, src| DexOp::MoveTwoReg {
                    move_type,
                    dest,
                    src,
                })
                .parse_next(input)?
            } else if let Ok(move_type) = OneRegMoveType::from_str(op) {
                parse_one_reg_op(|dest| DexOp::MoveOneReg { move_type, dest }).parse_next(input)?
            } else if let Ok(return_type) = ReturnType::from_str(op) {
                if let ReturnType::Void = return_type {
                    DexOp::Return {
                        return_type,
                        src: None,
                    }
                } else {
                    parse_one_reg_op(|src| DexOp::Return {
                        return_type,
                        src: Some(src),
                    })
                    .parse_next(input)?
                }
            } else if let Ok(cond_type) = ConditionType::from_str(op) {
                parse_one_reg_and_label(|reg1, offset| DexOp::Condition {
                    cond_type,
                    reg1,
                    offset,
                })
                .parse_next(input)?
            } else if let Ok(cond_type) = TwoRegConditionType::from_str(op) {
                parse_two_reg_and_label(|reg1, reg2, offset| DexOp::TwoRegCondition {
                    cond_type,
                    reg1,
                    reg2,
                    offset,
                })
                .parse_next(input)?
            } else if let Ok(goto_type) = GotoType::from_str(op) {
                parse_label()
                    .map(|offset| DexOp::Goto { goto_type, offset })
                    .parse_next(input)?
            } else if let Ok(const_type) = ConstLiteralType::from_str(op) {
                match const_type {
                    ConstLiteralType::Const => {
                        parse_one_reg_and_literal::<i32, _>(|dest, value| DexOp::ConstLiteral {
                            const_type,
                            dest,
                            value: ConstLiteralValue::Const(value),
                        })
                        .parse_next(input)?
                    }
                    ConstLiteralType::Const4 => {
                        parse_one_reg_and_literal::<i8, _>(|dest, value| DexOp::ConstLiteral {
                            const_type,
                            dest,
                            value: ConstLiteralValue::Const4(value),
                        })
                        .parse_next(input)?
                    }
                    ConstLiteralType::Const16 => {
                        parse_one_reg_and_literal::<i16, _>(|dest, value| DexOp::ConstLiteral {
                            const_type,
                            dest,
                            value: ConstLiteralValue::Const16(value),
                        })
                        .parse_next(input)?
                    }
                    ConstLiteralType::ConstWide => {
                        parse_one_reg_and_literal::<i64, _>(|dest, value| DexOp::ConstLiteral {
                            const_type,
                            dest,
                            value: ConstLiteralValue::ConstWide(value),
                        })
                        .parse_next(input)?
                    }
                    ConstLiteralType::ConstWide16 => {
                        parse_one_reg_and_literal::<i16, _>(|dest, value| DexOp::ConstLiteral {
                            const_type,
                            dest,
                            value: ConstLiteralValue::ConstWide16(value),
                        })
                        .parse_next(input)?
                    }
                    ConstLiteralType::ConstWide32 => {
                        parse_one_reg_and_literal::<i32, _>(|dest, value| DexOp::ConstLiteral {
                            const_type,
                            dest,
                            value: ConstLiteralValue::ConstWide32(value),
                        })
                        .parse_next(input)?
                    }
                    ConstLiteralType::ConstHigh16 => parse_const_high16().parse_next(input)?,
                    ConstLiteralType::ConstWideHigh16 => {
                        parse_const_wide_high16().parse_next(input)?
                    }
                }
            } else if let Ok(arith_type) = LitArithType8::from_str(op) {
                parse_two_reg_and_literal::<i8, _>(|dest, src, literal| DexOp::LitArith8 {
                    arith_type,
                    dest,
                    src,
                    literal,
                })
                .parse_next(input)?
            } else if let Ok(arith_type) = LitArithType16::from_str(op) {
                parse_two_reg_and_literal::<i16, _>(|dest, src, literal| DexOp::LitArith16 {
                    arith_type,
                    dest,
                    src,
                    literal,
                })
                .parse_next(input)?
            } else if let Ok(convert_type) = ConvertType::from_str(op) {
                parse_two_reg_op(|dest, src| DexOp::Convert {
                    convert_type,
                    dest,
                    src,
                })
                .parse_next(input)?
            } else if let Ok(cmp_type) = CmpType::from_str(op) {
                parse_three_reg_op(|dest, src1, src2| DexOp::Cmp {
                    cmp_type,
                    dest,
                    src1,
                    src2,
                })
                .parse_next(input)?
            } else if let Ok(switch_type) = SwitchType::from_str(op) {
                parse_one_reg_and_label(|reg, offset| DexOp::Switch {
                    switch_type,
                    reg,
                    offset,
                })
                .parse_next(input)?
            } else {
                let (t, v) = op.split_once('-').unwrap_or((op, ""));

                if let Ok(arith_type) = ArithUnaryType::from_str(t) {
                    let operand_type = ArithOperandType::from_str(v)
                        .map_err(|_| ErrMode::Backtrack(InputError::at(*input)))?;

                    parse_two_reg_op(|dest, src| DexOp::ArithUnary {
                        arith_type,
                        operand_type,
                        dest,
                        src,
                    })
                    .parse_next(input)?
                } else if let Ok(arith_type) = ArithType::from_str(t) {
                    if let Ok(operand_type) = ArithOperandType::from_str(v) {
                        parse_three_reg_op(|dest, src1, src2| DexOp::Arith {
                            arith_type,
                            operand_type,
                            dest,
                            src1,
                            src2,
                        })
                        .parse_next(input)?
                    } else {
                        let operand_type = ArithOperand2AddrType::from_str(v)
                            .map_err(|_| ErrMode::Backtrack(InputError::at(*input)))?;
                        parse_two_reg_op(|dest, src| DexOp::Arith2Addr {
                            arith_type,
                            operand_type,
                            dest,
                            src,
                        })
                        .parse_next(input)?
                    }
                } else if let Ok(access_type) = ArrayAccessType::from_str(t) {
                    let value_type = ArrayValueType::from_str(v)
                        .map_err(|_| ErrMode::Backtrack(InputError::at(*input)))?;
                    parse_three_reg_op(|reg, arr, idx| DexOp::ArrayAccess {
                        access_type,
                        value_type,
                        reg,
                        arr,
                        idx,
                    })
                    .parse_next(input)?
                } else if let Ok(access_type) = DynamicFieldAccessType::from_str(t) {
                    let value_type = FieldValueType::from_str(v)
                        .map_err(|_| ErrMode::Backtrack(InputError::at(*input)))?;
                    parse_two_reg_and_fieldref(move |reg, object, field| {
                        DexOp::DynamicFieldAccess {
                            access_type,
                            value_type,
                            reg,
                            object,
                            field,
                        }
                    })
                    .parse_next(input)?
                } else if let Ok(access_type) = StaticFieldAccessType::from_str(t) {
                    let value_type = FieldValueType::from_str(v)
                        .map_err(|_| ErrMode::Backtrack(InputError::at(*input)))?;
                    parse_one_reg_and_fieldref(move |reg, field| DexOp::StaticFieldAccess {
                        access_type,
                        value_type,
                        reg,
                        field,
                    })
                    .parse_next(input)?
                } else {
                    return Err(ErrMode::Backtrack(InputError::at(*input)));
                }
            }
        }
    };

    Ok(op_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_string() {
        let mut input = r#"const-string v0, "builder""#;
        let instr = parse_dex_op(&mut input).unwrap();
        assert_eq!(
            instr,
            DexOp::Const {
                const_type: ConstType::String,
                dest: Register::Local(0),
                value: StringOrTypeSig::String(Cow::Borrowed("builder"))
            }
        );
    }

    #[test]
    fn test_parse_method_ref() {
        let mut input = r#"Landroidx/core/content/res/TypedArrayUtils;->getNamedString(Landroid/content/res/TypedArray;Lorg/xmlpull/v1/XmlPullParser;Ljava/lang/String;I)Ljava/lang/String;"#;
        let _ = parse_method_ref().parse_next(&mut input).unwrap();
    }

    #[test]
    fn test_invoke_direct() {
        let mut input = r#"invoke-direct {p0}, Ljava/lang/Object;-><init>()V"#;
        let _ = parse_dex_op(&mut input).unwrap();
    }

    #[test]
    fn test_invoke_interface() {
        let mut input = "invoke-interface/range {v6 .. v12}, Lzpf;->a(JIIILxpf;)V";
        let _ = parse_dex_op(&mut input).unwrap();
    }

    #[test]
    fn test_parse_literal_int() {
        let i: i8 = parse_int_lit().parse_next(&mut "-0x5").unwrap();
        assert_eq!(i, -5);

        let i: i8 = parse_int_lit().parse_next(&mut "50").unwrap();
        assert_eq!(i, 50);

        let i: i8 = parse_int_lit().parse_next(&mut "5\n").unwrap();
        assert_eq!(i, 5);

        let i: i16 = parse_int_lit().parse_next(&mut "-0x7c05").unwrap();
        assert_eq!(i, -0x7c05);

        let i: i32 = parse_int_lit().parse_next(&mut "0x7fffffff").unwrap();
        assert_eq!(i, 0x7fffffff);

        let i: i32 = parse_int_lit().parse_next(&mut "-0x80000000").unwrap();
        assert_eq!(i, -0x80000000);

        let i: i32 = parse_int_lit().parse_next(&mut "-0x80000000").unwrap();
        let sixteen: i16 = (i >> 16) as i16;
        assert_eq!(sixteen, -0x8000);
    }

    #[test]
    fn test_filled_new_array() {
        let mut input = "filled-new-array {v0, v1}, Ljava/lang/String;";
        let instr = parse_dex_op(&mut input).unwrap();
        assert_eq!(
            instr,
            DexOp::FilledNewArray {
                registers: vec![Register::Local(0), Register::Local(1)],
                class: StringOrTypeSig::TypeSig(
                    parse_typesignature()
                        .parse_next(&mut "Ljava/lang/String;")
                        .unwrap()
                )
            }
        );
    }

    #[test]
    fn test_filled_new_array_range() {
        let mut input = "filled-new-array/range {v0 .. v2}, [I";
        let instr = parse_dex_op(&mut input).unwrap();
        assert_eq!(
            instr,
            DexOp::FilledNewArrayRange {
                registers: RegisterRange {
                    start: Register::Local(0),
                    end: Register::Local(2)
                },
                class: StringOrTypeSig::TypeSig(
                    parse_typesignature().parse_next(&mut "[I").unwrap()
                )
            }
        );
    }
}
