use std::{borrow::Cow, fmt};

use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{map, opt},
    error::Error,
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated},
};

use crate::{
    object_identifier::{ObjectIdentifier, parse_object_identifier},
    op::dex_op::{DexOp, parse_dex_op},
    parse_int_lit, ws,
};

pub mod dex_op;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label<'a>(pub Cow<'a, str>);

impl fmt::Display for Label<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Prepend a colon when printing
        write!(f, ":{}", self.0)
    }
}

/// Parse a label in smali syntax, e.g. ":cond_0"
pub fn parse_label<'a>() -> impl Parser<&'a str, Output = Label<'a>, Error = Error<&'a str>> {
    map(
        preceded(
            char(':'),
            take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '$'),
        ),
        |s| Label(Cow::Borrowed(s)),
    )
}

/// Represents a protected range in a try/catch directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TryRange<'a> {
    pub start: Label<'a>,
    pub end: Label<'a>,
}

pub fn parse_try_range<'a>() -> impl Parser<&'a str, Output = TryRange<'a>, Error = Error<&'a str>>
{
    map(
        delimited(
            ws(char('{')),
            (terminated(ws(parse_label()), tag("..")), ws(parse_label())),
            ws(char('}')),
        ),
        |(start, end)| TryRange { start, end },
    )
}

impl fmt::Display for TryRange<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format similar to smali: "{:try_start .. :try_end}"
        write!(f, "{{{} .. {}}}", self.start, self.end)
    }
}

/// Represents a catch block directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatchDirective<'a> {
    /// A catch directive with an exception type.
    Catch {
        exception: ObjectIdentifier<'a>, // e.g. "Ljava/lang/Exception;"
        try_range: TryRange<'a>,
        handler: Label<'a>,
    },
    /// A catch-all directive.
    CatchAll {
        try_range: TryRange<'a>,
        handler: Label<'a>,
    },
}

pub fn parse_catch_directive<'a>()
-> impl Parser<&'a str, Output = CatchDirective<'a>, Error = Error<&'a str>> {
    alt((
        map(
            preceded(
                tag(".catch"),
                (
                    ws(parse_object_identifier()),
                    ws(parse_try_range()),
                    ws(parse_label()),
                ),
            ),
            |(exception, try_range, handler)| CatchDirective::Catch {
                exception,
                try_range,
                handler,
            },
        ),
        map(
            preceded(tag(".catchall"), (ws(parse_try_range()), ws(parse_label()))),
            |(try_range, handler)| CatchDirective::CatchAll { try_range, handler },
        ),
    ))
}

impl fmt::Display for CatchDirective<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatchDirective::Catch {
                exception,
                try_range,
                handler,
            } => {
                // Print as: .catch <exception> <try_range> <handler>
                write!(f, ".catch {exception} {try_range} {handler}")
            }
            CatchDirective::CatchAll { try_range, handler } => {
                // Print as: .catchall <try_range> <handler>
                write!(f, ".catchall {try_range} {handler}")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArrayDataElement {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl fmt::Display for ArrayDataElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayDataElement::Byte(b) => write!(f, "{b:#x}t"),
            ArrayDataElement::Short(s) => write!(f, "{s:#x}s"),
            ArrayDataElement::Int(i) => write!(f, "{i:#x}"),
            ArrayDataElement::Long(l) => write!(f, "{l:#x}l"),
            ArrayDataElement::Float(fl) => write!(f, "{:#x}f", fl.to_bits()),
            ArrayDataElement::Double(d) => write!(f, "{:#x}d", d.to_bits()),
        }
    }
}

/// Represents a .array-data directive.
#[derive(Debug, PartialEq, Clone)]
pub struct ArrayDataDirective {
    /// The element width as specified in the header.
    pub width: u32,
    /// The parsed array elements.
    pub elements: Vec<ArrayDataElement>,
}

pub fn parse_array_data_directive<'a>()
-> impl Parser<&'a str, Output = ArrayDataDirective, Error = Error<&'a str>> {
    map(
        delimited(
            ws(tag(".array-data")),
            (
                ws(parse_int_lit::<u32>()),
                many0(ws((
                    parse_int_lit::<i64>(),
                    opt(alt((char('t'), char('s'), char('l'), char('f'), char('d')))),
                ))),
            ),
            ws(tag(".end array-data")),
        ),
        |(width, e)| ArrayDataDirective {
            width,
            elements: e
                .into_iter()
                .map(|(value, postfix)| {
                    if let Some(postfix) = postfix {
                        match postfix {
                            't' => ArrayDataElement::Byte(value as i8),
                            's' => ArrayDataElement::Short(value as i16),
                            'l' => ArrayDataElement::Long(value),
                            'f' => ArrayDataElement::Float(f32::from_bits(value as u32)),
                            'd' => ArrayDataElement::Double(f64::from_bits(value as u64)),
                            _ => unreachable!(),
                        }
                    } else {
                        match width {
                            1 => ArrayDataElement::Byte(value as i8),
                            2 => ArrayDataElement::Short(value as i16),
                            4 => ArrayDataElement::Int(value as i32),
                            8 => ArrayDataElement::Long(value),
                            _ => ArrayDataElement::Int(value as i32),
                        }
                    }
                })
                .collect(),
        },
    )
}

