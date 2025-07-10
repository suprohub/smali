//! TODO:
//! What about multispace0 and space0? Where i can set this?
//! Is there any .smali grammar docs? Like c++

use std::{
    borrow::Cow,
    fmt::{self, Debug},
};

use winnow::{
    ModalParser, ModalResult, Parser,
    ascii::{alphanumeric1, digit1, space0, space1},
    combinator::{alt, delimited, preceded, separated},
    error::InputError,
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

#[derive(Debug, Clone, PartialEq)]
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

/// A high-level representation of a DEX operation.
///
/// This enum “lifts” many opcodes so that literal values and symbolic references
/// (e.g. for strings, classes, methods, fields, call sites, prototypes) are stored
/// directly rather than as indices.
#[derive(Debug, Clone, PartialEq)]
pub enum DexOp<'a> {
    // Group A: constants, moves, returns, etc.
    ConstString {
        dest: Register,
        value: StringOrTypeSig<'a>,
    },
    ConstStringJumbo {
        dest: Register,
        value: StringOrTypeSig<'a>,
    },
    Nop,
    Move {
        dest: Register,
        src: Register,
    },
    MoveFrom16 {
        dest: Register,
        src: Register,
    },
    Move16 {
        dest: Register,
        src: Register,
    },
    MoveWide {
        dest: Register,
        src: Register,
    },
    MoveWideFrom16 {
        dest: Register,
        src: Register,
    },
    MoveWide16 {
        dest: Register,
        src: Register,
    },
    MoveObject {
        dest: Register,
        src: Register,
    },
    MoveObjectFrom16 {
        dest: Register,
        src: Register,
    },
    MoveObject16 {
        dest: Register,
        src: Register,
    },
    MoveResult {
        dest: Register,
    },
    MoveResultWide {
        dest: Register,
    },
    MoveResultObject {
        dest: Register,
    },
    MoveException {
        dest: Register,
    },
    ReturnVoid,
    Return {
        src: Register,
    },
    ReturnWide {
        src: Register,
    },
    ReturnObject {
        src: Register,
    },
    Const4 {
        dest: Register,
        value: i8,
    },
    Const16 {
        dest: Register,
        value: i16,
    },
    Const {
        dest: Register,
        value: i32,
    },
    ConstHigh16 {
        dest: Register,
        value: i16,
    },
    ConstWide16 {
        dest: Register,
        value: i16,
    },
    ConstWide32 {
        dest: Register,
        value: i32,
    },
    ConstWide {
        dest: Register,
        value: i64,
    },
    ConstWideHigh16 {
        dest: Register,
        value: i16,
    },
    ConstClass {
        dest: Register,
        class: StringOrTypeSig<'a>,
    },
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
    Goto {
        offset: Label<'a>,
    },
    Goto16 {
        offset: Label<'a>,
    },
    Goto32 {
        offset: Label<'a>,
    },
    PackedSwitch {
        reg: Register,
        offset: Label<'a>,
    },
    SparseSwitch {
        reg: Register,
        offset: Label<'a>,
    },
    CmplFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    CmpgFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    CmplDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    CmpgDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    CmpLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },

    // Group B: Array, field, and invocation operations.
    AGet {
        dest: Register,
        array: Register,
        index: Register,
    },
    AGetWide {
        dest: Register,
        array: Register,
        index: Register,
    },
    AGetObject {
        dest: Register,
        array: Register,
        index: Register,
    },
    AGetBoolean {
        dest: Register,
        array: Register,
        index: Register,
    },
    AGetByte {
        dest: Register,
        array: Register,
        index: Register,
    },
    AGetChar {
        dest: Register,
        array: Register,
        index: Register,
    },
    AGetShort {
        dest: Register,
        array: Register,
        index: Register,
    },
    APut {
        src: Register,
        array: Register,
        index: Register,
    },
    APutWide {
        src: Register,
        array: Register,
        index: Register,
    },
    APutObject {
        src: Register,
        array: Register,
        index: Register,
    },
    APutBoolean {
        src: Register,
        array: Register,
        index: Register,
    },
    APutByte {
        src: Register,
        array: Register,
        index: Register,
    },
    APutChar {
        src: Register,
        array: Register,
        index: Register,
    },
    APutShort {
        src: Register,
        array: Register,
        index: Register,
    },
    IGet {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IGetWide {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IGetObject {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IGetBoolean {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IGetByte {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IGetChar {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IGetShort {
        dest: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPut {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPutWide {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPutObject {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPutBoolean {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPutByte {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPutChar {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    IPutShort {
        src: Register,
        object: Register,
        field: FieldRef<'a>,
    },
    SGet {
        dest: Register,
        field: FieldRef<'a>,
    },
    SGetWide {
        dest: Register,
        field: FieldRef<'a>,
    },
    SGetObject {
        dest: Register,
        field: FieldRef<'a>,
    },
    SGetBoolean {
        dest: Register,
        field: FieldRef<'a>,
    },
    SGetByte {
        dest: Register,
        field: FieldRef<'a>,
    },
    SGetChar {
        dest: Register,
        field: FieldRef<'a>,
    },
    SGetShort {
        dest: Register,
        field: FieldRef<'a>,
    },
    SPut {
        src: Register,
        field: FieldRef<'a>,
    },
    SPutWide {
        src: Register,
        field: FieldRef<'a>,
    },
    SPutObject {
        src: Register,
        field: FieldRef<'a>,
    },
    SPutBoolean {
        src: Register,
        field: FieldRef<'a>,
    },
    SPutByte {
        src: Register,
        field: FieldRef<'a>,
    },
    SPutChar {
        src: Register,
        field: FieldRef<'a>,
    },
    SPutShort {
        src: Register,
        field: FieldRef<'a>,
    },
    InvokeVirtual {
        registers: Vec<Register>,
        method: MethodRef<'a>,
    },
    InvokeSuper {
        registers: Vec<Register>,
        method: MethodRef<'a>,
    },
    InvokeInterface {
        registers: Vec<Register>,
        method: MethodRef<'a>,
    },
    InvokeVirtualRange {
        range: RegisterRange,
        method: MethodRef<'a>,
    },
    InvokeSuperRange {
        range: RegisterRange,
        method: MethodRef<'a>,
    },
    InvokeDirectRange {
        range: RegisterRange,
        method: MethodRef<'a>,
    },
    InvokeStaticRange {
        range: RegisterRange,
        method: MethodRef<'a>,
    },
    InvokeInterfaceRange {
        range: RegisterRange,
        method: MethodRef<'a>,
    },
    InvokeDirect {
        registers: Vec<Register>,
        method: MethodRef<'a>,
    },
    InvokeStatic {
        registers: Vec<Register>,
        method: MethodRef<'a>,
    },

    // Group C: Arithmetic operations (non-2addr).
    AddInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    SubInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    MulInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    DivInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    RemInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    AndInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    OrInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    XorInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    ShlInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    ShrInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    UshrInt {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    AddLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    SubLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    MulLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    DivLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    RemLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    AndLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    OrLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    XorLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    ShlLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    ShrLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    UshrLong {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    AddFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    SubFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    MulFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    DivFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    RemFloat {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    AddDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    SubDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    MulDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    DivDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },
    RemDouble {
        dest: Register,
        src1: Register,
        src2: Register,
    },

    // Group D: Arithmetic operations (2addr variants).
    AddInt2Addr {
        reg: Register,
        src: Register,
    },
    SubInt2Addr {
        reg: Register,
        src: Register,
    },
    MulInt2Addr {
        reg: Register,
        src: Register,
    },
    DivInt2Addr {
        reg: Register,
        src: Register,
    },
    RemInt2Addr {
        reg: Register,
        src: Register,
    },
    AndInt2Addr {
        reg: Register,
        src: Register,
    },
    OrInt2Addr {
        reg: Register,
        src: Register,
    },
    XorInt2Addr {
        reg: Register,
        src: Register,
    },
    ShlInt2Addr {
        reg: Register,
        src: Register,
    },
    ShrInt2Addr {
        reg: Register,
        src: Register,
    },
    UshrInt2Addr {
        reg: Register,
        src: Register,
    },
    AddLong2Addr {
        reg: Register,
        src: Register,
    },
    SubLong2Addr {
        reg: Register,
        src: Register,
    },
    MulLong2Addr {
        reg: Register,
        src: Register,
    },
    DivLong2Addr {
        reg: Register,
        src: Register,
    },
    RemLong2Addr {
        reg: Register,
        src: Register,
    },
    AndLong2Addr {
        reg: Register,
        src: Register,
    },
    OrLong2Addr {
        reg: Register,
        src: Register,
    },
    XorLong2Addr {
        reg: Register,
        src: Register,
    },
    ShlLong2Addr {
        reg: Register,
        src: Register,
    },
    ShrLong2Addr {
        reg: Register,
        src: Register,
    },
    UshrLong2Addr {
        reg: Register,
        src: Register,
    },
    AddFloat2Addr {
        reg: Register,
        src: Register,
    },
    SubFloat2Addr {
        reg: Register,
        src: Register,
    },
    MulFloat2Addr {
        reg: Register,
        src: Register,
    },
    DivFloat2Addr {
        reg: Register,
        src: Register,
    },
    RemFloat2Addr {
        reg: Register,
        src: Register,
    },
    AddDouble2Addr {
        reg: Register,
        src: Register,
    },
    SubDouble2Addr {
        reg: Register,
        src: Register,
    },
    MulDouble2Addr {
        reg: Register,
        src: Register,
    },
    DivDouble2Addr {
        reg: Register,
        src: Register,
    },
    RemDouble2Addr {
        reg: Register,
        src: Register,
    },

    // Additional conversion operations:
    IntToByte {
        dest: Register,
        src: Register,
    },
    IntToChar {
        dest: Register,
        src: Register,
    },
    IntToShort {
        dest: Register,
        src: Register,
    },

    // Literal arithmetic operations using lit8 encoding:
    AddIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    RSubIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    MulIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    DivIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    RemIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    AndIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    OrIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    XorIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    ShlIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    ShrIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },
    UshrIntLit8 {
        dest: Register,
        src: Register,
        literal: i8,
    },

    // Conditional combinator operations now using SmaliRegister:
    IfEq {
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    IfNe {
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    IfLt {
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    IfGe {
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    IfGt {
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    IfLe {
        reg1: Register,
        reg2: Register,
        offset: Label<'a>,
    },
    IfEqz {
        reg: Register,
        offset: Label<'a>,
    },
    IfNez {
        reg: Register,
        offset: Label<'a>,
    },
    IfLtz {
        reg: Register,
        offset: Label<'a>,
    },
    IfGez {
        reg: Register,
        offset: Label<'a>,
    },
    IfGtz {
        reg: Register,
        offset: Label<'a>,
    },
    IfLez {
        reg: Register,
        offset: Label<'a>,
    },

    // Arithmetic operations:
    NegInt {
        dest: Register,
        src: Register,
    },
    NotInt {
        dest: Register,
        src: Register,
    },
    NegLong {
        dest: Register,
        src: Register,
    },
    NotLong {
        dest: Register,
        src: Register,
    },
    NegFloat {
        dest: Register,
        src: Register,
    },
    NegDouble {
        dest: Register,
        src: Register,
    },

    // Conversion operations added to the DexOp enum:
    IntToLong {
        dest: Register,
        src: Register,
    },
    IntToFloat {
        dest: Register,
        src: Register,
    },
    IntToDouble {
        dest: Register,
        src: Register,
    },
    LongToInt {
        dest: Register,
        src: Register,
    },
    LongToFloat {
        dest: Register,
        src: Register,
    },
    LongToDouble {
        dest: Register,
        src: Register,
    },
    FloatToInt {
        dest: Register,
        src: Register,
    },
    FloatToLong {
        dest: Register,
        src: Register,
    },
    FloatToDouble {
        dest: Register,
        src: Register,
    },
    DoubleToInt {
        dest: Register,
        src: Register,
    },
    DoubleToLong {
        dest: Register,
        src: Register,
    },
    DoubleToFloat {
        dest: Register,
        src: Register,
    },

    AddIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    RSubIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    MulIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    DivIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    RemIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    AndIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    OrIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },
    XorIntLit16 {
        dest: Register,
        src: Register,
        literal: i16,
    },

    // Group E: Polymorphic, custom and method handle/type constants.
    InvokePolymorphic {
        registers: Vec<Register>,
        method: MethodRef<'a>,
        proto: Cow<'a, str>,
    },
    InvokePolymorphicRange {
        range: RegisterRange,
        method: MethodRef<'a>,
        proto: Cow<'a, str>,
    },
    InvokeCustom {
        registers: Vec<Register>,
        call_site: Cow<'a, str>,
    },
    InvokeCustomRange {
        range: RegisterRange,
        call_site: Cow<'a, str>,
    },
    ConstMethodHandle {
        dest: Register,
        method_handle: StringOrTypeSig<'a>,
    },
    ConstMethodType {
        dest: Register,
        proto: StringOrTypeSig<'a>,
    },
    Unused {
        opcode: u8,
    },
}

impl fmt::Display for DexOp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Group A
            DexOp::ConstString { dest, value } => {
                write!(f, "const-string {dest}, \"{value}\"")
            }
            DexOp::ConstStringJumbo { dest, value } => {
                write!(f, "const-string/jumbo {dest}, \"{value}\"")
            }
            DexOp::Nop => write!(f, "nop"),
            DexOp::Move { dest, src } => write!(f, "move {dest}, {src}"),
            DexOp::MoveFrom16 { dest, src } => write!(f, "move/from16 {dest}, {src}"),
            DexOp::Move16 { dest, src } => write!(f, "move/16 {dest} , {src}"),
            DexOp::MoveWide { dest, src } => write!(f, "move-wide {dest}, {src}"),
            DexOp::MoveWideFrom16 { dest, src } => {
                write!(f, "move-wide/from16 {dest}, {src}")
            }
            DexOp::MoveWide16 { dest, src } => write!(f, "move-wide/16 {dest} , {src}"),
            DexOp::MoveObject { dest, src } => write!(f, "move-object {dest}, {src}"),
            DexOp::MoveObjectFrom16 { dest, src } => {
                write!(f, "move-object/from16 {dest}, {src}")
            }
            DexOp::MoveObject16 { dest, src } => {
                write!(f, "move-object/16 {dest} , {src}")
            }
            DexOp::MoveResult { dest } => write!(f, "move-result {dest}"),
            DexOp::MoveResultWide { dest } => write!(f, "move-result-wide {dest}"),
            DexOp::MoveResultObject { dest } => write!(f, "move-result-object {dest}"),
            DexOp::MoveException { dest } => write!(f, "move-exception {dest}"),
            DexOp::ReturnVoid => write!(f, "return-void"),
            DexOp::Return { src } => write!(f, "return {src}"),
            DexOp::ReturnWide { src } => write!(f, "return-wide {src}"),
            DexOp::ReturnObject { src } => write!(f, "return-object {src}"),
            DexOp::Const4 { dest, value } => write!(f, "const/4 {dest}, {value}"),
            DexOp::Const16 { dest, value } => write!(f, "const/16 {dest}, {value}"),
            DexOp::Const { dest, value } => write!(f, "const {dest}, {value}"),
            DexOp::ConstHigh16 { dest, value } => {
                write!(f, "const/high16 {dest}, 0x{value:0x}0000")
            }
            DexOp::ConstWide16 { dest, value } => {
                write!(f, "const-wide/16 {dest}, {value}")
            }
            DexOp::ConstWide32 { dest, value } => {
                write!(f, "const-wide/32 {dest}, {value}")
            }
            DexOp::ConstWide { dest, value } => {
                write!(f, "const-wide {dest}, 0x{value:0x}L")
            }
            DexOp::ConstWideHigh16 { dest, value } => {
                write!(f, "const-wide/high16 {dest}, 0x{value:0x}000000000000L")
            }
            DexOp::ConstClass { dest, class } => write!(f, "const-class {dest}, {class}"),
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
                let regs: Vec<String> = registers.iter().map(|r| format!("{r}")).collect();
                write!(f, "filled-new-array {{{}}}, {}", regs.join(", "), class)
            }
            DexOp::FilledNewArrayRange { registers, class } => {
                write!(f, "filled-new-array/range {registers}, {class}")
            }
            DexOp::FillArrayData { reg, offset } => {
                write!(f, "fill-array-data {reg}, {offset}")
            }
            DexOp::Throw { src } => write!(f, "throw {src}"),
            DexOp::Goto { offset } => write!(f, "goto {offset}"),
            DexOp::Goto16 { offset } => write!(f, "goto/16 {offset}"),
            DexOp::Goto32 { offset } => write!(f, "goto/32 {offset}"),
            DexOp::PackedSwitch { reg, offset } => {
                write!(f, "packed-switch {reg}, {offset}")
            }
            DexOp::SparseSwitch { reg, offset } => {
                write!(f, "sparse-switch {reg}, {offset}")
            }
            DexOp::CmplFloat { dest, src1, src2 } => {
                write!(f, "cmpl-float {dest}, {src1}, {src2}")
            }
            DexOp::CmpgFloat { dest, src1, src2 } => {
                write!(f, "cmpg-float {dest}, {src1}, {src2}")
            }
            DexOp::CmplDouble { dest, src1, src2 } => {
                write!(f, "cmpl-double {dest}, {src1}, {src2}")
            }
            DexOp::CmpgDouble { dest, src1, src2 } => {
                write!(f, "cmpg-double {dest}, {src1}, {src2}")
            }
            DexOp::CmpLong { dest, src1, src2 } => {
                write!(f, "cmp-long {dest}, {src1}, {src2}")
            }
            // Group B: Array, field and invocation operations.
            DexOp::AGet { dest, array, index } => {
                write!(f, "aget {dest}, {array}, {index}")
            }
            DexOp::AGetWide { dest, array, index } => {
                write!(f, "aget-wide {dest}, {array}, {index}")
            }
            DexOp::AGetObject { dest, array, index } => {
                write!(f, "aget-object {dest}, {array}, {index}")
            }
            DexOp::AGetBoolean { dest, array, index } => {
                write!(f, "aget-boolean {dest}, {array}, {index}")
            }
            DexOp::AGetByte { dest, array, index } => {
                write!(f, "aget-byte {dest}, {array}, {index}")
            }
            DexOp::AGetChar { dest, array, index } => {
                write!(f, "aget-char {dest}, {array}, {index}")
            }
            DexOp::AGetShort { dest, array, index } => {
                write!(f, "aget-short {dest}, {array}, {index}")
            }
            DexOp::APut { src, array, index } => write!(f, "aput {src}, {array}, {index}"),
            DexOp::APutWide { src, array, index } => {
                write!(f, "aput-wide {src}, {array}, {index}")
            }
            DexOp::APutObject { src, array, index } => {
                write!(f, "aput-object {src}, {array}, {index}")
            }
            DexOp::APutBoolean { src, array, index } => {
                write!(f, "aput-boolean {src}, {array}, {index}")
            }
            DexOp::APutByte { src, array, index } => {
                write!(f, "aput-byte {src}, {array}, {index}")
            }
            DexOp::APutChar { src, array, index } => {
                write!(f, "aput-char {src}, {array}, {index}")
            }
            DexOp::APutShort { src, array, index } => {
                write!(f, "aput-short {src}, {array}, {index}")
            }
            DexOp::IGet {
                dest,
                object,
                field,
            } => write!(f, "iget {dest}, {object}, {field}"),
            DexOp::IGetWide {
                dest,
                object,
                field,
            } => write!(f, "iget-wide {dest}, {object}, {field}"),
            DexOp::IGetObject {
                dest,
                object,
                field,
            } => write!(f, "iget-object {dest}, {object}, {field}"),
            DexOp::IGetBoolean {
                dest,
                object,
                field,
            } => write!(f, "iget-boolean {dest}, {object}, {field}"),
            DexOp::IGetByte {
                dest,
                object,
                field,
            } => write!(f, "iget-byte {dest}, {object}, {field}"),
            DexOp::IGetChar {
                dest,
                object,
                field,
            } => write!(f, "iget-char {dest}, {object}, {field}"),
            DexOp::IGetShort {
                dest,
                object,
                field,
            } => write!(f, "iget-short {dest}, {object}, {field}"),
            DexOp::IPut { src, object, field } => {
                write!(f, "iput {src}, {object}, {field}")
            }
            DexOp::IPutWide { src, object, field } => {
                write!(f, "iput-wide {src}, {object}, {field}")
            }
            DexOp::IPutObject { src, object, field } => {
                write!(f, "iput-object {src}, {object}, {field}")
            }
            DexOp::IPutBoolean { src, object, field } => {
                write!(f, "iput-boolean {src}, {object}, {field}")
            }
            DexOp::IPutByte { src, object, field } => {
                write!(f, "iput-byte {src}, {object}, {field}")
            }
            DexOp::IPutChar { src, object, field } => {
                write!(f, "iput-char {src}, {object}, {field}")
            }
            DexOp::IPutShort { src, object, field } => {
                write!(f, "iput-short {src}, {object}, {field}")
            }
            DexOp::SGet { dest, field } => write!(f, "sget {dest}, {field}"),
            DexOp::SGetWide { dest, field } => write!(f, "sget-wide {dest}, {field}"),
            DexOp::SGetObject { dest, field } => write!(f, "sget-object {dest}, {field}"),
            DexOp::SGetBoolean { dest, field } => {
                write!(f, "sget-boolean {dest}, {field}")
            }
            DexOp::SGetByte { dest, field } => write!(f, "sget-byte {dest}, {field}"),
            DexOp::SGetChar { dest, field } => write!(f, "sget-char {dest}, {field}"),
            DexOp::SGetShort { dest, field } => write!(f, "sget-short {dest}, {field}"),
            DexOp::SPut { src, field } => write!(f, "sput {src}, {field}"),
            DexOp::SPutWide { src, field } => write!(f, "sput-wide {src}, {field}"),
            DexOp::SPutObject { src, field } => write!(f, "sput-object {src}, {field}"),
            DexOp::SPutBoolean { src, field } => write!(f, "sput-boolean {src}, {field}"),
            DexOp::SPutByte { src, field } => write!(f, "sput-byte {src}, {field}"),
            DexOp::SPutChar { src, field } => write!(f, "sput-char {src}, {field}"),
            DexOp::SPutShort { src, field } => write!(f, "sput-short {src}, {field}"),
            DexOp::InvokeVirtual { registers, method } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-virtual {{{regs}}}, {method}")
            }
            DexOp::InvokeSuper { registers, method } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-super {{{regs}}}, {method}")
            }
            DexOp::InvokeInterface { registers, method } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-interface {{{regs}}}, {method}")
            }
            DexOp::InvokeVirtualRange { range, method } => {
                write!(f, "invoke-virtual/range {range}, {method}")
            }
            DexOp::InvokeSuperRange { range, method } => {
                write!(f, "invoke-super/range {range}, {method}")
            }
            DexOp::InvokeDirectRange { range, method } => {
                write!(f, "invoke-direct/range {range}, {method}")
            }
            DexOp::InvokeStaticRange { range, method } => {
                write!(f, "invoke-static/range {range}, {method}")
            }
            DexOp::InvokeInterfaceRange { range, method } => {
                write!(f, "invoke-interface/range {range}, {method}")
            }

            // Group C: Arithmetic (non-2addr)
            DexOp::AddInt { dest, src1, src2 } => {
                write!(f, "add-int {dest}, {src1}, {src2}")
            }
            DexOp::SubInt { dest, src1, src2 } => {
                write!(f, "sub-int {dest}, {src1}, {src2}")
            }
            DexOp::MulInt { dest, src1, src2 } => {
                write!(f, "mul-int {dest}, {src1}, {src2}")
            }
            DexOp::DivInt { dest, src1, src2 } => {
                write!(f, "div-int {dest}, {src1}, {src2}")
            }
            DexOp::RemInt { dest, src1, src2 } => {
                write!(f, "rem-int {dest}, {src1}, {src2}")
            }
            DexOp::AndInt { dest, src1, src2 } => {
                write!(f, "and-int {dest}, {src1}, {src2}")
            }
            DexOp::OrInt { dest, src1, src2 } => {
                write!(f, "or-int {dest}, {src1}, {src2}")
            }
            DexOp::XorInt { dest, src1, src2 } => {
                write!(f, "xor-int {dest}, {src1}, {src2}")
            }
            DexOp::ShlInt { dest, src1, src2 } => {
                write!(f, "shl-int {dest}, {src1}, {src2}")
            }
            DexOp::ShrInt { dest, src1, src2 } => {
                write!(f, "shr-int {dest}, {src1}, {src2}")
            }
            DexOp::UshrInt { dest, src1, src2 } => {
                write!(f, "ushr-int {dest}, {src1}, {src2}")
            }
            DexOp::AddLong { dest, src1, src2 } => {
                write!(f, "add-long {dest}, {src1}, {src2}")
            }
            DexOp::SubLong { dest, src1, src2 } => {
                write!(f, "sub-long {dest}, {src1}, {src2}")
            }
            DexOp::MulLong { dest, src1, src2 } => {
                write!(f, "mul-long {dest}, {src1}, {src2}")
            }
            DexOp::DivLong { dest, src1, src2 } => {
                write!(f, "div-long {dest}, {src1}, {src2}")
            }
            DexOp::RemLong { dest, src1, src2 } => {
                write!(f, "rem-long {dest}, {src1}, {src2}")
            }
            DexOp::AndLong { dest, src1, src2 } => {
                write!(f, "and-long {dest}, {src1}, {src2}")
            }
            DexOp::OrLong { dest, src1, src2 } => {
                write!(f, "or-long {dest}, {src1}, {src2}")
            }
            DexOp::XorLong { dest, src1, src2 } => {
                write!(f, "xor-long {dest}, {src1}, {src2}")
            }
            DexOp::ShlLong { dest, src1, src2 } => {
                write!(f, "shl-long {dest}, {src1}, {src2}")
            }
            DexOp::ShrLong { dest, src1, src2 } => {
                write!(f, "shr-long {dest}, {src1}, {src2}")
            }
            DexOp::UshrLong { dest, src1, src2 } => {
                write!(f, "ushr-long {dest}, {src1}, {src2}")
            }
            DexOp::AddFloat { dest, src1, src2 } => {
                write!(f, "add-float {dest}, {src1}, {src2}")
            }
            DexOp::SubFloat { dest, src1, src2 } => {
                write!(f, "sub-float {dest}, {src1}, {src2}")
            }
            DexOp::MulFloat { dest, src1, src2 } => {
                write!(f, "mul-float {dest}, {src1}, {src2}")
            }
            DexOp::DivFloat { dest, src1, src2 } => {
                write!(f, "div-float {dest}, {src1}, {src2}")
            }
            DexOp::RemFloat { dest, src1, src2 } => {
                write!(f, "rem-float {dest}, {src1}, {src2}")
            }
            DexOp::AddDouble { dest, src1, src2 } => {
                write!(f, "add-double {dest}, {src1}, {src2}")
            }
            DexOp::SubDouble { dest, src1, src2 } => {
                write!(f, "sub-double {dest}, {src1}, {src2}")
            }
            DexOp::MulDouble { dest, src1, src2 } => {
                write!(f, "mul-double {dest}, {src1}, {src2}")
            }
            DexOp::DivDouble { dest, src1, src2 } => {
                write!(f, "div-double {dest}, {src1}, {src2}")
            }
            DexOp::RemDouble { dest, src1, src2 } => {
                write!(f, "rem-double {dest}, {src1}, {src2}")
            }

            // Group D: 2addr arithmetic operations.
            DexOp::AddInt2Addr { reg, src } => write!(f, "add-int/2addr {reg}, {src}"),
            DexOp::SubInt2Addr { reg, src } => write!(f, "sub-int/2addr {reg}, {src}"),
            DexOp::MulInt2Addr { reg, src } => write!(f, "mul-int/2addr {reg}, {src}"),
            DexOp::DivInt2Addr { reg, src } => write!(f, "div-int/2addr {reg}, {src}"),
            DexOp::RemInt2Addr { reg, src } => write!(f, "rem-int/2addr {reg}, {src}"),
            DexOp::AndInt2Addr { reg, src } => write!(f, "and-int/2addr {reg}, {src}"),
            DexOp::OrInt2Addr { reg, src } => write!(f, "or-int/2addr {reg}, {src}"),
            DexOp::XorInt2Addr { reg, src } => write!(f, "xor-int/2addr {reg}, {src}"),
            DexOp::ShlInt2Addr { reg, src } => write!(f, "shl-int/2addr {reg}, {src}"),
            DexOp::ShrInt2Addr { reg, src } => write!(f, "shr-int/2addr {reg}, {src}"),
            DexOp::UshrInt2Addr { reg, src } => write!(f, "ushr-int/2addr {reg}, {src}"),
            DexOp::AddLong2Addr { reg, src } => write!(f, "add-long/2addr {reg}, {src}"),
            DexOp::SubLong2Addr { reg, src } => write!(f, "sub-long/2addr {reg}, {src}"),
            DexOp::MulLong2Addr { reg, src } => write!(f, "mul-long/2addr {reg}, {src}"),
            DexOp::DivLong2Addr { reg, src } => write!(f, "div-long/2addr {reg}, {src}"),
            DexOp::RemLong2Addr { reg, src } => write!(f, "rem-long/2addr {reg}, {src}"),
            DexOp::AndLong2Addr { reg, src } => write!(f, "and-long/2addr {reg}, {src}"),
            DexOp::OrLong2Addr { reg, src } => write!(f, "or-long/2addr {reg}, {src}"),
            DexOp::XorLong2Addr { reg, src } => write!(f, "xor-long/2addr {reg}, {src}"),
            DexOp::ShlLong2Addr { reg, src } => write!(f, "shl-long/2addr {reg}, {src}"),
            DexOp::ShrLong2Addr { reg, src } => write!(f, "shr-long/2addr {reg}, {src}"),
            DexOp::UshrLong2Addr { reg, src } => write!(f, "ushr-long/2addr {reg}, {src}"),
            DexOp::AddFloat2Addr { reg, src } => write!(f, "add-float/2addr {reg}, {src}"),
            DexOp::SubFloat2Addr { reg, src } => write!(f, "sub-float/2addr {reg}, {src}"),
            DexOp::MulFloat2Addr { reg, src } => write!(f, "mul-float/2addr {reg}, {src}"),
            DexOp::DivFloat2Addr { reg, src } => write!(f, "div-float/2addr {reg}, {src}"),
            DexOp::RemFloat2Addr { reg, src } => write!(f, "rem-float/2addr {reg}, {src}"),
            DexOp::AddDouble2Addr { reg, src } => {
                write!(f, "add-double/2addr {reg}, {src}")
            }
            DexOp::SubDouble2Addr { reg, src } => {
                write!(f, "sub-double/2addr {reg}, {src}")
            }
            DexOp::MulDouble2Addr { reg, src } => {
                write!(f, "mul-double/2addr {reg}, {src}")
            }
            DexOp::DivDouble2Addr { reg, src } => {
                write!(f, "div-double/2addr {reg}, {src}")
            }
            DexOp::RemDouble2Addr { reg, src } => {
                write!(f, "rem-double/2addr {reg}, {src}")
            }
            // Group E: Polymorphic, custom and method handle/type constants.
            DexOp::InvokePolymorphic {
                registers,
                method,
                proto,
            } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-polymorphic {{{regs}}}, {method}, {proto}")
            }
            DexOp::InvokePolymorphicRange {
                range,
                method,
                proto,
            } => {
                write!(f, "invoke-polymorphic/range {range}, {method}, {proto}")
            }
            DexOp::InvokeCustom {
                registers,
                call_site,
            } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-custom {{{regs}}}, {call_site}")
            }
            DexOp::InvokeCustomRange { range, call_site } => {
                write!(f, "invoke-custom/range {range}, {call_site}")
            }
            DexOp::ConstMethodHandle {
                dest,
                method_handle,
            } => write!(f, "const-method-handle {dest}, {method_handle}"),
            DexOp::ConstMethodType { dest, proto } => {
                write!(f, "const-method-type {dest}, {proto}")
            }

            // Conditional combinatores:
            DexOp::IfEq { reg1, reg2, offset } => {
                write!(f, "if-eq {reg1}, {reg2}, {offset}")
            }
            DexOp::IfNe { reg1, reg2, offset } => {
                write!(f, "if-ne {reg1}, {reg2}, {offset}")
            }
            DexOp::IfLt { reg1, reg2, offset } => {
                write!(f, "if-lt {reg1}, {reg2}, {offset}")
            }
            DexOp::IfGe { reg1, reg2, offset } => {
                write!(f, "if-ge {reg1}, {reg2}, {offset}")
            }
            DexOp::IfGt { reg1, reg2, offset } => {
                write!(f, "if-gt {reg1}, {reg2}, {offset}")
            }
            DexOp::IfLe { reg1, reg2, offset } => {
                write!(f, "if-le {reg1}, {reg2}, {offset}")
            }

            // Conditional combinator operations with a single register:
            DexOp::IfEqz { reg, offset } => write!(f, "if-eqz {reg}, {offset}"),
            DexOp::IfNez { reg, offset } => write!(f, "if-nez {reg}, {offset}"),
            DexOp::IfLtz { reg, offset } => write!(f, "if-ltz {reg}, {offset}"),
            DexOp::IfGez { reg, offset } => write!(f, "if-gez {reg}, {offset}"),
            DexOp::IfGtz { reg, offset } => write!(f, "if-gtz {reg}, {offset}"),
            DexOp::IfLez { reg, offset } => write!(f, "if-lez {reg}, {offset}"),

            // Invocation operations
            DexOp::InvokeDirect { registers, method } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-direct {{{regs}}}, {method}")
            }
            DexOp::InvokeStatic { registers, method } => {
                let regs = registers
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "invoke-static {{{regs}}}, {method}")
            }

            // Arithmetic operations
            DexOp::NegInt { dest, src } => write!(f, "neg-int {dest}, {src}"),
            DexOp::NotInt { dest, src } => write!(f, "not-int {dest}, {src}"),
            DexOp::NegLong { dest, src } => write!(f, "neg-long {dest}, {src}"),
            DexOp::NotLong { dest, src } => write!(f, "not-long {dest}, {src}"),
            DexOp::NegFloat { dest, src } => write!(f, "neg-float {dest}, {src}"),
            DexOp::NegDouble { dest, src } => write!(f, "neg-double {dest}, {src}"),

            // Conversion operations:
            DexOp::IntToLong { dest, src } => write!(f, "int-to-long {dest}, {src}"),
            DexOp::IntToFloat { dest, src } => write!(f, "int-to-float {dest}, {src}"),
            DexOp::IntToDouble { dest, src } => write!(f, "int-to-double {dest}, {src}"),
            DexOp::LongToInt { dest, src } => write!(f, "long-to-int {dest}, {src}"),
            DexOp::LongToFloat { dest, src } => write!(f, "long-to-float {dest}, {src}"),
            DexOp::LongToDouble { dest, src } => write!(f, "long-to-double {dest}, {src}"),
            DexOp::FloatToInt { dest, src } => write!(f, "float-to-int {dest}, {src}"),
            DexOp::FloatToLong { dest, src } => write!(f, "float-to-long {dest}, {src}"),
            DexOp::FloatToDouble { dest, src } => {
                write!(f, "float-to-double {dest}, {src}")
            }
            DexOp::DoubleToInt { dest, src } => write!(f, "double-to-int {dest}, {src}"),
            DexOp::DoubleToLong { dest, src } => write!(f, "double-to-long {dest}, {src}"),
            DexOp::DoubleToFloat { dest, src } => {
                write!(f, "double-to-float {dest}, {src}")
            }

            // Additional conversion variants:
            DexOp::IntToByte { dest, src } => write!(f, "int-to-byte {dest}, {src}"),
            DexOp::IntToChar { dest, src } => write!(f, "int-to-char {dest}, {src}"),
            DexOp::IntToShort { dest, src } => write!(f, "int-to-short {dest}, {src}"),

            // Arithmetic literal operations (example for int):
            DexOp::AddIntLit16 { dest, src, literal } => {
                write!(f, "add-int/lit16 {dest}, {src}, {literal}")
            }
            DexOp::RSubIntLit16 { dest, src, literal } => {
                write!(f, "rsub-int {dest}, {src}, {literal}")
            }
            DexOp::MulIntLit16 { dest, src, literal } => {
                write!(f, "mul-int/lit16 {dest}, {src}, {literal}")
            }
            DexOp::DivIntLit16 { dest, src, literal } => {
                write!(f, "div-int/lit16 {dest}, {src}, {literal}")
            }
            DexOp::RemIntLit16 { dest, src, literal } => {
                write!(f, "rem-int/lit16 {dest}, {src}, {literal}")
            }
            DexOp::AndIntLit16 { dest, src, literal } => {
                write!(f, "and-int/lit16 {dest}, {src}, {literal}")
            }
            DexOp::OrIntLit16 { dest, src, literal } => {
                write!(f, "or-int/lit16 {dest}, {src}, {literal}")
            }
            DexOp::XorIntLit16 { dest, src, literal } => {
                write!(f, "xor-int/lit16 {dest}, {src}, {literal}")
            }

            // Literal arithmetic operations (lit8 variants):
            DexOp::AddIntLit8 { dest, src, literal } => {
                write!(f, "add-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::RSubIntLit8 { dest, src, literal } => {
                write!(f, "rsub-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::MulIntLit8 { dest, src, literal } => {
                write!(f, "mul-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::DivIntLit8 { dest, src, literal } => {
                write!(f, "div-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::RemIntLit8 { dest, src, literal } => {
                write!(f, "rem-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::AndIntLit8 { dest, src, literal } => {
                write!(f, "and-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::OrIntLit8 { dest, src, literal } => {
                write!(f, "or-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::XorIntLit8 { dest, src, literal } => {
                write!(f, "xor-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::ShlIntLit8 { dest, src, literal } => {
                write!(f, "shl-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::ShrIntLit8 { dest, src, literal } => {
                write!(f, "shr-int/lit8 {dest}, {src}, {literal}")
            }
            DexOp::UshrIntLit8 { dest, src, literal } => {
                write!(f, "ushr-int/lit8 {dest}, {src}, {literal}")
            }

            // Unused - shouldn't come across this
            DexOp::Unused { .. } => {
                panic!("Attempted fmt display on Unused operation")
            }
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
        one_of('{'),
        separated(0.., parse_register(), (space0, one_of(','), space0)),
        one_of('}'),
    )
}

fn parse_const_high16<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_int_lit::<i32>(),
        )
            .map(|(dest, _, value32)| {
                let value = (value32 >> 16) as i16;
                DexOp::ConstHigh16 { dest, value }
            }),
    )
}

fn parse_const_wide_high16<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_int_lit::<i64>(),
        )
            .map(|(dest, _, value64)| {
                let value = (value64 >> 48) as i16;
                DexOp::ConstWideHigh16 { dest, value }
            }),
    )
}

/// Parses a register range enclosed in braces, e.g. "{v0 .. v6}".
/// Returns a tuple (first_reg, last_reg)
fn parse_register_range<'a>() -> impl ModalParser<&'a str, RegisterRange, InputError<&'a str>> {
    delimited(
        delimited(space0, one_of('{'), space0),
        (
            parse_register(),
            delimited(space0, literal(".."), space0),
            parse_register(),
        )
            .map(|(start, _, end)| RegisterRange { start, end }),
        delimited(space0, one_of('}'), space0),
    )
}

fn parse_invoke_polymorphic<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (
            parse_register_list(),
            delimited(space0, one_of(','), space0),
            parse_method_ref(),
            delimited(space0, one_of(','), space0),
            alphanumeric1,
        )
            .map(
                |(registers, _, method, _, proto)| DexOp::InvokePolymorphic {
                    registers,
                    method,
                    proto: Cow::Borrowed(proto),
                },
            ),
    )
}

fn parse_invoke_polymorphic_range<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
{
    preceded(
        space1,
        (
            parse_register_range(),
            delimited(space0, one_of(','), space0),
            parse_method_ref(),
            delimited(space0, one_of(','), space0),
            alphanumeric1,
        )
            .map(
                |(range, _, method, _, proto)| DexOp::InvokePolymorphicRange {
                    range,
                    method,
                    proto: Cow::Borrowed(proto),
                },
            ),
    )
}

fn parse_invoke_custom<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (
            parse_register_list(),
            delimited(space0, one_of(','), space0),
            alphanumeric1,
        )
            .map(|(registers, _, call_site)| DexOp::InvokeCustom {
                registers,
                call_site: Cow::Borrowed(call_site),
            }),
    )
}

fn parse_invoke_custom_range<'a>() -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>> {
    preceded(
        space1,
        (
            parse_register_range(),
            delimited(space0, one_of(','), space0),
            alphanumeric1,
        )
            .map(|(range, _, call_site)| DexOp::InvokeCustomRange {
                range,
                call_site: Cow::Borrowed(call_site),
            }),
    )
}

fn parse_invoke<'a, F>(constructor: F) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Vec<Register>, MethodRef<'a>) -> DexOp<'a>,
{
    preceded(
        space1,
        (
            parse_register_list(),
            delimited(space0, one_of(','), space0),
            parse_method_ref(),
        )
            .map(move |(registers, _, method)| constructor(registers, method)),
    )
}

macro_rules! invoke_case {
    ($variant:ident, $input:ident) => {
        parse_invoke(|regs, method| DexOp::$variant {
            registers: regs,
            method,
        })
        .parse_next(&mut $input)?
    };
}

fn parse_one_reg_op<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register) -> DexOp<'a>,
{
    preceded(space1, parse_register().map(constructor))
}
macro_rules! one_reg_case {
    ($variant:ident, $field:ident, $input:ident) => {
        parse_one_reg_op(|r| DexOp::$variant { $field: r }).parse_next(&mut $input)?
    };
}

/// Helper function: it consumes a space, then a register, then a comma (with optional spaces), then another register.
fn parse_two_reg_op<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register) -> DexOp<'a>,
{
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_register(),
        )
            .map(move |(r1, _, r2)| constructor(r1, r2)),
    )
}

