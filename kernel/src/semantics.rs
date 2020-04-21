use crate::parser;

use alloc::collections::linked_list::LinkedList;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::BTreeSet;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::{ToString, String};

#[derive(Debug)]
pub struct TypingErr<'t> {
    msg: String,
    ast: &'t parser::Expr
}

impl<'t> TypingErr<'t> {
    fn new(msg: &str, ast: &'t parser::Expr) -> TypingErr<'t> {
        TypingErr{msg: msg.to_string(), ast: ast}
    }
}

#[derive(Debug)]
enum TypedExpr<'t> {
    IfExpr(Box::<IfNode<'t>>),
    LetExpr(Box::<LetNode<'t>>),
    LitNum(NumNode<'t>),
    LitBool(BoolNode<'t>),
    IDExpr(IDNode<'t>),
    MatchExpr(Box::<MatchNode<'t>>),
    ApplyExpr(Exprs<'t>),
    ListExpr(Exprs<'t>),
    TupleExpr(Exprs<'t>),
}

#[derive(Debug)]
struct NumNode<'t> {
    num: i64,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct BoolNode<'t> {
    val: bool,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct IDNode<'t> {
    id: &'t str,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct IfNode<'t> {
    cond_expr: TypedExpr<'t>,
    then_expr: TypedExpr<'t>,
    else_expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
enum LetPat<'t> {
    LetPatID(IDNode<'t>),
    LetPatTuple(LetPatTupleNode<'t>)
}

#[derive(Debug)]
struct LetPatTupleNode<'t> {
    pattern: Vec::<LetPat<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct LetNode<'t> {
    def_vars: Vec<DefVar<'t>>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct DefVar<'t> {
    pattern: LetPat<'t>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct MatchNode<'t> {
    expr: TypedExpr<'t>,
    cases: Vec<MatchCase<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
enum MatchPat<'t> {
    MatchPatNum(NumNode<'t>),
    MatchPatBool(BoolNode<'t>),
    MatchPatID(IDNode<'t>),
    MatchPatTuple(MatchPatTupleNode<'t>),
    MatchPatData(MatchPatDataNode<'t>)
}

#[derive(Debug)]
struct MatchPatTupleNode<'t> {
    pattern: Vec<MatchPat<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct MatchPatDataNode<'t> {
    ty: TIDNode<'t>,
    pattern: Vec<MatchPat<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct MatchCase<'t> {
    pattern: MatchPat<'t>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct Exprs<'t> {
    exprs: Vec<TypedExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct TIDNode<'t> {
    id: &'t str,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct TypeBoolNode<'t> {
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct TypeIntNode<'t> {
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct DataType<'t> {
    name: DataTypeName<'t>,
    members: Vec<DataTypeMem<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct DataTypeName<'t> {
    id: TIDNode<'t>,
    type_args: Vec<IDNode<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct DataTypeMem<'t> {
    id: TIDNode<'t>,
    types: Vec<Type<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
enum Type<'t> {
    TypeBool(TypeBoolNode<'t>),
    TypeInt(TypeIntNode<'t>),
    TypeList(TypeListNode<'t>),
    TypeTuple(TypeTupleNode<'t>),
    TypeFun(TypeFunNode<'t>),
    TypeData(TypeDataNode<'t>),
    TypeID(IDNode<'t>)
}

#[derive(Debug)]
struct TypeListNode<'t> {
    ty: Box::<Type<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct TypeTupleNode<'t> {
    ty: Vec<Type<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
enum Effect {
    IO,
    Pure
}

#[derive(Debug)]
struct TypeFunNode<'t> {
    effect: Effect,
    args: Vec<Type<'t>>,
    ret: Box<Type<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct TypeDataNode<'t> {
    id: TIDNode<'t>,
    type_args: Vec<Type<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct Defun<'t> {
    id: IDNode<'t>,
    args: Vec<IDNode<'t>>,
    fun_type: Type<'t>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
pub struct Context<'t> {
    funs: BTreeMap<&'t str, Defun<'t>>,
    data: BTreeMap<&'t str, DataType<'t>>
}

impl<'t> Context<'t> {
    fn new(funs: BTreeMap<&'t str, Defun<'t>>, data: BTreeMap<&'t str, DataType<'t>>) -> Context<'t> {
        Context{funs: funs, data: data}
    }

    pub fn typing(&self) -> Result<(), TypingErr> {
        self.check_data()
    }

    fn check_data(&self) -> Result<(), TypingErr> {
        for (_, d) in self.data.iter() {
            check_data(d)?;
        }

        Ok(())
    }
}

pub fn exprs2context(exprs: &LinkedList<parser::Expr>) -> Result<Context, TypingErr> {
    let mut funs = BTreeMap::new();
    let mut data = BTreeMap::new();
    let msg = "error: top expression must be data, defun, or export";

    for e in exprs {
        match e {
            parser::Expr::Apply(es) => {
                let mut iter = es.iter();

                match iter.next() {
                    Some(parser::Expr::ID(id)) => {
                        if id == "defun" || id == "export" {
                            let f = expr2defun(e)?;

                            if funs.contains_key(f.id.id) {
                                let msg = format!("error: function {:?} is multiply defined", f.id.id);
                                return Err(TypingErr{msg: msg, ast: e});
                            }

                            funs.insert(f.id.id, f);
                        } else if id == "data" {
                            let d = expr2data(e)?;
                            if data.contains_key(d.name.id.id) {
                                let msg = format!("error: data type {:?} is multiply defined", d.name.id.id);
                                return Err(TypingErr{msg: msg, ast: e});
                            }

                            data.insert(d.name.id.id, d);
                        } else {
                            return Err(TypingErr::new(msg, e));
                        }
                    }
                    _ => {
                        return Err(TypingErr::new(msg, e));
                    }
                }
            }
            _ => {
                return Err(TypingErr::new(msg, e));
            }
        }
    }

    Ok(Context::new(funs, data))
}

fn check_data<'t>(data: &'t DataType) -> Result<(), TypingErr<'t>> {
    let mut args = BTreeSet::new();
    for arg in data.name.type_args.iter() {
        args.insert(arg.id);
    }

    for mem in data.members.iter() {
        check_data_mem(mem, &args)?;
    }

    Ok(())
}

fn check_data_mem<'t>(mem: &'t DataTypeMem, args: &BTreeSet<&str>) -> Result<(), TypingErr<'t>> {
    for it in mem.types.iter() {
        check_type(it, args)?
    }

    Ok(())
}

fn check_type<'t>(ty: &'t Type, args: &BTreeSet<&str>) -> Result<(), TypingErr<'t>> {
    match ty {
        Type::TypeID(id) => {
            if !args.contains(id.id) {
                let msg = format!("error: {:?} is undefined", id.id);
                return Err(TypingErr{msg: msg, ast: id.ast})
            }
        }
        Type::TypeList(list) => {
            check_type(&list.ty, args)?;
        }
        Type::TypeTuple(tuple) => {
            for it in tuple.ty.iter() {
                check_type(it, args)?;
            }
        }
        Type::TypeData(data) => {
            for it in data.type_args.iter() {
                check_type(it, args)?;
            }
        }
        Type::TypeFun(fun) => {
            for it in fun.args.iter() {
                check_type(it, args)?
            }

            check_type(&fun.ret, args)?
        }
        _ => {}
    }

    Ok(())
}

/// $DATA := ( data $DATA_NAME $MEMBER+ )
fn expr2data(expr: &parser::Expr) -> Result<DataType, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();
            iter.next(); // must be "data"

            // $DATA_NAME
            let data_name;
            match iter.next() {
                Some(e) => {
                    data_name = expr2data_name(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: require data name", expr))
                }
            }

            // $MEMBER+
            let mut mems = Vec::new();
            for mem in iter.next() {
                let data_mem = expr2data_mem(mem)?;
                mems.push(data_mem);
            }

            Ok(DataType{name: data_name, members: mems, ast: expr})
        }
        _ => {
            Err(TypingErr::new("error", expr))
        }
    }
}

/// $DATA_NAME := $TID | ( $TID $ID* )
fn expr2data_name(expr: &parser::Expr) -> Result<DataTypeName, TypingErr> {
    match expr {
        parser::Expr::ID(_) => {
            let tid = expr2type_id(expr)?;
            Ok(DataTypeName{id: tid, type_args: Vec::new(), ast: expr})
        }
        parser::Expr::Apply(exprs) => {
            let mut args = Vec::new();
            let mut iter = exprs.iter();
            let tid;

            match iter.next() {
                Some(e) => {
                    tid = expr2type_id(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: must type identifier (with type arguments)", expr))
                }
            }

            for it in iter {
                let id = expr2id(it)?;
                args.push(id);
            }

            Ok(DataTypeName{id: tid, type_args: args, ast: expr})
        }
        _ => {
            Err(TypingErr::new("error: must type identifier (with type arguments)", expr))
        }
    }
}

fn expr2type_id(expr: &parser::Expr) -> Result<TIDNode, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            match id.chars().nth(0) {
                Some(c) => {
                    if 'A' <= c && c <= 'Z' {
                        Ok(TIDNode{id: id, ast: expr})
                    } else {
                        Err(TypingErr::new("error: the first character must be captal", expr))
                    }
                }
                _ => {
                    Err(TypingErr::new("error", expr))
                }
            }
        }
        _ => {
            Err(TypingErr::new("error: must be type identifier", expr))
        }
    }
}

fn expr2id(expr: &parser::Expr) -> Result<IDNode, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            match id.chars().nth(0) {
                Some(c) => {
                    if 'A' <= c && c <= 'Z' {
                        Err(TypingErr::new("error: the first character must not be captal", expr))
                    } else {
                        Ok(IDNode{id: id, ast: expr})
                    }
                }
                _ => {
                    Err(TypingErr::new("error", expr))
                }
            }
        }
        _ => {
            Err(TypingErr::new("error: must be identifier", expr))
        }
    }
}

/// $MEMBER := $TID | ( $TID $TYPE* )
fn expr2data_mem(expr: &parser::Expr) -> Result<DataTypeMem, TypingErr> {
    match expr {
        parser::Expr::ID(_) => {
            // $TID
            let tid = expr2type_id(expr)?;
            Ok(DataTypeMem{id: tid, types: Vec::new(), ast: expr})
        }
        parser::Expr::Apply(exprs) => {
            // ( $TID $TYPE* )
            let mut iter = exprs.iter();
            let tid;

            match iter.next() {
                Some(e) => {
                    tid = expr2type_id(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: must type identifier", expr))
                }
            }

            let mut types = Vec::new();
            for it in iter {
                let pt = expr2type(it)?;
                types.push(pt);
            }

            Ok(DataTypeMem{id: tid, types: types , ast: expr})
        }
        _ => {
            Err(TypingErr::new("error: must be type identifier (with types)", expr))
        }
    }
}

/// $DEFUN := ( $HEAD_DEFUN $ID ( $ID* ) $TYPE_FUN $EXPR )
fn expr2defun(expr: &parser::Expr) -> Result<Defun, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();

            // $HEAD_DEFUN := export | defun
            iter.next(); // must be "export" or "defun"

            // $ID
            let id;
            match iter.next() {
                Some(e) => {
                    id = expr2id(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: require function name", expr));
                }
            }

            // ( $ID* )
            let mut args = Vec::new();
            match iter.next() {
                Some(parser::Expr::Apply(exprs)) => {
                    for it in exprs.iter() {
                        let arg = expr2id(it)?;
                        args.push(arg);
                    }
                }
                _ => {
                    return Err(TypingErr::new("error: require arguments", expr));
                }
            }

            // $TYPE_FUN
            let fun;
            match iter.next() {
                Some(e) => {
                    fun = expr2type_fun(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: require function type", expr));
                }
            }

            // $EXPR
            let body;
            match iter.next() {
                Some(e) => {
                    body = expr2typed_expr(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: require expression", expr));
                }
            }

            Ok(Defun{id: id, args: args, fun_type: fun, expr: body, ast: expr})
        }
        _ => {
            Err(TypingErr::new("error", expr))
        }
    }
}

/// $TYPE_FUN := ( $EFFECT ( -> $TYPES $TYPES ) )
fn expr2type_fun(expr: &parser::Expr) -> Result<Type, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();

            // $EFFECT := Pure | IO
            let effect;
            let e = iter.next();
            match e {
                Some(parser::Expr::ID(eff)) => {
                    if eff == "IO" {
                        effect = Effect::IO;
                    } else if eff == "Pure" {
                        effect = Effect::Pure;
                    } else {
                        return Err(TypingErr::new("error: effect must be \"Pure\" or \"IO\"", e.unwrap()));
                    }
                }
                _ => {
                    return Err(TypingErr::new("error: invalid effect", expr));
                }
            }

            // ( -> $TYPES $TYPE )
            let e1 = iter.next();
            let args;
            let ret;
            match e1 {
                Some(parser::Expr::Apply(exprs2)) => {
                    let mut iter2 = exprs2.iter();
                    let e2 = iter2.next();
                    match e2 {
                        Some(parser::Expr::ID(arr)) => {
                            if arr != "->" {
                                return Err(TypingErr::new("error: must be \"->\"", e2.unwrap()));
                            }
                        }
                        _ => {
                            return Err(TypingErr::new("error: require \"->\"", e1.unwrap()));
                        }
                    }

                    // $TYPES := $TYPE | ( $TYPE* )
                    match iter2.next() {
                        Some(t) => {
                            args = expr2types(t)?;
                        }
                        _ => {
                            return Err(TypingErr::new("error: require types for arguments", e1.unwrap()));
                        }
                    }

                    // $TYPE
                    match iter2.next() {
                        Some(t) => {
                            ret = expr2type(t)?;
                        }
                        _ => {
                            return Err(TypingErr::new("error: require type for return value", e1.unwrap()));
                        }
                    }
                }
                _ => {
                    return Err(TypingErr::new("error: require function type", expr));
                }
            }

            Ok(Type::TypeFun(TypeFunNode{effect: effect, args: args, ret: Box::new(ret), ast: expr}))
        }
        _ => {
            Err(TypingErr::new("error", expr))
        }
    }
}

/// $TYPES := $TYPE | ( $TYPE* )
fn expr2types(expr: &parser::Expr) -> Result<Vec<Type>, TypingErr> {
    match expr {
        parser::Expr::Apply(types) => {
            // ( $TYPES* )
            Ok(list_types2vec_types(types)?)
        }
        t => {
            // $TYPE
            let mut v = Vec::new();
            v.push(expr2type(t)?);
            Ok(v)
        }
    }
}

/// $TYPE := Int | Bool | $TYPE_LIST | $TYPE_TUPLE | $TYPE_FUN | $TYPE_DATA | $ID
fn expr2type(expr: &parser::Expr) -> Result<Type, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            // Int | Bool | $TID
            if id == "Int" {
                Ok(Type::TypeInt(TypeIntNode{ast: expr}))
            } else if id == "Bool" {
                Ok(Type::TypeBool(TypeBoolNode{ast: expr}))
            } else {
                let c = id.chars().nth(0).unwrap();
                if 'A' <= c && c <= 'Z' {
                    let tid = expr2type_id(expr)?;
                    Ok(Type::TypeData(TypeDataNode{id: tid, type_args: Vec::new(), ast: expr}))
                } else {
                    Ok(Type::TypeID(expr2id(expr)?))
                }
            }
        }
        parser::Expr::List(list) => {
            // $TYPE_LIST := '( $TYPE )
            if list.len() != 1 {
                return Err(TypingErr::new("error: require exactly one type as a type argument for list type", expr));
            }

            match list.iter().next() {
                Some(e) => {
                    let ty = Box::new(expr2type(e)?);
                    Ok(Type::TypeList(TypeListNode{ty: ty, ast: e}))
                }
                _ => {
                    Err(TypingErr::new("error: require type", expr))
                }
            }
        }
        parser::Expr::Tuple(tuple) => {
            // $TYPE_TUPLE := [ $TYPE+ ]
            if tuple.len() < 1 {
                return Err(TypingErr::new("error: require more than or equal to oen type", expr));
            }

            let mut types = Vec::new();
            for it in tuple {
                types.push(expr2type(it)?);
            }

            Ok(Type::TypeTuple(TypeTupleNode{ty: types, ast: expr}))
        }
        parser::Expr::Apply(exprs) => {
            // ( $TID $TYPE* )
            let mut iter = exprs.iter();

            // $TID
            let tid;
            let e = iter.next();
            match e {
                Some(parser::Expr::ID(id)) => {
                    // $TYPE_FUN
                    if id == "Pure" || id == "IO" {
                        let ty = expr2type_fun(e.unwrap())?;
                        return Ok(ty);
                    }
                    tid = expr2type_id(e.unwrap())?;
                }
                _ => {
                    return Err(TypingErr::new("error: require type", expr));
                }
            }

            // $TYPE*
            let mut args = Vec::new();
            for it in iter {
                args.push(expr2type(it)?);
            }

            Ok(Type::TypeData(TypeDataNode{id: tid, type_args: args, ast: expr}))
        }
        _ => {
            Err(TypingErr::new("error: must be type", expr))
        }
    }
}

fn list_types2vec_types(exprs: &LinkedList<parser::Expr>) -> Result<Vec<Type>, TypingErr> {
    let mut v = Vec::new();
    for e in exprs {
        v.push(expr2type(e)?);
    }

    Ok(v)
}

/// $EXPR := $LITERAL | $ID | $LET | $IF | $MATCH | $LIST | $TUPLE | $APPLY
fn expr2typed_expr(expr: &parser::Expr) -> Result<TypedExpr, TypingErr> {
    match expr {
        parser::Expr::Num(num) => {
            Ok(TypedExpr::LitNum(NumNode{num: *num, ast: expr}))
        }
        parser::Expr::Bool(val) => {
            Ok(TypedExpr::LitBool(BoolNode{val: *val, ast: expr}))
        }
        parser::Expr::ID(id) => {
            Ok(TypedExpr::IDExpr(IDNode{id: id, ast: expr}))
        }
        parser::Expr::List(list) => {
            let mut elms = Vec::new();
            for it in list {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(TypedExpr::ListExpr(Exprs{exprs: elms, ast: expr}))
        }
        parser::Expr::Tuple(tuple) => {
            let mut elms = Vec::new();
            for it in tuple {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(TypedExpr::TupleExpr(Exprs{exprs: elms, ast: expr}))
        }
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();

            match iter.next() {
                Some(parser::Expr::ID(id)) => {
                    if id == "if" {
                        return Ok(expr2if(expr)?);
                    } else if id == "let" {
                        return Ok(expr2let(expr)?);
                    } else if id == "match" {
                        return Ok(expr2match(expr)?);
                    }
                }
                Some(_) => { () }
                None => {
                    return Err(TypingErr::new("error: require function application", expr));
                }
            }

            let mut elms = Vec::new();
            for it in exprs {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(TypedExpr::ApplyExpr(Exprs{exprs: elms, ast: expr}))
        }
    }
}

/// $IF := ( if $EXPR $EXPR $EXPR )
fn expr2if(expr: &parser::Expr) -> Result<TypedExpr, TypingErr> {
    let exprs;
    match expr {
        parser::Expr::Apply(e) => {
            exprs = e;
        }
        _ => {
            return Err(TypingErr::new("error: if expression", expr));
        }
    }

    let mut iter = exprs.iter();
    iter.next(); // must be "if"

    let f = |next, msg| {
        match next {
            Some(e) => {
                return expr2typed_expr(e);
            }
            _ => {
                return Err(TypingErr::new(msg, expr));
            }
        }
    };

    let cond = f(iter.next(), "error: if requires condition")?;
    let then = f(iter.next(), "error: if requires then expression")?;
    let else_expr = f(iter.next(), "error: if requires else expression")?;

    Ok(TypedExpr::IfExpr(Box::new(IfNode{cond_expr: cond, then_expr: then, else_expr: else_expr, ast: expr})))
}

/// $LET := ( let ( $DEFVAR+ ) $EXPR )
fn expr2let(expr: &parser::Expr) -> Result<TypedExpr, TypingErr> {
    let exprs;
    match expr {
        parser::Expr::Apply(e) => {
            exprs = e;
        }
        _ => {
            return Err(TypingErr::new("error: if expression", expr));
        }
    }

    let mut iter = exprs.iter();
    iter.next(); // must be "let"

    // ( $DEFVAR+ )
    let mut def_vars = Vec::new();
    let e = iter.next();
    match e {
        Some(parser::Expr::Apply(dvs)) => {
            if dvs.len() == 0 {
                return Err(TypingErr::new("error: require variable binding", e.unwrap()));
            }

            for it in dvs.iter() {
                def_vars.push(expr2def_vars(it)?);
            }
        }
        _ => {
            return Err(TypingErr::new("error: require variable binding", expr));
        }
    }

    // $EXPR
    let body;
    let e = iter.next();
    match e {
        Some(body_expr) => {
            body = expr2typed_expr(body_expr)?;
        }
        _ => {
            return Err(TypingErr::new("error: require body", expr));
        }
    }

    Ok(TypedExpr::LetExpr(Box::new(LetNode{def_vars: def_vars, expr: body, ast: expr})))
}

/// $LETPAT := $ID | [ $LETPAT ]
fn expr2letpat(expr: &parser::Expr) -> Result<LetPat, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            // $ID
            let c = id.chars().nth(0).unwrap();
            if 'A' <= c && c <= 'Z' {
                Err(TypingErr::new("error: invalid pattern", expr))
            } else {
                Ok(LetPat::LetPatID(IDNode{id: id, ast: expr}))
            }
        }
        parser::Expr::Tuple(tuple) => {
            // [ $LETPAT ]
            if tuple.len() == 0 {
                return Err(TypingErr::new("error: require at least one pattern", expr));
            }

            let mut pattern = Vec::new();
            for it in tuple {
                pattern.push(expr2letpat(it)?);
            }

            Ok(LetPat::LetPatTuple(LetPatTupleNode{pattern: pattern, ast: expr}))
        }
        _ => {
            Err(TypingErr::new("error: invalid pattern", expr))
        }
    }
}

/// $DEFVAR := ( $LETPAT $EXPR )
fn expr2def_vars(expr: &parser::Expr) -> Result<DefVar, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            if exprs.len() != 2 {
                return Err(TypingErr::new("invalid variable definition", expr))
            }

            let mut iter = exprs.iter();

            let pattern = expr2letpat(iter.next().unwrap())?;  // $LETPAT
            let body = expr2typed_expr(iter.next().unwrap())?; // $EXPR

            Ok(DefVar{pattern: pattern, expr: body, ast: expr})
        }
        _ => {
            Err(TypingErr::new("must be variable definition(s)", expr))
        }
    }
}

/// $PATTERN := $LITERAL | $ID | $TID | [ $PATTERN+ ] | ( $TID $PATTERN* )
fn expr2mpat(expr: &parser::Expr) -> Result<MatchPat, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            let c = id.chars().nth(0).unwrap();
            if 'A' <= c && c <= 'Z' {
                // $TID
                let tid = expr2type_id(expr)?;
                Ok(MatchPat::MatchPatData(MatchPatDataNode{ty: tid, pattern: Vec::new(), ast: expr}))
            } else {
                // $ID
                let id_node = expr2id(expr)?;
                Ok(MatchPat::MatchPatID(id_node))
            }
        }
        parser::Expr::Bool(val) => {
            // $LITERAL
            Ok(MatchPat::MatchPatBool(BoolNode{val: *val, ast: expr}))
        }
        parser::Expr::Num(num) => {
            // $LITERAL
            Ok(MatchPat::MatchPatNum(NumNode{num: *num, ast: expr}))
        }
        parser::Expr::Tuple(exprs) => {
            // [ $PATTERN+ ]
            let mut pattern = Vec::new();
            for it in exprs {
                pattern.push(expr2mpat(it)?);
            }

            Ok(MatchPat::MatchPatTuple(MatchPatTupleNode{pattern: pattern, ast: expr}))
        }
        parser::Expr::Apply(exprs) => {
            // ( $TID $PATTERN* )
            let mut iter = exprs.iter();
            let first = iter.next();
            let tid;
            match first {
                Some(e) => {
                    tid = expr2type_id(e)?
                }
                _ => {
                    return Err(TypingErr::new("error: invalid pattern", expr));
                }
            }

            let mut pattern = Vec::new();
            for it in iter {
                pattern.push(expr2mpat(it)?);
            }

            Ok(MatchPat::MatchPatData(MatchPatDataNode{ty: tid, pattern: pattern, ast: expr}))
        }
        _ => {
            Err(TypingErr::new("error: list pattern is not supported", expr))
        }
    }
}

