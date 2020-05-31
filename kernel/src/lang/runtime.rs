use super::parser;
use super::semantics;
use super::{LispErr, Pos};

// use crate::driver;

use alloc::collections::linked_list::LinkedList;
use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use alloc::string::{ToString, String};

type Expr<'a> = semantics::LangExpr<'a>;
type Pattern<'a> = semantics::Pattern<'a>;

struct RuntimeErr {
    msg: String,
    pos: Pos
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
enum RTData {
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
struct LabeledData {
    label: String,
    data: Option<Vec<RTData>>
}

struct RootObject {
    objects: LinkedList<LabeledData>
}

impl RootObject {
    fn new() -> RootObject {
        RootObject{objects: LinkedList::new()}
    }

    fn make_obj(&mut self, label: String, data: Option<Vec<RTData>>) -> *const LabeledData {
        let obj = LabeledData{label: label, data: data};
        self.objects.push_back(obj);
        self.objects.back().unwrap() as *const LabeledData
    }
}

pub(crate) fn eval(code: &str, ctx: &semantics::Context) -> Result<LinkedList<String>, LispErr> {
    let mut ps = parser::Parser::new(code);
    let exprs;
    match ps.parse() {
        Ok(e) => {
            exprs = e;
        }
        Err(e) => {
            let msg = format!("Syntax Error: {}", e.msg);
            return Err(LispErr{msg: msg, pos: e.pos});
        }
    }

    let mut typed_exprs = LinkedList::new();
    for expr in &exprs {
        match semantics::typing_expr(expr, ctx) {
            Ok(e) => {
                typed_exprs.push_back(e);
            }
            Err(e) => {
                let msg = format!("Typing Error: {}", e.msg);
                return Err(LispErr{msg: msg, pos: e.pos});
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
                let msg = format!("(RuntimeErr [{:?} (Pos {:?} {:?})])", e.msg, e.pos.line, e.pos.column);
                result.push_back(msg);
                return Ok(result);
            }
        }

    }

    Ok(result)
}

fn eval_expr(expr: &Expr, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    match expr {
        Expr::LitNum(e)     => Ok(RTData::Int(e.num)),
        Expr::LitBool(e)    => Ok(RTData::Bool(e.val)),
        Expr::IfExpr(e)     => eval_if(&e, ctx, root, vars),
        Expr::DataExpr(e)   => eval_data(&e, ctx, root, vars),
        Expr::ListExpr(e)   => eval_list(&e, ctx, root, vars),
        Expr::LetExpr(e)    => eval_let(&e, ctx, root, vars),
        Expr::MatchExpr(e)  => eval_match(&e, ctx, root, vars),
        Expr::IDExpr(e)     => eval_id(&e, vars),
        Expr::ApplyExpr(e)  => eval_apply(&e, ctx, root, vars),
        Expr::TupleExpr(e)  => eval_tuple(&e, ctx, root, vars),
        Expr::LambdaExpr(e) => {
            let pos = e.ast.get_pos();
            return Err(RuntimeErr{msg: "not yet implemented".to_string(), pos: pos})
        }
    }
}

fn eval_tuple(expr: &semantics::Exprs, ctx: &semantics::Context, root: &mut RootObject, vars: &mut Variables) -> Result<RTData, RuntimeErr> {
    let mut v = Vec::new();
    for e in expr.exprs.iter() {
        v.push(eval_expr(e, ctx, root, vars)?);
    }

    let elm = root.make_obj("Tuple".to_string(), Some(v));

    Ok(RTData::LData(elm))
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
            return Err(RuntimeErr{msg: "empty application".to_string(), pos: pos})
        }
    }

    let fun_name;
    match eval_expr(&fun_expr, ctx, root, vars)? {
        RTData::Defun(f) => {
            fun_name = f;
        }
        _ => {
            let pos = fun_expr.get_ast().get_pos();
            return Err(RuntimeErr{msg: "not function".to_string(), pos: pos})
        }
    }

    if ctx.built_in.contains(&fun_name) {
        let mut v = Vec::new();
        for e in iter {
            let data = eval_expr(&e, ctx, root, vars)?;
            v.push(data);
        }
        return eval_built_in(fun_name, v, expr.ast.get_pos(), ctx);
    }

