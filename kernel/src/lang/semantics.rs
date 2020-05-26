use super::parser;
// use crate::driver;

use alloc::collections::linked_list::LinkedList;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::BTreeSet;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::{ToString, String};

type ID = u64;
type Sbst = BTreeMap<ID, Type>;

#[derive(Debug, Clone)]
enum Type {
    TCon(Tycon),
    TVar(ID)
}

#[derive(Debug, Clone)]
struct Tycon {
    id: String,
    args: Vec<Type>,
}

fn ty_bool() -> Type {
    Type::TCon(Tycon{id: "Bool".to_string(), args: Vec::new()})
}

fn ty_int() -> Type {
    Type::TCon(Tycon{id: "Int".to_string(), args: Vec::new()})
}

fn ty_var(n: ID) -> Type {
    Type::TVar(n)
}

fn ty_tuple(types: Vec<Type>) -> Type {
    Type::TCon(Tycon{id: "Tuple".to_string(), args: types})
}

fn ty_list(ty: Type) -> Type {
    Type::TCon(Tycon{id: "List".to_string(), args: vec!(ty)})
}

fn ty_fun(effect: &Effect, args: Vec<Type>, ret: Type) -> Type {
    let tuple = ty_tuple(args);
    let ty_effect = match effect {
        Effect::Pure => Type::TCon(Tycon{id: "Pure".to_string(), args: Vec::new()}),
        Effect::IO   => Type::TCon(Tycon{id: "IO".to_string(), args: Vec::new()})
    };
    Type::TCon(Tycon{id: "->".to_string(), args: vec!(ty_effect, tuple, ret)})
}

fn ty_fun_gen_effect(n: ID, args: Vec<Type>, ret: Type) -> Type {
    let tuple = ty_tuple(args);
    let ty_effect = ty_var(n);
    Type::TCon(Tycon{id: "->".to_string(), args: vec!(ty_effect, tuple, ret)})
}

#[derive(Debug)]
pub struct FunTypes {
    fun_types: BTreeMap<String, LinkedList<Type>>
}

impl FunTypes {
    fn new() -> FunTypes {
        FunTypes{fun_types: BTreeMap::new()}
    }

    fn insert(&mut self, key: &String, val: Type) {
        match self.fun_types.get_mut(key) {
            Some(list) => {
                list.push_back(val);
            }
            None => {
                let mut list = LinkedList::new();
                list.push_back(val);
                self.fun_types.insert(key.to_string(), list);
            }
        }
    }

    fn contains(&self, key: &String, val: &Type) -> bool {
        match self.fun_types.get(key) {
            Some(list) => {
                for t in list {
                    match unify(&val, &t) {
                        Some(_) => {
                            return true;
                        }
                        None => ()
                    }
                }
                false
            }
            None => false
        }
    }
}

#[derive(Debug)]
struct VarType {
    var_stack: LinkedList<BTreeMap<String, LinkedList<Type>>>
}

impl VarType {
    fn new() -> VarType {
        let mut var_type = VarType{var_stack: LinkedList::new()};
        var_type.push();
        var_type
    }

    fn push(&mut self) {
        self.var_stack.push_back(BTreeMap::new());
    }

    fn pop(&mut self) {
        self.var_stack.pop_back();
    }

    fn insert(&mut self, key: String, val: Type) {
        match self.var_stack.back_mut() {
            Some(m) => {
                match m.get_mut(&key) {
                    Some(v) => {
                        v.push_back(val);
                    }
                    None => {
                        let mut v = LinkedList::new();
                        v.push_back(val);
                        m.insert(key, v);
                    }
                }
            }
            None => {
                panic!("failed to insert");
            }
        }
    }

    fn get(&self, key: &String) -> Option<&Type> {
        for m in self.var_stack.iter().rev() {
            match m.get(key) {
                Some(list) => {
                    return list.back();
                }
                None => ()
            }
        }

        None
    }
}

#[derive(Debug)]
pub struct TypingErr {
    pub msg: String,
    pub pos: parser::Pos
}

impl TypingErr {
    fn new(msg: &str, ast: &parser::Expr) -> TypingErr {
        // TODO: line, column
        TypingErr{msg: msg.to_string(), pos: ast.get_pos()}
    }
}

