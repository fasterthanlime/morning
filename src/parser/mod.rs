use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::char,
    combinator::{all_consuming, cut, map, opt},
    error::{context, ParseError},
    multi::{many0, many1, separated_list},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult, InputTake,
};

mod errors;
mod span;

use super::ast::*;
pub use errors::*;
pub use span::*;

pub fn file<E: ParseError<Span>>(i: Span) -> IResult<Span, File, E> {
    let (i, loc) = loc(i)?;

    all_consuming(map(tuple(loc, many0(file_item)), |(loc, items)| {
        File::new(loc, items)
    }))(i)
}

/// f, but skip whitespace before and after (including newlines)
fn spaced<O, E: ParseError<Span>, F>(f: F) -> impl Fn(Span) -> IResult<Span, O, E>
where
    F: Fn(Span) -> IResult<Span, O, E>,
{
    terminated(preceded(sp, f), sp)
}

/// All whitespace (including newlines)
fn sp<E: ParseError<Span>>(i: Span) -> IResult<Span, Span, E> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

/// Whitespace excluding newlines
fn linesp<E: ParseError<Span>>(i: Span) -> IResult<Span, Span, E> {
    let chars = " \t";

    take_while(move |c| chars.contains(c))(i)
}

// Consumes nothing, returns a span for location information
fn loc<E: ParseError<Span>>(i: Span) -> IResult<Span, Span, E> {
    let o = i.take(0);
    Ok((i, o))
}
