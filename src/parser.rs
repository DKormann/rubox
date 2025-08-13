

use im::HashMap;
use std::{rc::Rc};
use pest::Parser as PestParser;
use pest_derive::Parser;


use crate::ast::*;

/// Create a new (empty) frame whose parent is `parent`.
fn env_extend(parent: Option<EnvRef>) -> EnvRef {
    Rc::new(EnvData {
        bindings: HashMap::new(),
        parent,
    })
}


#[derive(Parser)]
#[grammar = "funscript.pest"]
struct FunscriptParser;

pub fn parse(input: &str) -> Result<Expr, pest::error::Error<Rule>> {
  let mut pairs = FunscriptParser::parse(Rule::program, input)?;
  let pair = pairs.next().unwrap();

  build_expr(pair.into_inner().next().unwrap())
}

fn build_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, pest::error::Error<Rule>> {
  match pair.as_rule() {
    Rule::expr => build_expr(pair.into_inner().next().unwrap()),
    Rule::let_ => {
      let mut inner = pair.into_inner();
      let name = inner.next().unwrap().as_str().to_string();
      let init = build_expr(inner.next().unwrap())?;
      let body = build_expr(inner.next().unwrap())?;

      Ok(mk_let(name, init, body))
    }
    Rule::fun => {
      let mut inner = pair.into_inner();
      let params_pair = inner.next().unwrap();
      let params = build_params(params_pair);
      let body = build_expr(inner.next().unwrap())?;
      Ok(mk_fn(params,body))
    }
    // Rule::call => {
    //   let mut inner: pest::iterators::Pairs<'_, Rule> = pair.into_inner();
    //   let mut current = build_expr(inner.next().unwrap())?; // primary
    //   for arglist in inner { // one or more arglists
    //     let args = build_arglist(arglist)?;
    //     current = mk_call(current, args);
    //   }
    //   Ok(current)
    // }
    Rule::primary => build_expr(pair.into_inner().next().unwrap()),
    Rule::ident => Ok(Expr::Var(pair.as_str().to_string())),
    Rule::literal => build_literal(pair),
    Rule::int | Rule::float | Rule::string | Rule::boolean | Rule::null | Rule::undefined => build_literal(pair),
    Rule::array => build_array(pair),
    Rule::object => build_object(pair),
    Rule::binop => {
      let mut inner = pair.into_inner();
      let left = build_expr(inner.next().unwrap())?;
      let op = inner.next().unwrap().as_str().to_string();
      let right = build_expr(inner.next().unwrap())?;
      Ok(mk_binop(left, op, right))
    },

    Rule::index => {
      let mut inner = pair.into_inner();
      let primary = build_expr(inner.next().unwrap())?;
      let index = build_expr(inner.next().unwrap())?;
      Ok(mk_index(primary,index))
    },

    Rule::access_chain =>{

      let mut inner = pair.into_inner();
      let primary = build_expr(inner.next().unwrap())?;

      let mut current = primary;
      for p in inner {
        match p.as_rule() {
          Rule::field => {
            let prop = p.as_str().to_string();
            current = mk_access(current, prop.split_once('.').unwrap().1.to_string());
          }
          Rule::index => {
            let index = build_expr(p.into_inner().next().unwrap())?;
            current = mk_index(current, index);
          }
          Rule::arglist => {
            let args = build_arglist(p)?;
            current = mk_call(current, args);
          }
          _ => unreachable!("unhandled rule: {:?}", p.as_rule()),
        }
      };
      Ok(current)
    },

    _ => unreachable!("unhandled rule: {:?}", pair.as_rule()),
  }
}



fn build_params(pair: pest::iterators::Pair<Rule>) -> Vec<String> {
  let mut params: Vec<String> = Vec::new();
  let inner = pair.into_inner();
  for p in inner { // zero or more idents separated by commas
    params.push(p.as_str().to_string());
  }
  params
}

fn build_arglist(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Expr>, pest::error::Error<Rule>> {
  let mut args: Vec<Expr> = Vec::new();
  for p in pair.into_inner() {
    let built = build_expr(p.clone())?;
    args.push(built);
  }
  Ok(args)
}

fn build_literal(pair: pest::iterators::Pair<Rule>) -> Result<Expr, pest::error::Error<Rule>> {
  let rule = pair.as_rule();
  let text = pair.as_str();
  let value = match rule {
    Rule::float => Value::Float(text.parse::<f64>().unwrap()),
    Rule::int => Value::Int(text.parse::<i32>().unwrap()),
    Rule::string => {
      // strip surrounding quotes and unescape basic escapes
      let raw = &text[1..text.len()-1];
      Value::String(unescape_basic_string(raw))
    }
    Rule::boolean => Value::Boolean(text == "true"),
    Rule::null => Value::Null,
    Rule::undefined => Value::Undefined,
    Rule::literal => {
      // delegate to the single inner literal
      return build_literal(pair.into_inner().next().unwrap());
    }
    _ => unreachable!(),
    };
    Ok(Expr::Value(Box::new(value)))
}

fn unescape_basic_string(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  let mut chars = input.chars();
  while let Some(c) = chars.next() {
    if c == '\\' {
      if let Some(n) = chars.next() {
        match n {
          'n' => out.push('\n'),
          'r' => out.push('\r'),
          't' => out.push('\t'),
          '"' => out.push('"'),
          '\\' => out.push('\\'),
          other => {
            out.push('\\');
            out.push(other);
          }
        }
      } else {
        out.push('\\');
      }
    } else {
      out.push(c);
    }
  }
  out
}

fn build_array(pair: pest::iterators::Pair<Rule>) -> Result<Expr, pest::error::Error<Rule>> {
  let mut elems: Vec<ArrElem> = Vec::new();
  for p in pair.into_inner() { // expr items
    elems.push(ArrElem::Expr(build_expr(p)?));
  }
  Ok(Expr::Array(elems))
}

fn build_object(pair: pest::iterators::Pair<Rule>) -> Result<Expr, pest::error::Error<Rule>> {
  let mut props: Vec<ObjElem> = Vec::new();
  for p in pair.into_inner() { // prop items
    let mut inner = p.into_inner();
    let key = inner.next().unwrap().as_str().to_string();
    let val = build_expr(inner.next().unwrap())?;
    props.push(ObjElem::Expr((key, val)));
  }
  Ok(Expr::Object(props))
}
