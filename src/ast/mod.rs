use crate::parser::Span;

pub struct File {
    pub loc: Span,
    pub imports: Vec<Import>,
}

pub enum FileItem {
    Import(Import),
    Function(Function),
}

pub struct Import {
    pub loc: Span,
    pub path: String,
}

pub struct Function {
    pub loc: Span,
    pub name: String,
    pub body: Vec<Statement>,
}

pub struct Statement {
    pub loc: Span,
}

impl File {
    pub fn new(loc: Span, items: Vec<FileItem>) {}
}
