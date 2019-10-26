#![allow(dead_code)]

use once_cell::sync::Lazy;
use std::rc::Rc;
use std::sync::Mutex;

pub struct Func {
    block: Block,
}

pub struct Block {
    pub head: Rc<Label>,
    pub locals: Vec<Rc<Local>>,
    pub ops: Vec<Op>,
}

pub struct Label {
    pub name: String,
}

static LABEL_SEED: Lazy<Mutex<i64>> = Lazy::new(|| Mutex::new(0));

impl Label {
    pub fn new() -> Self {
        let mut seed = LABEL_SEED.lock().unwrap();
        let l = Self {
            name: format!("label_{}", seed),
        };
        *seed += 1;
        l
    }
}

pub struct Local {
    block: Rc<Block>,

    name: String,
    typ: Type,
}

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

pub enum Op {
    Mov(Mov),
    Add(Add),
    Jge(Jge),
    Jmp(Jmp),
    Label(Label),
}

pub struct Mov {
    pub dst: Location,
    pub src: Location,
}

pub struct Add {
    pub lhs: Location,
    pub rhs: Location,
}

pub struct Cmp {
    pub lhs: Location,
    pub rhs: Location,
}

pub struct Jge {
    pub dest: Rc<Label>,
}

pub struct Jmp {
    pub dest: Rc<Label>,
}

pub enum Location {
    Displaced(Displaced),
    Register(Register),
    Variable(Rc<Local>),
    Immediate(i64),
}

pub struct Displaced {
    pub register: Register,
    pub displacement: i64,
}

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
