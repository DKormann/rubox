// Needed imports for inner type definitions
use std::rc::Rc;
use im::HashMap;

#[derive(Clone, Debug)]
pub enum Expr {
    Var(String),
    Int(i32),
    Fn(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),

    

}

#[derive(Clone, Debug)]
pub enum Value {
    #[allow(dead_code)]
    Int(i32),
    Closure(Closure),
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub params: Vec<String>,
    pub body: Expr,
    pub env: EnvRef,
}

#[derive(Clone, Debug)]
pub struct EnvData {
    pub bindings: HashMap<String, VRef>,
    pub parent:   Option<EnvRef>,
}
pub type EnvRef = Rc<EnvData>;
pub type VRef   = Rc<Value>;
