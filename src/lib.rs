
#![allow(unused)]

mod parser;
mod runtime;
mod tests;
mod ast;

use crate::parser::parse;
use crate::runtime::eval;
