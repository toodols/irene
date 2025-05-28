use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{tag, take_while, take_while1},
        is_not,
    },
    character::{complete::alphanumeric1, one_of},
    combinator::recognize,
    multi::many0,
    sequence::{delimited, preceded},
};
use nom_locate::LocatedSpan;
type Span<'a> = LocatedSpan<&'a str>;

pub struct Program<'a> {
    span: Span<'a>,
    commands: Vec<Command<'a>>,
}

pub enum CommandType<'a> {
    EmptyCommand { span: Span<'a> },
    Command(Command<'a>),
}

impl<'a> CommandType<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        match self {
            Self::EmptyCommand { span } => span,
            Self::Command(c) => &c.span,
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Self::EmptyCommand { .. } => true,
            _ => false,
        }
    }
}

pub struct CommandPath<'a> {
    span: Span<'a>,
    components: Vec<&'a str>,
}

pub struct Command<'a> {
    span: Span<'a>,
    command_path: CommandPath<'a>,
    arguments: Vec<Argument<'a>>,
}

pub struct Body<'a> {
    span: Span<'a>,
    commands: Vec<Command<'a>>,
}

pub struct Text<'a> {
    span: Span<'a>,
    quoted: bool,
}

pub struct Subcall<'a>(Body<'a>);
impl<'a> Subcall<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        &self.0.span
    }
}
pub struct Function<'a>(Body<'a>);
impl<'a> Function<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        &self.0.span
    }
}
pub enum Argument<'a> {
    Subcall(Subcall<'a>),
    Function(Function<'a>),
    Word(Text<'a>),
}

impl<'a> Argument<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        match self {
            Self::Subcall(b) => &b.span(),
            Self::Function(b) => &b.span(),
            Self::Word(w) => &w.span,
        }
    }
}

fn parse_quoted_string<'a>(input: Span<'a>) -> IResult<Span<'a>, Text<'a>> {
    delimited(
        tag("\""),
        recognize(many0(alt((preceded(tag("\\"), tag("\"")), is_not("\\\""))))),
        tag("\""),
    )
    .parse(input)
    .map(|(input, span)| {
        (
            input,
            Text {
                span,
                quoted: true,
            },
        )
    })
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
}

// [a-zA-Z0-9_]+
fn parse_word<'a>(input: Span<'a>) -> IResult<Span<'a>, Text<'a>> {
    take_while1(is_valid_char)(input).map(|(_, span)| {
        (
            span,
            Text {
                span,
                quoted: false,
            },
        )
    })
}

fn parse_text<'a>(input: Span<'a>) -> IResult<Span<'a>, Text<'a>> {
    parse_word.or(parse_quoted_string).parse(input)
}