/// Macro for two-register operations. You specify the variant name and the names of the fields.
macro_rules! two_reg_case {
    ($variant:ident, $field1:ident, $field2:ident, $input:ident) => {
        parse_two_reg_op(|r1, r2| DexOp::$variant {
            $field1: r1,
            $field2: r2,
        })
        .parse_next(&mut $input)?
    };
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
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_register(),
        )
            .map(move |(r1, _, r2, _, r3)| constructor(r1, r2, r3)),
    )
}

/// Macro for three-register operations.
/// You supply the enum variant and the field names for each register,
/// along with the input.
macro_rules! three_reg_case {
    ($variant:ident, $field1:ident, $field2:ident, $field3:ident, $input:ident) => {
        parse_three_reg_op(|r1, r2, r3| DexOp::$variant {
            $field1: r1,
            $field2: r2,
            $field3: r3,
        })
        .parse_next(&mut $input)?
    };
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
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_int_lit::<T>(),
        )
            .map(move |(reg, _, literal)| constructor(reg, literal)),
    )
}

macro_rules! one_reg_lit_case {
    ($variant:ident, $field:ident, $lit_ty:ty, $input:ident) => {
        parse_one_reg_and_literal::<$lit_ty, _>(|r, lit| DexOp::$variant {
            $field: r,
            value: lit,
        })
        .parse_next(&mut $input)?
    };
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
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_int_lit::<T>(),
        )
            .map(move |(r1, _, r2, _, literal)| constructor(r1, r2, literal)),
    )
}

