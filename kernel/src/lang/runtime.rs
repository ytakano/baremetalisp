use super::parser;
use super::semantics;

//use crate::driver;

use alloc::collections::linked_list::LinkedList;
use alloc::vec::Vec;
use alloc::string::{ToString, String};

type Expr<'a> = semantics::LangExpr<'a>;

#[derive(Debug)]
pub enum RTData<'t> {
    Int(i64),
    Bool(bool),
    LData(&'t LabeledData<'t>),
    RuntimeErr(String)
}

#[derive(Debug)]
pub struct LabeledData<'t> {
    label: String,
    data: Vec<RTData<'t>>
}

pub struct RootObject<'t> {
    objects: LinkedList<LabeledData<'t>>
}

impl<'t> RootObject<'t> {
    pub fn new() -> RootObject<'t> {
        RootObject{objects: LinkedList::new()}
    }
}

#[derive(Debug)]
pub struct EvalErr {
    pub msg: String
}

pub fn eval<'t>(code: &str, ctx: &semantics::Context, root: RootObject<'t>) -> Result<LinkedList<RTData<'t>>, EvalErr> {
    let mut ps = parser::Parser::new(code);
    let exprs;
    match ps.parse() {
        Ok(e) => {
            exprs = e;
        }
        Err(e) => {
            let msg = format!("{:?}:{:?}: {}", e.pos.line, e.pos.column, e.msg);
            return Err(EvalErr{msg: msg});
        }
    }

    let mut typed_exprs = LinkedList::new();
    for expr in &exprs {
        match semantics::typing_expr(expr, ctx) {
            Ok(e) => {
                typed_exprs.push_back(e);
            }
            Err(e) => {
                let msg = format!("{:?}:{:?}: {}", e.pos.line, e.pos.column, e.msg);
                return Err(EvalErr{msg: msg});
            }
        }
    }

    let mut result = LinkedList::new();
    for expr in &typed_exprs {
        result.push_back(eval_expr(expr));
    }

    Ok(result)
}

fn eval_expr<'t>(expr: &Expr) -> RTData<'t> {
    match expr {
        Expr::LitNum(e) => RTData::Int(e.num),
        Expr::LitBool(e) => RTData::Bool(e.val),
        Expr::IfExpr(e)  => eval_if(&e),
        _ => RTData::RuntimeErr("not yet implemented".to_string())
    }
}

fn eval_if<'t>(expr: &semantics::IfNode) -> RTData<'t> {
    let cond = eval_expr(&expr.cond_expr);
    let flag;
    match cond {
        RTData::Bool(e) => {
            flag = e;
        }
        _ => {
            let pos = expr.cond_expr.get_ast().get_pos();
            let msg = format!("{:?}:{:?}: type mismatched", pos.line, pos.column);
            return RTData::RuntimeErr(msg);
        }
    }

    if flag {
        eval_expr(&expr.then_expr)
    } else {
        eval_expr(&expr.else_expr)
    }
}

fn eval_let<'t>(expr: &semantics::LetNode) -> RTData<'t> {
    for def in &expr.def_vars {

    }

    RTData::RuntimeErr("not yet implemented".to_string())
}
