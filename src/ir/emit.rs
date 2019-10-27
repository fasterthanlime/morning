use crate::*;
use std::io::{self, Write};

static CODE_INDENT: &'static str = "            ";

type Result = std::result::Result<(), io::Error>;

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

    pub fn push<F, R>(&mut self, b: BlockRef, f: F) -> R
    where
        F: Fn(&mut Self) -> R,
    {
        self.blocks.push(b);
        let r = f(self);
        self.exit_n_blocks(1);
        self.blocks.pop();
        r
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

    pub fn exit_all_blocks(&mut self) -> Result {
        let n = self.blocks.len();
        self.exit_n_blocks(n)
    }

    pub fn exit_n_blocks(&mut self, n: usize) -> Result {
        let mut dealloc_size = 0i64;
        for b in self.blocks.iter().rev().take(n) {
            dealloc_size += b.borrow(self.f).locals_girth(self.f);
        }

        emit_op(
            self,
            &Op::comment(format!(
                "exiting {} {}",
                n,
                match n {
                    1 => "block",
                    _ => "blocks",
                }
            )),
        )?;
        emit_op(self, &Op::add(Reg::RSP, dealloc_size))?;
        Ok(())
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

pub fn emit_all(w: &mut dyn io::Write, funcs: &[&Func]) -> Result {
    for f in funcs {
        if f.public {
            write!(w, "{}global {}\n", CODE_INDENT, f.name)?;
        }
    }

    write!(w, "{}section .text\n", CODE_INDENT)?;

    for f in funcs {
        write!(w, "{}:\n", f.name)?;
        emit_func(w, f)?;
    }

    Ok(())
}

fn emit_func(w: &mut dyn io::Write, f: &Func) -> Result {
    let entry = f.entry;
    let mut st = Stack::new(w, f);

    emit_op(&mut st, &Op::mov(Reg::RBP, Reg::RSP))?;
    st.push(entry, |st| -> Result {
        emit_block(st, entry)?;
        emit_op(st, &Op::Ret(None))?;
        Ok(())
    })?;

    Ok(())
}

fn instruction<F>(st: &mut Stack, name: &str, f: F) -> Result
where
    F: FnOnce(&mut Stack) -> Result,
{
    write!(st, "{}{:<10}", CODE_INDENT, name)?;
    f(st)?;
    write!(st, "\n")?;
    Ok(())
}

fn comment(st: &mut Stack, text: &str) -> Result {
    write!(st, "{}; {}\n", CODE_INDENT, text)?;
    Ok(())
}

fn emit_block(st: &mut Stack, block: BlockRef) -> Result {
    let block = block.borrow(st.f);

    comment(st, "block prologue start")?;
    emit_op(st, &Op::sub(Reg::RSP, block.locals_girth(st.f)))?;

    for op in &block.ops {
        emit_op(st, op)?;
    }

    Ok(())
}

fn emit_op(st: &mut Stack, op: &Op) -> Result {
    match op {
        Op::Block(ref b) => st.push(*b, |st| -> Result {
            emit_block(st, *b)?;
            Ok(())
        })?,
        Op::Label(ref l) => {
            let l = l.borrow(st.f);
            write!(st, "{}:\n", l.name)?;
        }
        Op::Xor(ref o) => {
            instruction(st, "xor", |st| {
                emit_opsize(st, &o.lhs)?;
                emit_location(st, &o.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &o.rhs)?;
                Ok(())
            })?;
        }
        Op::Add(ref o) => {
            instruction(st, "add", |st| {
                emit_opsize(st, &o.lhs)?;
                emit_location(st, &o.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &o.rhs)?;
                Ok(())
            })?;
        }
        Op::Sub(ref o) => {
            instruction(st, "sub", |st| {
                emit_opsize(st, &o.lhs)?;
                emit_location(st, &o.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &o.rhs)?;
                Ok(())
            })?;
        }
        Op::Mov(ref o) => {
            instruction(st, "mov", |st| {
                emit_opsize(st, &o.dst)?;
                emit_location(st, &o.dst)?;
                write!(st, ", ")?;
                emit_location(st, &o.src)?;
                Ok(())
            })?;
        }
        Op::Cmp(ref o) => {
            instruction(st, "cmp", |st| {
                emit_opsize(st, &o.lhs)?;
                emit_location(st, &o.lhs)?;
                write!(st, ", ")?;
                emit_location(st, &o.rhs)?;
                Ok(())
            })?;
        }
        Op::Jg(ref o) => {
            instruction(st, "jg", |st| {
                let l = o.dst.borrow(st.f);
                write!(st, "{}", l.name)?;
                Ok(())
            })?;
        }
        Op::Jmp(ref o) => {
            instruction(st, "jmp", |st| {
                let l = o.dst.borrow(st.f);
                write!(st, "{}", l.name)?;
                Ok(())
            })?;
        }
        Op::Ret(ref o) => {
            if let Some(o) = o {
                emit_op(st, &Op::mov(Reg::RAX, *o))?;
            }

            st.exit_all_blocks()?;

            instruction(st, "ret", |st| {
                write!(st, "0")?;
                Ok(())
            })?;
        }
        Op::Comment(ref c) => {
            comment(st, c.as_ref().map(|s| &s[..]).unwrap_or(""))?;
        }
    }

    Ok(())
}

fn emit_opsize(st: &mut Stack, loc: &Location) -> Result {
    if loc.is_displaced() {
        let op_size = byte_width_to_opsize(loc.byte_width(st.f));
        write!(st, "{} ", op_size)?;
    }
    Ok(())
}

fn emit_location(st: &mut Stack, loc: &Location) -> Result {
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
