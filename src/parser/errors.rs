use colored::*;
use nom::{
    error::{VerboseError, VerboseErrorKind},
    Err,
};
use std::fmt;
use std::iter::repeat;
use std::path::Path;
use std::rc::Rc;

use crate::{ast, parser};
use parser::Span;

/// A parsing, checking, or emitting error
pub enum Error {
    IO(std::io::Error),
    Source(SourceError),
    Diag(parser::Diagnostic),
    Unknown(UnknownError),
}

impl<'a> fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => write!(f, "{}", e),
            Error::Source(e) => write!(f, "{:#?}", e),
            Error::Diag(d) => write!(f, "{}", d),
            Error::Unknown(_) => write!(f, "unknown error"),
        }
    }
}

impl<'a> fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {}

pub struct SourceError {
    inner: VerboseError<parser::Span>,
}

impl<'a> fmt::Debug for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        print_errors(f, &self.inner)
    }
}

pub struct UnknownError {
    source: Rc<Source>,
}

impl fmt::Debug for UnknownError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "An unknown parsing error occured in {}",
            self.source.name()
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<parser::Diagnostic> for Error {
    fn from(e: parser::Diagnostic) -> Self {
        Error::Diag(e)
    }
}

/// A `.mor` source file
pub struct Source {
    pub input: String,
    name: String,
    pub lines: Vec<String>,
}

impl Source {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Rc<Self>, std::io::Error> {
        let path = path.as_ref();

        let mut input = String::new();
        {
            use std::fs::File;
            use std::io::Read;
            let mut f = File::open(path)?;
            f.read_to_string(&mut input)?;
        }

        let name = path.to_str().unwrap();
        Ok(Self::new(name.into(), input))
    }

    #[allow(unused)]
    pub fn from_string<S>(input: S) -> Rc<Self>
    where
        S: Into<String>,
    {
        Self::new("<memory>".into(), input.into())
    }

    pub fn new(name: String, input: String) -> Rc<Self> {
        let lines = input.lines().map(String::from).collect::<Vec<_>>();
        Rc::new(Self { name, input, lines })
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

fn do_parse<P, O>(source: Rc<Source>, p: P) -> Result<O, Error>
where
    P: Fn(Span) -> nom::IResult<Span, O, VerboseError<parser::Span>>,
{
    let span = parser::Span {
        source: source.clone(),
        offset: 0,
        len: source.input.len(),
    };
    let res = p(span);
    match res {
        Err(Err::Error(e)) | Err(Err::Failure(e)) => Err(Error::Source(SourceError { inner: e })),
        Err(_) => Err(Error::Unknown(UnknownError {
            source: source.clone(),
        })),
        Ok((_, output)) => Ok(output),
    }
}

pub fn parse(source: Rc<Source>) -> Result<ast::Unit, Error> {
    do_parse(source, parser::unit)
}

#[derive(Debug)]
pub struct Diagnostic {
    pos: Position,
    caret_color: Color,
    prefix: String,
    message: String,
}

pub struct DiagnosticBuilder {
    pos: Position,
    caret_color: Color,
    prefix: String,
    message: Option<String>,
}

const EMPTY_PREFIX: &str = "";

impl DiagnosticBuilder {
    pub fn new(pos: Position) -> Self {
        Self {
            pos: pos.clone(),
            caret_color: Color::Blue,
            prefix: EMPTY_PREFIX.into(),
            message: None,
        }
    }

    pub fn caret_color(mut self, caret_color: Color) -> Self {
        self.caret_color = caret_color;
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    pub fn build(self) -> Diagnostic {
        Diagnostic {
            pos: self.pos,
            caret_color: self.caret_color,
            prefix: self.prefix,
            message: self.message.unwrap_or_else(|| "".into()),
        }
    }
}

impl Diagnostic {
    pub fn print(&self) {
        print!("{}", self)
    }

    pub fn write(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl DiagnosticBuilder {
    pub fn print(self) {
        self.build().print()
    }

    pub fn write(self, f: &mut fmt::Formatter) -> fmt::Result {
        self.build().write(f)
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut pos = self.pos.clone();
        if self.pos.span.slice().starts_with("}") {
            let span = &self.pos.span;
            let haystack = &span.source.input[0..span.offset];

            let wsp = " \t\r\n";
            if let Some(index) = haystack.rfind(|c| !wsp.contains(c)) {
                let index = index + 1;
                let new_slice = &haystack[index..];

                let span = Span {
                    source: span.source.clone(),
                    offset: index,
                    len: 1,
                };
                pos = span.position();
            }
        }

        let caret_color = self.caret_color;
        let prefix = &self.prefix;
        let message = &self.message;

        let text_line = &pos.span.source.lines[pos.line];
        let loc = format!(
            "{}:{}:{}:",
            pos.span.source.name(),
            pos.line + 1,
            pos.column + 1
        );
        writeln!(f, "{}{} {}", prefix, loc.bold(), message)?;
        writeln!(f, "{}{}", prefix, text_line.dimmed())?;

        writeln!(
            f,
            "{}{}{}{}",
            prefix,
            repeat(' ').take(pos.column).collect::<String>(),
            "^".color(caret_color).bold(),
            repeat('~')
                .take(std::cmp::min(
                    match pos.span.len {
                        0 => 0,
                        x => x - 1,
                    },
                    text_line.len() - pos.column
                ))
                .collect::<String>()
                .color(caret_color)
                .bold()
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Position {
    pub span: Span,
    pub line: usize,
    pub column: usize,
}

impl Position {
    fn diag(&self, message: String) -> DiagnosticBuilder {
        DiagnosticBuilder::new(self.clone()).message(message)
    }

    pub fn diag_info(&self, message: String) -> DiagnosticBuilder {
        self.diag(message).caret_color(Color::Blue)
    }

    pub fn diag_err(&self, message: String) -> DiagnosticBuilder {
        self.diag(message).caret_color(Color::Red)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.span.source.name, self.line, self.column)
    }
}

pub fn print_errors(f: &mut fmt::Formatter, e: &VerboseError<Span>) -> fmt::Result {
    let mut errors = e.errors.clone();
    errors.reverse();

    writeln!(f)?;
    for (span, kind) in errors.iter() {
        let pos = span.position();

        match kind {
            VerboseErrorKind::Char(c) => {
                pos.diag_err(format!(
                    "expected '{}', found {}",
                    c,
                    span.chars().next().unwrap_or_else(|| '\0')
                ))
                .write(f)?;
            }
            VerboseErrorKind::Context(s) => {
                pos.diag_info(format!("In {}", s)).write(f)?;
            }
            VerboseErrorKind::Nom(ek) => {
                pos.diag_err(format!(
                    "parsing error: {}",
                    format!("{:#?}", ek).red().bold()
                ))
                .write(f)?;
            }
        }
    }

    Ok(())
}