macro_rules! two_reg_lit_case {
    ($variant:ident, $field1:ident, $field2:ident, $lit_ty:ty, $input:ident) => {
        parse_two_reg_and_literal::<$lit_ty, _>(|r1, r2, lit| DexOp::$variant {
            $field1: r1,
            $field2: r2,
            literal: lit,
        })
        .parse_next(&mut $input)?
    };
}

fn parse_one_reg_and_fieldref<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, FieldRef) -> DexOp + 'a,
{
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_field_ref(),
        )
            .map(move |(dest, _, field)| constructor(dest, field)),
    )
}

macro_rules! one_reg_fieldref_case {
    ($variant:ident, $reg:ident, $input:ident) => {
        parse_one_reg_and_fieldref(|reg, field| DexOp::$variant { $reg: reg, field })
            .parse_next(&mut $input)?
    };
}

fn parse_two_reg_and_fieldref<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register, FieldRef) -> DexOp + 'a,
{
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_field_ref(),
        )
            .map(move |(reg1, _, reg2, _, field)| constructor(reg1, reg2, field)),
    )
}

macro_rules! two_reg_fieldref_case {
    ($variant:ident, $reg1:ident, $input:ident) => {
        parse_two_reg_and_fieldref(|reg1, object, field| DexOp::$variant {
            $reg1: reg1,
            object,
            field,
        })
        .parse_next(&mut $input)?
    };
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
                write!(f, "{s}")
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
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            alt((
                parse_string_lit().map(|s| StringOrTypeSig::String(Cow::Borrowed(s))),
                parse_typesignature().map(StringOrTypeSig::TypeSig),
            )),
        )
            .map(move |(reg, _, literal)| constructor(reg, literal)),
    )
}

