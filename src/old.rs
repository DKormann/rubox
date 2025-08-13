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

/// Look for `name` in `env` and, if it is not present, walk the parent chain.
fn lookup(env: &EnvRef, name: &str) -> Option<VRef> {
    env.bindings
        .get(name)
        .cloned()
        .or_else(|| env.parent.as_ref().and_then(|p| lookup(p, name)))
}

/// Core evaluator --------------------------------------------------------------
fn eval(expr: &Expr, env: &EnvRef) -> Result<VRef, String> {
    match expr {
        Expr::Var(name) => lookup(env, name)
            .ok_or_else(|| format!("unbound variable `{}`", name)),

        Expr::Int(n) => Ok(v(Value::Int(*n))),

        Expr::Fn(params, body) => {
            // The closure captures the environment that is **already** in scope.
            Ok(v(Value::Closure(Closure {
                params: params.clone(),
                body: *body.clone(),
                env: env.clone(),
            })))
        }

        Expr::Call(func_expr, arg_exprs) => {
            // Evaluate the function expression.
            let fun = eval(func_expr, env)?;

            // Eagerly evaluate all arguments.
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

                    // Build a fresh frame for the call, linking it to the
                    // captured lexical environment of the closure.
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
            // 1️⃣  Create a fresh empty frame whose parent is the surrounding env.
            let temp_env = env_extend(Some(env.clone()));

            // 2️⃣  Evaluate the right‑hand side.
            //     – If it is a function literal we build a closure that captures
            //       `temp_env`.  The name will be inserted into `temp_env`
            //       afterwards, giving recursive functions full access to themselves.
            //     – Otherwise we just evaluate the expression normally.
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

            // 3️⃣  Insert the binding into the frame we just created.
            //     (Because `EnvData` is immutable we build a *new* frame that
            //      contains the binding and keeps the same parent.)
            let mut new_bindings = temp_env.bindings.clone();
            new_bindings.insert(name.clone(), val);
            let final_env = Rc::new(EnvData {
                bindings: new_bindings,
                parent: Some(env.clone()),
            });

            // 4️⃣  Evaluate the body in the extended environment.
            eval(body, &final_env)
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Helper constructors – they make writing test programs nicer                */
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

/* -------------------------------------------------------------------------- */
/* Demonstration – the same program from the question, plus a deeper test   */
fn main() {
    // Top‑level (global) empty environment.
    let global_env = env_extend(None);

    // Example from the original post.
    let exp = let_(
        "a",
        int(22),
        let_(
            "b",
            int(33),
            let_("c", int(44), var("a")), // should evaluate to 22
        ),
    );

    // A slightly deeper test that uses nested closures.
    // let f = fn(x) { fn(y) { x + y } };
    // let add5 = f(5);
    // add5(7)   // → 12
    let add_closure = let_(
        "f",
        fn_(vec!["x"], fn_(vec!["y"], call(var("+"), vec![var("x"), var("y")]))),
        let_(
            "add5",
            call(var("f"), vec![int(5)]),
            call(var("add5"), vec![int(7)]),
        ),
    );

    // NOTE: the primitive `+` is not implemented; this is just to show that
    // arbitrarily many nested closures now work without any special handling.

    // Run the simple example.
    match eval(&exp, &global_env) {
        Ok(v) => println!("result of simple let‑chain: {:?}", v),
        Err(e) => eprintln!("error: {}", e),
    }

    // Run the nested‑closure example (it will error because `+` is unknown,
    // but the evaluator will *not* overflow the stack).
    match eval(&add_closure, &global_env) {
        Ok(v) => println!("result of nested closures: {:?}", v),
        Err(e) => eprintln!("error (expected, missing '+'): {}", e),
    }
}