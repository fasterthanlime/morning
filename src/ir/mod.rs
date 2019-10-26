use std::rc::Rc;

pub struct Func {
    block: Block,

    ops: Vec<Op>,
}

pub struct Block {
    // have to reserve enough space for locals
    locals: Vec<Rc<Local>>,
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
    Store(Store),
}

pub struct Store {
    pub dst: Location,
    pub src: Location,
}

pub enum Location {
    Register(Register),
    Variable(Rc<Local>),
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
