use super::parser;
use super::semantics;

use alloc::collections::linked_list::LinkedList;
use alloc::vec::Vec;

enum RTData<'t> {
    Int(u64),
    Bool(bool),
    Label(&'t str, Vec<RTData<'t>>)
}

struct RootObject<'t> {
    objects: LinkedList<RTData<'t>>
}

impl<'t> RootObject<'t> {
    pub fn new() -> RootObject<'t> {
        RootObject{objects: LinkedList::new()}
    }
}

pub struct Evaluator<'t> {
    root: RootObject<'t>,
}

impl<'t> Evaluator<'t> {
    pub fn new() -> Evaluator<'t> {
        Evaluator{root: RootObject::new()}
    }

    pub fn eval(&mut self, code: &str, ctx: &semantics::Context<'t>) {
        self.root = RootObject::new();

        let mut ps = parser::Parser::new(code);
        let exprs;
        match ps.parse() {
            Ok(e) => {
                exprs = e;
            }
            Err(_e) => {
                // TODO: return error message
                return;
            }
        }

        let mut typed_exprs = Vec::new();
        for expr in &exprs {
            match semantics::typing_expr(expr, ctx) {
                Ok(e) => {
                    typed_exprs.push(e);
                }
                Err(_e) => {
                    // TODO: return error message
                    return;
                }
            }
        }
    }
}