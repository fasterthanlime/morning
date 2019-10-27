#![allow(dead_code)]

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
    pub entry: BlockRef,
    pub blocks: Vec<Block>,
}

impl Func {
    pub fn new() -> Self {
        let blocks = vec![];
        let mut f = Func {
            entry: BlockRef(0),
            blocks,
        };
        f.add_block(); // add entry block
        f
    }

    pub fn add_block(&mut self) -> BlockRef {
        let start_owned = Label::new();
        let block = BlockRef(self.blocks.len());
        let start = LabelRef(block, 0);
        let ops = vec![Op::Label(start)];

        let block_owned = Block {
            own_ref: block,
            start,
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
    pub start: LabelRef,
    pub locals: Vec<Local>,
    pub labels: Vec<Label>,
    pub ops: Vec<Op>,
}

impl Block {
    pub fn add_local<S: Into<String>>(&mut self, name: S, typ: Type) -> LocalRef {
        let local_owned = Local {
            name: name.into(),
            typ,
        };
        let local = LocalRef(self.own_ref, self.locals.len());
        self.locals.push(local_owned);
        local
    }

    pub fn add_label(&mut self) -> LabelRef {
        let label_owned = Label::new();
        let label = LabelRef(self.own_ref, self.labels.len());
        self.labels.push(label_owned);
        label
    }

    pub fn add_op(&mut self, op: Op) {
        self.ops.push(op)
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

impl Type {
    pub fn byte_width(&self) -> i64 {
        match self {
            Self::I64 => 8,
        }
    }
}

#[derive(Debug)]
pub enum Op {
    Mov(Mov),
    Add(Add),
    Jge(Jge),
    Jmp(Jmp),
    Label(LabelRef),
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
pub struct Cmp {
    pub lhs: Location,
    pub rhs: Location,
}

#[derive(Debug)]
pub struct Jge {
    pub dst: LabelRef,
}

#[derive(Debug)]
pub struct Jmp {
    pub dst: LabelRef,
}

#[derive(Debug)]
pub enum Location {
    Displaced(Displaced),
    Register(Register),
    Local(LocalRef),
    Immediate(i64),
}

#[derive(Debug)]
pub struct Displaced {
    pub register: Register,
    pub displacement: i64,
}

#[derive(Debug)]
pub enum Register {
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
