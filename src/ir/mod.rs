#![allow(dead_code)]

pub mod emit;

use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct BlockRef(usize);

impl BlockRef {
    pub fn borrow(self, f: &Func) -> &Block {
        &f.blocks[self.0]
    }

    pub fn borrow_mut(self, f: &mut Func) -> &mut Block {
        &mut f.blocks[self.0]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LabelRef(BlockRef, usize);

impl LabelRef {
    pub fn borrow(self, f: &Func) -> &Label {
        &self.0.borrow(f).labels[self.1]
    }

    pub fn borrow_mut(self, f: &mut Func) -> &mut Label {
        &mut self.0.borrow_mut(f).labels[self.1]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalRef(BlockRef, usize);

impl LocalRef {
    pub fn borrow(self, f: &Func) -> &Local {
        &self.0.borrow(f).locals[self.1]
    }

    pub fn borrow_mut(self, f: &mut Func) -> &mut Local {
        &mut self.0.borrow_mut(f).locals[self.1]
    }
}

#[derive(Debug)]
pub struct Func {
    pub public: bool,
    pub name: String,
    pub entry: BlockRef,
    pub blocks: Vec<Block>,
}

impl Func {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let blocks = vec![];
        let mut f = Func {
            name: name.into(),
            public: false,
            entry: BlockRef(0),
            blocks,
        };
        f.push_block(); // add entry block
        f
    }

    pub fn push_block(&mut self) -> BlockRef {
        let start_owned = Label::new();
        let block = BlockRef(self.blocks.len());
        let ops = vec![];

        let block_owned = Block {
            own_ref: block,
            locals: Vec::new(),
            labels: vec![start_owned],
            ops,
        };
        self.blocks.push(block_owned);
        block
    }
}

#[derive(Debug)]
pub struct Block {
    pub own_ref: BlockRef,
    pub locals: Vec<Local>,
    pub labels: Vec<Label>,
    pub ops: Vec<Op>,
}

impl Block {
    pub fn push_local<S: Into<String>>(&mut self, name: S, typ: Type) -> LocalRef {
        let local_owned = Local {
            name: name.into(),
            typ,
        };
        let local = LocalRef(self.own_ref, self.locals.len());
        self.locals.push(local_owned);
        local
    }

    pub fn new_label(&mut self) -> LabelRef {
        let label_owned = Label::new();
        let label = LabelRef(self.own_ref, self.labels.len());
        self.labels.push(label_owned);
        label
    }

    pub fn push_op<O: Into<Op>>(&mut self, op: O) {
        let op = op.into();
        match op {
            Op::Xor(ref o) => {
                if o.lhs.is_displaced() && o.rhs.is_displaced() {
                    self.push_op(Op::mov(Reg::RAX, o.lhs));
                    self.push_op(Op::xor(Reg::RAX, o.rhs));
                    self.push_op(Op::mov(o.lhs, Reg::RAX));
                    return;
                }
            }
            Op::Add(ref o) => {
                if o.lhs.is_displaced() && o.rhs.is_displaced() {
                    self.push_op(Op::mov(Reg::RAX, o.lhs));
                    self.push_op(Op::add(Reg::RAX, o.rhs));
                    self.push_op(Op::mov(o.lhs, Reg::RAX));
                    return;
                }
            }
            _ => {}
        }

        self.ops.push(op)
    }

    pub fn locals_girth(&self, f: &Func) -> i64 {
        let mut res = 0i64;
        for l in &self.locals {
            res += l.typ.byte_width(f);
        }
        res
    }
}

#[derive(Debug)]
pub struct Label {
    pub name: String,
}

static LABEL_SEED: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

impl Label {
    fn new() -> Self {
        let mut seed = LABEL_SEED.lock().unwrap();
        let l = Self {
            name: format!("label_{}", seed),
        };
        *seed += 1;
        l
    }
}

#[derive(Debug)]
pub struct Local {
    name: String,
    typ: Type,
}

#[derive(Debug)]
pub enum Type {
    I64,
}

pub trait Girthy {
    fn byte_width(&self, f: &Func) -> i64;
}

impl Girthy for Type {
    fn byte_width(&self, _f: &Func) -> i64 {
        match self {
            Self::I64 => 8,
        }
    }
}

pub trait Operand {
    fn op(self) -> Op;
}

macro_rules! impl_operand {
    ($variant: ident($typ: ident)) => {
        impl Into<Op> for $typ {
            fn into(self) -> Op {
                Op::$variant(self)
            }
        }
    };
    ($($variant: ident($typ: ident)),*$(,)?) => {
        $(impl_operand!($variant($typ));)*
    }
}

#[derive(Debug)]
pub enum Op {
    Xor(Xor),
    Mov(Mov),
    Add(Add),
    Cmp(Cmp),
    Sub(Sub),
    Jg(Jg),
    Jmp(Jmp),
    Label(LabelRef),
    Ret(Option<Location>),

    Comment(Option<String>),
}

impl_operand!(
    Xor(Xor),
    Mov(Mov),
    Add(Add),
    Sub(Sub),
    Cmp(Cmp),
    Jg(Jg),
    Jmp(Jmp),
    Label(LabelRef),
);

impl Op {
    pub fn xor<L: Into<Location>, R: Into<Location>>(lhs: L, rhs: R) -> Self {
        Xor {
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
        .into()
    }

    pub fn mov<L: Into<Location>, R: Into<Location>>(lhs: L, rhs: R) -> Self {
        Mov {
            dst: lhs.into(),
            src: rhs.into(),
        }
        .into()
    }

    pub fn add<L: Into<Location>, R: Into<Location>>(lhs: L, rhs: R) -> Self {
        Add {
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
        .into()
    }

    pub fn sub<L: Into<Location>, R: Into<Location>>(lhs: L, rhs: R) -> Self {
        Sub {
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
        .into()
    }

    pub fn cmp<L: Into<Location>, R: Into<Location>>(lhs: L, rhs: R) -> Self {
        Cmp {
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
        .into()
    }

    pub fn jg<D: Into<LabelRef>>(target: D) -> Self {
        Jg { dst: target.into() }.into()
    }

    pub fn jmp<D: Into<LabelRef>>(target: D) -> Self {
        Jmp { dst: target.into() }.into()
    }

    pub fn label(l: LabelRef) -> Self {
        l.into()
    }

    pub fn ret_none() -> Self {
        Self::Ret(None)
    }

    pub fn ret_some<L: Into<Location>>(l: L) -> Self {
        Self::Ret(Some(l.into()))
    }

    pub fn comment<N: Into<String>>(n: N) -> Self {
        Self::Comment(Some(n.into()))
    }
}

#[derive(Debug)]
pub struct Xor {
    pub lhs: Location,
    pub rhs: Location,
}

#[derive(Debug)]
pub struct Mov {
    pub dst: Location,
    pub src: Location,
}

#[derive(Debug)]
pub struct Add {
    pub lhs: Location,
    pub rhs: Location,
}

#[derive(Debug)]
pub struct Sub {
    pub lhs: Location,
    pub rhs: Location,
}

#[derive(Debug)]
pub struct Cmp {
    pub lhs: Location,
    pub rhs: Location,
}

#[derive(Debug)]
pub struct Jg {
    pub dst: LabelRef,
}

#[derive(Debug)]
pub struct Jmp {
    pub dst: LabelRef,
}

#[derive(Debug, Clone, Copy)]
pub enum Location {
    Displaced(Displaced),
    Register(Reg),
    Local(LocalRef),
    Imm64(i64),
}

impl Girthy for Location {
    fn byte_width(&self, f: &Func) -> i64 {
        match self {
            Self::Displaced(ref d) => d.register.byte_width(f),
            Self::Register(ref r) => r.byte_width(f),
            Self::Local(ref l) => {
                let l = l.borrow(f);
                l.typ.byte_width(f)
            }
            Self::Imm64(_) => 8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Displaced {
    pub register: Reg,
    pub displacement: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum Reg {
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    RBP,
    RSP,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Girthy for Reg {
    fn byte_width(&self, _f: &Func) -> i64 {
        match self {
            Self::RAX
            | Self::RBX
            | Self::RCX
            | Self::RDX
            | Self::RSI
            | Self::RDI
            | Self::RBP
            | Self::RSP
            | Self::R8
            | Self::R9
            | Self::R10
            | Self::R11
            | Self::R12
            | Self::R13
            | Self::R14
            | Self::R15 => 8,
        }
    }
}

impl Reg {
    pub fn write_nasm_name(self, w: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        let s = format!("{:?}", self);
        write!(w, "{}", s.to_lowercase())?;
        Ok(())
    }
}

impl Into<Location> for LocalRef {
    fn into(self) -> Location {
        Location::Local(self)
    }
}

impl Into<Location> for Reg {
    fn into(self) -> Location {
        Location::Register(self)
    }
}

impl Location {
    fn is_displaced(self) -> bool {
        match self {
            Location::Displaced(_) => true,
            Location::Local(_) => true,
            _ => false,
        }
    }
}

impl Reg {
    fn displaced(self, displacement: i64) -> Location {
        Location::Displaced(Displaced {
            register: self,
            displacement,
        })
    }
}

impl Into<Location> for i64 {
    fn into(self) -> Location {
        Location::Imm64(self)
    }
}

pub fn byte_width_to_opsize(width: i64) -> &'static str {
    match width {
        1 => "byte",
        2 => "word",
        4 => "dword",
        8 => "qword",
        _ => "<invalid_width>",
    }
}
