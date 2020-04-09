use crate::parser;

use alloc::collections::linked_list::LinkedList;
use alloc::vec::Vec;
use alloc::boxed::Box;

struct TypingErr<'t> {
    msg: &'static str,
    ast: &'t parser::Expr
}

enum TypedExpr<'t> {
    IfExpr(Box::<IfNode<'t>>),
    LetExpr(Box::<LetNode<'t>>),
    LitNum(NumNode<'t>),
    LitBool(BoolNode<'t>),
    IDExpr(IDNode<'t>),
    MatchExpr(Box::<MatchNode<'t>>),
    ApplyExpr(ApplyNode<'t>)
}

struct NumNode<'t> {
    num: i64,
    ast: &'t parser::Expr
}

struct BoolNode<'t> {
    val: bool,
    ast: &'t parser::Expr
}

struct IDNode<'t> {
    id: &'t str,
    ast: &'t parser::Expr
}

struct IfNode<'t> {
    cond_expr: TypedExpr<'t>,
    then_expr: TypedExpr<'t>,
    else_expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

enum LetPat<'t> {
    LetPatNone(TypedExpr<'t>),
    LetPatID(TypedExpr<'t>),
    LetPatData(LetPatDataNode<'t>),
    LetPatTuple(LetPatTupleNode<'t>)
}

struct LetPatDataNode<'t> {
    ty: TypedExpr<'t>,
    pattern: LinkedList::<LetPat<'t>>,
    ast: &'t parser::Expr
}

struct LetPatTupleNode<'t> {
    pattern: LinkedList::<LetPat<'t>>,
    ast: &'t parser::Expr
}

struct LetNode<'t> {
    def_vars: LinkedList<DefVar<'t>>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

struct DefVar<'t> {
    var: LetPat<'t>,
    ty: Option<TypedExpr<'t>>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

struct MatchNode<'t> {
    expr: &'t parser::Expr,
    cases: LinkedList<(MatchPat<'t>, TypedExpr<'t>)>,
    ast: &'t parser::Expr
}

enum MatchPat<'t> {
    MatchPatNone(TypedExpr<'t>),
    MatchPatNum(TypedExpr<'t>),
    MatchPatBool(TypedExpr<'t>),
    MatchPatID(TypedExpr<'t>),
    MatchPatTuple(MatchPatTupleNode<'t>),
    MatchPatData(MatchPatDataNode<'t>)
}

struct MatchPatTupleNode<'t> {
    pattern: LinkedList<MatchPat<'t>>,
    ast: &'t parser::Expr
}

struct MatchPatDataNode<'t> {
    ty: TypedExpr<'t>,
    pattern: LinkedList<MatchPat<'t>>,
    ast: &'t parser::Expr
}

struct ApplyNode<'t> {
    exprs: LinkedList<TypedExpr<'t>>,
    ast: &'t parser::Expr
}

struct TIDNode<'t> {
    id: &'t str,
    ast: &'t parser::Expr
}

enum PrimType<'t> {
    PrimTypeBool(TypeBoolNode<'t>),
    PrimTypeInt(TypeIntNode<'t>),
    PrimTypeList(PrimTypeListNode<'t>),
    PrimTypeTuple(PrimTypeTupleNode<'t>)
}

struct TypeBoolNode<'t> {
    ast: &'t parser::Expr
}

struct TypeIntNode<'t> {
    ast: &'t parser::Expr
}

struct PrimTypeListNode<'t> {
    ty: Box::<PrimType<'t>>,
    ast: &'t parser::Expr
}

struct PrimTypeTupleNode<'t> {
    ty: Vec<PrimType<'t>>,
    ast: &'t parser::Expr
}

struct DataType<'t> {
    name: DataTypeName<'t>,
    members: Vec<DataTypeMem<'t>>,
    ast: &'t parser::Expr
}

struct DataTypeName<'t> {
    id: TIDNode<'t>,
    type_args: Vec<IDNode<'t>>,
    ast: &'t parser::Expr
}

struct DataTypeMem<'t> {
    id: TIDNode<'t>,
    types: Vec<PrimType<'t>>,
    ast: &'t parser::Expr
}

enum Type<'t> {
    TypeBool(TypeBoolNode<'t>),
    TypeInt(TypeIntNode<'t>),
    TypeList(TypeListNode<'t>),
    TypeTuple(TypeTupleNode<'t>),
    TypeFun(TypeFunNode<'t>),
    TypeData(TypeDataNode<'t>)
}

struct TypeListNode<'t> {
    ty: Box::<Type<'t>>,
    ast: &'t parser::Expr
}

struct TypeTupleNode<'t> {
    ty: Vec<Type<'t>>,
    ast: &'t parser::Expr
}

enum Effect {
    IO,
    Pure
}

struct TypeFunNode<'t> {
    effect: Effect,
    args: Vec<Type<'t>>,
    ret: Box<Type<'t>>,
    ast: &'t parser::Expr
}

struct TypeDataNode<'t> {
    id: TIDNode<'t>,
    type_args: Vec<PrimType<'t>>,
    ast: &'t parser::Expr
}

struct Defun<'t> {
    id: IDNode<'t>,
    args: Vec<IDNode<'t>>,
    fun_type: Type<'t>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
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
                    return Err(TypingErr{msg: "error: require data name", ast: expr})
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
            Err(TypingErr{msg: "error", ast: expr})
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
                    return Err(TypingErr{msg: "error: must type identifier (with type arguments)", ast: expr})
                }
            }

            for it in iter {
                let id = expr2id(it)?;
                args.push(id);
            }

            Ok(DataTypeName{id: tid, type_args: args, ast: expr})
        }
        _ => {
            Err(TypingErr{msg: "error: must type identifier (with type arguments)", ast: expr})
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
                        Err(TypingErr{msg: "error: the first character must be captal", ast: expr})
                    }
                }
                _ => {
                    Err(TypingErr{msg: "error", ast: expr})
                }
            }
        }
        _ => {
            Err(TypingErr{msg: "error: must be type identifier", ast: expr})
        }
    }
}

