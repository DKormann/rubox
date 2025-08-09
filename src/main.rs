use im::{HashMap, Vector};
use std::rc::Rc;

type VRef = Rc<Value>;
type EnvRef = Rc<Env>;

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Float(f32),
    Int(i32),
    Str(String),
    Null,
    Object(HashMap<String, VRef>),
    Array(Vector<VRef>),
    Function(Function),
}

#[derive(Debug, Clone)]
struct Function {
    params: Vec<String>,
    body: Expr,
    env: EnvRef, // captured lexical env
}

#[derive(Debug, Clone)]
struct Env {
    map: HashMap<String, VRef>,
    parent: Option<EnvRef>,
}

impl Env {
    fn new(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(Env { map: HashMap::new(), parent })
    }
    fn extend(&self, name: String, val: VRef) -> EnvRef {
        Rc::new(Env {
            map: self.map.update(name, val),
            parent: self.parent.clone(),
        })
    }
    fn lookup(&self, name: &str) -> Option<VRef> {
        self.map.get(name).cloned().or_else(|| {
            self.parent.as_ref().and_then(|p| p.lookup(name))
        })
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Var(String),
    Bool(bool),
    Float(f32),
    Int(i32),
    Str(String),
    Null,
    Object(Vec<(String, Expr)>),
    Array(Vec<Expr>),
    Fn(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>),
    Binary(Box<Expr>, String, Box<Expr>),
    Unary(String, Box<Expr>),
    Prop(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
enum RuntimeError {
    TypeError(String),
    NotFound(String),
    ArityError(usize, usize),
}

fn v(val: Value) -> VRef { Rc::new(val) }

fn eval(expr: &Expr, env: EnvRef) -> Result<VRef, RuntimeError> {
    use Expr::*;
    match expr {
        Var(name) => env.lookup(name).ok_or(RuntimeError::NotFound(name.clone())),
        Bool(b) => Ok(v(Value::Bool(*b))),
        Float(n) => Ok(v(Value::Float(*n))),
        Int(n) => Ok(v(Value::Int(*n))),
        Str(s) => Ok(v(Value::Str(s.clone()))),
        Null => Ok(v(Value::Null)),
        Object(fields) => {
            let mut map = HashMap::new();
            for (k, e) in fields {
                let val = eval(e, env.clone())?;
                map.insert(k.clone(), val);
            }
            Ok(v(Value::Object(map)))
        }
        Array(items) => {
            let mut vec = Vector::new();
            for e in items {
                vec.push_back(eval(e, env.clone())?);
            }
            Ok(v(Value::Array(vec)))
        }
        Fn(params, body) => Ok(v(Value::Function(Function {
            params: params.clone(),
            body: *body.clone(),
            env: env.clone(),
        }))),
        Call(fn_expr, args_expr) => {
            let fval = eval(fn_expr, env.clone())?;
            let args: Result<Vec<_>, _> = args_expr.iter().map(|a| eval(a, env.clone())).collect();
            let args = args?;
            match fval.as_ref() {
                Value::Function(fun) => {
                    if args.len() != fun.params.len() {
                        return Err(RuntimeError::ArityError(fun.params.len(), args.len()));
                    }
                    let mut new_map = HashMap::new();
                    for (p, arg) in fun.params.iter().zip(args) {
                        new_map.insert(p.clone(), arg);
                    }
                    let call_env = Rc::new(Env {
                        map: new_map,
                        parent: Some(fun.env.clone()),
                    });
                    eval(&fun.body, call_env)
                }
                _ => Err(RuntimeError::TypeError("call on non-function".into())),
            }
        }
        Let(name, rhs, body) => {

            //allow for recursive function
            let val = eval(rhs, env.clone())?;
            let new_env = Rc::new(Env {
                map: env.map.update(name.clone(), val),
                parent: env.parent.clone(),
            });
            eval(body, new_env)
        }
        
        
        IfElse(c, t, f) => match eval(c, env.clone())?.as_ref() {
            Value::Bool(true) => eval(t, env),
            Value::Bool(false) => eval(f, env),
            _ => Err(RuntimeError::TypeError("ternary needs bool".into())),
        },
        Binary(a, op, b) => {
            let av = eval(a, env.clone())?;
            let bv = eval(b, env)?;
            eval_binary(op, av, bv)
        }
        Unary(op, e) => {
            let ev = eval(e, env)?;
            match (op.as_str(), ev.as_ref()) {
                ("!", Value::Bool(b)) => Ok(v(Value::Bool(!b))),
                ("-", Value::Float(n)) => Ok(v(Value::Float(-n))),
                _ => Err(RuntimeError::TypeError("invalid unary".into())),
            }
        }
        Prop(obj_expr, key) => match eval(obj_expr, env)?.as_ref() {
            Value::Object(map) => map.get(key).cloned().ok_or(RuntimeError::NotFound(key.clone())),
            _ => Err(RuntimeError::TypeError("prop on non-object".into())),
        },
        Index(arr_expr, idx_expr) => {
            let coll = eval(arr_expr, env.clone())?;
            let idxv = eval(idx_expr, env)?;
            match (coll.as_ref(), idxv.as_ref()) {
                (Value::Array(vec), Value::Int(n)) => {
                    vec.get(*n as usize).cloned().ok_or(RuntimeError::NotFound("index".into()))
                }
                (Value::Str(s), Value::Int(n)) => {
                    s.chars().nth(*n as usize)
                        .map(|c| v(Value::Str(c.to_string())))
                        .ok_or(RuntimeError::NotFound("char index".into()))
                }
                _ => Err(RuntimeError::TypeError("invalid index".into())),
            }
        }
    }
}

fn eval_binary(op: &str, a: VRef, b: VRef) -> Result<VRef, RuntimeError> {
    use Value::*;
    match (a.as_ref(), b.as_ref()) {
        (Float(x), Float(y)) => {
            let res = match op {
                "+" => Float(x + y),
                "-" => Float(x - y),
                "*" => Float(x * y),
                "/" => Float(x / y),
                "==" => Bool(x == y),
                "<"  => Bool(x < y),
                ">"  => Bool(x > y),
                _ => return Err(RuntimeError::TypeError(format!("op {}", op))),
            };
            Ok(v(res))
        }
        (Int(x), Int(y)) => {
            let res = match op {
                "+" => Int(x + y),
                "-" => Int(x - y),
                "*" => Int(x * y),
                "/" => Int(x / y),
                "==" => Bool(x == y),
                "<"  => Bool(x < y),
                ">"  => Bool(x > y),
                _ => return Err(RuntimeError::TypeError(format!("op {}", op))),
            };
            Ok(v(res))
        }
        (Str(s1), Str(s2)) if op == "+" => Ok(v(Str(format!("{}{}", s1, s2)))),
        (Bool(b1), Bool(b2)) if op == "&&" => Ok(v(Bool(*b1 && *b2))),
        (Bool(b1), Bool(b2)) if op == "||" => Ok(v(Bool(*b1 || *b2))),
        _ => Err(RuntimeError::TypeError("unsupported binary".into())),
    }
}


fn var (name:&'static str) -> Expr {
    Expr::Var(name.into())
}


fn binary(op:&'static str, a:Expr, b:Expr) -> Expr {
    Expr::Binary(Box::new(a), op.into(), Box::new(b))
}

fn let_(name:&'static str, val:Expr, body:Expr) -> Expr {
    Expr::Let(name.into(), Box::new(val), Box::new(body))
}

fn if_else(c:Expr, t:Expr, f:Expr) -> Expr {
    Expr::IfElse(Box::new(c), Box::new(t), Box::new(f))
}

fn fn_(params:Vec<&'static str>, body:Expr) -> Expr {
    Expr::Fn(params.into_iter().map(|s| s.into()).collect(), Box::new(body))
}

fn call(fn_expr:Expr, args_expr:Vec<Expr>) -> Expr {
    Expr::Call(Box::new(fn_expr), args_expr)
}

fn int_(n:i32) -> Expr{
    Expr::Int(n)
}

// f self x = y
// fix f = f (fix f)


fn recfun (name:&'static str, params:Vec<&'static str>, body:Expr) -> Expr{


    let mut v = vec![name];
    v.extend(params.iter().cloned());
    let sfun = fn_(v,body,);


    let slet = let_(name, sfun, sfun);
    slet
    
}


fn main() {
    let global = Env::new(None);
    let expr = let_("x", int_(1), binary("+", var("x"), int_(2)));


    let expr2 = call(fn_(vec!["x"], binary("+", var("x"), int_(2))), vec![int_(3)]);

    let expr3: Expr = let_(
        "fib",
        fn_(vec!["n".into()], if_else(
                binary("<", var("n"), int_(2)),
                int_(1),
                binary("+", 
                    call(var("fib"), vec![binary("-", var("n"),int_(1))]),
                    call(var("fib"), vec![binary("-", var("n"),int_(2))])
                ),)),

        call(var("fib"), vec![int_(5)]));


    match eval(&expr3, global) {
        Ok(v) => println!("Result = {:?}", v.as_ref()),
        Err(e) => println!("Error: {:?}", e),
    }
}
