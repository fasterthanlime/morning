use crate::parser::Span;

#[derive(Debug)]
pub struct Unit {
    pub funs: Vec<FunctionDeclaration>,
}

#[derive(Debug)]
pub enum UnitItem {
    FunctionDeclaration(FunctionDeclaration),
}

#[derive(Debug)]
pub struct Identifier {
    pub loc: Span,
    pub value: String,
}

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub body: Block,
}

#[derive(Debug)]
pub struct Parameter {
    pub name: Identifier,
    pub typ: TypeReference,
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<Statement>,
}

#[derive(Debug)]
pub struct Statement {
    pub loc: Span,
}

impl Unit {
    pub fn new(mut items: Vec<UnitItem>) -> Self {
        let mut file = Unit { funs: Vec::new() };

        for item in items.drain(..) {
            match item {
                UnitItem::FunctionDeclaration(fun) => file.funs.push(fun),
            }
        }

        file
    }
}

impl Identifier {
    pub fn new(loc: Span) -> Self {
        Self {
            loc: loc.clone(),
            value: loc.slice().into(),
        }
    }
}

#[derive(Debug)]
pub struct TypeReference {
    pub id: Identifier,
}
