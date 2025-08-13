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


impl From<Value> for Expr {
  fn from(val: Value) -> Self { Expr::Value(Box::new(val)) }
}

impl From<&Value> for Expr {
  fn from(val: &Value) -> Self { Expr::Value(Box::new(val.clone())) }
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




impl From<i32> for Value {
  fn from(n: i32) -> Self { Value::Int(n) }
}
impl From<f64> for Value {
  fn from(n: f64) -> Self { Value::Float(n) }
}
impl From<bool> for Value {
  fn from(b: bool) -> Self { Value::Boolean(b) }
}
impl From<String> for Value {
  fn from(s: String) -> Self { Value::String(s) }
}
impl From<&str> for Value {
  fn from(s: &str) -> Self { Value::String(s.into()) }
}

// Collections
impl From<Vec<Rc<Value>>> for Value {
  fn from(v: Vec<Rc<Value>>) -> Self { Value::Array(v) }
}
impl From<Vec<Value>> for Value {
  fn from(v: Vec<Value>) -> Self { Value::Array(v.into_iter().map(Rc::new).collect()) }
}
impl From<im::HashMap<String, Rc<Value>>> for Value {
  fn from(m: im::HashMap<String, Rc<Value>>) -> Self { Value::Object(m) }
}



use std::convert::TryFrom;

#[derive(Debug)]
pub struct ValueCastError(&'static str);

impl TryFrom<&Value> for i32 {
  type Error = ValueCastError;
  fn try_from(v: &Value) -> Result<Self, Self::Error> {
    match v { Value::Int(n) => Ok(*n), _ => Err(ValueCastError("expected Int")) }
  }
}
impl TryFrom<&Value> for f64 {
  type Error = ValueCastError;
  fn try_from(v: &Value) -> Result<Self, Self::Error> {
    match v { Value::Float(n) => Ok(*n), _ => Err(ValueCastError("expected Float")) }
  }
}
impl TryFrom<&Value> for bool {
  type Error = ValueCastError;
  fn try_from(v: &Value) -> Result<Self, Self::Error> {
    match v { Value::Boolean(b) => Ok(*b), _ => Err(ValueCastError("expected Boolean")) }
  }
}
impl TryFrom<&Value> for String {
  type Error = ValueCastError;
  fn try_from(v: &Value) -> Result<Self, Self::Error> {
    match v { Value::String(s) => Ok(s.clone()), _ => Err(ValueCastError("expected String")) }
  }
}
impl TryFrom<&Value> for Vec<Rc<Value>> {
  type Error = ValueCastError;
  fn try_from(v: &Value) -> Result<Self, Self::Error> {
    match v { Value::Array(items) => Ok(items.clone()), _ => Err(ValueCastError("expected Array")) }
  }
}
impl TryFrom<&Value> for im::HashMap<String, Rc<Value>> {
  type Error = ValueCastError;
  fn try_from(v: &Value) -> Result<Self, Self::Error> {
    match v { Value::Object(m) => Ok(m.clone()), _ => Err(ValueCastError("expected Object")) }
  }
}


impl From<i32> for Expr { fn from(n: i32) -> Self { Value::from(n).into() } }
impl From<f64> for Expr { fn from(n: f64) -> Self { Value::from(n).into() } }
impl From<bool> for Expr { fn from(b: bool) -> Self { Value::from(b).into() } }
impl From<&str> for Expr { fn from(s: &str) -> Self { Value::from(s).into() } }
impl From<String> for Expr { fn from(s: String) -> Self { Value::from(s).into() } }

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
pub fn mk_let(name: String, value: Expr, body: Expr) -> Expr {
  Expr::Let(name, Box::new(value), Box::new(body))
}

pub fn mk_array(elems: Vec<Expr>) -> Expr {
  Expr::Array(elems.into_iter().map(|e| ArrElem::Expr(e)).collect())
}

pub fn mk_index(primary: Expr, index: Expr) -> Expr {
  Expr::Index(Box::new(primary), Box::new(index))
}

pub fn mk_access(primary: Expr, property: String) -> Expr {
  Expr::Access(Box::new(primary), property)
}

pub fn mk_binop(left: Expr, op: &'static str, right: Expr) -> Expr {
  Expr::Binop(Box::new(left), op.into(), Box::new(right))
}

