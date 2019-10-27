use crate::*;
use std::io::{self, Write};

static CODE_INDENT: &'static str = "              ";

struct Stack<'a> {
    w: &'a mut dyn io::Write,
    f: &'a Func,
    blocks: Vec<BlockRef>,
}

impl<'a> Stack<'a> {
    pub fn new(w: &'a mut dyn io::Write, f: &'a Func) -> Self {
        Self {
            w,
            f,
            blocks: Vec::new(),
        }
    }

    pub fn top(&self) -> BlockRef {
        self.blocks[self.blocks.len() - 1]
    }

    pub fn push(&mut self, b: BlockRef) {
        self.blocks.push(b)
    }

    pub fn pop(&mut self) {
        self.blocks.pop();
    }

    pub fn offset(&self, l: LocalRef) -> i64 {
        let mut offset = 0i64;
        for b in &self.blocks[..self.blocks.len() - 1] {
            let b = b.borrow(self.f);
            for l in &b.locals {
                offset += l.typ.byte_width(self.f);
            }
        }

        {
            let b = self.top();
            let b = b.borrow(self.f);
            for l in &b.locals[..(l.1 + 1)] {
                offset += l.typ.byte_width(self.f);
            }
        }

        offset
    }
}

impl<'a> io::Write for Stack<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

pub fn emit_main(w: &mut dyn io::Write, f: &Func) -> Result<(), std::io::Error> {
    write!(w, "{}global _start\n", CODE_INDENT)?;
    write!(w, "{}section .text\n", CODE_INDENT)?;

    write!(w, "_start:\n")?;

    emit(w, f)?;
    Ok(())
}

pub fn emit(w: &mut dyn io::Write, f: &Func) -> Result<(), std::io::Error> {
    let entry = f.entry;
    let mut st = Stack::new(w, f);
    st.push(entry);
    emit_block(&mut st, entry)?;
    emit_op(&mut st, &Op::Ret(None))?;

    Ok(())
}

fn instruction<F>(w: &mut Stack, name: &str, f: F) -> Result<(), std::io::Error>
where
    F: FnOnce(&mut Stack) -> Result<(), std::io::Error>,
{
    write!(w, "{}{:<10}", CODE_INDENT, name)?;
    f(w)?;
    write!(w, "\n")?;
    Ok(())
}

fn emit_block(st: &mut Stack, block: BlockRef) -> Result<(), std::io::Error> {
    let block = block.borrow(st.f);

    emit_op(st, &Op::mov(Reg::RBP, Reg::RSP))?;
    emit_op(st, &Op::sub(Reg::RSP, block.locals_girth(st.f)))?;

    for op in &block.ops {
        emit_op(st, op)?;
    }

    Ok(())
}

fn emit_op(st: &mut Stack, op: &Op) -> Result<(), std::io::Error> {
    match op {
        Op::Label(ref l) => {
            let l = l.borrow(st.f);
            write!(st, "{}:\n", l.name)?;
        }
        Op::Add(ref a) => {
            instruction(st, "add", |st| {
                emit_location(st, &a.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &a.rhs)?;
                Ok(())
            })?;
        }
        Op::Sub(ref s) => {
            instruction(st, "sub", |st| {
                emit_location(st, &s.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &s.rhs)?;
                Ok(())
            })?;
        }
        Op::Mov(ref m) => {
            instruction(st, "mov", |st| {
                match &m.dst {
                    Location::Displaced(_) | Location::Local(_) => {
                        let op_size = byte_width_to_opsize(m.dst.byte_width(st.f));
                        write!(st, "{} ", op_size)?;
                    }
                    _ => {}
                };
                emit_location(st, &m.dst)?;
                write!(st, ", ")?;
                emit_location(st, &m.src)?;
                Ok(())
            })?;
        }
        Op::Cmp(ref c) => {
            instruction(st, "cmp", |st| {
                emit_location(st, &c.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &c.rhs)?;
                Ok(())
            })?;
        }
        Op::Jg(ref j) => {
            instruction(st, "jg", |st| {
                let l = j.dst.borrow(st.f);
                write!(st, "{}", l.name)?;
                Ok(())
            })?;
        }
        Op::Jmp(ref j) => {
            instruction(st, "jmp", |st| {
                let l = j.dst.borrow(st.f);
                write!(st, "{}", l.name)?;
                Ok(())
            })?;
        }
        Op::Ret(ref l) => {
            if let Some(l) = l {
                emit_op(st, &Op::mov(Reg::RAX, *l))?;
            }

            let block = st.top().borrow(st.f);
            emit_op(st, &Op::add(Reg::RSP, block.locals_girth(st.f)))?;

            instruction(st, "ret", |st| {
                write!(st, "0")?;
                Ok(())
            })?;
        }
    }

    Ok(())
}

fn emit_location(st: &mut Stack, loc: &Location) -> Result<(), std::io::Error> {
    match loc {
        Location::Register(r) => r.write_nasm_name(st)?,
        Location::Local(l) => {
            emit_location(st, &Reg::RBP.displaced(-st.offset(*l)))?;
        }
        Location::Displaced(d) => {
            write!(st, "[")?;
            d.register.write_nasm_name(st)?;

            if d.displacement > 0 {
                write!(st, "+{}", d.displacement)?;
            } else if d.displacement < 0 {
                write!(st, "-{}", -d.displacement)?;
            }
            write!(st, "]")?;
        }
        Location::Imm64(v) => write!(st, "{}", v)?,
    }

    Ok(())
}
