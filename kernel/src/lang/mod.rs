pub mod parser;
pub mod semantics;
pub mod runtime;

use alloc::collections::linked_list::LinkedList;
use alloc::string::String;

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct LispErr {
    pub msg: String,
    pub pos: Pos,
}

impl LispErr {
    fn new(msg: String, pos: Pos) -> LispErr {
        LispErr{msg: msg, pos: pos}
    }
}

pub fn init(code: &str) -> Result<LinkedList<parser::Expr>, LispErr> {
    let mut ps = parser::Parser::new(code);
    match ps.parse() {
        Ok(e) => {
            Ok(e)
        }
        Err(e) => {
            let msg = format!("Syntax Error: {}", e.msg);
            Err(LispErr::new(msg, e.pos))
        }
    }
}

pub fn typing(exprs: &LinkedList<parser::Expr>) -> Result<semantics::Context, LispErr> {
    match semantics::exprs2context(exprs) {
        Ok(c) => {
            Ok(c)
        }
        Err(e) => {
            let msg = format!("Typing Error: {}", e.msg);
            Err(LispErr::new(msg, e.pos))
        }
    }
}

pub fn eval(code: &str, ctx: &semantics::Context) -> Result<LinkedList<String>, LispErr> {
    runtime::eval(code, ctx)
}