use std::rc::Rc;

use im::HashMap;


/// The language syntax ---------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
  Value(Box<Value>),
  Var(String),
  Fn(Vec<String>, Box<Expr>),
  Call(Box<Expr>, Vec<Expr>),
  Let(String, Box<Expr>, Box<Expr>),
  Array(Vec<ArrElem>),
  Object(Vec<ObjElem>),
  Index(Box<Expr>, Box<Expr>),
  Access(Box<Expr>, String),
  Binop(Box<Expr>, String, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrElem{
  Expr(Expr),
  Spread(Expr)
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjElem{
  Expr((String,Expr)),
  Spread(Expr)
}



/// Runtime values --------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Closure(Closure),
    Float(f64),
    Array(Vec<Rc<Value>>),

    Object(HashMap<String, Rc<Value>>),
    String(String),
    Boolean(bool),
    Null,
    Undefined
}


#[derive(Clone, Debug, PartialEq)]
pub struct Closure {
    pub params: Vec<String>,
    pub body: Expr,
    pub env: EnvRef,            // lexical environment captured at definition time
}

/// Environment -----------------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct EnvData {
    pub bindings: HashMap<String, VRef>,
    pub parent:   Option<EnvRef>,
}
pub type EnvRef = Rc<EnvData>;
pub type VRef   = Rc<Value>;

fn v(val: Value) -> VRef {
    Rc::new(val)
}




pub fn mk_var(s: &'static str) -> Expr {
  Expr::Var(s.into())
}
pub fn mk_int(n: i32) -> Expr {
  Expr::Value(Box::new(Value::Int(n)))
}

#[allow(dead_code)]
pub fn mk_fn(params: Vec<String>, body: Expr) -> Expr {
  Expr::Fn(
    params.into_iter().map(|s| s.into()).collect(),
    Box::new(body),
  )
}

#[allow(dead_code)]
pub fn mk_call(func: Expr, args: Vec<Expr>) -> Expr {
  Expr::Call(Box::new(func), args)
}
pub fn mk_let(name: &'static str, value: Expr, body: Expr) -> Expr {
  Expr::Let(name.into(), Box::new(value), Box::new(body))
}

pub fn mk_array(elems: Vec<Expr>) -> Expr {
  Expr::Array(elems.into_iter().map(|e| ArrElem::Expr(e)).collect())
}

pub fn mk_index(primary: Expr, index: Expr) -> Expr {
  Expr::Index(Box::new(primary), Box::new(index))
}

pub fn mk_access(primary: Expr, property: &'static str) -> Expr {
  Expr::Access(Box::new(primary), property.into())
}

pub fn mk_binop(left: Expr, op: &'static str, right: Expr) -> Expr {
  Expr::Binop(Box::new(left), op.into(), Box::new(right))
}