macro_rules! one_reg_string_case {
    ($variant:ident, $field:ident, $string:ident, $input:ident) => {
        parse_one_reg_and_string(|r, lit| DexOp::$variant {
            $field: r,
            $string: lit,
        })
        .parse_next(&mut $input)?
    };
}

fn parse_two_reg_and_string<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Register, StringOrTypeSig<'a>) -> DexOp<'a>,
{
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_register(),
            delimited(space0, one_of(','), space0),
            alt((
                parse_string_lit().map(|s| StringOrTypeSig::String(Cow::Borrowed(s))),
                parse_typesignature().map(StringOrTypeSig::TypeSig),
            )),
        )
            .map(move |(reg1, _, reg2, _, literal)| constructor(reg1, reg2, literal)),
    )
}

macro_rules! two_reg_string_case {
    ($variant:ident, $reg:ident, $string:ident, $input:ident) => {
        parse_two_reg_and_string(|dest, reg, lit| DexOp::$variant {
            dest,
            $reg: reg,
            $string: lit,
        })
        .parse_next(&mut $input)?
    };
}

fn parse_one_reg_and_label<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(Register, Label<'a>) -> DexOp<'a>,
{
    preceded(
        space1,
        (
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_label(),
        )
            .map(move |(reg, _, label)| constructor(reg, label)),
    )
}

macro_rules! one_reg_label_case {
    ($variant:ident, $input:ident) => {
        parse_one_reg_and_label(|reg, offset| DexOp::$variant { reg, offset })
            .parse_next(&mut $input)?
    };
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
            delimited(space0, one_of(','), space0),
            parse_register(),
            delimited(space0, one_of(','), space0),
            parse_label(),
        )
            .map(move |(reg1, _, reg2, _, label)| constructor(reg1, reg2, label)),
    )
}

