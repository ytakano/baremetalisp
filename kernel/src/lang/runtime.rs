use super::parser;
use super::semantics;

//use crate::driver;

use alloc::collections::linked_list::LinkedList;
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use alloc::string::{ToString, String};

type Expr<'a> = semantics::LangExpr<'a>;
type Pattern<'a> = semantics::Pattern<'a>;

struct Variables {
    vars: LinkedList<BTreeMap<String, RTData>>
}

impl Variables {
    fn new() -> Variables {
        let mut list = LinkedList::new();
        list.push_back(BTreeMap::new());
        Variables{vars: list}
    }

    fn push(&mut self) {
        self.vars.push_back(BTreeMap::new());
    }

    fn pop(&mut self) {
        self.vars.pop_back();
    }

    fn insert(&mut self, id: String, data: RTData) {
        let m = self.vars.back_mut().unwrap();
        m.insert(id, data);
    }

    fn get(&mut self, id: &String) -> Option<&RTData> {
        let m = self.vars.back_mut().unwrap();
        m.get(id)
    }
}

#[derive(Debug, Clone)]
pub enum RTData {
    Int(i64),
    Bool(bool),
    LData(*const LabeledData),
    RuntimeErr(String)
}

#[derive(Debug)]
pub struct LabeledData {
    label: String,
    data: Option<Vec<RTData>>
}

pub struct RootObject {
    objects: LinkedList<LabeledData>
}

impl RootObject {
    pub fn new() -> RootObject {
        RootObject{objects: LinkedList::new()}
    }

    fn make_obj(&mut self, label: String, data: Option<Vec<RTData>>) -> *const LabeledData {
        let obj = LabeledData{label: label, data: data};
        self.objects.push_back(obj);
        self.objects.back().unwrap() as *const LabeledData
    }
}

#[derive(Debug)]
pub struct EvalErr {
    pub msg: String
}

pub fn eval(code: &str, ctx: &semantics::Context, mut root: RootObject) -> Result<LinkedList<RTData>, EvalErr> {
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
        let mut vars = Variables::new();
        result.push_back(eval_expr(expr, ctx, &mut root, &mut vars));
    }

    Ok(result)
}

fn eval_expr(expr: &Expr, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> RTData {
    match expr {
        Expr::LitNum(e)   => RTData::Int(e.num),
        Expr::LitBool(e)  => RTData::Bool(e.val),
        Expr::IfExpr(e)   => eval_if(&e, ctx, root, vars),
        Expr::DataExpr(e) => eval_data(&e, ctx, root, vars),
        _ => RTData::RuntimeErr("not yet implemented".to_string())
    }
}

fn eval_if(expr: &semantics::IfNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> RTData {
    let cond = eval_expr(&expr.cond_expr, ctx ,root, vars);
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
        eval_expr(&expr.then_expr, ctx, root, vars)
    } else {
        eval_expr(&expr.else_expr, ctx, root, vars)
    }
}

fn eval_data(expr: &semantics::DataNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> RTData {
    let data = if expr.exprs.len() == 0 {
        None
    } else {
        let mut v = Vec::new();
        for e in &expr.exprs {
            v.push(eval_expr(e, ctx, root, vars));
        }
        Some(v)
    };

    let ptr = root.make_obj(expr.label.id.to_string(), data);

    RTData::LData(ptr)
}

fn eval_let(expr: &semantics::LetNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> RTData {
    vars.push();

    for def in &expr.def_vars {
        let data = eval_expr(&def.expr, ctx, root, vars);
        if !eval_pat(&def.pattern, data, vars) {
            let pos = def.pattern.get_ast().get_pos();
            let msg = format!("{:?}:{:?}: failed pattern matching", pos.line, pos.column);
            return RTData::RuntimeErr(msg);
        }
    }

    let result = eval_expr(&expr.expr, ctx, root, vars);
    vars.pop();

    result
}

fn eval_pat(pat: &Pattern, data: RTData, vars: &mut Variables) -> bool {
    match pat {
        Pattern::PatID(p) => {
            vars.insert(p.id.to_string(), data);
            true
        }
        Pattern::PatNum(p) => {
            match data {
                RTData::Int(n) => n == p.num,
                _ => false
            }
        }
        Pattern::PatBool(p) => {
            match data {
                RTData::Bool(n) => n == p.val,
                _ => false
            }
        }
        Pattern::PatNil(_) => {
            match data {
                RTData::LData(ptr) => {
                    unsafe {
                        (*ptr).label == "Nil"
                    }
                }
                _ => false
            }
        }
        Pattern::PatTuple(p) => {
            match data {
                RTData::LData(ptr) => {
                    if unsafe { (*ptr).label == "Tuple"} {
                        return false;
                    }

                    match unsafe { &(*ptr).data } {
                        Some(rds) => {
                            for (pat2, rd) in p.pattern.iter().zip(rds.iter()) {
                                if !eval_pat(pat2, rd.clone(), vars) {
                                    return false;
                                }
                            }
                            true
                        }
                        None => true
                    }
                }
                _ => false
            }
        }
        Pattern::PatData(p) => {
            match data {
                RTData::LData(ptr) => {
                    if unsafe { (*ptr).label == p.label.id} {
                        return false;
                    }

                    match unsafe { &(*ptr).data } {
                        Some(rds) => {
                            for (pat2, rd) in p.pattern.iter().zip(rds.iter()) {
                                if !eval_pat(pat2, rd.clone(), vars) {
                                    return false;
                                }
                            }
                            true
                        }
                        None => true
                    }
                }
                _ => false
            }
        }
    }
}