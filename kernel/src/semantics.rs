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
}

struct TypeListNode<'t> {
    ty: Box::<Type<'t>>,
    ast: &'t parser::Expr
}

struct TypeTupleNode<'t> {
    ty: LinkedList<Type<'t>>,
    ast: &'t parser::Expr
}

enum Effect {
    IO,
    Pure
}

struct TypeFunNode<'t> {
    effect: Effect,
    args: Vec<Type<'t>>,
    ret: Vec<Type<'t>>,
    ast: &'t parser::Expr
}

struct TypeDataNode<'t> {
    id: TIDNode<'t>,
    type_args: LinkedList<PrimType<'t>>,
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

            // TODO:
            // $TYPE_FUN

            Err(TypingErr{msg: "not yet implemented", ast: expr})
        }
        _ => {
            Err(TypingErr{msg: "error", ast: expr})
        }
    }
}

/// $TYPE_FUN := ( $EFFECT ( -> $TYPES $TYPES ) )
fn expr2type_fun(expr: &parser::Expr) -> Result<TypeFunNode, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs) => {
            let mut iter = exprs.iter();

            // $EFFECT := Pure | IO
            let effect;
            match iter.next() {
                Some(e@parser::Expr::ID(eff)) => {
                    if eff == "IO" {
                        effect = Effect::IO;
                    } else if eff == "Pure" {
                        effect = Effect::Pure;
                    } else {
                        return Err(TypingErr{msg: "error: effect must be \"Pure\" or \"IO\"", ast: e});
                    }
                }
                _ => {
                    return Err(TypingErr{msg: "error: invalid effect", ast: expr});
                }
            }

            // ( -> $TYPES $TYPES )
            match iter.next() {
                Some(e1@parser::Expr::Apply(exprs)) => {
                    let mut iter2 = exprs.iter();
                    match iter2.next() {
                        Some(e2@parser::Expr::ID(arr)) => {
                            if arr != "->" {
                                return Err(TypingErr{msg: "error: must be \"->\"", ast: e2});
                            }
                        }
                        _ => {
                            return Err(TypingErr{msg: "error: require function type", ast: e1});
                        }
                    }

                    // $TYPES := $TYPE | ( $TYPE* )
                    let mut args = Vec::new();
                    match iter2.next() {
                        Some(parser::Expr::Apply(types)) => {
                            // TODO:
                            // $TYPES
                            for it in types {

                            }
                        }
                        Some(t) => {
                            // TODO:
                            // $TYPE
                        }
                        _ => {
                            return Err(TypingErr{msg: "error: require types for arguments", ast: e1});
                        }
                    }
                }
                _ => {
                    return Err(TypingErr{msg: "error: require function type", ast: expr});
                }
            }

            Err(TypingErr{msg: "error", ast: expr})
        }
        _ => {
            Err(TypingErr{msg: "error", ast: expr})
        }
    }
}
