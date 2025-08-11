
#[allow(dead_code)]
// mod types;
// use types::{Closure, EnvData, EnvRef, Expr, Value, VRef};
#[allow(unreachable_code)]


#[derive(Clone, Debug)]
pub enum Expr {
    Var(String),
    Int(i32),
    Fn(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Array(Vec<VecItem>),
    Object(Vec<ObjectItem>),
    Get(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum VecItem {
    Expr(Expr),
    Spread(Expr)
}

#[derive(Clone, Debug)]
pub enum ObjectItem {
    KeyVal(String, Expr),
    Spread(Expr)
}


#[derive(Clone, Debug)]
pub enum Value {
    #[allow(dead_code)]
    Int(i32),
    Closure(Closure),
    String(String),
    Array(Vec<VRef>),
    Object(im::HashMap<String, VRef>),
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


use im::HashMap;
use std::rc::Rc;



fn v(val: Value) -> VRef {
    Rc::new(val)
}

fn env_extend(parent: Option<EnvRef>) -> EnvRef {
    Rc::new(EnvData {
        bindings: HashMap::new(),
        parent,
    })
}

fn lookup(env: &EnvRef, name: &str) -> Option<VRef> {
    env.bindings
        .get(name)
        .cloned()
        .or_else(|| env.parent.as_ref().and_then(|p| lookup(p, name)))
}

fn eval(expr: &Expr, env: &EnvRef) -> Result<VRef, String> {
    match expr {
        Expr::Var(name) => lookup(env, name)
            .ok_or_else(|| format!("unbound variable `{}`", name)),

        Expr::Int(n) => Ok(v(Value::Int(*n))),

        Expr::Fn(params, body) => {

            Ok(v(Value::Closure(Closure {
                params: params.clone(),
                body: *body.clone(),
                env: env.clone(),
            })))
        }

        Expr::Call(func_expr, arg_exprs) => {
            // Evaluate the function expression.
            let fun = eval(func_expr, env)?;

            let mut arg_vals = Vec::with_capacity(arg_exprs.len());
            for a in arg_exprs {
                arg_vals.push(eval(a, env)?);
            }

            match fun.as_ref() {
                Value::Closure(cl) => {
                    if cl.params.len() != arg_vals.len() {
                        return Err(format!(
                            "arity mismatch: expected {} arguments, got {}",
                            cl.params.len(),
                            arg_vals.len()
                        ));
                    }

                    let mut frame = HashMap::new();
                    for (param, arg) in cl.params.iter().zip(arg_vals) {
                        frame.insert(param.clone(), arg);
                    }

                    let call_env = Rc::new(EnvData {
                        bindings: frame,
                        parent: Some(cl.env.clone()),
                    });

                    eval(&cl.body, &call_env)
                }
                _ => Err("attempted call a non‑function value".into()),
            }
        }

        Expr::Let(name, val_expr, body) => {
            let temp_env = env_extend(Some(env.clone()));
            let val = match val_expr.as_ref() {
                Expr::Fn(params, f_body) => {
                    v(Value::Closure(Closure {
                        params: params.clone(),
                        body: *f_body.clone(),
                        env: temp_env.clone(),
                    }))
                }
                _ => eval(val_expr, &temp_env)?,
            };

            let mut new_bindings = temp_env.bindings.clone();
            new_bindings.insert(name.clone(), val);
            let final_env = Rc::new(EnvData {
                bindings: new_bindings,
                parent: Some(env.clone()),
            });

            eval(body, &final_env)
        }
        Expr::Array(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items {
                match item{
                    VecItem::Expr(e) => arr.push(eval(&e, env)?),
                    VecItem::Spread(e) => {
                        let spread = eval(&e, env)?;
                        let res = match spread.as_ref() {
                            Value::Array(arr) => arr.clone(),
                            _ => return Err("spread operator used with non-array value".into()),
                        };
                        arr.extend(res);
                    }
                }
            }
            Ok(v(Value::Array(arr)))
        }
        Expr::Object(items) => {
            let mut obj = HashMap::new();
            for item in items {
                match item {
                    ObjectItem::KeyVal(key, val) => {
                        obj.insert(key.clone(), eval(&val, env)?);
                    }
                    ObjectItem::Spread(e) => {
                        let spread = eval(&e, env)?;
                        match spread.as_ref() {
                            Value::Object(inner) => {
                                inner.iter().for_each(|(k, v)| {
                                    obj.insert(k.clone(), v.clone());
                                });
                            }
                            _ => return Err("spread operator used with non-object value".into()),
                        }
                    }
                }
            }
            Ok(v(Value::Object(obj)))
        }
        Expr::Get(obj_expr, index) => {
            let obj: Rc<Value> = eval(obj_expr, env)?;
            match obj.as_ref() {
                Value::Object(obj) => {
                    let idx = eval(index, env)?;
                    match idx.as_ref() {
                        Value::String(s) => {
                            let res = obj.get(s).ok_or_else(|| format!("key `{}` not found", s))?;
                            return Ok(res.clone());
                        },
                        _ => Err(format!("cant index object with non-string index {:?}", idx))
                    }
                },
                Value::Array(arr) => {
                    let idx = eval(index, env)?;
                    match idx.as_ref() {
                        Value::Int(i) => {
                            let res = arr.get(*i as usize).ok_or_else(|| "index out of bounds".to_string())?;
                            return Ok(res.clone());
                        },
                        _ => Err("attempted to index array with non-number index".into())
                    }
                },

                _ => Err("attempted to get property of non-object value".into()),
            }
        }
    }
}

fn var(s: &'static str) -> Expr {
    Expr::Var(s.into())
}
fn int(n: i32) -> Expr {
    Expr::Int(n)
}

fn fn_(params: Vec<&'static str>, body: Expr) -> Expr {
    Expr::Fn(
        params.into_iter().map(|s| s.into()).collect(),
        Box::new(body),
    )
}

fn call(func: Expr, args: Vec<Expr>) -> Expr {
    Expr::Call(Box::new(func), args)
}
fn let_(name: &'static str, value: Expr, body: Expr) -> Expr {
    Expr::Let(name.into(), Box::new(value), Box::new(body))
}

fn arr(items: Vec<Expr>) -> Expr {
    Expr::Array(items.into_iter().map(|e| VecItem::Expr(e)).collect())
}

fn get(obj: Expr, index: Expr) -> Expr {
    Expr::Get(Box::new(obj), Box::new(index))
}


fn test_let(){
    let global_env = env_extend(None);
    let exp = let_(
        "a",
        int(22),
        let_(
            "b",
            int(33),
            let_("c", int(44), var("a")), // should evaluate to 22
        ),
    );
    let res = eval(&exp, &global_env);
    match res {
        Ok(v) => println!("result: {:?}", v),
        Err(e) => eprintln!("error: {}", e),
    }
}


fn test_arr(){
    let global_env = env_extend(None);
    let exp = let_(
        "a",
        arr(vec![int(1), int(2), int(3)]),
        var("a"),
    );
    let res = eval(&exp, &global_env);
    match res {
        Ok(v) => println!("result: {:?}", v),
        Err(e) => eprintln!("error: {}", e),
    }

    let exp2 = let_(
        "a",
        arr(vec![int(1), int(2), int(3)]),
        get(var("a"), int(1)),
    );
    let res2 = eval(&exp2, &global_env);
    match res2 {
        Ok(v) => println!("result: {:?}", v),
        Err(e) => eprintln!("error: {}", e),
    }
}


fn main() {


    test_let();
    test_arr();
}