#[derive(Debug, Clone)]
pub(crate) enum LangExpr<'t> {
    IfExpr(Box::<IfNode<'t>>),
    LetExpr(Box::<LetNode<'t>>),
    LitNum(NumNode<'t>),
    LitBool(BoolNode<'t>),
    IDExpr(IDNode<'t>),
    DataExpr(DataNode<'t>),
    MatchExpr(Box::<MatchNode<'t>>),
    ApplyExpr(Exprs<'t>),
    ListExpr(Exprs<'t>),
    TupleExpr(Exprs<'t>),
}

impl<'t> LangExpr<'t> {
    pub(crate) fn get_ast(&self) -> &'t parser::Expr {
        match self {
            LangExpr::IfExpr(e)    => e.ast,
            LangExpr::LetExpr(e)   => e.ast,
            LangExpr::LitNum(e)    => e.ast,
            LangExpr::LitBool(e)   => e.ast,
            LangExpr::IDExpr(e)    => e.ast,
            LangExpr::DataExpr(e)  => e.ast,
            LangExpr::MatchExpr(e) => e.ast,
            LangExpr::ApplyExpr(e) => e.ast,
            LangExpr::ListExpr(e)  => e.ast,
            LangExpr::TupleExpr(e) => e.ast,
        }
    }

    fn apply_sbst(&mut self, sbst: &Sbst) {
        let app = |opty: &Option<Type>| {
            match opty {
                Some(t) => {
                    Some(t.apply_sbst(sbst))
                }
                None => None
            }
        };

        match self {
            LangExpr::IfExpr(e) => {
                e.cond_expr.apply_sbst(sbst);
                e.then_expr.apply_sbst(sbst);
                e.else_expr.apply_sbst(sbst);
                e.ty = app(&e.ty);
            },
            LangExpr::LetExpr(e) => {
                for dv in e.def_vars.iter_mut() {
                    dv.pattern.apply_sbst(sbst);
                    dv.expr.apply_sbst(sbst);
                    dv.ty = app(&dv.ty);
                }
                e.expr.apply_sbst(sbst);
                e.ty = app(&e.ty);
            },
            LangExpr::LitNum(_) => (),
            LangExpr::LitBool(_) => (),
            LangExpr::IDExpr(e) => {
                e.ty = app(&e.ty);
            },
            LangExpr::DataExpr(e) => {
                for it in e.exprs.iter_mut() {
                    it.apply_sbst(sbst);
                }
                e.ty = app(&e.ty);
            },
            LangExpr::MatchExpr(e) => {
                e.expr.apply_sbst(sbst);
                for cs in e.cases.iter_mut() {
                    cs.pattern.apply_sbst(sbst);
                    cs.expr.apply_sbst(sbst);
                    cs.ty = app(&cs.ty);
                }
            },
            LangExpr::ApplyExpr(e) => {
                for it in e.exprs.iter_mut() {
                    it.apply_sbst(sbst);
                }
                e.ty = app(&e.ty);
            },
            LangExpr::ListExpr(e)  => {
                for it in e.exprs.iter_mut() {
                    it.apply_sbst(sbst);
                }
                e.ty = app(&e.ty);
            },
            LangExpr::TupleExpr(e) => {
                for it in e.exprs.iter_mut() {
                    it.apply_sbst(sbst);
                }
                e.ty = app(&e.ty);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NumNode<'t> {
    pub(crate) num: i64,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
pub(crate) struct BoolNode<'t> {
    pub(crate) val: bool,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
pub(crate) struct IDNode<'t> {
    pub(crate) id: &'t str,
    ast: &'t parser::Expr,
    ty: Option<Type>,
}

#[derive(Debug, Clone)]
pub(crate) struct IfNode<'t> {
    pub(crate) cond_expr: LangExpr<'t>,
    pub(crate) then_expr: LangExpr<'t>,
    pub(crate) else_expr: LangExpr<'t>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct LetNode<'t> {
    pub(crate) def_vars: Vec<DefVar<'t>>,
    pub(crate) expr: LangExpr<'t>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct DefVar<'t> {
    pub(crate) pattern: Pattern<'t>,
    pub(crate) expr: LangExpr<'t>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct MatchNode<'t> {
    pub(crate) expr: LangExpr<'t>,
    pub(crate) cases: Vec<MatchCase<'t>>,
    pub(crate) ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct DataNode<'t> {
    pub(crate) label: TIDNode<'t>,
    pub(crate) exprs: Vec<LangExpr<'t>>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) enum Pattern<'t> {
    PatNum(NumNode<'t>),
    PatBool(BoolNode<'t>),
    PatID(IDNode<'t>),
    PatTuple(PatTupleNode<'t>),
    PatData(PatDataNode<'t>),
    PatNil(PatNilNode<'t>),
}

impl<'t> Pattern<'t> {
    pub(crate) fn get_ast(&self) -> &'t parser::Expr {
        match self {
            Pattern::PatNum(e)   => e.ast,
            Pattern::PatBool(e)  => e.ast,
            Pattern::PatID(e)    => e.ast,
            Pattern::PatTuple(e) => e.ast,
            Pattern::PatData(e)  => e.ast,
            Pattern::PatNil(e)   => e.ast,
        }
    }

    fn apply_sbst(&mut self, sbst: &Sbst) {
        let app = |opty: &Option<Type>| {
            match opty {
                Some(t) => {
                    Some(t.apply_sbst(sbst))
                }
                None => None
            }
        };

        match self {
            Pattern::PatID(e) => {
                e.ty = app(&e.ty);
            }
            Pattern::PatTuple(e) => {
                for pat in e.pattern.iter_mut() {
                    pat.apply_sbst(sbst);
                }
                e.ty = app(&e.ty);
            }
            Pattern::PatData(e) => {
                for pat in e.pattern.iter_mut() {
                    pat.apply_sbst(sbst);
                }
                e.ty = app(&e.ty);
            }
            Pattern::PatNil(e) => {
                e.ty = app(&e.ty);
            }
            _ => (),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PatTupleNode<'t> {
    pub(crate) pattern: Vec<Pattern<'t>>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct PatDataNode<'t> {
    pub(crate) label: TIDNode<'t>,
    pub(crate) pattern: Vec<Pattern<'t>>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct PatNilNode<'t> {
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct MatchCase<'t> {
    pub(crate) pattern: Pattern<'t>,
    pub(crate) expr: LangExpr<'t>,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct Exprs<'t> {
    pub(crate) exprs: Vec<LangExpr<'t>>,
    pub(crate) ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
pub(crate) struct TIDNode<'t> {
    pub(crate) id: &'t str,
    ast: &'t parser::Expr,
    ty: Option<Type>
}

#[derive(Debug, Clone)]
struct TEBoolNode<'t> {
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct TEIntNode<'t> {
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct DataType<'t> {
    name: DataTypeName<'t>,
    members: Vec<DataTypeMem<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct DataTypeName<'t> {
    id: TIDNode<'t>,
    type_args: Vec<IDNode<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct DataTypeMem<'t> {
    id: TIDNode<'t>,
    types: Vec<TypeExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
enum TypeExpr<'t> {
    TEBool(TEBoolNode<'t>),
    TEInt(TEIntNode<'t>),
    TEList(TEListNode<'t>),
    TETuple(TETupleNode<'t>),
    TEFun(TEFunNode<'t>),
    TEData(TEDataNode<'t>),
    TEID(IDNode<'t>)
}

#[derive(Debug, Clone)]
struct TEListNode<'t> {
    ty: Box::<TypeExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct TETupleNode<'t> {
    ty: Vec<TypeExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
enum Effect {
    IO,
    Pure
}

#[derive(Debug, Clone)]
struct TEFunNode<'t> {
    effect: Effect,
    args: Vec<TypeExpr<'t>>,
    ret: Box<TypeExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct TEDataNode<'t> {
    id: TIDNode<'t>,
    type_args: Vec<TypeExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
pub(crate) struct Defun<'t> {
    exported: bool,
    id: IDNode<'t>,
    pub(crate) args: Vec<IDNode<'t>>,
    fun_type: TypeExpr<'t>,
    effect: Effect,
    pub(crate) expr: LangExpr<'t>,
    ast: &'t parser::Expr,
    ty: Option<Type>,
}

trait TApp<'t>: Sized {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<Self, TypingErr>;
}

impl<'t> TApp<'t> for DataType<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<DataType<'t>, TypingErr> {
        let mut mems = Vec::new();
        for m in self.members.iter() {
            mems.push(m.apply(ty)?);
        }

        Ok(DataType{
            name: self.name.clone(),
            members: mems,
            ast: self.ast
        })
    }
}

impl<'t> TApp<'t> for DataTypeMem<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<DataTypeMem<'t>, TypingErr> {
        let mut v = Vec::new();
        for it in self.types.iter() {
            v.push(it.apply(ty)?);
        }

        Ok(DataTypeMem{
            id: self.id.clone(),
            types: v,
            ast: self.ast
        })
    }
}

impl<'t> TApp<'t> for TypeExpr<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TypeExpr<'t>, TypingErr> {
        match self {
            TypeExpr::TEData(data) => {
                Ok(TypeExpr::TEData(data.apply(ty)?))
            }
            TypeExpr::TEList(list) => {
                Ok(TypeExpr::TEList(list.apply(ty)?))
            }
            TypeExpr::TETuple(tuple) => {
                Ok(TypeExpr::TETuple(tuple.apply(ty)?))
            }
            TypeExpr::TEFun(fun) => {
                Ok(TypeExpr::TEFun(fun.apply(ty)?))
            }
            TypeExpr::TEID(id) => {
                match ty.get(id.id) {
                    Some(t) => {
                        Ok(t.clone())
                    }
                    _ => {
                        Ok(TypeExpr::TEID(id.clone()))
                    }
                }
            }
            _ => {
                Ok(self.clone())
            }
        }
    }
}

impl<'t> TApp<'t> for TEListNode<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TEListNode<'t>, TypingErr> {
        Ok(TEListNode{
            ty: Box::new(self.ty.apply(ty)?),
            ast: self.ast
        })
    }
}

impl<'t> TApp<'t> for TETupleNode<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TETupleNode<'t>, TypingErr> {
        let mut v = Vec::new();
        for it in self.ty.iter() {
            v.push(it.apply(ty)?);
        }

        Ok(TETupleNode{ty: v, ast: self.ast})
    }
}

impl<'t> TApp<'t> for TEFunNode<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TEFunNode<'t>, TypingErr> {
        let mut v = Vec::new();
        for it in self.args.iter() {
            v.push(it.apply(ty)?);
        }

        Ok(TEFunNode{
            effect: self.effect.clone(),
            args: v,
            ret: Box::new(self.ret.apply(ty)?),
            ast: self.ast
        })
    }
}

impl<'t> TApp<'t> for TEDataNode<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TEDataNode<'t>, TypingErr> {
        let mut v = Vec::new();
        for it in self.type_args.iter() {
            v.push(it.apply(ty)?);
        }

        Ok(TEDataNode{
            id: self.id.clone(),
            type_args: v,
            ast: self.ast
        })
    }
}

#[derive(Debug)]
pub struct Context<'t> {
    pub(crate) funs: BTreeMap<String, Defun<'t>>,
    data: BTreeMap<&'t str, DataType<'t>>,
    label2data: BTreeMap<&'t str, &'t str>
}

impl<'t> Context<'t> {
    fn new(funs: BTreeMap<String, Defun<'t>>, data: BTreeMap<&'t str, DataType<'t>>) -> Context<'t> {
        Context{funs: funs,
                data: data,
                label2data: BTreeMap::new()}
    }

    fn typing(&mut self) -> Result<(), TypingErr> {
        self.check_data_def()?;
        self.check_label()?;
        self.check_data_rec()?;
        self.check_defun_type()?;
        self.typing_functions()?;
        self.check_defun_type_after_infer()?;

        Ok(())
    }

    fn check_label(&mut self) -> Result<(), TypingErr> {
        for (_, dt) in &self.data {
            for mem in &dt.members {
                if self.label2data.contains_key(mem.id.id) {
                    let msg = format!("{:?} is multiply defined", mem.id.id);
                    return Err(TypingErr{msg: msg, pos: mem.id.ast.get_pos()});
                }

                self.label2data.insert(mem.id.id, dt.name.id.id);
            }
        }

        Ok(())
    }

    fn typing_functions(&mut self) -> Result<(), TypingErr> {
        let mut funs = BTreeMap::new();
        for (_, defun) in self.funs.iter() {
            let defun = defun.clone();
            let defun = self.typing_defun(defun)?;
            funs.insert(defun.id.id.to_string(), defun);
        }

        self.funs = funs;

        Ok(())
    }

    fn typing_defun(&self, mut defun: Defun<'t>) -> Result<Defun<'t>, TypingErr> {
        let mut var_type = VarType::new();
        let mut num_tv = 0;
        let mut args_orig = Vec::new();

        // initialize types of arguments
        for t in &defun.args {
            let tv = ty_var(num_tv);
            var_type.insert(t.id.to_string(), tv.clone());
            args_orig.push(tv);
            num_tv += 1;
        }

        // infer type of the expression
        let sbst = Sbst::new();
        let (ret, sbst) = self.typing_expr(&mut defun.expr, sbst, &mut var_type, &mut num_tv)?;

        let args = args_orig.iter().into_iter().map(|x| x.apply_sbst(&sbst)).collect();

        let fun_type1 = self.to_type(&defun.fun_type, &mut num_tv).unwrap(); // defined type
        let fun_type2 = ty_fun(&defun.effect, args, ret);                    // inferred type

        // check defined function types with inferred type
        let s1;
        match unify(&fun_type1, &fun_type2) {
            None => {
                let msg = format!("error: function type was inferred as {:?} but defined as {:?}", fun_type2, fun_type1);
                return Err(TypingErr{msg: msg, pos: defun.ast.get_pos()});
            }
            Some(s) => {
                s1 = s
            }
        }

        let sbst = compose(&s1, &sbst);

        // update function type
        defun.ty = Some(fun_type1.apply_sbst(&sbst));

        // update types in the expression
        defun.expr.apply_sbst(&sbst);

        // update types of arguments
        for (arg, ty) in defun.args.iter_mut().zip(args_orig.iter()) {
            arg.ty = Some(ty.apply_sbst(&sbst));
        }

        Ok(defun)
    }

    fn typing_expr<'a>(&self, expr: &mut LangExpr<'a>, sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        match expr {
            LangExpr::LitBool(_)   => Ok((ty_bool(), sbst)),
            LangExpr::LitNum(_)    => Ok((ty_int(), sbst)),
            LangExpr::IfExpr(e)    => self.typing_if(e, sbst, var_type, num_tv),
            LangExpr::IDExpr(e)    => self.typing_var(e, sbst, var_type, num_tv),
            LangExpr::LetExpr(e)   => self.typing_let(e, sbst, var_type, num_tv),
            LangExpr::MatchExpr(e) => self.typing_match(e, sbst, var_type, num_tv),
            LangExpr::TupleExpr(e) => self.typing_tuple(e, sbst, var_type, num_tv),
            LangExpr::ListExpr(e)  => self.typing_list(e, sbst, var_type, num_tv),
            LangExpr::ApplyExpr(e) => self.typing_app(e, sbst, var_type, num_tv),
            LangExpr::DataExpr(e)  => self.typing_data(e, sbst, var_type, num_tv),
        }
    }

    fn typing_data<'a>(&self, expr: &mut DataNode<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let data_type;
        let label_types;
        // get type of label and types of label's elements
        match self.get_type_of_label(expr.label.id, num_tv) {
            Ok((t, m)) => {
                data_type = t;
                label_types = m;
            }
            Err(msg) => {
                return Err(TypingErr{msg: msg, pos: expr.ast.get_pos()});
            }
        }

        // check the number of elements
        if label_types.len() != expr.exprs.len() {
            let msg = format!("error: {:?} requires exactly {:?} arguments but actually passed {:?}", expr.label.id, label_types.len(), expr.exprs.len());
            return Err(TypingErr{msg: msg, pos: expr.ast.get_pos()});
        }

        // check types of the elements and arguments
        for (e, ty) in expr.exprs.iter_mut().zip(label_types.iter()) {
            let r = self.typing_expr(e, sbst, var_type, num_tv)?;
            sbst = r.1;
            let t0 = ty.apply_sbst(&sbst);
            let t1 = r.0.apply_sbst(&sbst);
            let s1;

            match unify(&t0, &t1) {
                Some(s) => {
                    s1 = s;
                }
                None => {
                    let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", t0, t1);
                    return Err(TypingErr{msg: msg, pos: e.get_ast().get_pos()});
                }
            }
            sbst = compose(&s1, &sbst);
        }

        expr.ty = Some(data_type.clone());

        Ok((data_type, sbst))
    }

    fn typing_app<'a>(&self, expr: &mut Exprs<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let mut iter = expr.exprs.iter_mut();

        // get function
        let mut e1;
        match iter.next() {
            Some(e) => {
                e1 = e;
            }
            None => {
                return Err(TypingErr::new("error: require function", expr.ast));
            }
        }

        // get function type
        let r = self.typing_expr(&mut e1, sbst, var_type, num_tv)?;
        sbst = r.1;
        let t1 = r.0;

        // get arguments
        let mut v = Vec::new();
        for mut e in iter {
            let r = self.typing_expr(&mut e, sbst, var_type, num_tv)?;
            sbst = r.1;
            v.push(r.0);
        }

        // get return type
        let ret = ty_var(*num_tv);
        *num_tv += 1;

        // get inferred function type
        let fun_ty = ty_fun_gen_effect(*num_tv, v, ret.clone());
        *num_tv += 1;

        match unify(&t1, &fun_ty) {
            Some(s1) => {
                sbst = compose(&s1, &sbst);
            }
            None => {
                let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", fun_ty, t1);
                return Err(TypingErr{msg: msg, pos: expr.ast.get_pos()});
            }
        }

        let t = ret.apply_sbst(&sbst);
        expr.ty = Some(t.clone());

        Ok((t, sbst))
    }

    fn typing_tuple<'a>(&self, expr: &mut Exprs<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let mut v = Vec::new();
        for e in expr.exprs.iter_mut() {
            let (t, s) = self.typing_expr(e, sbst, var_type, num_tv)?;
            sbst = s;
            v.push(t);
        }

        let ty = ty_tuple(v);
        expr.ty = Some(ty.clone());

        Ok((ty, sbst))
    }

    fn typing_list<'a>(&self, expr: &mut Exprs<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let mut ty = None; // type of the first element

        for e in expr.exprs.iter_mut() {
            let (t, s) = self.typing_expr(e, sbst, var_type, num_tv)?;
            sbst = s;
            match &ty {
                None => {
                    ty = Some(t);
                }
                Some(t0) => {
                    let t0 = t0.apply_sbst(&sbst);
                    // check current element's type is same as the first element's type
                    match unify(&t0, &t) {
                        Some(s1) => {
                            sbst = compose(&s1, &sbst);
                        }
                        None => {
                            let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", t0, t);
                            return Err(TypingErr{msg: msg, pos: e.get_ast().get_pos()});
                        }
                    }
                }
            }
        }

        match ty {
            Some(t0) => {
                let tyls = ty_list(t0.apply_sbst(&sbst));
                expr.ty = Some(tyls.clone());
                Ok((tyls, sbst))
            }
            None => {
                // Nil
                let t = ty_var(*num_tv);
                let tyls = ty_list(t);
                *num_tv += 1;
                expr.ty = Some(tyls.clone());
                Ok((tyls, sbst))
            }
        }
    }

    fn typing_match<'a>(&self, expr: &mut MatchNode<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        // for (match e_0 (c_1 e_1) (c_2 e_2) ... (c_n e_n))

        // get e_0's type
        let r = self.typing_expr(&mut expr.expr, sbst, var_type, num_tv)?;
        let mut type_head = r.0;
        sbst = r.1;

        let mut e_ty = None;
        for cs in expr.cases.iter_mut() {
            var_type.push();

            // get c_n's type
            let (pat_ty, s) = self.typing_pat(&mut cs.pattern, sbst, var_type, num_tv)?;
            sbst = s;

            // check types of e_0 and c_n are same
            let s1;
            type_head = type_head.apply_sbst(&sbst);
            match unify(&type_head, &pat_ty) {
                Some(s) => {
                    s1 = s;
                }
                None => {
                    let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", type_head, pat_ty);
                    return Err(TypingErr{msg: msg, pos: cs.pattern.get_ast().get_pos()});
                }
            }

            sbst = compose(&s1, &sbst);

            // get e_n's type
            let (ty, s) = self.typing_expr(&mut cs.expr, sbst, var_type, num_tv)?;
            sbst = s;

            // check types of e_{n-1} and e_n are same
            match e_ty {
                Some(t_prev) => {
                    let s1;
                    match unify(&t_prev, &ty) {
                        Some(s) => {
                            s1 = s;
                        }
                        None => {
                            let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", t_prev, ty);
                            return Err(TypingErr{msg: msg, pos: cs.ast.get_pos()});
                        }
                    }

                    sbst = compose(&s1, &sbst);
                }
                None => ()
            }

            let ty = ty.apply_sbst(&sbst);
            cs.ty = Some(ty.clone());
            e_ty = Some(ty);

            var_type.pop();
        }

        expr.ty = e_ty.clone();

        Ok((e_ty.unwrap(), sbst))
    }

    fn typing_var<'a>(&self, expr: &mut IDNode<'a>, sbst: Sbst, var_type: &VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let ty;
        match var_type.get(&expr.id.to_string()) {
            Some(t) => {
                ty = t.apply_sbst(&sbst);
            }
            None => {
                // look up function
                match self.funs.get(&expr.id.to_string()) {
                    Some(defun) => {
                        ty = self.to_type(&defun.fun_type, num_tv).unwrap();
                    }
                    None => {
                        let msg = format!("error: {:?} is not defined", expr.id);
                        return Err(TypingErr{msg: msg, pos: expr.ast.get_pos()});
                    }
                }
            }
        }

        expr.ty = Some(ty.clone());

        Ok((ty, sbst))
    }

    fn typing_if<'a>(&self, expr: &mut IfNode<'a>, sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        // condition
        let (ty_cond, sbst) = self.typing_expr(&mut expr.cond_expr, sbst, var_type, num_tv)?;

        // check the type of condition is Bool
        let s1;
        match unify(&ty_bool(), &ty_cond) {
            Some(s) => {
                s1 = s;
            }
            None => {
                let msg = format!("error: condition of if expression must be Bool, but found {:?}", ty_cond);
                return Err(TypingErr{msg: msg, pos: expr.cond_expr.get_ast().get_pos()});
            }
        }

        let sbst = compose(&s1, &sbst);

        // then and else expressions
        let (ty_then, sbst) = self.typing_expr(&mut expr.then_expr, sbst, var_type, num_tv)?;
        let (ty_else, sbst) = self.typing_expr(&mut expr.else_expr, sbst, var_type, num_tv)?;

        // check types of expressions are same
        let s1;
        match unify(&ty_then, &ty_else) {
            Some(s) => {
                s1 = s;
            }
            None => {
                let msg = format!("error: when (if c e1 e2), the types of e1 and e2 must be same\n  e1: {:?}\n  e2: {:?}", ty_then, ty_else);
                return Err(TypingErr{msg: msg, pos: expr.else_expr.get_ast().get_pos()});
            }
        }

        let sbst = compose(&s1, &sbst);
        let ty = ty_then.apply_sbst(&sbst);

        expr.ty = Some(ty.clone());

        Ok((ty, sbst))
    }

    fn typing_let<'a>(&self, expr: &mut LetNode<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        var_type.push();

        for dv in expr.def_vars.iter_mut() {
            let (t1, s) = self.typing_expr(&mut dv.expr, sbst, var_type, num_tv)?;
            let (t2, s) = self.typing_pat(&mut dv.pattern, s, var_type, num_tv)?;
            sbst = s;

            let s1;
            match unify(&t1, &t2) {
                Some(s) => {
                    s1 = s;
                }
                None => {
                    let msg = format!("error: mismatched type\n   left: {:?}\n  right: {:?}", t2, t1);
                    return Err(TypingErr{msg: msg, pos: dv.ast.get_pos()});
                }
            }
            sbst = compose(&s1, &sbst);
            dv.ty = Some(t1.apply_sbst(&sbst));
        }

        let r = self.typing_expr(&mut expr.expr, sbst, var_type, num_tv)?;

        var_type.pop();
        expr.ty = Some(r.0.clone());

        Ok(r)
    }

    fn typing_pat<'a>(&self, expr: &mut Pattern<'a>, sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        match expr {
            Pattern::PatBool(_)  => Ok((ty_bool(), sbst)),
            Pattern::PatNum(_)   => Ok((ty_int(), sbst)),
            Pattern::PatID(e)    => self.typing_pat_id(e, sbst, var_type, num_tv),
            Pattern::PatData(e)  => self.typing_pat_data(e, sbst, var_type, num_tv),
            Pattern::PatTuple(e) => self.typing_pat_tuple(e, sbst, var_type, num_tv),
            Pattern::PatNil(e)   => self.typing_pat_nil(e, sbst, num_tv),
        }
    }

    fn typing_pat_tuple<'a>(&self, expr: &mut PatTupleNode<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let mut v = Vec::new();
        for pat in expr.pattern.iter_mut() {
            let (t, s) = self.typing_pat(pat, sbst, var_type, num_tv)?;
            sbst = s;
            v.push(t);
        }

        let ty = ty_tuple(v);
        expr.ty = Some(ty.clone());

        Ok((ty, sbst))
    }

    fn typing_pat_id<'a>(&self, expr: &mut IDNode<'a>, sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        // generate new type variable (internal representation)
        let ty = ty_var(*num_tv);
        *num_tv += 1;
        expr.ty = Some(ty.clone());

        if expr.id != "_" {
            var_type.insert(expr.id.to_string(), ty.clone());
        }

        Ok((ty, sbst))
    }

    fn typing_pat_data<'a>(&self, expr: &mut PatDataNode<'a>, mut sbst: Sbst, var_type: &mut VarType, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        // get the type of label and the types of label's elements
        let data_type;   // type of label
        let label_types; // types of label's elements
        match self.get_type_of_label(expr.label.id, num_tv) {
            Ok((t, m)) => {
                data_type = t;
                label_types = m;
            }
            Err(msg) => {
                return Err(TypingErr{msg: msg, pos: expr.label.ast.get_pos()});
            }
        }

        // check the number of arguments
        if label_types.len() != expr.pattern.len() {
            let msg = format!("error: {:?} requires exactly {:?} arguments but actually passed {:?}", expr.label.id, label_types.len(), expr.pattern.len());
            return Err(TypingErr{msg: msg, pos: expr.label.ast.get_pos()});
        }

        // check type of each element
        for (pat, lt) in expr.pattern.iter_mut().zip(label_types.iter()) {
            let r = self.typing_pat(pat, sbst, var_type, num_tv)?;
            sbst = r.1;
            let lt = lt.apply_sbst(&sbst);
            let s1;
            match unify(&lt, &r.0) {
                Some(s) => {
                    s1 = s;
                }
                None => {
                    let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", lt, r.0);
                    return Err(TypingErr{msg: msg, pos: pat.get_ast().get_pos()});
                }
            }
            sbst = compose(&s1, &sbst);
        }

        let ty = data_type.apply_sbst(&sbst);
        expr.ty = Some(ty.clone());

        Ok((ty, sbst))
    }

    fn typing_pat_nil<'a>(&self, expr: &mut PatNilNode<'a>, sbst: Sbst, num_tv: &mut ID) -> Result<(Type, Sbst), TypingErr> {
        let tv = ty_var(*num_tv);
        *num_tv += 1;
        let ty = ty_list(tv);
        expr.ty = Some(ty.clone());
        Ok((ty, sbst))
    }

    fn to_type(&self, expr: &TypeExpr<'t>, num_tv: &mut ID) -> Result<Type, String> {
        let mut tv2type = BTreeMap::new();
        self.get_tv2type_from_type_expr(expr, num_tv, &mut tv2type);
        self.apply_tv2type_to_type_expr(expr, &tv2type)
    }

    fn get_tv2type_from_type_expr(&self, expr: &TypeExpr<'t>, num_tv: &mut ID, tv2type: &mut BTreeMap<&'t str, Type>) {
        match expr {
            TypeExpr::TEList(e) => {
                self.get_tv2type_from_type_expr(&e.ty, num_tv, tv2type);
            }
            TypeExpr::TETuple(e) => {
                for it in &e.ty {
                    self.get_tv2type_from_type_expr(it, num_tv, tv2type);
                }
            }
            TypeExpr::TEFun(e) => {
                for it in &e.args {
                    self.get_tv2type_from_type_expr(it, num_tv, tv2type);
                }
                self.get_tv2type_from_type_expr(&e.ret, num_tv, tv2type);
            }
            TypeExpr::TEData(e) => {
                for it in &e.type_args {
                    self.get_tv2type_from_type_expr(it, num_tv, tv2type);
                }
            }
            TypeExpr::TEID(e) => {
                tv2type.insert(e.id, ty_var(*num_tv));
                *num_tv += 1;
            }
            _ => ()
        }
    }

    /// If
    /// ```
    /// (data (Tree t)
    ///   (Node (Tree t) (Tree t))
    ///   Leaf)
    /// ```
    /// then get_type_of_label("Node", 2)
    /// returns Ok((Tree (TVar 2)), vec!((Tree (TVar 2)), ((Tree (TVar 2))))
    fn get_type_of_label(&self, label: &'t str, num_tv: &mut ID) -> Result<(Type, Vec<Type>), String> {
        // find the name of data of the label
        let data_name;
        match self.label2data.get(label) {
            Some(n) => {
                data_name = n;
            }
            None => {
                match label {
                    // built-in data type
                    // (data (List a)
                    //     (Cons a (List a))
                    //     Nil)
                    "Cons" => {
                        let tv = ty_var(*num_tv);
                        let ty = ty_list(tv.clone());
                        *num_tv += 1;
                        return Ok((ty.clone(), vec!(tv, ty)));
                    }
                    "Nil" => {
                        let tv = ty_var(*num_tv);
                        let ty = ty_list(tv.clone());
                        *num_tv += 1;
                        return Ok((ty.clone(), vec!()));
                    }
                    _ => {
                        let msg = format!("error: {:?} is not defined", label);
                        return Err(msg);
                    }
                }
            }
        }

        // find corresponding data
        let data_node;
        match self.data.get(data_name) {
            Some(n) => {
                data_node = n;
            }
            None => {
                let msg = format!("error: could not find data of label {:?}", label);
                return Err(msg);
            }
        }

        // get the type of the data
        let mut types = Vec::new();
        for i in 0..data_node.name.type_args.len() {
            types.push(ty_var(i as ID + *num_tv));
            *num_tv += 1;
        }

        // generate a map from type variable to type
        let mut tv2type = BTreeMap::new();
        for (k, v) in data_node.name.type_args.iter().zip(types.iter()) {
            tv2type.insert(k.id, v.clone());
        }

        // find corresponding member
        let mut mem = None;
        for m in &data_node.members {
            if m.id.id == label {
                mem = Some(m);
                break;
            }
        }

        // return type of label and label's type
        match mem {
            Some(mem) => {
                let mut label_types = Vec::new();
                for t in &mem.types {
                    label_types.push(self.apply_tv2type_to_type_expr(t, &tv2type)?);
                }

                // the type of the data
                let data_type = Type::TCon(Tycon{id: data_name.to_string(), args: types});

                Ok((data_type, label_types))
            }
            None => {
                let msg = format!("error: could not find label {:?}", label);
                Err(msg)
            }
        }
    }

    /// If
    /// ```
    /// (data (Tree t)
    ///   (Node (Tree t) (Tree t))
    ///   Leaf)
    /// ```
    /// and tv2type = {t: Int} then
    /// apply_tv2type_to_type_expr((Tree t), tv2type) returns (Tree Int)
    fn apply_tv2type_to_type_expr(&self, type_expr: &TypeExpr<'t>, tv2type: &BTreeMap<&'t str, Type>) -> Result<Type, String> {
        match type_expr {
            TypeExpr::TEBool(_) => Ok(ty_bool()),
            TypeExpr::TEInt(_)  => Ok(ty_int()),
            TypeExpr::TEList(list) => {
                let t = self.apply_tv2type_to_type_expr(&list.ty, tv2type)?;
                Ok(ty_list(t))
            }
            TypeExpr::TETuple(tuple) => {
                let mut v = Vec::new();
                for t in &tuple.ty {
                    v.push(self.apply_tv2type_to_type_expr(t, tv2type)?);
                }
                Ok(ty_tuple(v))
            }
            TypeExpr::TEFun(fun) => {
                let mut args = Vec::new();
                for a in &fun.args {
                    args.push(self.apply_tv2type_to_type_expr(a, tv2type)?);
                }
                let r = self.apply_tv2type_to_type_expr(&fun.ret, tv2type)?;
                Ok(ty_fun(&fun.effect, args, r))
            }
            TypeExpr::TEData(data) => {
                let mut v = Vec::new();
                for t in &data.type_args {
                    v.push(self.apply_tv2type_to_type_expr(t, tv2type)?);
                }

                Ok(Type::TCon(Tycon{id: data.id.id.to_string(), args: v}))
            }
            TypeExpr::TEID(id) => {
                match tv2type.get(id.id) {
                    Some(t) => {
                        Ok(t.clone())
                    }
                    None => {
                        let msg = format!("type variable {:?} is undefined", id.id);
                        Err(msg)
                    }
                }
            }
        }
    }

    fn check_data_def(&self) -> Result<(), TypingErr> {
        for (_, d) in self.data.iter() {
            self.check_data_def_data(d)?;
        }

        Ok(())
    }

    fn check_data_def_data(&self, data: &DataType<'t>) -> Result<(), TypingErr> {
        let mut args = BTreeSet::new();
        for arg in data.name.type_args.iter() {
            if args.contains(arg.id) {
                let msg = format!("error: {:?} is multiply used", arg.id);
                return Err(TypingErr{msg: msg, pos: arg.ast.get_pos()});
            }

            args.insert(arg.id);
        }

        for mem in data.members.iter() {
            self.check_data_def_mem(mem, &args)?;
        }

        Ok(())
    }

    fn check_data_def_mem(&self, mem: &DataTypeMem<'t>, args: &BTreeSet<&str>) -> Result<(), TypingErr> {
        for it in mem.types.iter() {
            self.check_def_type(it, Some(args))?
        }

        Ok(())
    }

    fn check_def_type(&self, ty: &TypeExpr<'t>, args: Option<&BTreeSet<&str>>) -> Result<(), TypingErr> {
        match ty {
            TypeExpr::TEID(id) => {
                match args {
                    Some(m) => {
                        if !m.contains(id.id) {
                            let msg = format!("error: {:?} is undefined", id.id);
                            return Err(TypingErr{msg: msg, pos: id.ast.get_pos()})
                        }
                    }
                    None => ()
                }
            }
            TypeExpr::TEList(list) => {
                self.check_def_type(&list.ty, args)?;
            }
            TypeExpr::TETuple(tuple) => {
                for it in tuple.ty.iter() {
                    self.check_def_type(it, args)?;
                }
            }
            TypeExpr::TEData(data) => {
                match self.data.get(data.id.id) {
                    Some(dt) => {
                        if dt.name.type_args.len() != data.type_args.len() {
                            let msg = format!("error: {:?} takes {:?} type arguments but actually passed {:?}",
                                              data.id.id, dt.name.type_args.len(), data.type_args.len());
                            return Err(TypingErr{msg: msg, pos: data.id.ast.get_pos()});
                        }
                    }
                    None => {
                        let msg = format!("error: {:?} is unkown type", data.id.id);
                        return Err(TypingErr{msg: msg, pos: data.id.ast.get_pos()});
                    }
                }

                for it in data.type_args.iter() {
                    self.check_def_type(it, args)?;
                }
            }
            TypeExpr::TEFun(fun) => {
                for it in fun.args.iter() {
                    self.check_def_type(it, args)?
                }

                self.check_def_type(&fun.ret, args)?
            }
            _ => {}
        }

        Ok(())
    }

    /// check data definition is not infinite recursive
    fn check_data_rec(&self) -> Result<(), TypingErr> {
        let mut checked = LinkedList::new();
        for (_, d) in self.data.iter() {
            let mut visited = BTreeSet::new();
            let mut inst = LinkedList::new();
            inst.push_back(d.ast);
            if self.check_data_rec_data(d, &mut visited, &mut checked, &mut inst)? {
                let msg = format!("error: {:?}'s definition is infinitely recursive", d.name.id.id);
                return Err(TypingErr{msg: msg, pos: d.name.id.ast.get_pos()});
            }
            checked.push_back(d.clone());
        }

        Ok(())
    }

    /// Ok(true) if the type is inifinite recursive
    /// Ok(false) if the type is not recursive or limited recursive
    ///
    /// infinite recursive data
    /// ```
    /// (data Num (Succ Num))
    /// ```
    ///
    /// limited recursive date
    /// ```
    /// (data (Tree t)
    ///   (Node (Tree t) (Tree t))
    ///   Leaf)
    ///
    /// (data Num
    ///   (Succ Num)
    ///   Zero)
    /// ```
    fn check_data_rec_data(&self,
                           data: &DataType<'t>,
                           visited: &mut BTreeSet<&'t str>,
                           checked: &mut LinkedList<DataType<'t>>,
                           inst: &mut LinkedList<&'t parser::Expr>) -> Result<bool, TypingErr> {
        if visited.contains(data.name.id.id) {
            return Ok(true);
        }

        let mut ret = true;

        visited.insert(data.name.id.id);
        for mem in data.members.iter() {
            inst.push_back(mem.ast);
            let result = self.check_data_rec_mem(mem, visited, checked, inst)?;
            ret = result && ret;
            inst.pop_back();
        }

        Ok(ret)
    }

    fn check_data_rec_mem(&self,
                          mem: &DataTypeMem<'t>,
                          visited: &mut BTreeSet<&'t str>,
                          checked: &mut LinkedList<DataType<'t>>,
                          inst: &mut LinkedList<&'t parser::Expr>) -> Result<bool, TypingErr> {
        let mut ret = false;

        for ty in mem.types.iter() {
            if self.check_data_rec_ty(ty, visited, checked, inst)? {
                ret = true;
            }
        }

        Ok(ret)
    }

    fn check_data_rec_ty(&self,
                         ty: &TypeExpr<'t>,
                         visited: &mut BTreeSet<&'t str>,
                         checked: &mut LinkedList<DataType<'t>>,
                         inst: &mut LinkedList<&'t parser::Expr>) -> Result<bool, TypingErr> {
        match ty {
            TypeExpr::TEList(_list) => {
                Ok(false)
            }
            TypeExpr::TETuple(tuple) => {
                let mut ret = false;

                inst.push_back(tuple.ast);
                for it in tuple.ty.iter() {
                    if self.check_data_rec_ty(it, visited, checked, inst)? {
                        ret = true;
                    }
                }
                inst.pop_back();

                Ok(ret)
            }
            TypeExpr::TEData(data) => {
                let dt = self.type_data_node2data_type(data)?;
                inst.push_back(data.ast);
                let ret = self.check_data_rec_data(&dt, visited, checked, inst);
                inst.pop_back();
                ret
            }
            TypeExpr::TEFun(_fun) => {
                Ok(false)
            }
            _ => {
                Ok(false)
            }
        }
    }

    fn type_data_node2data_type(&self, data: &TEDataNode<'t>) -> Result<DataType<'t>, TypingErr> {
        let dt;
        match self.data.get(data.id.id) {
            Some(t) => {
                dt = t;
            }
            None => {
                return Err(TypingErr::new("error: no such type", data.id.ast));
            }
        }

        if data.type_args.len() != dt.name.type_args.len() {
            let msg = format!("error: {:?} takes {:?} type arguments but actually passed {:?}",
                              data.id.id, dt.name.type_args.len(), data.type_args.len());
            return Err(TypingErr{msg: msg, pos: data.ast.get_pos()});
        }

        let mut map = BTreeMap::new();
        for (k, v) in dt.name.type_args.iter().zip(data.type_args.iter()) {
            map.insert(k.id, v.clone());
        }

        dt.apply(&map)
    }

    fn check_defun_type(&self) -> Result<(), TypingErr> {
        for (_, fun) in self.funs.iter() {
            let set;
            let m = if fun.exported {
                set = BTreeSet::new();
                Some(&set)
            } else {
                None
            };
            self.check_def_type(&fun.fun_type, m)?;
        }

        Ok(())
    }

    fn check_defun_type_after_infer(&mut self) -> Result<(), TypingErr> {
        let mut m = FunTypes::new();
        for (_, fun) in self.funs.iter() {
            self.check_type_infer(fun, &mut m)?;
        }

        Ok(())
    }

    /// check type inference has been correctly done?
    ///
    /// If an inferred type of defun has no type variables,
    /// then the types of the expression must not contain type variables.
    ///
    /// If an effect of a function is Pure,
    /// then the expression must not contain IO function.
    fn check_type_infer(&self, defun: &Defun<'t>, fun_types: &mut FunTypes) -> Result<(), TypingErr> {
        // check effect
        check_type_has_io(&defun.ty, defun.ast, &Sbst::new(), &defun.effect)?;

        // if function type contains type variables, just return Ok
        match &defun.ty {
            Some(t) => {
                if has_tvar(t) {
                    return Ok(());
                }
            }
            None => {
                return Err(TypingErr::new("error: function type has not inferred yet", defun.ast));
            }
        }

        // get arguments
        let mut vars = VarType::new();
        vars.push();
        for arg in &defun.args {
            match &arg.ty {
                Some(t) => {
                    vars.insert(arg.id.to_string(), t.clone());
                }
                None => {
                    return Err(TypingErr::new("error: argument type has not inferred yet", arg.ast));
                }
            }
        }

        self.check_expr_type(&defun.expr, fun_types, &mut vars, &Sbst::new(), &defun.effect, true)
    }

    fn check_expr_type<'a>(&self, expr: &LangExpr<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        match expr {
            LangExpr::IDExpr(e)    => self.check_id_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::IfExpr(e)    => self.check_if_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::LetExpr(e)   => self.check_let_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::MatchExpr(e) => self.check_match_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::ApplyExpr(e) => self.check_exprs_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::ListExpr(e)  => self.check_exprs_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::TupleExpr(e) => self.check_exprs_type(&e, fun_types, vars, sbst, effect, chk_rec),
            LangExpr::DataExpr(e)  => self.check_data_type(&e, fun_types, vars, sbst, effect, chk_rec),
            _ => Ok(())
        }
    }

    fn check_data_type<'a>(&self, expr: &DataNode<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        check_type_has_no_tvars(&expr.ty, expr.ast, sbst)?;
        check_type_has_io(&expr.ty, expr.ast, sbst, effect)?;

        for e in &expr.exprs {
            self.check_expr_type(&e, fun_types, vars, sbst, effect, chk_rec)?;
        }
        Ok(())
    }

    fn check_exprs_type<'a>(&self, expr: &Exprs<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        check_type_has_no_tvars(&expr.ty, expr.ast, sbst)?;
        check_type_has_io(&expr.ty, expr.ast, sbst, effect)?;

        for e in &expr.exprs {
            self.check_expr_type(&e, fun_types, vars, sbst, effect, chk_rec)?;
        }
        Ok(())
    }

    fn check_match_type<'a>(&self, expr: &MatchNode<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        check_type_has_no_tvars(&expr.ty, expr.ast, sbst)?;
        check_type_has_io(&expr.ty, expr.ast, sbst, effect)?;

        self.check_expr_type(&expr.expr, fun_types, vars, sbst, effect, chk_rec)?;

        for c in &expr.cases {
            vars.push();
            self.check_pat_type(&c.pattern, vars, sbst, effect)?;
            self.check_expr_type(&c.expr, fun_types, vars, sbst, effect, chk_rec)?;
            vars.pop();
        }

        Ok(())
    }

    fn check_id_type<'a>(&self, expr: &IDNode<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        check_type_has_no_tvars(&expr.ty, expr.ast, sbst)?;
        check_type_has_io(&expr.ty, expr.ast, sbst, effect)?;
        match vars.get(&expr.id.to_string()) {
            Some(_) => (),
            None => {
                match self.funs.get(&expr.id.to_string()) {
                    Some(defun) => {
                        if !chk_rec && !defun.exported {
                            let msg = format!("error: {:?} is not defined", expr.id);
                            return Err(TypingErr{msg: msg, pos: expr.ast.get_pos()});
                        }

                        self.check_defun_type_recur(&expr.ty.as_ref().unwrap(), defun, fun_types, chk_rec)?;
                    }
                    None => {
                        let msg = format!("error: {:?} is not defined", expr.id);
                        return Err(TypingErr{msg: msg, pos: expr.ast.get_pos()});
                    }
                }
                ()
            }
        }
        Ok(())
    }

    fn check_defun_type_recur(&self, call_ty: &Type, defun: &Defun<'t>, fun_types: &mut FunTypes, chk_rec: bool) -> Result<(), TypingErr> {
        let defun_ty;

        // check only functions whose type has type variables
        match &defun.ty {
            Some(t) => {
                if !has_tvar(t) {
                    return Ok(());
                }
                defun_ty = t;
            }
            None => {
                return Err(TypingErr::new("error: function type has not inferred yet", defun.ast));
            }
        }

        // already checked?
        let id = defun.id.id.to_string();
        if fun_types.contains(&id, &call_ty) {
            return Ok(())
        }

        fun_types.insert(&id, call_ty.clone());

        // check type with caller side
        let sbst;
        match unify(call_ty, &defun_ty) {
            Some(s) => {
                sbst = s;
            }
            None => {
                let msg = format!("error: mismatched type\n  expected: {:?}\n    actual: {:?}", call_ty, defun_ty);
                return Err(TypingErr{msg: msg, pos: defun.ast.get_pos()});
            }
        }

        // check effect
        check_type_has_io(&defun.ty, defun.ast, &sbst, &defun.effect)?;

        // get arguments
        let mut vars = VarType::new();
        vars.push();
        for arg in &defun.args {
            match &arg.ty {
                Some(t) => {
                    vars.insert(arg.id.to_string(), t.apply_sbst(&sbst));
                }
                None => {
                    return Err(TypingErr::new("error: argument type has not inferred yet", arg.ast));
                }
            }
        }

        // check function type recursively
        self.check_expr_type(&defun.expr, fun_types, &mut vars, &sbst, &defun.effect, chk_rec)
    }

    fn check_if_type<'a>(&self, expr: &IfNode<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        check_type_has_no_tvars(&expr.ty, expr.ast, sbst)?;
        check_type_has_io(&expr.ty, expr.ast, sbst, effect)?;
        self.check_expr_type(&expr.cond_expr, fun_types, vars, sbst, effect, chk_rec)?;
        self.check_expr_type(&expr.then_expr, fun_types, vars, sbst, effect, chk_rec)?;
        self.check_expr_type(&expr.else_expr, fun_types, vars, sbst, effect, chk_rec)?;
        Ok(())
    }

    fn check_let_type<'a>(&self, expr: &LetNode<'a>, fun_types: &mut FunTypes, vars: &mut VarType, sbst: &Sbst, effect: &Effect, chk_rec: bool) -> Result<(), TypingErr> {
        check_type_has_no_tvars(&expr.ty, expr.ast, sbst)?;
        check_type_has_io(&expr.ty, expr.ast, sbst, effect)?;

        vars.push();

        for def in &expr.def_vars {
            self.check_expr_type(&def.expr, fun_types, vars, sbst, effect, chk_rec)?;
            self.check_pat_type(&def.pattern, vars, sbst, effect)?;
        }

        self.check_expr_type(&expr.expr, fun_types, vars, sbst, effect, chk_rec)?;
        vars.pop();

        Ok(())
    }

    fn check_pat_type<'a>(&self, expr: &Pattern<'a>, vars: &mut VarType, sbst: &Sbst, effect: &Effect) -> Result<(), TypingErr> {
        match expr {
            Pattern::PatID(e) => {
                check_type_has_no_tvars(&e.ty, e.ast, sbst)?;
                check_type_has_io(&e.ty, e.ast, sbst, effect)?;
                vars.insert(e.id.to_string(), e.ty.as_ref().unwrap().clone());
                Ok(())
            }
            Pattern::PatTuple(e) => {
                check_type_has_no_tvars(&e.ty, e.ast, sbst)?;
                check_type_has_io(&e.ty, e.ast, sbst, effect)?;
                for it in &e.pattern {
                    self.check_pat_type(it, vars, sbst, effect)?;
                }
                Ok(())
            }
            Pattern::PatData(e) => {
                check_type_has_no_tvars(&e.ty, e.ast, sbst)?;
                check_type_has_io(&e.ty, e.ast, sbst, effect)?;
                for it in &e.pattern {
                    self.check_pat_type(it, vars, sbst, effect)?;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }
}

fn check_type_has_no_tvars<'t>(ty: &Option<Type>, ast: &'t parser::Expr, sbst: &Sbst) -> Result<(), TypingErr> {
    match ty {
        Some(t) => {
            if has_tvar(&t.apply_sbst(sbst)) {
                let msg = format!("error: inferred type still contains type variables\n  type: {:?}", t);
                return Err(TypingErr{msg: msg, pos: ast.get_pos()});
            }
        }
        None => {
            return Err(TypingErr::new("error: type has not inferred yet", ast));
        }
    }
    Ok(())
}

fn check_type_has_io<'t>(ty: &Option<Type>, ast: &'t parser::Expr, sbst: &Sbst, effect: &Effect) -> Result<(), TypingErr> {
    match ty {
        Some(t) => {
            match effect {
                Effect::IO => {
                    if has_io(&t.apply_sbst(sbst)) {
                        let msg = format!("error: Pure function contains an IO function\n type: {:?}", t);
                        return Err(TypingErr{msg: msg, pos: ast.get_pos()});
                    }
                }
                _ => ()
            }
        }
        None => {
            return Err(TypingErr::new("error: type has not inferred yet", ast));
        }
    }

    Ok(())
}

