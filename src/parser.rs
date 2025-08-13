

use im::HashMap;
use std::rc::Rc;

/// The language syntax ---------------------------------------------------------
#[derive(Clone, Debug)]
enum Expr {
    Var(String),
    Int(i32),
    Fn(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Array(Vec<ArrElem>),
    Object(Vec<ObjElem>)

}

#[derive(Clone, Debug)]
enum ArrElem{
  Expr(Expr),
  Spread(Spread)
}

#[derive(Clone, Debug)]
enum ObjElem{
  Expr((String,Expr)),
  Spread(Spread)
}

#[derive(Clone, Debug)]
struct Spread {
  arg:Expr
}

/// Runtime values --------------------------------------------------------------
#[derive(Clone, Debug)]
enum Value {
    Int(i32),
    Closure(Closure),

}

#[derive(Clone, Debug)]
struct Closure {
    params: Vec<String>,
    body: Expr,
    env: EnvRef,            // lexical environment captured at definition time
}

/// Environment -----------------------------------------------------------------
#[derive(Clone, Debug)]
struct EnvData {
    bindings: HashMap<String, VRef>,
    parent:   Option<EnvRef>,
}
type EnvRef = Rc<EnvData>;
type VRef   = Rc<Value>;

fn v(val: Value) -> VRef {
    Rc::new(val)
}

/// Create a new (empty) frame whose parent is `parent`.
fn env_extend(parent: Option<EnvRef>) -> EnvRef {
    Rc::new(EnvData {
        bindings: HashMap::new(),
        parent,
    })
}



pub fn parse(input: &str) -> Result<Expr, pest::error::Error<String>> {
    todo!("implement me!")
}

