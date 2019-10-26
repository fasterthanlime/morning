#![allow(unused)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::char,
    combinator::{all_consuming, cut, map, opt},
    error::{context, ParseError, VerboseError},
    multi::{many0, many1, separated_list},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult, InputTake,
};

mod errors;
mod span;

use super::ast::*;
pub use errors::*;
pub use span::*;

pub type Res<T> = IResult<Span, T, VerboseError<Span>>;

pub fn unit(i: Span) -> Res<Unit> {
    map(many0(unit_item), move |items| Unit::new(items))(i)
}

fn unit_item(i: Span) -> Res<UnitItem> {
    map(fn_decl, |f| UnitItem::FunctionDeclaration(f))(i)
}

fn fn_decl(i: Span) -> Res<FunctionDeclaration> {
    spaced(context("fn declaration", |i| {
        let (i, _) = tag("fn")(i)?;
        cut(|i| {
            let (i, name) = spaced(identifier)(i)?;
            let (i, params) = param_list(i)?;
            let (i, body) = spaced(block)(i)?;

            let fun = FunctionDeclaration { body, params, name };
            return Ok((i, fun));
        })(i)
    }))(i)
}

fn param_list(i: Span) -> Res<Vec<Parameter>> {
    spaced(context("param list", |i| {
        delimited(
            tag("("),
            cut(separated_list(tag(","), spaced(parameter))),
            tag(")"),
        )(i)
    }))(i)
}

fn parameter(i: Span) -> Res<Parameter> {
    let (i, (name, typ)) = separated_pair(spaced(identifier), tag(":"), spaced(type_reference))(i)?;

    let p = Parameter { name, typ };
    Ok((i, p))
}

fn type_reference(i: Span) -> Res<TypeReference> {
    let (i, id) = spaced(identifier)(i)?;
    Ok((i, TypeReference { id }))
}

fn block(i: Span) -> Res<Block> {
    // let p = delimited(spaced(tag("{")), many0(statement), spaced(tag("}")));
    let p = delimited(spaced(tag("{")), cut(many0(tag("FIXME"))), spaced(tag("}")));
    map(p, |items| Block { items: Vec::new() })(i)
}

fn statement(i: Span) -> Res<Statement> {
    unimplemented!()
}

static VALID_ID_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

fn identifier(i: Span) -> Res<Identifier> {
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
fn linesp(i: Span) -> Res<Span> {
    let chars = " \t";

    take_while(move |c| chars.contains(c))(i)
}

// Consumes nothing, returns a span for location information
fn loc(i: Span) -> Res<Span> {
    let o = i.take(0);
    Ok((i, o))
}
