use super::parser;
use super::semantics;

use alloc::collections::linked_list::LinkedList;

enum RTData {
    Int(u64),
    Bool(bool)
}

struct RootObject {
    objects: LinkedList<RTData>
}

impl RootObject {
    pub fn new() -> RootObject {
        RootObject{objects: LinkedList::new()}
    }
}

pub struct Evaluator {
    root: RootObject,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        Evaluator{root: RootObject::new()}
    }

    pub fn eval(&mut self, code: &str, ctx: &semantics::Context) {
        self.root = RootObject::new();

        let mut ps = parser::Parser::new(code);
        let exprs;
        match ps.parse() {
            Ok(e) => {
                exprs = e;
            }
            Err(e) => {
                return;
            }
        }

        for expr in &exprs {
        }
    }
}