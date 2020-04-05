use crate::parser;

use alloc::collections::linked_list::LinkedList;
use alloc::boxed::Box;

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
    type_args: LinkedList<IDNode<'t>>,
    ast: &'t parser::Expr
}

struct DataTypeMem<'t> {
    id: TIDNode<'t>,
    types: LinkedList<PrimType<'t>>,
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