fn expr2id(expr: &parser::Expr) -> Result<IDNode, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            match id.chars().nth(0) {
                Some(c) => {
                    if 'A' <= c && c <= 'Z' {
                        Err(TypingErr{msg: "error: the first character must not be captal", ast: expr})
                    } else {
                        Ok(IDNode{id: id, ast: expr})
                    }
                }
                _ => {
                    Err(TypingErr{msg: "error", ast: expr})
                }
            }
        }
        _ => {
            Err(TypingErr{msg: "error: must be identifier", ast: expr})
        }
    }
}

/// $MEMBER := $TID | ( $TID $PRIM* )
fn expr2data_mem(expr: &parser::Expr) -> Result<DataTypeMem, TypingErr> {
    match expr {
        parser::Expr::ID(_) => {
            // $TID
            let tid = expr2type_id(expr)?;
            Ok(DataTypeMem{id: tid, types: Vec::new(), ast: expr})
        }
        parser::Expr::Apply(exprs) => {
            // ( $TID $PRIM* )
            let mut iter = exprs.iter();
            let tid;

            match iter.next() {
                Some(e) => {
                    tid = expr2type_id(e)?;
                }
                _ => {
                    return Err(TypingErr{msg: "error: must type identifier", ast: expr})
                }
            }

            let mut types = Vec::new();
            for it in iter {
                let pt = expr2prim(it)?;
                types.push(pt);
            }

            Ok(DataTypeMem{id: tid, types: types , ast: expr})
        }
        _ => {
            Err(TypingErr{msg: "error: must be type identifier (with types)", ast: expr})
        }
    }
}

