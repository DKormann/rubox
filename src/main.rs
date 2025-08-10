use im::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
enum Expr {
    Var(String),
    Int(i32),
    Fn(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    // ... add other forms
}

#[derive(Clone, Debug)]
enum Value {
    Int(i32),
    Closure(Closure),
}

#[derive(Clone, Debug)]
struct Closure {
    params: Vec<String>,
    body: Expr,
    env: EnvRef, // captured lexical env
}

type VRef = Rc<Value>;
type Env = HashMap<String, VRef>;
type EnvRef = Rc<Env>;

fn v(val: Value) -> VRef { Rc::new(val) }

fn env_new(parent: Option<&EnvRef>) -> EnvRef {
    // new empty frame with optional parent: we represent parent by closure capture,
    // so parent is not stored explicitly here (we'll use chained lookup below).
    Rc::new(HashMap::new())
}

// lookup walks parents manually by passing parent env when needed
fn lookup(env: &EnvRef, name: &str, parent: Option<&EnvRef>) -> Option<VRef> {
    env.get(name).cloned().or_else(|| {
        parent.and_then(|p| lookup(p, name, None))
    })
}

// Evaluate expression `expr` under environment `env`.
// `parent` is optional pointer to outer env when we constructed this env frame.
// In this sketch we pass parent explicitly when needed (you can wrap env+parent in a struct).
fn eval(expr: &Expr, env: &EnvRef, parent: Option<&EnvRef>) -> Result<VRef, String> {
    match expr {
        Expr::Var(name) => {
            lookup(env, name, parent).ok_or(format!("not found {}", name))
        }
        Expr::Int(n) => Ok(v(Value::Int(*n))),
        Expr::Fn(params, body) => {
            // Create a closure that captures the *env+parent* as the lexical environment.
            // We pack the environment into an EnvRef that represents this frame; to
            // allow the closure to see bindings in parent, we could wrap env+parent into one struct.
            // For simplicity here we create a single EnvRef representing the *current lexical environment*
            // by merging env with parent (or you can create wrapper struct).
            // **Important:** we DO NOT evaluate the body now.
            let captured_env = Rc::new(env.clone()); // shallow copy: env is persistent
            Ok(v(Value::Closure(Closure {
                params: params.clone(),
                body: (*body).clone(),
                env: captured_env,
            })))
        }
        Expr::Call(func_expr, arg_exprs) => {
            let fun = eval(func_expr, env, parent)?;
            // eager eval of args (strict language)
            let mut arg_vals = Vec::with_capacity(arg_exprs.len());
            for a in arg_exprs {
                arg_vals.push(eval(a, env, parent)?);
            }
            match fun.as_ref() {
                Value::Closure(cl) => {
                    if cl.params.len() != arg_vals.len() {
                        return Err("arity mismatch".into());
                    }
                    // create a fresh frame mapping params -> args
                    let mut frame = HashMap::new();
                    for (p, argv) in cl.params.iter().zip(arg_vals.into_iter()) {
                        frame.insert(p.clone(), argv);
                    }
                    let call_env = Rc::new(frame); // frame has no parent in this simple sketch
                    // evaluate body in call_env where lookup should check call_env then cl.env (the captured env)
                    eval(&cl.body, &call_env, Some(&cl.env))
                }
                _ => Err("not a function".into()),
            }
        }
        Expr::Let(name, val_expr, body) => {
            // Recursion-friendly: if RHS is a function literal, construct closure that captures
            // a frame that will contain the binding itself.
            let mut frame = HashMap::new();
            let temp_env = Rc::new(frame.clone()); // frame currently empty
            let val = match val_expr.as_ref() {
                Expr::Fn(params, body_expr) => {
                    // closure captures temp_env (which we will insert into below)
                    v(Value::Closure(Closure {
                        params: params.clone(),
                        body: (*body_expr).clone(),
                        env: temp_env.clone(),
                    }))
                }
                _ => {
                    // evaluate non-function RHS in temp_env so that RHS can reference the name if needed
                    eval(val_expr, &temp_env, Some(env))?
                }
            };

            // insert binding into frame (frame was used to build temp_env)
            let mut final_frame = temp_env.as_ref().clone();
            final_frame.insert(name.clone(), val);
            let final_env = Rc::new(final_frame);

            // evaluate body in final_env; allow lookups to fall back to outer env
            eval(body, &final_env, Some(env))
        }
    }
}


fn main(){
    
}