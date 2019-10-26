use crate::parser::Span;

#[derive(Debug, Clone)]
pub struct Unit {
    pub funs: Vec<FunctionDeclaration>,
}

#[derive(Debug, Clone)]
pub enum UnitItem {
    FunctionDeclaration(FunctionDeclaration),
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub loc: Span,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub params: Vec<Parameter>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: Identifier,
    pub typ: TypeReference,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    Return(Return),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Return {
    pub expr: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Identifier,
    pub typ: Option<TypeReference>,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Call(Call),
    BinaryExpression(BinaryExpresion),
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    FloatingLiteral(FloatingLiteral),
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub loc: Span,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct FloatingLiteral {
    pub loc: Span,
    pub value: f64,
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

#[derive(Debug, Clone)]
pub struct TypeReference {
    pub id: Identifier,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub target: Box<Expression>,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpresion {
    pub operator: BinaryOperator,
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Mul,
    Div,
}
