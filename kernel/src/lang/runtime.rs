use super::parser;
use super::semantics;

// use crate::driver;

use alloc::collections::linked_list::LinkedList;
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use alloc::string::{ToString, String};

type Expr<'a> = semantics::LangExpr<'a>;
type Pattern<'a> = semantics::Pattern<'a>;

struct RuntimeErr {
    msg: String
}

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
    Defun(String),
    LData(*const LabeledData),
}

impl RTData {
    fn get_by_lisp(&self) -> String {
        match self {
            RTData::Int(n)   => format!("{:?}", n),
            RTData::Bool(n)  => format!("{:?}", n),
            RTData::Defun(n) => format!("{}", n),
            RTData::LData(n) => {
                let mut msg = format!("({}", unsafe { &(*(*n)).label });
                match unsafe { (*(*n)).data.as_ref() } {
                    Some(ld) => {
                        for d in ld.iter() {
                            msg = format!("{} {}", msg, d.get_by_lisp());
                        }
                        format!("{})", msg)
                    }
                    None => {
                        format!("{})", msg)
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct LabeledData {
    label: String,
    data: Option<Vec<RTData>>
}

struct RootObject {
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

pub fn eval(code: &str, ctx: &semantics::Context) -> Result<LinkedList<String>, EvalErr> {
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

    let mut root = RootObject::new();
    let mut result = LinkedList::new();
    for expr in &typed_exprs {
        let mut vars = Variables::new();
        match eval_expr(expr, ctx, &mut root, &mut vars) {
            Ok(val) => {
                result.push_back(val.get_by_lisp());
            }
            Err(e) => {
                let msg = format!("(RuntimeErr {:?})", e.msg);
                result.push_back(msg);
                return Ok(result);
            }
        }

    }

    Ok(result)
}

fn eval_expr(expr: &Expr, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    match expr {
        Expr::LitNum(e)    => Ok(RTData::Int(e.num)),
        Expr::LitBool(e)   => Ok(RTData::Bool(e.val)),
        Expr::IfExpr(e)    => eval_if(&e, ctx, root, vars),
        Expr::DataExpr(e)  => eval_data(&e, ctx, root, vars),
        Expr::ListExpr(e)  => eval_list(&e, ctx, root, vars),
        Expr::LetExpr(e)   => eval_let(&e, ctx, root, vars),
        Expr::MatchExpr(e) => eval_match(&e, ctx, root, vars),
        Expr::IDExpr(e)    => eval_id(&e, vars),
        Expr::ApplyExpr(e) => eval_apply(&e, ctx, root, vars),
        _ => Err(RuntimeErr{msg: "not yet implemented".to_string()})
    }
}

fn eval_apply(expr: &semantics::Exprs, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let mut iter = expr.exprs.iter();
    let fun_expr;
    match iter.next() {
        Some(e) => {
            fun_expr = e;
        }
        None => {
            let pos = expr.ast.get_pos();
            let msg = format!("{:?}:{:?}: empty application", pos.line, pos.column);
            return Err(RuntimeErr{msg: msg})
        }
    }

    let fun_name;
    match eval_expr(&fun_expr, ctx, root, vars)? {
        RTData::Defun(f) => {
            fun_name = f;
        }
        _ => {
            let pos = fun_expr.get_ast().get_pos();
            let msg = format!("{:?}:{:?}: not function", pos.line, pos.column);
            return Err(RuntimeErr{msg: msg})
        }
    }

    let fun;
    match ctx.funs.get(&fun_name) {
        Some(f) => {
            fun = f;
        }
        None => {
            let pos = fun_expr.get_ast().get_pos();
            let msg = format!("{:?}:{:?}: {:?} is not defined", fun_name, pos.line, pos.column);
            return Err(RuntimeErr{msg: msg})
        }
    }

    let mut vars_fun = Variables::new();
    for (e, arg) in iter.zip(fun.args.iter()) {
        let data = eval_expr(&e, ctx, root, vars)?;
        vars_fun.insert(arg.id.to_string(), data);
    }

    eval_expr(&fun.expr, ctx, root, &mut vars_fun)
}

fn eval_match(expr: &semantics::MatchNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let data = eval_expr(&expr.expr, ctx, root, vars)?;

    for c in &expr.cases {
        vars.push();
        if eval_pat(&c.pattern, data.clone(), vars) {
            let retval = eval_expr(&c.expr, ctx, root, vars)?;
            vars.pop();
            return Ok(retval);
        }
        vars.pop();
    }

    let pos = expr.ast.get_pos();
    let msg = format!("{:?}:{:?}: pattern-matching is not exhaustive", pos.line, pos.column);
    Err(RuntimeErr{msg: msg})
}

fn eval_id(expr: &semantics::IDNode, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let id = expr.id.to_string();
    match vars.get(&id) {
        Some(data) => Ok(data.clone()),
        None => Ok(RTData::Defun(id))
    }
}

fn eval_list(expr: &semantics::Exprs, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let mut elm = root.make_obj("Nil".to_string(), None);
    for e in expr.exprs.iter().rev() {
        let val = eval_expr(e, ctx, root, vars)?;
        elm = root.make_obj("Cons".to_string(), Some(vec!(val, RTData::LData(elm))));
    }

    Ok(RTData::LData(elm))
}

fn eval_if(expr: &semantics::IfNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let cond = eval_expr(&expr.cond_expr, ctx ,root, vars)?;
    let flag;
    match cond {
        RTData::Bool(e) => {
            flag = e;
        }
        _ => {
            let pos = expr.cond_expr.get_ast().get_pos();
            let msg = format!("{:?}:{:?}: type mismatched", pos.line, pos.column);
            return Err(RuntimeErr{msg: msg});
        }
    }

    if flag {
        eval_expr(&expr.then_expr, ctx, root, vars)
    } else {
        eval_expr(&expr.else_expr, ctx, root, vars)
    }
}

fn eval_data(expr: &semantics::DataNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let data = if expr.exprs.len() == 0 {
        None
    } else {
        let mut v = Vec::new();
        for e in &expr.exprs {
            v.push(eval_expr(e, ctx, root, vars)?);
        }
        Some(v)
    };

    let ptr = root.make_obj(expr.label.id.to_string(), data);

    Ok(RTData::LData(ptr))
}

fn eval_let(expr: &semantics::LetNode, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    vars.push();

    for def in &expr.def_vars {
        let data = eval_expr(&def.expr, ctx, root, vars)?;
        if !eval_pat(&def.pattern, data, vars) {
            let pos = def.pattern.get_ast().get_pos();
            let msg = format!("{:?}:{:?}: failed pattern matching", pos.line, pos.column);
            return Err(RuntimeErr{msg: msg});
        }
    }

    let result = eval_expr(&expr.expr, ctx, root, vars)?;
    vars.pop();

    Ok(result)
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
                    if unsafe { (*ptr).label != p.label.id} {
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