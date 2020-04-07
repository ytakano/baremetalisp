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
    ty: LinkedList<PrimType<'t>>,
    ast: &'t parser::Expr
}

struct DataType<'t> {
    name: DataTypeName<'t>,
    member: DataTypeMem<'t>,
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

struct TypeFunNode<'t> {
    args: LinkedList<Type<'t>>,
    ret: LinkedList<Type<'t>>,
    ast: &'t parser::Expr
}

struct TypeDataNode<'t> {
    id: TIDNode<'t>,
    type_args: LinkedList<PrimType<'t>>,
    ast: &'t parser::Expr
}

struct Defun<'t> {
    id: IDNode<'t>,
    args: LinkedList<IDNode<'t>>,
    fun_type: Type<'t>,
    expr: TypedExpr<'t>,
    ast: &'t parser::Expr
}

/// $DATA := ( data $DATA_NAME $MEMBER+ )
fn expr2data(expr: &parser::Expr) -> Result<DataType,TypingErr> {
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

            // TODO:
            // $MEMBER+

            // Ok(DataType{name: data_name, ast: expr})

            Err(TypingErr{msg: "error", ast: expr})
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

fn expr2type_id(expr: &parser::Expr) -> Result<TIDNode,TypingErr> {
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

fn expr2id(expr: &parser::Expr) -> Result<IDNode,TypingErr> {
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