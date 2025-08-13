#[allow(dead_code)]
#[allow(unreachable_code)]

use im::HashMap;
use std::rc::Rc;



use crate::ast::*;

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

pub fn eval(expr: &Expr)->Result<VRef, String>{
  do_eval(expr,&env_extend(None))
}

fn do_eval(expr: &Expr, env: &EnvRef) -> Result<VRef, String> {
  match expr {
  Expr::Var(name) => lookup(env, name)
  .ok_or_else(|| format!("unbound variable {}", name)),


      // Expr::Int(n) => Ok(v(Value::Int(*n))),
      Expr::Value(val) => Ok(val.clone().into()),

      Expr::Fn(params, body) => {

          Ok(v(Value::Closure(Closure {
              params: params.clone(),
              body: *body.clone(),
              env: env.clone(),
          })))
      }

      Expr::Call(func_expr, arg_exprs) => {
          // Evaluate the function expression.
          let fun = do_eval(func_expr, env)?;

          let mut arg_vals = Vec::with_capacity(arg_exprs.len());
          for a in arg_exprs {
              arg_vals.push(do_eval(a, env)?);
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

                  do_eval(&cl.body, &call_env)
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
              _ => do_eval(val_expr, &temp_env)?,
          };

          let mut new_bindings = temp_env.bindings.clone();
          new_bindings.insert(name.clone(), val);
          let final_env = Rc::new(EnvData {
              bindings: new_bindings,
              parent: Some(env.clone()),
          });

          do_eval(body, &final_env)
      }
      Expr::Array(arr) =>{
        let mut narr: Vec<Rc<Value>> = Vec::new();
        for elem in arr {
          match elem {
            ArrElem::Expr(e) => {
              let val = do_eval(e,env)?;
              narr.push(val);
            },
            ArrElem::Spread(e) => {
              let val = do_eval(e,env)?;
              match val.as_ref() {
                Value::Array(arr) => {
                  for v in arr {
                    narr.push(v.clone())
                  }
                }
                _=>return Err("attempted to spread a non-array value".into())
              }
            }
          }
        }
        Ok(v(Value::Array(narr)))
      },
      Expr::Object(obj_expr) => {
          let mut obj:HashMap<String, Rc<Value>> = HashMap::new();
          for elem in obj_expr {
            match elem {
              ObjElem::Expr((key,value)) => {
                let val = do_eval(value,env)?;
                obj.insert(key.clone(), val);
              },
              ObjElem::Spread(e) => {
                let val = do_eval(e,env)?;
                match val.as_ref() {
                  Value::Object(obj_map) => {
                    for (k,v_ref) in obj_map {
                      obj.insert(k.clone(), v_ref.clone());
                    }
                  }
                  _=>return Err("attempted to spread a non-object value".into())
                }
              }
            }
          }


          Ok(v(Value::Object(obj)))
      }
      _ => todo!(),
  }
}



pub fn object(elems: Vec<(&'static str, Expr)>) -> Expr {
  Expr::Object(elems.into_iter().map(|(k,v)| ObjElem::Expr((k.into(),v))).collect())
}
