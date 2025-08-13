


#[cfg(test)]
mod tests {
  use std::array;
  use crate::ast::*;
  use crate::runtime::*;

  #[test]
  fn test_let(){
    let code = "let x = 2; x";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = mk_let("x", mk_int(2), mk_var("x"));
    assert_eq!(ast, expect_ast);
    let res = crate::runtime::eval(&ast).expect("eval failed");
    let expect_res = crate::runtime::eval(&mk_int(2)).expect("eval failed");
    assert_eq!(res, expect_res);

  }

  #[test]
  fn test_let_chain(){
    let code = "let x = 2; let y = x; y";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = mk_let("x", mk_int(2), mk_let("y", mk_var("x"), mk_var("y")));
    assert_eq!(ast, expect_ast);
    let res = crate::runtime::eval(&ast).expect("eval failed");
    let expect_res = crate::runtime::eval(&mk_int(2)).expect("eval failed");
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
    let expect_ast = mk_fn(vec!["x".into()], mk_var("x"));
    assert_eq!(ast, expect_ast);
  }

  #[test]
  fn test_parse_call(){
    let code = "fn(22)";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = mk_call(mk_var("fn"), vec![mk_int(22)]);
    assert_eq!(ast, expect_ast);
  }

  #[test]
  fn test_parse_array(){
    let code = "[1,2,3]";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = mk_array(vec![mk_int(1), mk_int(2), mk_int(3)]);
    assert_eq!(ast, expect_ast);
  }

  #[test]
  fn test_parse_object(){
    let code = "{a: 1, b: 2}";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = object(vec![("a", mk_int(1)), ("b", mk_int(2))]);
    assert_eq!(ast, expect_ast);
  }


  #[test]
  fn test_parse_index(){
    let code = "([1])[0]";
    let ast = crate::parse(code).expect("parse failed");
    let expect_ast = mk_index(mk_array(vec![mk_int(1)]), mk_int(0));
    assert_eq!(ast, expect_ast);
    
  }


  #[test]
  fn fn_call_eval(){
    let code = "((x)=>x)(22)";
    test_code_equiv(code, "22");

    test_code_equiv("((x,y)=>y)(1,2)", "2");
    test_code_equiv("((x,y)=>y)(1,2)", "1");
  }

  #[test]
  fn arr_index_eval(){
    test_code_equiv("([1,2,3])[2]", "3");
  }


  #[test]
  fn obj_get_eval(){
    test_code_equiv("({a:22}).a", "22");
  }



}



