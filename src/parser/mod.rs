#![allow(unused)]

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

pub fn file<E: ParseError<Span>>(i: Span) -> IResult<Span, Unit, E> {
    map(many0(file_item), move |items| Unit::new(items))(i)
}

fn file_item<E: ParseError<Span>>(i: Span) -> IResult<Span, UnitItem, E> {
    dbg!("in file_item");
    map(fn_decl, |f| UnitItem::FunctionDeclaration(f))(i)
}

fn fn_decl<E: ParseError<Span>>(i: Span) -> IResult<Span, FunctionDeclaration, E> {
    println!("looking for fn tag at {:?}", i);
    let (i, _) = spaced(tag("fn"))(i)?;
    dbg!("got fn");
    let (i, name) = spaced(identifier)(i)?;
    dbg!(&name);
    let (i, params) = param_list(i)?;
    dbg!(&params);
    let (i, body) = spaced(block)(i)?;
    dbg!(&body);

    let fun = FunctionDeclaration { body, params, name };
    return Ok((i, fun));
}

fn param_list<E: ParseError<Span>>(i: Span) -> IResult<Span, Vec<Parameter>, E> {
    spaced(delimited(
        tag("("),
        separated_list(tag(","), spaced(parameter)),
        tag(")"),
    ))(i)
}

fn parameter<E: ParseError<Span>>(i: Span) -> IResult<Span, Parameter, E> {
    let (i, (name, typ)) = separated_pair(spaced(identifier), tag(":"), spaced(type_reference))(i)?;

    let p = Parameter { name, typ };
    Ok((i, p))
}

fn type_reference<E: ParseError<Span>>(i: Span) -> IResult<Span, TypeReference, E> {
    let (i, id) = spaced(identifier)(i)?;
    Ok((i, TypeReference { id }))
}

fn block<E: ParseError<Span>>(i: Span) -> IResult<Span, Block, E> {
    // let p = delimited(spaced(tag("{")), many0(statement), spaced(tag("}")));
    let p = delimited(spaced(tag("{")), many0(tag("FIXME")), spaced(tag("}")));
    map(p, |items| Block { items: Vec::new() })(i)
}

fn statement<E: ParseError<Span>>(i: Span) -> IResult<Span, Statement, E> {
    unimplemented!()
}

static VALID_ID_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

fn identifier<E: ParseError<Span>>(i: Span) -> IResult<Span, Identifier, E> {
    let (i, span) = take_while(|c| VALID_ID_CHARS.contains(c))(i)?;

    let id = Identifier::new(span);
    Ok((i, id))
}

/// f, but skip whitespace before and after (including newlines, and comments)
fn spaced<O, E: ParseError<Span>, F>(f: F) -> impl Fn(Span) -> IResult<Span, O, E>
where
    F: Fn(Span) -> IResult<Span, O, E>,
{
    preceded(
        many0(alt((
            tag(" "),
            tag("\t"),
            tag("\r"),
            tag("\n"),
            preceded(tag("//"), take_until("\n")),
        ))),
        f,
    )
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