impl fmt::Display for ArrayDataDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the header. We'll print the width in hex.
        writeln!(f, ".array-data {:#x}", self.width)?;
        // Print elements in groups (here, 4 per line).
        for chunk in self.elements.chunks(4) {
            write!(f, "    ")?;
            for elem in chunk {
                write!(f, "{elem} ")?;
            }
            writeln!(f)?;
        }
        write!(f, ".end array-data")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PackedSwitchDirective<'a> {
    pub first_key: i32,
    pub targets: Vec<Label<'a>>,
}

pub fn parse_packed_switch_directive<'a>()
-> impl Parser<&'a str, Output = PackedSwitchDirective<'a>, Error = Error<&'a str>> {
    map(
        delimited(
            ws(tag(".packed-switch")),
            (ws(parse_int_lit::<i32>()), many1(ws(parse_label()))),
            ws(tag(".end packed-switch")),
        ),
        |(first_key, targets)| PackedSwitchDirective { first_key, targets },
    )
}

impl fmt::Display for PackedSwitchDirective<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the header with the first key in hex.
        writeln!(f, ".packed-switch {:#x}", self.first_key)?;
        // Print each target label, indented.
        for target in &self.targets {
            writeln!(f, "    {target}")?;
        }
        // Print the footer without a trailing newline.
        write!(f, ".end packed-switch")
    }
}

/// An entry in a sparse-switch directive: a key and its corresponding target label.
#[derive(Debug, PartialEq, Clone)]
pub struct SparseSwitchEntry<'a> {
    pub key: i32,
    pub target: Label<'a>,
}

pub fn parse_sparse_switch_entry<'a>()
-> impl Parser<&'a str, Output = SparseSwitchEntry<'a>, Error = Error<&'a str>> {
    map(
        (
            terminated(ws(parse_int_lit::<i32>()), tag("->")),
            ws(parse_label()),
        ),
        |(key, target)| SparseSwitchEntry { key, target },
    )
}

impl fmt::Display for SparseSwitchEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format the key in hexadecimal followed by "->" and the label.
        write!(f, "{:#x} -> {}", self.key, self.target)
    }
}

/// The sparse-switch directive.
#[derive(Debug, PartialEq, Clone)]
pub struct SparseSwitchDirective<'a> {
    pub entries: Vec<SparseSwitchEntry<'a>>,
}

pub fn parse_sparse_switch_directive<'a>()
-> impl Parser<&'a str, Output = SparseSwitchDirective<'a>, Error = Error<&'a str>> {
    map(
        delimited(
            ws(tag(".sparse-switch")),
            many0(parse_sparse_switch_entry()),
            ws(tag(".end sparse-switch")),
        ),
        |entries| SparseSwitchDirective { entries },
    )
}

impl fmt::Display for SparseSwitchDirective<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print the header.
        writeln!(f, ".sparse-switch")?;
        // Print each entry indented.
        for entry in &self.entries {
            writeln!(f, "    {entry}")?;
        }
        // Print the footer.
        write!(f, ".end sparse-switch")
    }
}

/// An enum representing operations within a method, these can be a label, a line number or a dex operation as a String.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Op<'a> {
    Label(Label<'a>),
    Line(u32),
    Op(DexOp<'a>),
    Catch(CatchDirective<'a>),
    ArrayData(ArrayDataDirective),
    PackedSwitch(PackedSwitchDirective<'a>),
    SparseSwitch(SparseSwitchDirective<'a>),
}

pub fn parse_op<'a>() -> impl Parser<&'a str, Output = Op<'a>, Error = Error<&'a str>> {
    alt((
        map(ws(parse_label()), Op::Label),
        map(
            preceded(ws(tag(".line")), ws(parse_int_lit::<u32>())),
            Op::Line,
        ),
        map(ws(parse_dex_op), Op::Op),
        map(parse_catch_directive(), Op::Catch),
        map(parse_array_data_directive(), Op::ArrayData),
        map(parse_packed_switch_directive(), Op::PackedSwitch),
        map(parse_sparse_switch_directive(), Op::SparseSwitch),
    ))
}

mod tests {

    #[test]
    fn test_array_data() {
        use super::*;
        use nom::Parser;
        let input = r#".array-data 4
                                0x0
                                0x3f800000
                             .end array-data"#;
        let ad = parse_array_data_directive().parse_complete(input).unwrap();
        println!("{ad:?}");
    }

    #[test]
    fn test1() {
        use super::*;
        use nom::Parser;
        let input = "\n    invoke-direct {p0}, Ljava/lang/Object;-><init>()V\n\n";
        let (_, a) = parse_op().parse(input).unwrap();
        println!("{a:?}");

        let input = "\n    :goto_0\n\n";
        let (_, a) = parse_op().parse(input).unwrap();
        println!("{a:?}");
    }
}
