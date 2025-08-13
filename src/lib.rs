
#![allow(unused)]

mod parser;
mod runtime;

use crate::parser::parse;
use crate::runtime::eval;

#[cfg(test)]
mod tests {
    use crate::runtime::{call, fn_, int, let_, var};

  #[test]
  fn test_let(){
    let code = "let x = 2; x";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = let_("x", int(2), var("x"));
    assert_eq!(ast, expect_ast);
    let res = crate::runtime::eval(&ast).expect("eval failed");
    let expect_res = crate::runtime::eval(&int(2)).expect("eval failed");
    assert_eq!(res, expect_res);

  }

  #[test]
  fn test_let_chain(){
    let code = "let x = 2; let y = x; y";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = let_("x", int(2), let_("y", var("x"), var("y")));
    assert_eq!(ast, expect_ast);
    let res = crate::runtime::eval(&ast).expect("eval failed");
    let expect_res = crate::runtime::eval(&int(2)).expect("eval failed");
    assert_eq!(res, expect_res);
  }
  
  fn test_code_equiv(a:&str,b:&str){
    let ast_a = crate::parse(a).expect("parse failed");
    let ast_b = crate::parse(b).expect("parse failed");
    let res_a = crate::eval(&ast_a).expect("eval failed");
    let res_b = crate::eval(&ast_b).expect("eval failed");
    assert_eq!(res_a, res_b);
  }

  #[test]
  fn test_let_chain_equiv(){
    test_code_equiv("let x = 2; let y = x; y", "2");
  }

  #[test]
  fn test_brace_equiv(){
    test_code_equiv("(22)", "22");
  }

  #[test]
  fn test_fn_def(){

    let code = "((x)=>x)";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = fn_(vec!["x"], var("x"));
    assert_eq!(ast, expect_ast);
  }

  #[test]
  fn test_var_call(){
    let code = "(fn)(22)";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = call(var("fn"), vec![int(22)]);
    assert_eq!(ast, expect_ast);
  }
    


}



