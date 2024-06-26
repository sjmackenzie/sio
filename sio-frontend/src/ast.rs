use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::fmt;

use crate::position::WithSpan;

pub type Identifier = String;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnaryOperator {
    Bang,
    Minus,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BinaryOperator {
    Slash,
    Star,
    Plus,
    Minus,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    BangEqual,
    EqualEqual,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LogicalOperator {
    And,
    Or,
} 

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<WithSpan<Expr>>, WithSpan<BinaryOperator>, Box<WithSpan<Expr>>),
    Grouping(Box<WithSpan<Expr>>),
    Number(f64),
    Boolean(bool),
    Nil,
    String(String),
    Call(Box<WithSpan<Expr>>, Vec<WithSpan<Expr>>),
    UrlCall(Vec<WithSpan<String>>, Vec<WithSpan<Expr>>),
    Unary(WithSpan<UnaryOperator>, Box<WithSpan<Expr>>),
    Variable(WithSpan<Identifier>),
    Logical(Box<WithSpan<Expr>>, WithSpan<LogicalOperator>, Box<WithSpan<Expr>>),
    Assign(WithSpan<Identifier>, Box<WithSpan<Expr>>),
    Get(Box<WithSpan<Expr>>, WithSpan<Identifier>),
    Set(Box<WithSpan<Expr>>, WithSpan<Identifier>, Box<WithSpan<Expr>>),
    List(Vec<WithSpan<Expr>>),
    ListGet(Box<WithSpan<Expr>>, Box<WithSpan<Expr>>),
    ListSet(Box<WithSpan<Expr>>, Box<WithSpan<Expr>>, Box<WithSpan<Expr>>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Url(Box<WithSpan<Identifier>>, HierarchicalName),
    Expression(Box<WithSpan<Expr>>),
    Print(Box<WithSpan<Expr>>),
    If(Box<WithSpan<Expr>>, Box<WithSpan<Stmt>>, Option<Box<WithSpan<Stmt>>>),
    Block(Vec<WithSpan<Stmt>>),
    //Let(WithSpan<Identifier>, WithSpan<Identifier>, Option<WithSpan<Expr>>),
    //LetMultiple(Vec<WithSpan<Identifier>>, WithSpan<Identifier>),
    Let(WithSpan<Identifier>, Option<WithSpan<Expr>>),
    LetMultiple(Vec<WithSpan<Identifier>>),
    Thread(Vec<WithSpan<Stmt>>),
    Function(Function),
    Use(WithSpan<String>, Option<Vec<WithSpan<String>>>),
    Module(Module),
    Return(Box<WithSpan<Expr>>),
}

pub type Ast = Vec<WithSpan<Stmt>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Module {
    Corporal { name: HierarchicalName, stmts: Vec<WithSpan<Stmt>> },
    Major { name: HierarchicalName, stmts: Vec<WithSpan<Stmt>> },
    Brigadier { name: HierarchicalName, stmts: Vec<WithSpan<Stmt>>},
    General { name: HierarchicalName, stmts: Vec<WithSpan<Stmt>>},
}

#[derive(Debug, PartialEq, Clone)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param {
    pub name: WithSpan<Identifier>,
    pub param_type: WithSpan<Identifier>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub visibility: Visibility,
    pub name: Option<WithSpan<Identifier>>,
    pub params: Vec<Param>,
    pub return_type: Option<WithSpan<Identifier>>,
    pub body: Vec<WithSpan<Stmt>>,
}

     
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchicalName {
    pub parts: Vec<WithSpan<String>>,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Bang => write!(f, "!"),
            UnaryOperator::Minus => write!(f, "-"),
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Slash => write!(f, "/"),
            BinaryOperator::Star => write!(f, "*"),
            BinaryOperator::Plus => write!(f, "+"),
            BinaryOperator::Minus => write!(f, "-"),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::BangEqual => write!(f, "!="),
            BinaryOperator::EqualEqual => write!(f, "=="),
        }
    }
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "&&"),
            LogicalOperator::Or => write!(f, "||"),
        }
    }
}

impl HierarchicalName {
    pub fn new(parts: Vec<WithSpan<String>>) -> Self {
        Self { parts }
    }

    pub fn to_string(&self) -> String {
        self.parts.iter()
            .map(|part| part.value.clone())
            .collect::<Vec<String>>()
            .join("::")
    }
}
