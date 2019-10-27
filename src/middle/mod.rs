use crate::{ast, ir};
use std::io;

type Result = std::result::Result<(), io::Error>;

pub struct File {}

pub fn transform(w: &mut dyn io::Write, u: &ast::Unit) -> Result {
    let mut funs = Vec::new();

    for af in &u.funs {
        funs.push(transform_fdecl(af));
    }

    let v: Vec<_> = funs.iter().collect();
    ir::emit::emit_all(w, &v[..])?;

    Ok(())
}

fn transform_fdecl(af: &ast::FDecl) -> ir::Func {
    let f = ir::Func::new(af.name.value.clone());

    f
}
