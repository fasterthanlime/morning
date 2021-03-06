#![allow(unused)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::char,
    combinator::{all_consuming, cut, map, map_res, opt, recognize},
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
    all_consuming(terminated(
        map(many0(unit_item), move |items| Unit::new(items)),
        // FIXME: that looks big dumb, but it eats up
        // trailing whitespace so all_consuming is happy..
        spaced(tag("")),
    ))(i)
}

fn unit_item(i: Span) -> Res<UnitItem> {
    map(fn_decl, |f| UnitItem::FDecl(f))(i)
}

fn fn_decl(i: Span) -> Res<FDecl> {
    spaced(context("fn declaration", |i| {
        let (i, public) = opt(stag("pub"))(i)?;
        let (i, _) = stag("fn")(i)?;
        cut(move |i| {
            let (i, name) = spaced(identifier)(i)?;
            let (i, params) = param_list(i)?;
            let (i, body) = spaced(block)(i)?;

            let fun = FDecl {
                body,
                params,
                name,
                public: public.is_some(),
            };
            return Ok((i, fun));
        })(i)
    }))(i)
}

fn param_list(i: Span) -> Res<Vec<Param>> {
    spaced(context("param list", |i| {
        delimited(
            tag("("),
            cut(separated_list(tag(","), spaced(parameter))),
            tag(")"),
        )(i)
    }))(i)
}

fn parameter(i: Span) -> Res<Param> {
    let (i, (name, typ)) = separated_pair(spaced(identifier), tag(":"), spaced(type_reference))(i)?;

    let p = Param { name, typ };
    Ok((i, p))
}

fn type_reference(i: Span) -> Res<TypeRef> {
    let (i, id) = spaced(identifier)(i)?;
    Ok((i, TypeRef { id }))
}

fn block(i: Span) -> Res<Block> {
    let p = spaced(delimited(stag("{"), many0(statement), cut(stag("}"))));
    map(p, |items| Block { items })(i)
}

fn statement(i: Span) -> Res<Statement> {
    alt((
        map(block, Statement::Block),
        map(loop_st, Statement::Loop),
        map(if_st, Statement::If),
        terminated(
            spaced(alt((
                map(stag("break"), |_| Statement::Break),
                map(stag("continue"), |_| Statement::Continue),
                map(return_st, Statement::Return),
                map(var_decl, Statement::VDecl),
                map(expression, Statement::Expr),
            ))),
            cut(stag(";")),
        ),
    ))(i)
}

fn if_st(i: Span) -> Res<If> {
    let (i, _) = stag("if")(i)?;
    spaced(context(
        "if",
        map(
            tuple((spaced(expression), spaced(block))),
            |(cond, body)| If { cond, body },
        ),
    ))(i)
}

fn loop_st(i: Span) -> Res<Loop> {
    let (i, _) = stag("loop")(i)?;
    spaced(context("loop", map(block, |body| Loop { body })))(i)
}

fn return_st(i: Span) -> Res<Return> {
    let (i, _) = stag("return")(i)?;
    spaced(context("return statement", |i| {
        let (i, expr) = opt(expression)(i)?;
        let ret = Return { expr };
        Ok((i, ret))
    }))(i)
}

fn var_decl(i: Span) -> Res<VDecl> {
    spaced(context("let binding", |i| {
        let (i, _) = tag("let")(i)?;
        cut(|i| {
            let (i, name) = spaced(identifier)(i)?;
            let (i, typ) = opt(preceded(stag(":"), spaced(type_reference)))(i)?;
            let (i, value) = opt(preceded(stag("="), spaced(expression)))(i)?;
            let vd = VDecl { name, typ, value };
            Ok((i, vd))
        })(i)
    }))(i)
}

fn expression(i: Span) -> Res<Expr> {
    let (mut i, mut expr) = inner_expression(i)?;

    loop {
        match postfix_expression(expr.clone(), i.clone()) {
            Ok((i2, expr2)) => {
                i = i2;
                expr = expr2;
                continue;
            }
            Err(_) => return Ok((i, expr)),
        }
    }
}

fn inner_expression(i: Span) -> Res<Expr> {
    spaced(alt((
        delimited(stag("("), spaced(expression), stag(")")),
        map(block, Expr::Block),
        map(float_lit, Expr::FloatLit),
        map(int_lit, Expr::IntLit),
        map(identifier, Expr::Identifier),
    )))(i)
}

fn postfix_expression(lhs: Expr, i: Span) -> Res<Expr> {
    alt((
        map(call(&lhs), Expr::Call),
        map(binary_expression(&lhs), Expr::Bexp),
    ))(i)
}

fn call<'a>(target: &'a Expr) -> impl Fn(Span) -> Res<Call> + 'a {
    move |i| {
        let (i, args) = delimited(stag("("), separated_list(stag(","), expression), stag(")"))(i)?;

        let c = Call {
            target: Box::new(target.clone()),
            args,
        };
        Ok((i, c))
    }
}

fn binary_expression<'a>(lhs: &'a Expr) -> impl Fn(Span) -> Res<Bexp> + 'a {
    move |i| {
        let (i, operator) = spaced(binary_operator_ex)(i)?;
        let (i, rhs) = spaced(expression)(i)?;

        Ok((i, operator.as_expr(Box::new(lhs.clone()), Box::new(rhs))))
    }
}

fn binary_operator_ex(i: Span) -> Res<BopEx> {
    alt((
        map(tag("+="), |_| BopEx::Ass(AssOp::Plus)),
        map(tag("-="), |_| BopEx::Ass(AssOp::Minus)),
        map(tag("*="), |_| BopEx::Ass(AssOp::Mul)),
        map(tag("/="), |_| BopEx::Ass(AssOp::Div)),
        map(tag("+"), |_| BopEx::Base(Bop::Plus)),
        map(tag("-"), |_| BopEx::Base(Bop::Minus)),
        map(tag("*"), |_| BopEx::Base(Bop::Mul)),
        map(tag("/"), |_| BopEx::Base(Bop::Div)),
        map(tag("="), |_| BopEx::Base(Bop::Assign)),
        map(tag(">="), |_| BopEx::Base(Bop::GtEq)),
        map(tag(">"), |_| BopEx::Base(Bop::Gt)),
        map(tag("<="), |_| BopEx::Base(Bop::LtEq)),
        map(tag("<"), |_| BopEx::Base(Bop::Lt)),
    ))(i)
}

fn float_lit(i: Span) -> Res<FloatLit> {
    let (i, slice) = alt((
        recognize(tuple((tag("."), digits))),
        recognize(tuple((digits, tag("."), opt(digits)))),
    ))(i)?;

    let loc = slice.clone();
    let (_, value) = all_consuming(nom::number::complete::double)(slice)?;
    let fl = FloatLit { loc, value };
    Ok((i, fl))
}

fn int_lit(i: Span) -> Res<IntLit> {
    map_res(digits, move |span: Span| {
        match span.slice().parse::<i64>() {
            Ok(value) => {
                let il = IntLit { loc: span, value };
                Ok(il)
            }
            Err(_) => Err(nom::Err::Error(nom::error::ErrorKind::Tag)),
        }
    })(i)
}

fn digits(i: Span) -> Res<Span> {
    let int_chars = "0123456789";
    take_while1(move |c| int_chars.contains(c))(i)
}

static VALID_ID_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

fn identifier(i: Span) -> Res<Id> {
    let (i, span) = take_while1(|c| VALID_ID_CHARS.contains(c))(i)?;

    let id = Id::new(span);
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

fn stag(s: &'static str) -> impl Fn(Span) -> Res<Span> {
    spaced(tag(s))
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
