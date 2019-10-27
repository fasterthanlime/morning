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

struct Stack<'a> {
    f: &'a mut ir::Func,
    items: Vec<Item>,
}

impl<'a> Stack<'a> {
    pub fn new(f: &'a mut ir::Func) -> Self {
        let items = vec![Item::Block(f.entry)];
        Self { f, items }
    }

    pub fn block(&mut self) -> &mut ir::Block {
        let block = self
            .items
            .iter()
            .rev()
            .find_map(|i| match i {
                Item::Block(b) => Some(b),
                _ => None,
            })
            .expect("middle-end stack should always have a block");
        block.borrow_mut(self.f)
    }

    pub fn push(&mut self, item: Item) {
        self.items.push(item)
    }

    pub fn pop(&mut self) {
        self.items.pop();
    }
}

enum Item {
    Loop(Loop),
    Block(ir::BlockRef),
}

struct Loop {
    break_label: ir::LabelRef,
    loop_label: ir::LabelRef,
}

fn transform_fdecl(af: &ast::FDecl) -> ir::Func {
    let mut f = ir::Func::new(af.name.value.clone());

    for st in &af.body.items {
        f.entry.borrow_mut(&mut f).push_op(ir::Op::Comment(None));

        match st {
            ast::Statement::VDecl(vd) => {
                f.entry
                    .borrow_mut(&mut f)
                    .push_op(ir::Op::comment(&vd.name.loc.position()));
                let local = f
                    .entry
                    .borrow_mut(&mut f)
                    .push_local(vd.name.value.clone(), ir::Type::I64);

                if let Some(value) = vd.value.as_ref() {
                    match value {
                        ast::Expr::IntLit(ast::IntLit { value, .. }) => f
                            .entry
                            .borrow_mut(&mut f)
                            .push_op(ir::Op::mov(local, *value)),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    f
}

fn transform_st(af: &ast::FDecl, st: &ast::Statement) {}
