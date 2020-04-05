use crate::parser;

use alloc::collections::linked_list::LinkedList;
use alloc::boxed::Box;

enum TypedExpr<'t> {
    IfExpr(Box::<IfNode<'t>>),
    LetExpr(Box::<LetNode<'t>>),
    LitNum(NumNode<'t>),
    LitBool(BoolNode<'t>),
    IDExpr(IDNode<'t>),
    MatchExpr(Box::<MatchNode<'t>>)
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