    let fun;
    match ctx.funs.get(&fun_name) {
        Some(f) => {
            fun = f;
        }
        None => {
            let pos = fun_expr.get_ast().get_pos();
            let msg = format!("{:?} is not defined", fun_name);
            return Err(RuntimeErr{msg: msg, pos: pos});
        }
    }

    let mut vars_fun = Variables::new();
    for (e, arg) in iter.zip(fun.args.iter()) {
        let data = eval_expr(&e, ctx, root, vars)?;
        vars_fun.insert(arg.id.to_string(), data);
    }

    eval_expr(&fun.expr, ctx, root, &mut vars_fun)
}

fn get_int_int(args: Vec<RTData>, pos: Pos) -> Result<(i64, i64), RuntimeErr> {
    match (args[0].clone(), args[1].clone()) {
        (RTData::Int(n1), RTData::Int(n2)) => {
            Ok((n1, n2))
        }
        _ => {
            Err(RuntimeErr{msg: "there must be exactly 2 integers".to_string(), pos: pos})
        }
    }
}

fn get_int_int_int(args: Vec<RTData>, pos: Pos) -> Result<(i64, i64, i64), RuntimeErr> {
    match (args[0].clone(), args[1].clone(), args[2].clone()) {
        (RTData::Int(n1), RTData::Int(n2), RTData::Int(n3)) => {
            Ok((n1, n2, n3))
        }
        _ => {
            Err(RuntimeErr{msg: "there must be exactly 2 integers".to_string(), pos: pos})
        }
    }
}

fn get_bool_bool(args: Vec<RTData>, pos: Pos) -> Result<(bool, bool), RuntimeErr> {
    match (args[0].clone(), args[1].clone()) {
        (RTData::Bool(n1), RTData::Bool(n2)) => {
            Ok((n1, n2))
        }
        _ => {
            Err(RuntimeErr{msg: "there must be exactly 2 boolean values".to_string(), pos: pos})
        }
    }
}

fn get_bool(args: Vec<RTData>, pos: Pos) -> Result<bool, RuntimeErr> {
    match args[0].clone() {
        RTData::Bool(n) => {
            Ok(n)
        }
        _ => {
            Err(RuntimeErr{msg: "there must be exactly 2 boolean values".to_string(), pos: pos})
        }
    }
}

fn eval_built_in(fun_name: String, args: Vec<RTData>, pos: Pos, ctx: &semantics::Context) -> Result<RTData, RuntimeErr> {
    match fun_name.as_str() {
        "+" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Int(n1 + n2))
        }
        "-" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Int(n1 - n2))
        }
        "*" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Int(n1 * n2))
        }
        "/" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Int(n1 / n2))
        }
        "<" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Bool(n1 < n2))
        }
        ">" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Bool(n1 > n2))
        }
        "=" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Bool(n1 == n2))
        }
        "<=" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Bool(n1 <= n2))
        }
        ">=" => {
            let (n1, n2) = get_int_int(args, pos)?;
            Ok(RTData::Bool(n1 >= n2))
        }
        "and" => {
            let (n1, n2) = get_bool_bool(args, pos)?;
            Ok(RTData::Bool(n1 && n2))
        }
        "or" => {
            let (n1, n2) = get_bool_bool(args, pos)?;
            Ok(RTData::Bool(n1 || n2))
        }
        "xor" => {
            let (n1, n2) = get_bool_bool(args, pos)?;
            Ok(RTData::Bool(n1 ^ n2))
        }
        "not" => {
            let n = get_bool(args, pos)?;
            Ok(RTData::Bool(!n))
        }
        "call-rust" => {
            let (n1, n2, n3) = get_int_int_int(args, pos)?;
            Ok(RTData::Int((ctx.callback)(n1, n2, n3)))
        }
        _ => {
            Err(RuntimeErr{msg: "unknown built-in function".to_string(), pos: pos})
        }
    }
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
    Err(RuntimeErr{msg: "pattern-matching is not exhaustive".to_string(), pos: pos})
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
            return Err(RuntimeErr{msg: "type mismatched".to_string(), pos: pos});
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
            return Err(RuntimeErr{msg: "failed pattern matching".to_string(), pos: pos});
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