/// Does type contain IO?
fn has_io(ty: &Type) -> bool {
    match ty {
        Type::TCon(t) => {
            if t.id == "IO" {
                return true;
            }

            for arg in &t.args {
                if has_io(arg) {
                    return true;
                }
            }

            false
        }
        Type::TVar(_) => false
    }
}

/// Does type contain type variables?
fn has_tvar(ty: &Type) -> bool {
    match ty {
        Type::TCon(t) => {
            for arg in &t.args {
                if has_tvar(arg) {
                    return true;
                }
            }
            false
        }
        Type::TVar(_) => true
    }
}

pub(crate) fn typing_expr<'a, 'b>(expr: &'a parser::Expr, ctx: &Context<'b>) -> Result<LangExpr<'a>, TypingErr> {
    let mut expr = expr2typed_expr(expr)?;
    let mut num_tv = 0;
    let (_, sbst) = ctx.typing_expr(&mut expr, Sbst::new(), &mut VarType::new(), &mut num_tv)?;

    expr.apply_sbst(&sbst);

//    let msg = format!("sbst = {:#?}\nexpr = {:#?}\n", sbst, expr);
//    driver::uart::puts(&msg);

    // check call only exported functions
    ctx.check_expr_type(&expr, &mut FunTypes::new(), &mut VarType::new(), &Sbst::new(), &Effect::IO, false)?;

    Ok(expr)
}