macro_rules! two_reg_label_case {
    ($variant:ident, $input:ident) => {
        parse_two_reg_and_label(|reg1, reg2, offset| DexOp::$variant { reg1, reg2, offset })
            .parse_next(&mut $input)?
    };
}

fn parse_range_and_method<'a, F>(
    constructor: F,
) -> impl ModalParser<&'a str, DexOp<'a>, InputError<&'a str>>
where
    F: Fn(RegisterRange, MethodRef<'a>) -> DexOp<'a>,
{
    preceded(
        space1,
        (
            parse_register_range(),
            delimited(space0, one_of(','), space0),
            parse_method_ref(),
        )
            .map(move |(range, _, method)| constructor(range, method)),
    )
}

macro_rules! range_method_case {
    ($variant:ident, $input:ident) => {
        parse_range_and_method(|range, method| DexOp::$variant { range, method })
            .parse_next(&mut $input)?
    };
}

// Higher level parser for all operations
pub fn parse_dex_op<'a>(mut input: &mut &'a str) -> ModalResult<DexOp<'a>, InputError<&'a str>> {
    let op =
        take_while(1.., |c: char| c.is_alphanumeric() || c == '-' || c == '/').parse_next(input)?;
    let r = match op {
        // Invoke operations
        "invoke-static" => invoke_case!(InvokeStatic, input),
        "invoke-virtual" => invoke_case!(InvokeVirtual, input),
        "invoke-super" => invoke_case!(InvokeSuper, input),
        "invoke-interface" => invoke_case!(InvokeInterface, input),
        "invoke-direct" => invoke_case!(InvokeDirect, input),

        // One-register operations.
        "move-result" => one_reg_case!(MoveResult, dest, input),
        "move-result-wide" => one_reg_case!(MoveResultWide, dest, input),
        "move-result-object" => one_reg_case!(MoveResultObject, dest, input),
        "move-exception" => one_reg_case!(MoveException, dest, input),
        "return" => one_reg_case!(Return, src, input),
        "return-wide" => one_reg_case!(ReturnWide, src, input),
        "return-object" => one_reg_case!(ReturnObject, src, input),
        "monitor-enter" => one_reg_case!(MonitorEnter, src, input),
        "monitor-exit" => one_reg_case!(MonitorExit, src, input),
        "throw" => one_reg_case!(Throw, src, input),

        // Two register operations
        // Group A: Move operations.
        "move" => two_reg_case!(Move, dest, src, input),
        "move/from16" => two_reg_case!(MoveFrom16, dest, src, input),
        "move/16" => two_reg_case!(Move16, dest, src, input),
        "move-wide" => two_reg_case!(MoveWide, dest, src, input),
        "move-wide/from16" => two_reg_case!(MoveWideFrom16, dest, src, input),
        "move-wide/16" => two_reg_case!(MoveWide16, dest, src, input),
        "move-object" => two_reg_case!(MoveObject, dest, src, input),
        "move-object/from16" => two_reg_case!(MoveObjectFrom16, dest, src, input),
        "move-object/16" => two_reg_case!(MoveObject16, dest, src, input),
        // Group A: Array length.
        "array-length" => two_reg_case!(ArrayLength, dest, array, input),
        // Group A: Conversion operations.
        "int-to-byte" => two_reg_case!(IntToByte, dest, src, input),
        "int-to-char" => two_reg_case!(IntToChar, dest, src, input),
        "int-to-short" => two_reg_case!(IntToShort, dest, src, input),
        // Group A: Unary arithmetic operations.
        "neg-int" => two_reg_case!(NegInt, dest, src, input),
        "not-int" => two_reg_case!(NotInt, dest, src, input),
        "neg-long" => two_reg_case!(NegLong, dest, src, input),
        "not-long" => two_reg_case!(NotLong, dest, src, input),
        "neg-float" => two_reg_case!(NegFloat, dest, src, input),
        "neg-double" => two_reg_case!(NegDouble, dest, src, input),
        // Group C: Conversion arithmetic operations.
        "int-to-long" => two_reg_case!(IntToLong, dest, src, input),
        "int-to-float" => two_reg_case!(IntToFloat, dest, src, input),
        "int-to-double" => two_reg_case!(IntToDouble, dest, src, input),
        "long-to-int" => two_reg_case!(LongToInt, dest, src, input),
        "long-to-float" => two_reg_case!(LongToFloat, dest, src, input),
        "long-to-double" => two_reg_case!(LongToDouble, dest, src, input),
        "float-to-int" => two_reg_case!(FloatToInt, dest, src, input),
        "float-to-long" => two_reg_case!(FloatToLong, dest, src, input),
        "float-to-double" => two_reg_case!(FloatToDouble, dest, src, input),
        "double-to-int" => two_reg_case!(DoubleToInt, dest, src, input),
        "double-to-long" => two_reg_case!(DoubleToLong, dest, src, input),
        "double-to-float" => two_reg_case!(DoubleToFloat, dest, src, input),
        // Group D: 2addr arithmetic operations (using reg and src).
        "add-int/2addr" => two_reg_case!(AddInt2Addr, reg, src, input),
        "sub-int/2addr" => two_reg_case!(SubInt2Addr, reg, src, input),
        "mul-int/2addr" => two_reg_case!(MulInt2Addr, reg, src, input),
        "div-int/2addr" => two_reg_case!(DivInt2Addr, reg, src, input),
        "rem-int/2addr" => two_reg_case!(RemInt2Addr, reg, src, input),
        "and-int/2addr" => two_reg_case!(AndInt2Addr, reg, src, input),
        "or-int/2addr" => two_reg_case!(OrInt2Addr, reg, src, input),
        "xor-int/2addr" => two_reg_case!(XorInt2Addr, reg, src, input),
        "shl-int/2addr" => two_reg_case!(ShlInt2Addr, reg, src, input),
        "shr-int/2addr" => two_reg_case!(ShrInt2Addr, reg, src, input),
        "ushr-int/2addr" => two_reg_case!(UshrInt2Addr, reg, src, input),
        "add-long/2addr" => two_reg_case!(AddLong2Addr, reg, src, input),
        "sub-long/2addr" => two_reg_case!(SubLong2Addr, reg, src, input),
        "mul-long/2addr" => two_reg_case!(MulLong2Addr, reg, src, input),
        "div-long/2addr" => two_reg_case!(DivLong2Addr, reg, src, input),
        "rem-long/2addr" => two_reg_case!(RemLong2Addr, reg, src, input),
        "and-long/2addr" => two_reg_case!(AndLong2Addr, reg, src, input),
        "or-long/2addr" => two_reg_case!(OrLong2Addr, reg, src, input),
        "xor-long/2addr" => two_reg_case!(XorLong2Addr, reg, src, input),
        "shl-long/2addr" => two_reg_case!(ShlLong2Addr, reg, src, input),
        "shr-long/2addr" => two_reg_case!(ShrLong2Addr, reg, src, input),
        "ushr-long/2addr" => two_reg_case!(UshrLong2Addr, reg, src, input),
        "add-float/2addr" => two_reg_case!(AddFloat2Addr, reg, src, input),
        "sub-float/2addr" => two_reg_case!(SubFloat2Addr, reg, src, input),
        "mul-float/2addr" => two_reg_case!(MulFloat2Addr, reg, src, input),
        "div-float/2addr" => two_reg_case!(DivFloat2Addr, reg, src, input),
        "rem-float/2addr" => two_reg_case!(RemFloat2Addr, reg, src, input),
        "add-double/2addr" => two_reg_case!(AddDouble2Addr, reg, src, input),
        "sub-double/2addr" => two_reg_case!(SubDouble2Addr, reg, src, input),
        "mul-double/2addr" => two_reg_case!(MulDouble2Addr, reg, src, input),
        "div-double/2addr" => two_reg_case!(DivDouble2Addr, reg, src, input),
        "rem-double/2addr" => two_reg_case!(RemDouble2Addr, reg, src, input),

        // Three register operations
        // Group B: Array get/put operations.
        "aget" => three_reg_case!(AGet, dest, array, index, input),
        "aget-wide" => three_reg_case!(AGetWide, dest, array, index, input),
        "aget-object" => three_reg_case!(AGetObject, dest, array, index, input),
        "aget-boolean" => three_reg_case!(AGetBoolean, dest, array, index, input),
        "aget-byte" => three_reg_case!(AGetByte, dest, array, index, input),
        "aget-char" => three_reg_case!(AGetChar, dest, array, index, input),
        "aget-short" => three_reg_case!(AGetShort, dest, array, index, input),
        "aput" => three_reg_case!(APut, src, array, index, input),
        "aput-wide" => three_reg_case!(APutWide, src, array, index, input),
        "aput-object" => three_reg_case!(APutObject, src, array, index, input),
        "aput-boolean" => three_reg_case!(APutBoolean, src, array, index, input),
        "aput-byte" => three_reg_case!(APutByte, src, array, index, input),
        "aput-char" => three_reg_case!(APutChar, src, array, index, input),
        "aput-short" => three_reg_case!(APutShort, src, array, index, input),
        // Group C: Arithmetic operations (non-2addr and comparisons).
        "add-int" => three_reg_case!(AddInt, dest, src1, src2, input),
        "sub-int" => three_reg_case!(SubInt, dest, src1, src2, input),
        "mul-int" => three_reg_case!(MulInt, dest, src1, src2, input),
        "div-int" => three_reg_case!(DivInt, dest, src1, src2, input),
        "rem-int" => three_reg_case!(RemInt, dest, src1, src2, input),
        "and-int" => three_reg_case!(AndInt, dest, src1, src2, input),
        "or-int" => three_reg_case!(OrInt, dest, src1, src2, input),
        "xor-int" => three_reg_case!(XorInt, dest, src1, src2, input),
        "shl-int" => three_reg_case!(ShlInt, dest, src1, src2, input),
        "shr-int" => three_reg_case!(ShrInt, dest, src1, src2, input),
        "ushr-int" => three_reg_case!(UshrInt, dest, src1, src2, input),
        "add-long" => three_reg_case!(AddLong, dest, src1, src2, input),
        "sub-long" => three_reg_case!(SubLong, dest, src1, src2, input),
        "mul-long" => three_reg_case!(MulLong, dest, src1, src2, input),
        "div-long" => three_reg_case!(DivLong, dest, src1, src2, input),
        "rem-long" => three_reg_case!(RemLong, dest, src1, src2, input),
        "and-long" => three_reg_case!(AndLong, dest, src1, src2, input),
        "or-long" => three_reg_case!(OrLong, dest, src1, src2, input),
        "xor-long" => three_reg_case!(XorLong, dest, src1, src2, input),
        "shl-long" => three_reg_case!(ShlLong, dest, src1, src2, input),
        "shr-long" => three_reg_case!(ShrLong, dest, src1, src2, input),
        "ushr-long" => three_reg_case!(UshrLong, dest, src1, src2, input),
        "add-float" => three_reg_case!(AddFloat, dest, src1, src2, input),
        "sub-float" => three_reg_case!(SubFloat, dest, src1, src2, input),
        "mul-float" => three_reg_case!(MulFloat, dest, src1, src2, input),
        "div-float" => three_reg_case!(DivFloat, dest, src1, src2, input),
        "rem-float" => three_reg_case!(RemFloat, dest, src1, src2, input),
        "add-double" => three_reg_case!(AddDouble, dest, src1, src2, input),
        "sub-double" => three_reg_case!(SubDouble, dest, src1, src2, input),
        "mul-double" => three_reg_case!(MulDouble, dest, src1, src2, input),
        "div-double" => three_reg_case!(DivDouble, dest, src1, src2, input),
        "rem-double" => three_reg_case!(RemDouble, dest, src1, src2, input),
        // Comparison operations.
        "cmpl-float" => three_reg_case!(CmplFloat, dest, src1, src2, input),
        "cmpg-float" => three_reg_case!(CmpgFloat, dest, src1, src2, input),
        "cmpl-double" => three_reg_case!(CmplDouble, dest, src1, src2, input),
        "cmpg-double" => three_reg_case!(CmpgDouble, dest, src1, src2, input),
        "cmp-long" => three_reg_case!(CmpLong, dest, src1, src2, input),

        // One-register literal operations (constants):
        "const" => one_reg_lit_case!(Const, dest, i32, input),
        "const/4" => one_reg_lit_case!(Const4, dest, i8, input),
        "const/16" => one_reg_lit_case!(Const16, dest, i16, input),
        "const-wide" => one_reg_lit_case!(ConstWide, dest, i64, input),
        "const-wide/16" => one_reg_lit_case!(ConstWide16, dest, i16, input),
        "const-wide/32" => one_reg_lit_case!(ConstWide32, dest, i32, input),

        // Two-register literal operations (lit8):
        "add-int/lit8" => two_reg_lit_case!(AddIntLit8, dest, src, i8, input),
        "rsub-int/lit8" => two_reg_lit_case!(RSubIntLit8, dest, src, i8, input),
        "mul-int/lit8" => two_reg_lit_case!(MulIntLit8, dest, src, i8, input),
        "div-int/lit8" => two_reg_lit_case!(DivIntLit8, dest, src, i8, input),
        "rem-int/lit8" => two_reg_lit_case!(RemIntLit8, dest, src, i8, input),
        "and-int/lit8" => two_reg_lit_case!(AndIntLit8, dest, src, i8, input),
        "or-int/lit8" => two_reg_lit_case!(OrIntLit8, dest, src, i8, input),
        "xor-int/lit8" => two_reg_lit_case!(XorIntLit8, dest, src, i8, input),
        "shl-int/lit8" => two_reg_lit_case!(ShlIntLit8, dest, src, i8, input),
        "shr-int/lit8" => two_reg_lit_case!(ShrIntLit8, dest, src, i8, input),
        "ushr-int/lit8" => two_reg_lit_case!(UshrIntLit8, dest, src, i8, input),

        // Two-register literal operations (lit16):
        "add-int/lit16" => two_reg_lit_case!(AddIntLit16, dest, src, i16, input),
        "rsub-int" => two_reg_lit_case!(RSubIntLit16, dest, src, i16, input),
        "mul-int/lit16" => two_reg_lit_case!(MulIntLit16, dest, src, i16, input),
        "div-int/lit16" => two_reg_lit_case!(DivIntLit16, dest, src, i16, input),
        "rem-int/lit16" => two_reg_lit_case!(RemIntLit16, dest, src, i16, input),
        "and-int/lit16" => two_reg_lit_case!(AndIntLit16, dest, src, i16, input),
        "or-int/lit16" => two_reg_lit_case!(OrIntLit16, dest, src, i16, input),
        "xor-int/lit16" => two_reg_lit_case!(XorIntLit16, dest, src, i16, input),

        // One reg and field
        "sget" => one_reg_fieldref_case!(SGet, dest, input),
        "sget-wide" => one_reg_fieldref_case!(SGetWide, dest, input),
        "sget-object" => one_reg_fieldref_case!(SGetObject, dest, input),
        "sget-boolean" => one_reg_fieldref_case!(SGetBoolean, dest, input),
        "sget-byte" => one_reg_fieldref_case!(SGetByte, dest, input),
        "sget-char" => one_reg_fieldref_case!(SGetChar, dest, input),
        "sget-short" => one_reg_fieldref_case!(SGetShort, dest, input),
        "sput" => one_reg_fieldref_case!(SPut, src, input),
        "sput-wide" => one_reg_fieldref_case!(SPutWide, src, input),
        "sput-object" => one_reg_fieldref_case!(SPutObject, src, input),
        "sput-boolean" => one_reg_fieldref_case!(SPutBoolean, src, input),
        "sput-byte" => one_reg_fieldref_case!(SPutByte, src, input),
        "sput-char" => one_reg_fieldref_case!(SPutChar, src, input),
        "sput-short" => one_reg_fieldref_case!(SPutShort, src, input),

        // Two reg and field
        "iget" => two_reg_fieldref_case!(IGet, dest, input),
        "iget-wide" => two_reg_fieldref_case!(IGetWide, dest, input),
        "iget-object" => two_reg_fieldref_case!(IGetObject, dest, input),
        "iget-boolean" => two_reg_fieldref_case!(IGetBoolean, dest, input),
        "iget-byte" => two_reg_fieldref_case!(IGetByte, dest, input),
        "iget-char" => two_reg_fieldref_case!(IGetChar, dest, input),
        "iget-short" => two_reg_fieldref_case!(IGetShort, dest, input),
        "iput" => two_reg_fieldref_case!(IPut, src, input),
        "iput-wide" => two_reg_fieldref_case!(IPutWide, src, input),
        "iput-object" => two_reg_fieldref_case!(IPutObject, src, input),
        "iput-boolean" => two_reg_fieldref_case!(IPutBoolean, src, input),
        "iput-byte" => two_reg_fieldref_case!(IPutByte, src, input),
        "iput-char" => two_reg_fieldref_case!(IPutChar, src, input),
        "iput-short" => two_reg_fieldref_case!(IPutShort, src, input),

        // One reg & string
        "const-string" => one_reg_string_case!(ConstString, dest, value, input),
        "const-string/jumbo" => one_reg_string_case!(ConstStringJumbo, dest, value, input),
        "const-class" => one_reg_string_case!(ConstClass, dest, class, input),
        "check-cast" => one_reg_string_case!(CheckCast, dest, class, input),
        "new-instance" => one_reg_string_case!(NewInstance, dest, class, input),
        "const-method-handle" => {
            one_reg_string_case!(ConstMethodHandle, dest, method_handle, input)
        }
        "const-method-type" => one_reg_string_case!(ConstMethodType, dest, proto, input),

        // Two regs & string
        "instance-of" => two_reg_string_case!(InstanceOf, src, class, input),
        "new-array" => two_reg_string_case!(NewArray, size_reg, class, input),

        // Gotos = 1 label
        "goto" => preceded(space1, parse_label())
            .map(|offset| DexOp::Goto { offset })
            .parse_next(input)?,
        "goto/16" => preceded(space1, parse_label())
            .map(|offset| DexOp::Goto16 { offset })
            .parse_next(input)?,
        "goto/32" => preceded(space1, parse_label())
            .map(|offset| DexOp::Goto32 { offset })
            .parse_next(input)?,

        // One reg & label
        "if-eqz" => one_reg_label_case!(IfEqz, input),
        "if-nez" => one_reg_label_case!(IfNez, input),
        "if-ltz" => one_reg_label_case!(IfLtz, input),
        "if-gez" => one_reg_label_case!(IfGez, input),
        "if-gtz" => one_reg_label_case!(IfGtz, input),
        "if-lez" => one_reg_label_case!(IfLez, input),
        "packed-switch" => one_reg_label_case!(PackedSwitch, input),
        "sparse-switch" => one_reg_label_case!(SparseSwitch, input),
        "fill-array-data" => one_reg_label_case!(FillArrayData, input),

        // Arrays
        "filled-new-array" => preceded(
            space1,
            (
                parse_register_list(),
                delimited(space0, one_of(','), space0),
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
                delimited(space0, one_of(','), space0),
                parse_typesignature(),
            )
                .map(|(registers, _, class)| DexOp::FilledNewArrayRange {
                    registers,
                    class: StringOrTypeSig::TypeSig(class),
                }),
        )
        .parse_next(input)?,

        // Two regs & label
        "if-eq" => two_reg_label_case!(IfEq, input),
        "if-ne" => two_reg_label_case!(IfNe, input),
        "if-lt" => two_reg_label_case!(IfLt, input),
        "if-ge" => two_reg_label_case!(IfGe, input),
        "if-gt" => two_reg_label_case!(IfGt, input),
        "if-le" => two_reg_label_case!(IfLe, input),

        // Range and method
        "invoke-virtual/range" => range_method_case!(InvokeVirtualRange, input),
        "invoke-super/range" => range_method_case!(InvokeSuperRange, input),
        "invoke-direct/range" => range_method_case!(InvokeDirectRange, input),
        "invoke-static/range" => range_method_case!(InvokeStaticRange, input),
        "invoke-interface/range" => range_method_case!(InvokeInterfaceRange, input),

        // Oddities
        "invoke-polymorphic" => parse_invoke_polymorphic().parse_next(input)?,
        "invoke-polymorphic/range" => parse_invoke_polymorphic_range().parse_next(input)?,
        "invoke-custom" => parse_invoke_custom().parse_next(input)?,
        "invoke-custom/range" => parse_invoke_custom_range().parse_next(input)?,
        "const/high16" => parse_const_high16().parse_next(input)?,
        "const-wide/high16" => parse_const_wide_high16().parse_next(input)?,
        "nop" => DexOp::Nop,
        "return-void" => DexOp::ReturnVoid,

        _ => {
            panic!("Unhandled operation {op}")
        }
    };

    Ok(r)
}

