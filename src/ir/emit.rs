use crate::ir;
use std::io;

struct BlockStack(Vec<ir::BlockRef>);

impl BlockStack {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, b: ir::BlockRef) {
        self.0.push(b)
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn offset(&self, f: &ir::Func, l: ir::LocalRef) -> i64 {
        let mut offset = 0i64;
        for b in &self.0[..self.0.len() - 1] {
            let b = b.borrow(f);
            for l in &b.locals {
                offset += l.typ.byte_width();
            }
        }

        {
            let b = self.0[self.0.len() - 1];
            let b = b.borrow(f);
            for l in &b.locals[..(l.1 + 1)] {
                offset += l.typ.byte_width();
            }
        }

        offset
    }
}

pub fn emit(w: &mut dyn io::Write, f: &ir::Func) -> Result<(), std::io::Error> {
    let entry = f.entry;
    let mut stack = BlockStack::new();
    stack.push(entry);
    emit_block(w, &f, &mut stack, entry)?;

    Ok(())
}

fn emit_block(
    w: &mut dyn io::Write,
    f: &ir::Func,
    stack: &mut BlockStack,
    block: ir::BlockRef,
) -> Result<(), std::io::Error> {
    let block = block.borrow(f);
    for op in &block.ops {
        write!(w, "; {:?}\n", op)?;
        match op {
            ir::Op::Label(ref l) => {
                let l = l.borrow(f);
                write!(w, "{}:\n", l.name)?;
            }
            ir::Op::Mov(ref m) => {
                write!(w, "mov ")?;
                emit_location(w, f, stack, &m.dst)?;
                write!(w, ", ")?;
                emit_location(w, f, stack, &m.src)?;
                write!(w, "\n")?;
            }
            _ => unimplemented!(),
        }
    }

    Ok(())
}

fn emit_location(
    w: &mut dyn io::Write,
    f: &ir::Func,
    stack: &mut BlockStack,
    loc: &ir::Location,
) -> Result<(), std::io::Error> {
    match loc {
        ir::Location::Local(l) => write!(w, "[rbp-{}]", stack.offset(f, *l))?,
        ir::Location::Immediate(v) => write!(w, "{}", v)?,
        _ => unimplemented!(),
    }

    Ok(())
}
