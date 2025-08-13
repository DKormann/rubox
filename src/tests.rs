


#[cfg(test)]
mod tests {
use std::array;
use crate::ast::*;
use crate::parser::parse;
use crate::runtime::*;

  fn test_parse(code:&str, expect_ast:Expr){
    let ast = crate::parse(code).expect("parse failed");
    assert_eq!(ast, expect_ast);
  }

  #[test]
  fn test_let_parse(){
    let code = "let x = 2; x";
    test_parse(code, mk_let("x".into(), mk_int(2), mk_var("x")));
  }

  #[test]
  fn test_let_eval(){
    let code = "let x = 2; x";
    test_code_equiv(code, "2");
  }



  #[test]
  fn test_let_chain(){
    test_code_equiv("let x = 2; let y = x; y", "2");
  }
  
  fn test_code_equiv(a:&str,b:&str){
    let ast_a = crate::parse(a).expect("parse failed");
    let ast_b = crate::parse(b).expect("parse failed");
    let res_a = crate::eval(&ast_a).expect("eval failed");
    let res_b = crate::eval(&ast_b).expect("eval failed");
    assert_eq!(res_a, res_b);
  }

  #[test]
  fn test_let_chain_eval(){
    test_code_equiv("let x = 2; let y = x; y", "2");
  }

  #[test]
  fn test_brace_equiv(){
    test_code_equiv("(22)", "22");
  }

  #[test]
  fn test_fn_def(){
    let code = "((x)=>x)";
    test_parse(code, mk_fn(vec!["x".into()], mk_var("x")));
  }

  #[test]
  fn test_parse_call(){
    let code = "fn(22)";
    test_parse(code, mk_call(mk_var("fn"), vec![mk_int(22)]));
  }

  #[test]
  fn test_parse_binops(){
    test_parse("2+2", mk_binop(mk_int(2), "+".into(), mk_int(2)));
    test_parse("2-2", mk_binop(mk_int(2), "-".into(), mk_int(2)));
    test_parse("2*2", mk_binop(mk_int(2), "*".into(), mk_int(2)));
    test_parse("2/2", mk_binop(mk_int(2), "/".into(), mk_int(2)));
    test_parse("2==2", mk_binop(mk_int(2), "==".into(), mk_int(2)));
    test_parse("2!=2", mk_binop(mk_int(2), "!=".into(), mk_int(2)));
    test_parse("2>2", mk_binop(mk_int(2), ">".into(), mk_int(2)));
    test_parse("2<2", mk_binop(mk_int(2), "<".into(), mk_int(2)));
    test_parse("2>=2", mk_binop(mk_int(2), ">=".into(), mk_int(2)));
    test_parse("2<=2", mk_binop(mk_int(2), "<=".into(), mk_int(2)));
    test_parse("(a)=>a+1", mk_fn(vec!["a".into()], mk_binop(mk_var("a"), "+".into(), mk_int(1))));
  }

  #[test]
  fn test_parse_binops_eval(){


    test_code_equiv("2+2", "4");
    test_code_equiv("2-2", "0");
    test_code_equiv("2*2", "4");
    test_code_equiv("2/2", "1");
    test_code_equiv("2==2", "true");
    test_code_equiv("2!=2", "false");
    test_code_equiv("2>2", "false");
    test_code_equiv("2<2", "false");
    test_code_equiv("2>=2", "true");
    test_code_equiv("2<=2", "true");
    test_code_equiv("(a)=>a+1", "(a)=>a+1");
  }

  #[test]
  fn test_parse_array(){
    let code = "[1,2,3]";
    test_parse(code, mk_array(vec![mk_int(1), mk_int(2), mk_int(3)]));
  }

  #[test]
  fn test_parse_array_eval(){
    let code = "[1,2,3]";
    test_code_equiv(code, "[1,2,3]");
  }

  #[test]
  fn test_parse_object(){
    let code = "{a: 1, b: 2}";
    test_parse(code, object(vec![("a", mk_int(1)), ("b", mk_int(2))]))
  }


  #[test]
  fn test_parse_index(){
    let code = "a[0]";
    test_parse(code, mk_index(mk_var("a"), mk_int(0)));
  }

  #[test]
  fn test_parse_access(){
    let code = "a.b";
    test_parse(code, mk_access(mk_var("a"), "b".into()));

    let code = "a.b(22)";
    test_parse(code, mk_call(mk_access(mk_var("a".into()), "b".into()), vec![mk_int(22)]));
  }

  #[test]
  fn test_parse_access_chain(){
    let code = "a.b.c";
    test_parse(code, mk_access(mk_access(mk_var("a".into()), "b".into()), "c".into()));

    let code = "a.b.c(22)[3].d";
    test_parse(code, mk_access(mk_index(mk_call(mk_access(mk_access(mk_var("a".into()), "b".into()), "c".into()), vec![mk_int(22)]), mk_int(3)), "d".into()));

    let code = "(((a.b).c)(22))[3].d";
    test_parse(code, mk_access(mk_index(mk_call(mk_access(mk_access(mk_var("a".into()), "b".into()), "c".into()), vec![mk_int(22)]), mk_int(3)), "d".into()));
  }



  #[test]
  fn fn_call_eval(){
    test_code_equiv("((x)=>x)(22)", "22");
    test_code_equiv("((x,y)=>y)(1,2)", "2");
    test_code_equiv("((x,y)=>x)(1,2)", "1");
  }

  #[test]
  fn arr_index_eval(){
    test_code_equiv("([1,2,3])[2]", "3");
  }


  #[test]
  fn obj_get_eval(){
    test_code_equiv("({a:22}).a", "22");
  }

  #[test]
  fn full_eval_complex(){
    test_code_equiv("let o = {a:22}; o.a", "22");
    test_code_equiv("let o = {a:22}; let x = o.a; x", "22");
    test_code_equiv("let o = {f:(x)=>x}; (o.f)(22)", "22");
  }


  // #[test]
  // fn full_eval_fib(){
  //   let code = "
  //   let fib  = (n)=>(n+1);
  //   fib
  //   ";

  //   test_code_equiv(code, code);
  // }


}