#[cfg(test)]
mod tests {
    use crate::{
        object_identifier::parse_object_identifier,
        signature::method_signature::parse_method_parameter,
    };

    use super::*;

    #[test]
    fn test_const_string() {
        let mut input = r#"const-string v0, "builder""#;
        let instr = parse_dex_op(&mut input).unwrap();
        assert_eq!(
            instr,
            DexOp::ConstString {
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
    fn test_invoke_static() {
        let mut input = r#"invoke-static {p1, p2, v0, v1}, Landroidx/core/content/res/TypedArrayUtils;->getNamedString(Landroid/content/res/TypedArray;Lorg/xmlpull/v1/XmlPullParser;Ljava/lang/String;I)Ljava/lang/String;"#;
        let instr = parse_dex_op(&mut input).unwrap();
        let expected_method = MethodRef {
            class: parse_object_identifier().parse_next(&mut "Landroidx/core/content/res/TypedArrayUtils;").unwrap(),
            param: parse_method_parameter().parse_next(&mut "getNamedString(Landroid/content/res/TypedArray;Lorg/xmlpull/v1/XmlPullParser;Ljava/lang/String;I)Ljava/lang/String;").unwrap(),
        };
        assert_eq!(
            instr,
            DexOp::InvokeStatic {
                registers: vec![
                    Register::Parameter(1),
                    Register::Parameter(2),
                    Register::Local(0),
                    Register::Local(1)
                ],
                method: expected_method,
            }
        );
    }

    #[test]
    fn test_invoke_direct() {
        let mut input = r#"invoke-direct {p0}, Ljava/lang/Object;-><init>()V"#;
        let instr = parse_dex_op(&mut input).unwrap();
        let expected_method = MethodRef {
            class: parse_object_identifier()
                .parse_next(&mut "Ljava/lang/Object;")
                .unwrap(),
            param: parse_method_parameter()
                .parse_next(&mut "<init>()V")
                .unwrap(),
        };
        assert_eq!(
            instr,
            DexOp::InvokeDirect {
                registers: vec![Register::Parameter(0)],
                method: expected_method,
            }
        );
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