/// $PRIM := Int | Bool | $PRIM_LIST | $PRIM_TUPLE
fn expr2prim(expr: &parser::Expr) -> Result<PrimType, TypingErr> {
    match expr {
        parser::Expr::ID(_) => {
            // Int | Bool
            let tid = expr2type_id(expr)?;

            if tid.id == "Int" {
                return Ok(PrimType::PrimTypeInt(TypeIntNode{ast: expr}));
            } else if tid.id == "Bool" {
                return Ok(PrimType::PrimTypeBool(TypeBoolNode{ast: expr}));
            }

            Err(TypingErr{msg: "error: must be Int, Bool, list, or tuple", ast: expr})
        }
        parser::Expr::List(list) => {
            // $PRIM_LIST := '( $PRIM )
            if list.len() != 1 {
                return Err(TypingErr{msg: "error: require exactly one type as a type argument for list type", ast: expr});
            }

            match list.iter().next() {
                Some(e) => {
                    let ty = Box::new(expr2prim(e)?);
                    Ok(PrimType::PrimTypeList(PrimTypeListNode{ty: ty, ast: e}))
                }
                _ => {
                    Err(TypingErr{msg: "error: require primitive type", ast: expr})
                }
            }
        }
        parser::Expr::Tuple(tuple) => {
            // $PRIM_TUPLE := [ $PRIM+ ]
            if tuple.len() < 1 {
                return Err(TypingErr{msg: "error: require more than or equal to one for tuple type", ast: expr});
            }

            let mut types = Vec::new();
            for it in tuple.iter() {
                let ty = expr2prim(it)?;
                types.push(ty);
            }

            Ok(PrimType::PrimTypeTuple(PrimTypeTupleNode{ty: types, ast: expr}))
        }
        _ => {
            Err(TypingErr{msg: "error: must be primitive type", ast: expr})
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
                    return Err(TypingErr{msg: "error: require function name", ast: expr});
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
                    return Err(TypingErr{msg: "error: require arguments", ast: expr});
                }
            }

            // $TYPE_FUN
            let fun;
            match iter.next() {
                Some(e) => {
                    fun = expr2type_fun(e)?;
                }
                _ => {
                    return Err(TypingErr{msg: "error: require function type", ast: expr});
                }
            }

            // $EXPR
            let body;
            match iter.next() {
                Some(e) => {
                    body = expr2typed_expr(e)?;
                }
                _ => {
                    return Err(TypingErr{msg: "error: require expression", ast: expr});
                }
            }

            Ok(Defun{id: id, args: args, fun_type: fun, expr: body, ast: expr})
        }
        _ => {
            Err(TypingErr{msg: "error", ast: expr})
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
                        return Err(TypingErr{msg: "error: effect must be \"Pure\" or \"IO\"", ast: e.unwrap()});
                    }
                }
                _ => {
                    return Err(TypingErr{msg: "error: invalid effect", ast: expr});
                }
            }

            // ( -> $TYPES $TYPE )
            let e1 = iter.next();
            let args;
            let ret;
            match e1 {
                Some(parser::Expr::Apply(exprs)) => {
                    let mut iter2 = exprs.iter();
                    let e2 = iter.next();
                    match e2 {
                        Some(parser::Expr::ID(arr)) => {
                            if arr != "->" {
                                return Err(TypingErr{msg: "error: must be \"->\"", ast: e2.unwrap()});
                            }
                        }
                        _ => {
                            return Err(TypingErr{msg: "error: require function type", ast: e1.unwrap()});
                        }
                    }

                    // $TYPES := $TYPE | ( $TYPE* )
                    match iter2.next() {
                        Some(t) => {
                            args = expr2types(t)?;
                        }
                        _ => {
                            return Err(TypingErr{msg: "error: require types for arguments", ast: e1.unwrap()});
                        }
                    }

                    // $TYPE
                    match iter2.next() {
                        Some(t) => {
                            ret = expr2type(t)?;
                        }
                        _ => {
                            return Err(TypingErr{msg: "error: require type for return value", ast: e1.unwrap()});
                        }
                    }
                }
                _ => {
                    return Err(TypingErr{msg: "error: require function type", ast: expr});
                }
            }

            Ok(Type::TypeFun(TypeFunNode{effect: effect, args: args, ret: Box::new(ret), ast: expr}))
        }
        _ => {
            Err(TypingErr{msg: "error", ast: expr})
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

/// $TYPE := Int | Bool | $TYPE_LIST | $TYPE_TUPLE | $TYPE_FUN | $TYPE_DATA
fn expr2type(expr: &parser::Expr) -> Result<Type, TypingErr> {
    match expr {
        parser::Expr::ID(id) => {
            // Int | Bool | $TID
            if id == "Int" {
                Ok(Type::TypeInt(TypeIntNode{ast: expr}))
            } else if id == "Bool" {
                Ok(Type::TypeBool(TypeBoolNode{ast: expr}))
            } else {
                let tid = expr2type_id(expr)?;
                Ok(Type::TypeData(TypeDataNode{id: tid, type_args: Vec::new(), ast: expr}))
            }
        }
        parser::Expr::List(list) => {
            // $TYPE_LIST := '( $TYPE )
            if list.len() != 1 {
                return Err(TypingErr{msg: "error: require exactly one type as a type argument for list type", ast: expr});
            }

            match list.iter().next() {
                Some(e) => {
                    let ty = Box::new(expr2type(e)?);
                    Ok(Type::TypeList(TypeListNode{ty: ty, ast: e}))
                }
                _ => {
                    Err(TypingErr{msg: "error: require primitive type", ast: expr})
                }
            }
        }
        parser::Expr::Tuple(tuple) => {
            // $TYPE_TUPLE := [ $TYPE+ ]
            if tuple.len() < 1 {
                return Err(TypingErr{msg: "error: require more than or equal to oen type", ast: expr});
            }

            let mut types = Vec::new();
            for it in tuple {
                types.push(expr2type(it)?);
            }

            Ok(Type::TypeTuple(TypeTupleNode{ty: types, ast: expr}))
        }
        parser::Expr::Apply(exprs) => {
            // ( $TID $PRIM* )
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
                    return Err(TypingErr{msg: "error: require type", ast: expr});
                }
            }

            // $PRIM*
            let mut args = Vec::new();
            for it in iter {
                args.push(expr2prim(it)?);
            }

            Ok(Type::TypeData(TypeDataNode{id: tid, type_args: args, ast: expr}))
        }
        _ => {
            Err(TypingErr{msg: "error: must be type", ast: expr})
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
        parser::Expr::List(_list) => {
            Err(TypingErr{msg: "not yet implemented", ast: expr})
        }
        parser::Expr::Tuple(_tuple) => {
            Err(TypingErr{msg: "not yet implemented", ast: expr})
        }
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();

            match iter.next() {
                Some(parser::Expr::ID(id)) => {
                    if id == "if" {
                        return Ok(expr2if(expr)?);
                    } else {
                        return Err(TypingErr{msg: "not yet implemented", ast: expr});
                    }
                }
                _ => {
                    return Err(TypingErr{msg: "error: require function", ast: expr});
                }
            }
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
            return Err(TypingErr{msg: "error: if expression", ast: expr});
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
                return Err(TypingErr{msg: msg, ast: expr});
            }
        }
    };

    let cond = f(iter.next(), "error: if requires condition")?;
    let then = f(iter.next(), "error: if requires then expression")?;
    let else_expr = f(iter.next(), "error: if requires else expression")?;

    Ok(TypedExpr::IfExpr(Box::new(IfNode{cond_expr: cond, then_expr: then, else_expr: else_expr, ast: expr})))
}