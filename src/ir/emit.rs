use crate::ir;
use ir::{Girthy, Located};
use std::io;

static CODE_INDENT: &'static str = "              ";

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
                offset += l.typ.byte_width(f);
            }
        }

        {
            let b = self.0[self.0.len() - 1];
            let b = b.borrow(f);
            for l in &b.locals[..(l.1 + 1)] {
                offset += l.typ.byte_width(f);
            }
        }

        offset
    }
}

pub fn emit_main(w: &mut dyn io::Write, f: &ir::Func) -> Result<(), std::io::Error> {
    write!(w, "{}global _start\n", CODE_INDENT)?;
    write!(w, "{}section .text\n", CODE_INDENT)?;

    write!(w, "_start:\n")?;

    emit(w, f)?;
    Ok(())
}

pub fn emit(w: &mut dyn io::Write, f: &ir::Func) -> Result<(), std::io::Error> {
    let entry = f.entry;
    let mut stack = BlockStack::new();
    stack.push(entry);
    emit_block(w, &f, &mut stack, entry)?;
    emit_op(w, f, &mut stack, &ir::Op::Ret(None))?;

    Ok(())
}

fn instruction<F>(w: &mut dyn io::Write, name: &str, f: F) -> Result<(), std::io::Error>
where
    F: FnOnce(&mut dyn io::Write) -> Result<(), std::io::Error>,
{
    write!(w, "{}{:<10}", CODE_INDENT, name)?;
    f(w)?;
    write!(w, "\n")?;
    Ok(())
}

fn emit_block(
    w: &mut dyn io::Write,
    f: &ir::Func,
    stack: &mut BlockStack,
    block: ir::BlockRef,
) -> Result<(), std::io::Error> {
    let block = block.borrow(f);

    emit_op(
        w,
        f,
        stack,
        &ir::Op::Mov(ir::Mov {
            dst: ir::Location::Register(ir::Register::RBP),
            src: ir::Location::Register(ir::Register::RSP),
        }),
    )?;
    instruction(w, "sub", |w| {
        write!(w, "rsp, {}", block.locals_girth(f))?;
        Ok(())
    })?;

    for op in &block.ops {
        emit_op(w, f, stack, op)?;
    }

    Ok(())
}

fn emit_op(
    w: &mut dyn io::Write,
    f: &ir::Func,
    stack: &mut BlockStack,
    op: &ir::Op,
) -> Result<(), std::io::Error> {
    match op {
        ir::Op::Label(ref l) => {
            let l = l.borrow(f);
            write!(w, "{}:\n", l.name)?;
        }
        ir::Op::Mov(ref m) => {
            instruction(w, "mov", |w| {
                match &m.dst {
                    ir::Location::Displaced(_) | ir::Location::Local(_) => {
                        let op_size = ir::byte_width_to_opsize(m.dst.byte_width(f));
                        write!(w, "{} ", op_size)?;
                    }
                    _ => {}
                };
                emit_location(w, f, stack, &m.dst)?;
                write!(w, ", ")?;
                emit_location(w, f, stack, &m.src)?;
                Ok(())
            })?;
        }
        ir::Op::Cmp(ref c) => {
            instruction(w, "cmp", |w| {
                emit_location(w, f, stack, &c.lhs)?;
                write!(w, ", ")?;
                emit_location(w, f, stack, &c.rhs)?;
                Ok(())
            })?;
        }
        ir::Op::Jg(ref j) => {
            instruction(w, "jg", |w| {
                let l = j.dst.borrow(f);
                write!(w, "{}", l.name)?;
                Ok(())
            })?;
        }
        ir::Op::Ret(ref l) => {
            if let Some(l) = l {
                emit_op(
                    w,
                    f,
                    stack,
                    &ir::Op::Mov(ir::Mov {
                        dst: ir::Register::RAX.loc(),
                        src: *l,
                    }),
                )?;
            }

            instruction(w, "ret", |w| {
                write!(w, "0")?;
                Ok(())
            })?;
        }
        _ => unimplemented!(),
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
        ir::Location::Register(r) => r.write_nasm_name(w)?,
        ir::Location::Local(l) => {
            emit_location(
                w,
                f,
                stack,
                &ir::Register::RBP.displaced(stack.offset(f, *l)),
            )?;
        }
        ir::Location::Displaced(d) => {
            write!(w, "[")?;
            d.register.write_nasm_name(w)?;

            if d.displacement > 0 {
                write!(w, "+{}", d.displacement)?;
            } else if d.displacement < 0 {
                write!(w, "-{}", -d.displacement)?;
            }
            write!(w, "]")?;
        }
        ir::Location::Imm64(v) => write!(w, "{}", v)?,
    }

    Ok(())
}
