#[allow(dead_code)]
#[allow(unreachable_code)]

use im::HashMap;
use std::rc::Rc;



use crate::ast::*;

fn v(val: Value) -> VRef {
Rc::new(val)
}

fn stringify_value(v: &Value) -> String { format_value(v) }

fn format_value(v: &Value) -> String {
  match v {
    Value::Int(n) => n.to_string(),
    Value::Float(f) => {
      let mut s = f.to_string();
      if s.contains('.') { s } else { format!("{}.0", s) }
    }
    Value::String(s) => s.clone(),
    Value::Boolean(b) => b.to_string(),
    Value::Null => "null".into(),
    Value::Undefined => "undefined".into(),
    Value::Array(_) => "[Array]".into(),
    Value::Object(_) => "[Object]".into(),
    Value::Closure(_) => "[Function]".into(),
  }
}

fn cmp_numbers(a: f64, b: f64, op: &str) -> Result<VRef, String> {
  let res = match op { "==" => a == b, "!=" => a != b, ">" => a > b, "<" => a < b, ">=" => a >= b, "<=" => a <= b, _ => unreachable!() };
  Ok(v(Value::Boolean(res)))
}

fn cmp_strings(a: &String, b: &String, op: &str) -> Result<VRef, String> {
  let res = match op { "==" => a == b, "!=" => a != b, ">" => a > b, "<" => a < b, ">=" => a >= b, "<=" => a <= b, _ => unreachable!() };
  Ok(v(Value::Boolean(res)))
}

fn cmp_bools(a: bool, b: bool, op: &str) -> Result<VRef, String> {
  let res = match op { "==" => a == b, "!=" => a != b, ">" => a && !b, "<" => !a && b, ">=" => a || (!a && !b), "<=" => !a || (a && b), _ => unreachable!() };
  Ok(v(Value::Boolean(res)))
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
      Expr::Index(arr, idx)=> {
        let arr_val = do_eval(arr,env)?;
        let idx_val = do_eval(idx,env)?;
        match arr_val.as_ref() {
          Value::Array(items) => {
            use std::convert::TryFrom;
            let i: i32 = i32::try_from(idx_val.as_ref()).map_err(|_| "index must be an Int")?;
            if i < 0 || (i as usize) >= items.len() {
              return Err("index out of bounds".into());
            }
            Ok(items[i as usize].clone())
          }
          _=>return Err("attempted to index a non-array value".into())
        }
      },
      Expr::Access(primary, prop)=> {
        let primary_val = do_eval(primary,env)?;
        match primary_val.as_ref() {
          Value::Object(obj) => {
            obj.get(prop)
            .cloned()
            .ok_or_else(|| format!("property {} not found on object", prop))
          }
          _=>return Err("attempted to access a non-object value".into())
        }
      },
      Expr::Conditional(c,t,e) => {
        let v = do_eval(c, env)?;
        let truthy = match v.as_ref() {
          Value::Boolean(b) => *b,
          Value::Null | Value::Undefined => false,
          Value::Int(n) => *n != 0,
          Value::Float(f) => *f != 0.0,
          Value::String(s) => !s.is_empty(),
          Value::Array(a) => !a.is_empty(),
          Value::Object(o) => !o.is_empty(),
          Value::Closure(_) => true,
        };
        if truthy { do_eval(t, env) } else { do_eval(e, env) }
      },
      Expr::Binop(left, op, right) => {
        use std::convert::TryFrom;
        // Evaluate operands
        let lv = do_eval(left, env)?;
        let rv = do_eval(right, env)?;

        match op.as_str() {
          "+" => {
            match (lv.as_ref(), rv.as_ref()) {
              (Value::Int(a), Value::Int(b)) => Ok(v(Value::Int(a + b))),
              (Value::Float(a), Value::Float(b)) => Ok(v(Value::Float(a + b))),
              (Value::String(a), Value::String(b)) => Ok(v(Value::String(format!("{}{}", a, b)))),
              (Value::String(a), other) => Ok(v(Value::String(format!("{}{}", a, stringify_value(other))))),
              (other, Value::String(b)) => Ok(v(Value::String(format!("{}{}", stringify_value(other), b)))),
              (Value::Int(a), Value::Float(b)) => Ok(v(Value::Float(*a as f64 + b))),
              (Value::Float(a), Value::Int(b)) => Ok(v(Value::Float(a + *b as f64))),
              _ => Err("unsupported + operands".into()),
            }
          }
          "-" | "*" | "/" => {
            // Coerce numeric pairs (int/float)
            let as_float = match (lv.as_ref(), rv.as_ref()) {
              (Value::Int(a), Value::Int(b)) => (Some(*a as f64), Some(*b as f64)),
              (Value::Int(a), Value::Float(b)) => (Some(*a as f64), Some(*b)),
              (Value::Float(a), Value::Int(b)) => (Some(*a), Some(*b as f64)),
              (Value::Float(a), Value::Float(b)) => (Some(*a), Some(*b)),
              _ => (None, None),
            };
            if let (Some(a), Some(b)) = as_float {
              // If both were Ints, prefer Int results when exact
              if let (Value::Int(ai), Value::Int(bi)) = (lv.as_ref(), rv.as_ref()) {
                match op.as_str() {
                  "-" => return Ok(v(Value::Int(ai - bi))),
                  "*" => return Ok(v(Value::Int(ai * bi))),
                  "/" => {
                    if *bi == 0 { return Err("division by zero".into()); }
                    if ai % bi == 0 { return Ok(v(Value::Int(ai / bi))); }
                    return Ok(v(Value::Float(a / b)));
                  }
                  _ => unreachable!(),
                }
              }
              // Mixed numeric types -> Float
              let res = match op.as_str() { "-" => a - b, "*" => a * b, "/" => a / b, _ => unreachable!() };
              Ok(v(Value::Float(res)))
            } else {
              Err("numeric operator on non-numeric values".into())
            }
          }
          "==" | "!=" | ">" | "<" | ">=" | "<=" => {
            let res = match (lv.as_ref(), rv.as_ref(), op.as_str()) {
              // numeric comparisons (coerce to float)
              (Value::Int(a), Value::Int(b), op) => cmp_numbers(*a as f64, *b as f64, op),
              (Value::Int(a), Value::Float(b), op) => cmp_numbers(*a as f64, *b, op),
              (Value::Float(a), Value::Int(b), op) => cmp_numbers(*a, *b as f64, op),
              (Value::Float(a), Value::Float(b), op) => cmp_numbers(*a, *b, op),
              // string comparisons
              (Value::String(a), Value::String(b), op) => cmp_strings(a, b, op),
              // boolean comparisons
              (Value::Boolean(a), Value::Boolean(b), op) => cmp_bools(*a, *b, op),
              // equality/inequality for other types: pointer/variant equality
              (la, rb, "==") => Ok(v(Value::Boolean(std::mem::discriminant(la) == std::mem::discriminant(rb) && format_value(la) == format_value(rb)))),
              (la, rb, "!=") => Ok(v(Value::Boolean(!(std::mem::discriminant(la) == std::mem::discriminant(rb) && format_value(la) == format_value(rb))))),
              _ => Err("unsupported comparison operands".into()),
            }?;
            Ok(res)
          }
          _ => Err("unknown operator".into()),
        }
      },
  }
}



pub fn object(elems: Vec<(&'static str, Expr)>) -> Expr {
  Expr::Object(elems.into_iter().map(|(k,v)| ObjElem::Expr((k.into(),v))).collect())
}
