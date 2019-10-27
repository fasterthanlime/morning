use crate::{ast, ir};
use std::collections::HashMap;
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

struct Stack {
    f: ir::Func,
    items: Vec<Item>,
}

impl Stack {
    pub fn new(f: ir::Func) -> Self {
        let items = vec![Scope::new(f.entry).into()];
        Self { f, items }
    }

    pub fn f(&mut self) -> &mut ir::Func {
        &mut self.f
    }

    pub fn scope(&mut self) -> &Scope {
        self.items
            .iter()
            .rev()
            .find_map(|i| match i {
                Item::Scope(s) => Some(s),
                _ => None,
            })
            .expect("middle-end stack should always have a scope")
    }

    pub fn block(&mut self) -> &mut ir::Block {
        self.scope().block.borrow_mut(&mut self.f)
    }

    pub fn push<F, R, I>(&mut self, item: I, f: F) -> R
    where
        F: Fn(&mut Self) -> R,
        I: Into<Item>,
    {
        self.items.push(item.into());
        let r = f(self);
        self.items.pop();
        r
    }

    pub fn into_inner(self) -> ir::Func {
        if self.items.len() != 1 {
            panic!("middle-end stack should have exactly 1 items at the end of codegen")
        }
        self.f
    }
}

enum Item {
    Loop(Loop),
    Scope(Scope),
}

struct Loop {
    continue_label: ir::LabelRef,
    break_label: ir::LabelRef,
}

struct Scope {
    block: ir::BlockRef,
    names: HashMap<String, ir::LocalRef>,
}

impl Scope {
    pub fn new(block: ir::BlockRef) -> Self {
        Self {
            block,
            names: HashMap::new(),
        }
    }
}

impl Into<Item> for Scope {
    fn into(self) -> Item {
        Item::Scope(self)
    }
}

fn transform_fdecl(af: &ast::FDecl) -> ir::Func {
    let mut f = ir::Func::new(af.name.value.clone());
    f.public = af.public;
    let mut st = Stack::new(f);

    for stat in &af.body.items {
        transform_stat(&mut st, stat);
    }

    st.into_inner()
}

fn transform_stat(st: &mut Stack, stat: &ast::Statement) {
    match stat {
        ast::Statement::Expr(ex) => match ex {
            ast::Expr::Bexp(bexp) => match bexp.operator {
                ast::Bop::Assign => match bexp.lhs.as_ref() {
                    ast::Expr::Identifier(_id) => {
                        st.block().push_op(ir::Op::mov(ir::Reg::RAX, 420));
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        },
        ast::Statement::VDecl(vd) => {
            st.block()
                .push_op(ir::Op::comment(format!("vdecl {}", vd.name.value)));
            let local = st.f().push_local(vd.name.value.clone(), ir::Type::I64);

            if let Some(value) = vd.value.as_ref() {
                match value {
                    ast::Expr::IntLit(ast::IntLit { value, .. }) => {
                        st.block().push_op(ir::Op::mov(local, *value))
                    }
                    _ => {}
                }
            }
        }
        ast::Statement::Loop(l) => {
            let continue_label = st.block().new_label();
            let break_label = st.block().new_label();

            st.push(
                Item::Loop(Loop {
                    continue_label,
                    break_label,
                }),
                |st| {
                    let loop_block = st.f().push_block();
                    st.block().push_op(loop_block);
                    st.push(Scope::new(loop_block), |st| {
                        st.block().push_op(continue_label);
                        for stat in &l.body.items {
                            transform_stat(st, stat);
                        }
                        st.block().push_op(ir::Op::jmp(continue_label));
                        st.block().push_op(break_label);
                    });
                },
            );
        }
        _ => {}
    }
}