/// $CASE := ( $PATTERN $EXPR )
fn expr2case(expr: &parser::Expr) -> Result<MatchCase, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            if exprs.len() != 2 {
                return Err(TypingErr::new("error: case require exactly 2 expressions", expr));
            }

            let mut iter = exprs.iter();
            let pattern = expr2mpat(iter.next().unwrap())?;
            let body = expr2typed_expr(iter.next().unwrap())?;

            Ok(MatchCase{pattern: pattern, expr: body, ast: expr})
        }
        _ => {
            Err(TypingErr::new("error: invalid case", expr))
        }
    }
}

/// $MATCH := ( match $EXPR $CASE+ )
fn expr2match(expr: &parser::Expr) -> Result<TypedExpr, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();
            iter.next(); // must be "match"

            let cond;
            match iter.next() {
                Some(e) => {
                    cond = expr2typed_expr(e)?;
                }
                _ => {
                    return Err(TypingErr::new("error: no condition", expr));
                }
            }

            let mut cases = Vec::new();
            for it in iter {
                cases.push(expr2case(it)?);
            }

            let node = MatchNode{expr: cond, cases: cases, ast: expr};
            Ok(TypedExpr::MatchExpr(Box::new(node)))
        }
        _ => {
            Err(TypingErr::new("error: invalid match", expr))
        }
    }
}