use crate::parser::Span;

#[derive(Debug, Clone)]
pub struct Unit {
    pub funs: Vec<FDecl>,
}

#[derive(Debug, Clone)]
pub enum UnitItem {
    FDecl(FDecl),
}

#[derive(Debug, Clone)]
pub struct Id {
    pub loc: Span,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct FDecl {
    pub name: Id,
    pub params: Vec<Param>,
    pub body: Block,
    pub public: bool,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: Id,
    pub typ: TypeRef,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct If {
    pub cond: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Block(Block),
    Loop(Loop),
    If(If),

    Continue,
    Break,

    VDecl(VDecl),
    Return(Return),
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub struct Return {
    pub expr: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct VDecl {
    pub name: Id,
    pub typ: Option<TypeRef>,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Call(Call),
    Block(Block),
    Bexp(Bexp),
    Identifier(Id),
    IntLit(IntLit),
    FloatLit(FloatLit),
}

#[derive(Debug, Clone)]
pub struct IntLit {
    pub loc: Span,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct FloatLit {
    pub loc: Span,
    pub value: f64,
}

impl Unit {
    pub fn new(mut items: Vec<UnitItem>) -> Self {
        let mut file = Unit { funs: Vec::new() };

        for item in items.drain(..) {
            match item {
                UnitItem::FDecl(fun) => file.funs.push(fun),
            }
        }

        file
    }
}

impl Id {
    pub fn new(loc: Span) -> Self {
        Self {
            loc: loc.clone(),
            value: loc.slice().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeRef {
    pub id: Id,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub target: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Bexp {
    pub operator: Bop,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, Clone)]
pub enum Bop {
    Plus,
    Minus,
    Mul,
    Div,

    Gt,
    GtEq,
    Lt,
    LtEq,

    Assign,
}

impl Bop {
    pub fn as_expr(self, lhs: Box<Expr>, rhs: Box<Expr>) -> Bexp {
        Bexp {
            lhs,
            operator: self,
            rhs,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AssOp {
    Plus,
    Minus,
    Mul,
    Div,
}

impl AssOp {
    pub fn as_operator(self) -> Bop {
        match self {
            Self::Plus => Bop::Plus,
            Self::Minus => Bop::Minus,
            Self::Mul => Bop::Mul,
            Self::Div => Bop::Div,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BopEx {
    Base(Bop),
    Ass(AssOp),
}

impl BopEx {
    pub fn as_expr(self, lhs: Box<Expr>, rhs: Box<Expr>) -> Bexp {
        match self {
            Self::Base(operator) => operator.as_expr(lhs, rhs),
            Self::Ass(operator) => Bexp {
                lhs: lhs.clone(),
                operator: Bop::Assign,
                rhs: Box::new(Expr::Bexp(operator.as_operator().as_expr(lhs, rhs))),
            },
        }
    }
}