pub fn exprs2context(exprs: &LinkedList<parser::Expr>) -> Result<Context, TypingErr> {
    let mut funs = BTreeMap::new();
    let mut data = BTreeMap::new();
    let msg = "error: top expression must be data, defun, or export";

    for e in exprs {
        match e {
            parser::Expr::Apply(es, _) => {
                let mut iter = es.iter();

                match iter.next() {
                    Some(parser::Expr::ID(id, _)) => {
                        if id == "defun" || id == "export" {
                            let f = expr2defun(e)?;

                            if funs.contains_key(&f.id.id.to_string()) {
                                let msg = format!("error: function {:?} is multiply defined", f.id.id);
                                return Err(TypingErr{msg: msg, pos: f.id.ast.get_pos()});
                            }

                            funs.insert(f.id.id.to_string(), f);
                        } else if id == "data" {
                            let d = expr2data(e)?;
                            if data.contains_key(d.name.id.id) {
                                let msg = format!("error: data type {:?} is multiply defined", d.name.id.id);
                                return Err(TypingErr{msg: msg, pos: d.name.ast.get_pos()});
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

    let mut ctx = Context::new(funs, data);
    ctx.typing()?;

    Ok(ctx)
}

/// $DATA := ( data $DATA_NAME $MEMBER+ )
fn expr2data(expr: &parser::Expr) -> Result<DataType, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs, _) => {
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
            for mem in iter {
                let data_mem = expr2data_mem(mem)?;
                mems.push(data_mem);
            }

            Ok(DataType{name: data_name, members: mems, ast: expr})
        }
        _ => {
            Err(TypingErr::new("error: syntax error on data definition", expr))
        }
    }
}

/// $DATA_NAME := $TID | ( $TID $ID* )
fn expr2data_name(expr: &parser::Expr) -> Result<DataTypeName, TypingErr> {
    match expr {
        parser::Expr::ID(_, _) => {
            let tid = expr2type_id(expr)?;
            Ok(DataTypeName{id: tid, type_args: Vec::new(), ast: expr})
        }
        parser::Expr::Apply(exprs, _) => {
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
        parser::Expr::ID(id, _) => {
            match id.chars().nth(0) {
                Some(c) => {
                    if 'A' <= c && c <= 'Z' {
                        Ok(TIDNode{id: id, ast: expr, ty: None})
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
        parser::Expr::ID(id, _) => {
            match id.chars().nth(0) {
                Some(c) => {
                    if 'A' <= c && c <= 'Z' {
                        Err(TypingErr::new("error: the first character must not be captal", expr))
                    } else {
                        Ok(IDNode{id: id, ast: expr, ty: None})
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
        parser::Expr::ID(_, _) => {
            // $TID
            let tid = expr2type_id(expr)?;
            Ok(DataTypeMem{id: tid, types: Vec::new(), ast: expr})
        }
        parser::Expr::Apply(exprs, _) => {
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
        parser::Expr::Apply(exprs, _) => {
            let mut iter = exprs.iter();

            // $HEAD_DEFUN := export | defun
            let exported;
            match iter.next() {
                Some(parser::Expr::ID(id, _)) => {
                    exported = id == "export";
                }
                _ => {
                    return Err(TypingErr::new("error: require defun or export", expr));
                }
            }

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
                Some(parser::Expr::Apply(exprs, _)) => {
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

            let effect;
            match &fun {
                TypeExpr::TEFun(e) => {
                    effect = e.effect.clone();
                }
                _ => {
                    panic!("failed to get effect");
                }
            }
            Ok(Defun{exported: exported, id: id, args: args, fun_type: fun, effect: effect, expr: body, ast: expr, ty: None})
        }
        _ => {
            Err(TypingErr::new("error: syntax error on function definition", expr))
        }
    }
}

/// $TYPE_FUN := ( $EFFECT ( -> $TYPES $TYPE ) )
fn expr2type_fun(expr: &parser::Expr) -> Result<TypeExpr, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs, _) => {
            let mut iter = exprs.iter();

            // $EFFECT := Pure | IO
            let effect;
            let e = iter.next();
            match e {
                Some(parser::Expr::ID(eff, _)) => {
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
                Some(parser::Expr::Apply(exprs2, _)) => {
                    let mut iter2 = exprs2.iter();
                    let e2 = iter2.next();
                    match e2 {
                        Some(parser::Expr::ID(arr, _)) => {
                            if arr != "->" {
                                return Err(TypingErr::new("error: must be \"->\"", e2.unwrap()));
                            }
                        }
                        _ => {
                            return Err(TypingErr::new("error: require \"->\"", e1.unwrap()));
                        }
                    }

                    // $TYPES
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

            Ok(TypeExpr::TEFun(TEFunNode{effect: effect, args: args, ret: Box::new(ret), ast: expr}))
        }
        _ => {
            Err(TypingErr::new("error", expr))
        }
    }
}

/// $TYPES := ( $TYPE* )
fn expr2types(expr: &parser::Expr) -> Result<Vec<TypeExpr>, TypingErr> {
    match expr {
        parser::Expr::Apply(types, _) => {
            // ( $TYPES* )
            Ok(list_types2vec_types(types)?)
        }
        _ => {
            Err(TypingErr::new("error: require types of arguments", expr))
        }
    }
}

/// $TYPE := Int | Bool | $TYPE_LIST | $TYPE_TUPLE | $TYPE_FUN | $TYPE_DATA | $ID
fn expr2type(expr: &parser::Expr) -> Result<TypeExpr, TypingErr> {
    match expr {
        parser::Expr::ID(id, _) => {
            // Int | Bool | $TID
            if id == "Int" {
                Ok(TypeExpr::TEInt(TEIntNode{ast: expr}))
            } else if id == "Bool" {
                Ok(TypeExpr::TEBool(TEBoolNode{ast: expr}))
            } else {
                let c = id.chars().nth(0).unwrap();
                if 'A' <= c && c <= 'Z' {
                    let tid = expr2type_id(expr)?;
                    Ok(TypeExpr::TEData(TEDataNode{id: tid, type_args: Vec::new(), ast: expr}))
                } else {
                    Ok(TypeExpr::TEID(expr2id(expr)?))
                }
            }
        }
        parser::Expr::List(list, _) => {
            // $TYPE_LIST := '( $TYPE )
            if list.len() != 1 {
                return Err(TypingErr::new("error: require exactly one type as a type argument for list type", expr));
            }

            match list.iter().next() {
                Some(e) => {
                    let ty = Box::new(expr2type(e)?);
                    Ok(TypeExpr::TEList(TEListNode{ty: ty, ast: e}))
                }
                _ => {
                    Err(TypingErr::new("error: require type", expr))
                }
            }
        }
        parser::Expr::Tuple(tuple, _) => {
            // $TYPE_TUPLE := [ $TYPE* ]
            let mut types = Vec::new();
            for it in tuple {
                types.push(expr2type(it)?);
            }

            Ok(TypeExpr::TETuple(TETupleNode{ty: types, ast: expr}))
        }
        parser::Expr::Apply(exprs, _) => {
            // ( $TID $TYPE* )
            let mut iter = exprs.iter();

            // $TID
            let tid;
            let e = iter.next();
            match e {
                Some(parser::Expr::ID(id, _)) => {
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

            Ok(TypeExpr::TEData(TEDataNode{id: tid, type_args: args, ast: expr}))
        }
        _ => {
            Err(TypingErr::new("error: must be type", expr))
        }
    }
}

fn list_types2vec_types(exprs: &LinkedList<parser::Expr>) -> Result<Vec<TypeExpr>, TypingErr> {
    let mut v = Vec::new();
    for e in exprs {
        v.push(expr2type(e)?);
    }

    Ok(v)
}

/// $EXPR := $LITERAL | $ID | $TID | $LET | $IF | $MATCH | $LIST | $TUPLE | $GENDATA | $APPLY
fn expr2typed_expr(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
    match expr {
        parser::Expr::Num(num, _) => {
            Ok(LangExpr::LitNum(NumNode{num: *num, ast: expr}))
        }
        parser::Expr::Bool(val, _) => {
            Ok(LangExpr::LitBool(BoolNode{val: *val, ast: expr}))
        }
        parser::Expr::ID(id, _) => {
            let c = id.chars().nth(0).unwrap();
            if 'A' <= c && c <= 'Z' {
                // $TID
                let tid = expr2type_id(expr)?;
                Ok(LangExpr::DataExpr(DataNode{label: tid, exprs: Vec::new(), ast: expr, ty: None}))
            } else {
                Ok(LangExpr::IDExpr(IDNode{id: id, ast: expr, ty: None}))
            }
        }
        parser::Expr::List(list, _) => {
            let mut elms = Vec::new();
            for it in list {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(LangExpr::ListExpr(Exprs{exprs: elms, ast: expr, ty: None}))
        }
        parser::Expr::Tuple(tuple, _) => {
            let mut elms = Vec::new();
            for it in tuple {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(LangExpr::TupleExpr(Exprs{exprs: elms, ast: expr, ty: None}))
        }
        parser::Expr::Apply(exprs, _) => {
            if exprs.len() == 0 {
                return Err(TypingErr::new("error: empty expression", expr));
            }

            let mut iter = exprs.iter();

            match iter.next() {
                Some(parser::Expr::ID(id, _)) => {
                    let c = id.chars().nth(0).unwrap();
                    if 'A' <= c && c <= 'Z' {
                        // $TID
                        return Ok(expr2data_expr(expr)?);
                    } else if id == "if" {
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
            Ok(LangExpr::ApplyExpr(Exprs{exprs: elms, ast: expr, ty: None}))
        }
    }
}

/// $GENDATA := ( $TID $EXPR* )
fn expr2data_expr(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
    let exprs;
    match expr {
        parser::Expr::Apply(e, _) => {
            exprs = e;
        }
        _ => {
            return Err(TypingErr::new("error: data expression", expr));
        }
    }

    let mut iter = exprs.iter();
    let tid = expr2type_id(iter.next().unwrap())?;

    let mut v = Vec::new();
    for e in iter {
        v.push(expr2typed_expr(e)?);
    }

    Ok(LangExpr::DataExpr(DataNode{label: tid, exprs: v, ast: expr, ty: None}))
}

/// $IF := ( if $EXPR $EXPR $EXPR )
fn expr2if(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
    let exprs;
    match expr {
        parser::Expr::Apply(e, _) => {
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

    Ok(LangExpr::IfExpr(Box::new(IfNode{cond_expr: cond, then_expr: then, else_expr: else_expr, ast: expr, ty: None})))
}

/// $LET := ( let ( $DEFVAR+ ) $EXPR )
fn expr2let(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
    let exprs;
    match expr {
        parser::Expr::Apply(e, _) => {
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
        Some(parser::Expr::Apply(dvs, _)) => {
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

    Ok(LangExpr::LetExpr(Box::new(LetNode{def_vars: def_vars, expr: body, ast: expr, ty: None})))
}

/// $LETPAT := $ID | [ $LETPAT+ ] | ($TID $LETPAT+ )
fn expr2letpat(expr: &parser::Expr) -> Result<Pattern, TypingErr> {
    match expr {
        parser::Expr::ID(id, _) => {
            // $ID
            let c = id.chars().nth(0).unwrap();
            if 'A' <= c && c <= 'Z' {
                Err(TypingErr::new("error: invalid pattern", expr))
            } else {
                Ok(Pattern::PatID(IDNode{id: id, ast: expr, ty: None}))
            }
        }
        parser::Expr::Tuple(tuple, _) => {
            // [ $LETPAT+ ]
            if tuple.len() == 0 {
                return Err(TypingErr::new("error: require at least one pattern", expr));
            }

            let mut pattern = Vec::new();
            for it in tuple {
                pattern.push(expr2letpat(it)?);
            }

            Ok(Pattern::PatTuple(PatTupleNode{pattern: pattern, ast: expr, ty: None}))
        }
        parser::Expr::Apply(exprs, _) => {
            // ($TID $LETPAT+ )
            if exprs.len() < 2 {
                return Err(TypingErr::new("error: require label and at least one pattern", expr));
            }

            let mut iter = exprs.iter();
            let tid = expr2type_id(iter.next().unwrap())?;

            let mut v = Vec::new();
            for it in iter {
                v.push(expr2letpat(it)?);
            }

            Ok(Pattern::PatData(PatDataNode{label: tid, pattern: v, ast: expr, ty: None}))
        }
        _ => {
            Err(TypingErr::new("error: invalid pattern", expr))
        }
    }
}

/// $DEFVAR := ( $LETPAT $EXPR )
fn expr2def_vars(expr: &parser::Expr) -> Result<DefVar, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs, _) => {
            if exprs.len() != 2 {
                return Err(TypingErr::new("invalid variable definition", expr))
            }

            let mut iter = exprs.iter();

            let pattern = expr2letpat(iter.next().unwrap())?;  // $LETPAT
            let body = expr2typed_expr(iter.next().unwrap())?; // $EXPR

            Ok(DefVar{pattern: pattern, expr: body, ast: expr, ty: None})
        }
        _ => {
            Err(TypingErr::new("must be variable definition(s)", expr))
        }
    }
}

/// $PATTERN := $LITERAL | $ID | $TID | [ $PATTERN+ ] | ( $TID $PATTERN* ) | '()
fn expr2mpat(expr: &parser::Expr) -> Result<Pattern, TypingErr> {
    match expr {
        parser::Expr::ID(id, _) => {
            let c = id.chars().nth(0).unwrap();
            if 'A' <= c && c <= 'Z' {
                // $TID
                let tid = expr2type_id(expr)?;
                Ok(Pattern::PatData(PatDataNode{label: tid, pattern: Vec::new(), ast: expr, ty: None}))
            } else {
                // $ID
                let id_node = expr2id(expr)?;
                Ok(Pattern::PatID(id_node))
            }
        }
        parser::Expr::Bool(val, _) => {
            // $LITERAL
            Ok(Pattern::PatBool(BoolNode{val: *val, ast: expr}))
        }
        parser::Expr::Num(num, _) => {
            // $LITERAL
            Ok(Pattern::PatNum(NumNode{num: *num, ast: expr}))
        }
        parser::Expr::Tuple(exprs, _) => {
            // [ $PATTERN+ ]
            let mut pattern = Vec::new();
            for it in exprs {
                pattern.push(expr2mpat(it)?);
            }

            Ok(Pattern::PatTuple(PatTupleNode{pattern: pattern, ast: expr, ty: None}))
        }
        parser::Expr::Apply(exprs, _) => {
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

            Ok(Pattern::PatData(PatDataNode{label: tid, pattern: pattern, ast: expr, ty: None}))
        }
        parser::Expr::List(list, _) => {
            if list.len() > 0 {
                return Err(TypingErr::new("error: list pattern is not supported", expr));
            }

            Ok(Pattern::PatNil(PatNilNode{ast: expr, ty: None}))
        }
    }
}

/// $CASE := ( $PATTERN $EXPR )
fn expr2case(expr: &parser::Expr) -> Result<MatchCase, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs, _) => {
            if exprs.len() != 2 {
                return Err(TypingErr::new("error: case require exactly 2 expressions", expr));
            }

            let mut iter = exprs.iter();
            let pattern = expr2mpat(iter.next().unwrap())?;
            let body = expr2typed_expr(iter.next().unwrap())?;

            Ok(MatchCase{pattern: pattern, expr: body, ast: expr, ty: None})
        }
        _ => {
            Err(TypingErr::new("error: invalid case", expr))
        }
    }
}

/// $MATCH := ( match $EXPR $CASE+ )
fn expr2match(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
    match expr {
        parser::Expr::Apply(exprs, _) => {
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

            if cases.len() == 0 {
                return Err(TypingErr::new("error: require at least one case", expr))
            }

            let node = MatchNode{expr: cond, cases: cases, ast: expr, ty: None};
            Ok(LangExpr::MatchExpr(Box::new(node)))
        }
        _ => {
            Err(TypingErr::new("error: invalid match", expr))
        }
    }
}

impl Type {
    fn has_tvar(&self, id: ID) -> bool {
        match self {
            Type::TVar(n)  => id == *n,
            Type::TCon(tc) => tc.has_tvar(id)
        }
    }

    fn apply_sbst(&self, sbst: &Sbst) -> Type {
        match self {
            Type::TVar(n) => {
                match sbst.get(n) {
                    Some(t) => t.clone(),
                    None => self.clone()
                }
            }
            Type::TCon(tc) => tc.apply_sbst(sbst)
        }
    }
}

impl Tycon {
    fn has_tvar(&self, id: ID) -> bool {
        for t in &self.args {
            if t.has_tvar(id) {
                return true;
            }
        }

        false
    }

    fn apply_sbst(&self, sbst: &Sbst) -> Type {
        let mut v = Vec::new();
        for t in &self.args {
            v.push(t.apply_sbst(sbst));
        }

        Type::TCon(Tycon{id: self.id.clone(), args: v})
    }
}

fn unify(lhs: &Type, rhs: &Type) -> Option<Sbst> {
    let mut sbst = Sbst::new();
    match (lhs, rhs) {
        (Type::TVar(id1), Type::TVar(id2)) => {
            if id1 != id2 {
                sbst.insert(*id1, rhs.clone());
            }
            Some(sbst)
        }
        (Type::TVar(id), _) => {
            if rhs.has_tvar(*id) {
                return None;
            }
            sbst.insert(*id, rhs.clone());
            Some(sbst)
        }
        (_, Type::TVar(id)) => {
            if lhs.has_tvar(*id) {
                return None;
            }
            sbst.insert(*id, lhs.clone());
            Some(sbst)
        }
        (Type::TCon(ty_lhs), Type::TCon(ty_rhs)) => {
            if ty_lhs.id != ty_rhs.id || ty_lhs.args.len() != ty_rhs.args.len() {
                return None;
            }

            for (t1, t2) in ty_lhs.args.iter().zip(ty_rhs.args.iter()) {
                let s = unify(&t1.apply_sbst(&sbst), &t2.apply_sbst(&sbst))?;
                sbst = compose(&s, &sbst);
            }

            Some(sbst)
        }
    }
}

/// - S: substitution
/// - x: type variable
/// - T: type
///
/// S := x : T, S
///
/// S1・S2
/// compose(S1, S2) = {
///   x : T.apply_sbst(S1) if x : T in S2
///   x : T                if x : T in S1 and x not in domain(S2)
/// }
fn compose(s1: &Sbst, s2: &Sbst) -> Sbst {
    let mut sbst = Sbst::new();

    for (x, t) in s2.iter() {
        sbst.insert(*x, t.apply_sbst(s1));
    }

    for (x, t) in s1.iter() {
        sbst.entry(*x).or_insert(t.clone());
    }

    sbst
}