

use im::HashMap;
use std::{rc::Rc};
use pest::Parser as PestParser;
use pest_derive::Parser;

/// The language syntax ---------------------------------------------------------
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Value(Box<Value>),
    Var(String),
    Fn(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Array(Vec<ArrElem>),
    Object(Vec<ObjElem>)
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
    let mut pairs = FunscriptParser::parse(Rule::expr, input)?;
    let pair = pairs.next().unwrap();
    build_expr(pair)
}

fn build_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, pest::error::Error<Rule>> {
    match pair.as_rule() {
        Rule::expr => build_expr(pair.into_inner().next().unwrap()),
        Rule::let_ => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let init = build_expr(inner.next().unwrap())?;
            let body = build_expr(inner.next().unwrap())?;
            Ok(Expr::Let(name, Box::new(init), Box::new(body)))
        }
        Rule::fun => {
            let mut inner = pair.into_inner();
            let params_pair = inner.next().unwrap();
            let params = build_params(params_pair);
            let body = build_expr(inner.next().unwrap())?;
            Ok(Expr::Fn(params, Box::new(body)))
        }
        Rule::call => {
            let mut inner: pest::iterators::Pairs<'_, Rule> = pair.into_inner();
            let mut current = build_expr(inner.next().unwrap())?; // primary
            for arglist in inner { // one or more arglists
                let args = build_arglist(arglist)?;
                current = Expr::Call(Box::new(current), args);
            }
            Ok(current)
        }
        Rule::primary => build_expr(pair.into_inner().next().unwrap()),
        Rule::ident => Ok(Expr::Var(pair.as_str().to_string())),
        Rule::literal => build_literal(pair),
        Rule::int | Rule::float | Rule::string | Rule::boolean | Rule::null | Rule::undefined => build_literal(pair),
        Rule::array => build_array(pair),
        Rule::object => build_object(pair),
        _ => unreachable!("unhandled rule: {:?}", pair.as_rule()),
    }
}

fn build_params(pair: pest::iterators::Pair<Rule>) -> Vec<String> {
    let mut params: Vec<String> = Vec::new();
    let inner = pair.into_inner();
    for p in inner { // zero or more idents separated by commas
        if p.as_rule() == Rule::ident {
            params.push(p.as_str().to_string());
        }
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
        if p.as_rule() == Rule::expr {
            elems.push(ArrElem::Expr(build_expr(p)?));
        }
    }
    Ok(Expr::Array(elems))
}

fn build_object(pair: pest::iterators::Pair<Rule>) -> Result<Expr, pest::error::Error<Rule>> {
    let mut props: Vec<ObjElem> = Vec::new();
    for p in pair.into_inner() { // prop items
        if p.as_rule() == Rule::prop {
            let mut inner = p.into_inner();
            let key = inner.next().unwrap().as_str().to_string();
            let val = build_expr(inner.next().unwrap())?;
            props.push(ObjElem::Expr((key, val)));
        }
    }
    Ok(Expr::Object(props))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integer_literal() {
        let src = "1";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_variable() {
        let src = "x";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_simple_function_literal() {
        let src = "(x) => x";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_function_call() {
        let src = "f(1)";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_let_then_body_expression() {
        // matches E := let x = E; E
        let src = "let x = 1; x";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_array_constructor() {
        let src = "[1, 2, 3]";
        assert!(parse(src).is_ok());
    }

    #[test]
    fn parses_object_constructor() {
        let src = "{a: 1, b: 2}";
        assert!(parse(src).is_ok());
    }

    // // --- Invalid programs per grammar.md / language rules ---
    // #[test]
    // fn rejects_reassignment() {
    //     let src = "let x = 1; x = 2";
    //     assert!(parse(src).is_err());
    // }
}
