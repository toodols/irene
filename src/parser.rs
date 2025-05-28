use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{
        complete::{is_a, tag, take_while, take_while1},
        is_not, take_until,
    },
    character::{
        complete::{alphanumeric1, space0, space1},
        one_of,
    },
    combinator::recognize,
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, preceded},
};
use nom_locate::LocatedSpan;
type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug)]
pub struct Program<'a> {
    span: Span<'a>,
    commands: Vec<Command<'a>>,
}

#[derive(Debug)]
pub struct EmptyCommand<'a> {
    span: Span<'a>,
}

#[derive(Debug)]
pub enum CommandType<'a> {
    EmptyCommand(EmptyCommand<'a>),
    Command(Command<'a>),
}

impl<'a> CommandType<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        match self {
            Self::EmptyCommand(c) => &c.span,
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

#[derive(Debug)]
pub struct CommandPath<'a> {
    span: Span<'a>,
    components: Vec<&'a str>,
}

#[derive(Debug)]
pub struct Command<'a> {
    span: Span<'a>,
    command_path: CommandPath<'a>,
    arguments: Vec<Argument<'a>>,
}

#[derive(Debug)]
pub struct Body<'a> {
    span: Span<'a>,
    commands: Vec<Command<'a>>,
}

#[derive(Debug)]
pub struct String<'a> {
    span: Span<'a>,
    quoted: bool,
}

#[derive(Debug)]
pub struct Subcall<'a>(Body<'a>);
impl<'a> Subcall<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        &self.0.span
    }
}

#[derive(Debug)]
pub struct Function<'a>(Body<'a>);
impl<'a> Function<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        &self.0.span
    }
}

#[derive(Debug)]
pub enum Argument<'a> {
    Subcall(Subcall<'a>),
    Function(Function<'a>),
    String(String<'a>),
}

impl<'a> Argument<'a> {
    fn span(&'a self) -> &'a Span<'a> {
        match self {
            Self::Subcall(b) => &b.span(),
            Self::Function(b) => &b.span(),
            Self::String(w) => &w.span,
        }
    }
}

fn comment(input: Span) -> IResult<Span, Span> {
    delimited(tag("/*"), take_until("*/"), tag("*/")).parse(input)
}
fn whitespace(input: Span) -> IResult<Span, ()> {
    many1(alt((space1, comment))).map(|_| ()).parse(input)
}

fn parse_command_path(input: Span) -> IResult<Span, CommandPath> {
    separated_list1(tag("."), take_while1(alphanumeric_underscore))
        .map(|components| CommandPath {
            span: input,
            components: components
                .into_iter()
                .map(|s: Span| *s.fragment())
                .collect(),
        })
        .parse(input)
}

fn parse_arguments(input: Span) -> IResult<Span, Vec<Argument>> {
    separated_list0(tag("|"), parse_argument).parse(input)
}

fn parse_argument(input: Span) -> IResult<Span, Argument> {
    alt((
        parse_word.map(Argument::String),
        parse_quoted_string.map(Argument::String),
    ))
    .parse(input)
}

fn parse_quoted_string(input: Span) -> IResult<Span, String> {
    delimited(
        tag("\""),
        recognize(many0(alt((preceded(tag("\\"), tag("\"")), is_not("\\\""))))),
        tag("\""),
    )
    .parse(input)
    .map(|(input, span)| (input, String { span, quoted: true }))
}

fn alphanumeric_underscore(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'
}

// [a-zA-Z0-9_]+
fn parse_word(input: Span) -> IResult<Span, String> {
    take_while1(alphanumeric_underscore)
        .map(|span| String {
            span,
            quoted: false,
        })
        .parse(input)
}

#[test]
fn test_parse() {
    println!("{:#?}", parse_arguments(Span::new("abc|def")).unwrap());
}
