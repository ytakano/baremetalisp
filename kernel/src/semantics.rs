use crate::parser;

use alloc::collections::linked_list::LinkedList;
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::btree_set::BTreeSet;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::{ToString, String};

type ID = u64;

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
enum LangExpr<'t> {
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

#[derive(Debug, Clone)]
struct IDNode<'t> {
    id: &'t str,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct IfNode<'t> {
    cond_expr: LangExpr<'t>,
    then_expr: LangExpr<'t>,
    else_expr: LangExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
enum LetPat<'t> {
    LetPatID(IDNode<'t>),
    LetPatTuple(LetPatTupleNode<'t>),
    LetPatLabel(LetPatLabelNode<'t>)
}

#[derive(Debug)]
struct LetPatTupleNode<'t> {
    pattern: Vec::<LetPat<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct LetPatLabelNode<'t> {
    id: TIDNode<'t>,
    pattern: Vec::<LetPat<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct LetNode<'t> {
    def_vars: Vec<DefVar<'t>>,
    expr: LangExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct DefVar<'t> {
    pattern: LetPat<'t>,
    expr: LangExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct MatchNode<'t> {
    expr: LangExpr<'t>,
    cases: Vec<MatchCase<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
enum MatchPat<'t> {
    MatchPatNum(NumNode<'t>),
    MatchPatBool(BoolNode<'t>),
    MatchPatID(IDNode<'t>),
    MatchPatTuple(MatchPatTupleNode<'t>),
    MatchPatData(MatchPatDataNode<'t>),
    MatchPatElist(&'t parser::Expr),
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
    expr: LangExpr<'t>,
    ast: &'t parser::Expr
}

#[derive(Debug)]
struct Exprs<'t> {
    exprs: Vec<LangExpr<'t>>,
    ast: &'t parser::Expr
}

#[derive(Debug, Clone)]
struct TIDNode<'t> {
    id: &'t str,
    ast: &'t parser::Expr
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

#[derive(Debug)]
struct Defun<'t> {
    id: IDNode<'t>,
    args: Vec<IDNode<'t>>,
    fun_type: TypeExpr<'t>,
    expr: LangExpr<'t>,
    ast: &'t parser::Expr
}

trait TApp<'t>: Sized {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<Self, TypingErr<'t>>;
}

impl<'t> TApp<'t> for DataType<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<DataType<'t>, TypingErr<'t>> {
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
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<DataTypeMem<'t>, TypingErr<'t>> {
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
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TypeExpr<'t>, TypingErr<'t>> {
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
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TEListNode<'t>, TypingErr<'t>> {
        Ok(TEListNode{
            ty: Box::new(self.ty.apply(ty)?),
            ast: self.ast
        })
    }
}

impl<'t> TApp<'t> for TETupleNode<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TETupleNode<'t>, TypingErr<'t>> {
        let mut v = Vec::new();
        for it in self.ty.iter() {
            v.push(it.apply(ty)?);
        }

        Ok(TETupleNode{ty: v, ast: self.ast})
    }
}

impl<'t> TApp<'t> for TEFunNode<'t> {
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TEFunNode<'t>, TypingErr<'t>> {
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
    fn apply(&self, ty: &BTreeMap<&str, TypeExpr<'t>>) -> Result<TEDataNode<'t>, TypingErr<'t>> {
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
    funs: BTreeMap<&'t str, Defun<'t>>,
    data: BTreeMap<&'t str, DataType<'t>>,
    label2data: BTreeMap<&'t str, &'t str>,
    curr_id: ID,
}

impl<'t> Context<'t> {
    fn new(funs: BTreeMap<&'t str, Defun<'t>>, data: BTreeMap<&'t str, DataType<'t>>) -> Context<'t> {
        Context{funs: funs,
                data: data,
                label2data: BTreeMap::new(),
                curr_id: 0}
    }

    pub fn typing(&mut self) -> Result<(), TypingErr<'t>> {
        self.check_data_def()?;
        self.check_label()?;
        self.check_data_rec()?;
        self.check_defun_type()?;

        Ok(())
    }

    fn check_label(&mut self) -> Result<(), TypingErr<'t>> {
        for (_, dt) in &self.data {
            for mem in &dt.members {
                if self.label2data.contains_key(mem.id.id) {
                    let msg = format!("{:?} is multiply defined", mem.id.id);
                    return Err(TypingErr{msg: msg, ast: mem.id.ast});
                }

                self.label2data.insert(mem.id.id, dt.name.id.id);
            }
        }

        Ok(())
    }

    // TODO: implementing
    fn type_expr(&mut self, expr: &LangExpr<'t>) -> Type {
        match expr {
            LangExpr::LitBool(_) => ty_bool(),
            LangExpr::LitNum(_) => ty_int(),
            _ => Type::TCon(Tycon{id: "Fail".to_string(), args: Vec::new()})
        }
    }

    /// If
    /// ```
    /// (data (Tree t)
    ///   (Node (Tree t) (Tree t))
    ///   Leaf)
    /// ```
    /// then apply2data("Tree", vec!(Int)) returns (Tree Int)
    fn apply2data(&self, name: &'t str, types: &Vec<Type>) -> Result<Type, String> {
        match self.data.get(name) {
            Some(dt) => {
                if types.len() != dt.name.type_args.len() {
                    let msg = format!("{:?} requires {:?} type arguments but actually passed {:?}",
                                      name,
                                      dt.name.type_args.len(),
                                      types.len());
                    return Err(msg);
                }

                Ok(Type::TCon(Tycon{id: dt.name.id.id.to_string(),
                                    args: types.to_vec()}))
            }
            None => {
                let msg = format!("{:?} is undefined", name);
                Err(msg)
            }
        }
    }

    /// generate tymap for apply2type
    fn gen_tymap(&self, name: &'t str, types: &Vec<Type>) -> Result<BTreeMap<&'t str, Type>, String> {
        match self.data.get(name) {
            Some(dt) => {
                if types.len() != dt.name.type_args.len() {
                    let msg = format!("{:?} requires {:?} type arguments but actually passed {:?}",
                                      name,
                                      dt.name.type_args.len(),
                                      types.len());
                    return Err(msg);
                }

                let mut tymap = BTreeMap::new();

                for (k, v) in dt.name.type_args.iter().zip(types.iter()) {
                    tymap.insert(k.id, v.clone());
                }

                Ok(tymap)
            }
            None => {
                let msg = format!("{:?} is undefined", name);
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
    /// and tymap = {T: Int} then
    /// apply2type((Tree t), tymap) returns (Tree Int)
    fn apply2type(&self, type_expr: &TypeExpr<'t>, tymap: &BTreeMap<&'t str, Type>) -> Result<Type, String> {
        match type_expr {
            TypeExpr::TEBool(_) => Ok(ty_bool()),
            TypeExpr::TEInt(_)  => Ok(ty_int()),
            TypeExpr::TEList(list) => {
                let t = self.apply2type(&list.ty, tymap)?;
                Ok(Type::TCon(Tycon{id: "List".to_string(), args: vec!(t)}))
            }
            TypeExpr::TETuple(tuple) => {
                let mut v = Vec::new();
                for t in &tuple.ty {
                    v.push(self.apply2type(t, tymap)?);
                }

                Ok(Type::TCon(Tycon{id: "Tuple".to_string(), args: v}))
            }
            TypeExpr::TEFun(fun) => {
                let mut args = Vec::new();
                for a in &fun.args {
                    args.push(self.apply2type(a, tymap)?);
                }

                let r = self.apply2type(&fun.ret, tymap)?;

                let mut v = Vec::new();
                v.push(Type::TCon(Tycon{id: "Tuple".to_string(), args: args}));
                v.push(r);

                Ok(Type::TCon(Tycon{id: "->".to_string(), args: v}))
            }
            TypeExpr::TEData(data) => {
                let mut v = Vec::new();
                for t in &data.type_args {
                    v.push(self.apply2type(t, tymap)?);
                }

                Ok(Type::TCon(Tycon{id: data.id.id.to_string(), args: v}))
            }
            TypeExpr::TEID(id) => {
                match tymap.get(id.id) {
                    Some(t) => {
                        Ok(t.clone())
                    }
                    None => {
                        let msg = format!("{:?} is undefined", id.id);
                        Err(msg)
                    }
                }
            }
        }
    }

    fn check_data_def(&self) -> Result<(), TypingErr<'t>> {
        for (_, d) in self.data.iter() {
            self.check_data_def_data(d)?;
        }

        Ok(())
    }

    fn check_data_def_data(&self, data: &DataType<'t>) -> Result<(), TypingErr<'t>> {
        let mut args = BTreeSet::new();
        for arg in data.name.type_args.iter() {
            if args.contains(arg.id) {
                let msg = format!("error: {:?} is multiply used", arg.id);
                return Err(TypingErr{msg: msg, ast: arg.ast});
            }

            args.insert(arg.id);
        }

        for mem in data.members.iter() {
            self.check_data_def_mem(mem, &args)?;
        }

        Ok(())
    }

    fn check_data_def_mem(&self, mem: &DataTypeMem<'t>, args: &BTreeSet<&str>) -> Result<(), TypingErr<'t>> {
        for it in mem.types.iter() {
            self.check_def_type(it, args)?
        }

        Ok(())
    }

    fn check_def_type(&self, ty: &TypeExpr<'t>, args: &BTreeSet<&str>) -> Result<(), TypingErr<'t>> {
        match ty {
            TypeExpr::TEID(id) => {
                if !args.contains(id.id) {
                    let msg = format!("error: {:?} is undefined", id.id);
                    return Err(TypingErr{msg: msg, ast: id.ast})
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
                            return Err(TypingErr{msg: msg, ast: data.ast});
                        }
                    }
                    None => {
                        let msg = format!("error: {:?} is unkown type", data.id.id);
                        return Err(TypingErr{msg: msg, ast: data.id.ast});
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
    fn check_data_rec(&self) -> Result<(), TypingErr<'t>> {
        let mut checked = LinkedList::new();
        for (_, d) in self.data.iter() {
            let mut visited = BTreeSet::new();
            let mut inst = LinkedList::new();
            inst.push_back(d.ast);
            if self.check_data_rec_data(d, &mut visited, &mut checked, &mut inst)? {
                let msg = format!("error: {:?}'s definition is inifinete recursive", d.name.id.id);
                return Err(TypingErr{msg: msg, ast: d.ast});
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
                           inst: &mut LinkedList<&'t parser::Expr>) -> Result<bool, TypingErr<'t>> {
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
                          inst: &mut LinkedList<&'t parser::Expr>) -> Result<bool, TypingErr<'t>> {
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
                         inst: &mut LinkedList<&'t parser::Expr>) -> Result<bool, TypingErr<'t>> {
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

    fn type_data_node2data_type(&self, data: &TEDataNode<'t>) -> Result<DataType<'t>, TypingErr<'t>> {
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
            return Err(TypingErr{msg: msg, ast: data.ast});
        }

        let mut map = BTreeMap::new();
        for (k, v) in dt.name.type_args.iter().zip(data.type_args.iter()) {
            map.insert(k.id, v.clone());
        }

        dt.apply(&map)
    }

    fn check_defun_type(&self) -> Result<(), TypingErr<'t>> {
        let m = BTreeSet::new();
        for (_,fun) in self.funs.iter() {
            self.check_def_type(&fun.fun_type, &m)?;
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
            for mem in iter {
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
fn expr2type_fun(expr: &parser::Expr) -> Result<TypeExpr, TypingErr> {
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

                    // ( $TYPE* )
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
        parser::Expr::Apply(types) => {
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
        parser::Expr::ID(id) => {
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
        parser::Expr::List(list) => {
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
        parser::Expr::Tuple(tuple) => {
            // $TYPE_TUPLE := [ $TYPE+ ]
            if tuple.len() < 1 {
                return Err(TypingErr::new("error: require more than or equal to oen type", expr));
            }

            let mut types = Vec::new();
            for it in tuple {
                types.push(expr2type(it)?);
            }

            Ok(TypeExpr::TETuple(TETupleNode{ty: types, ast: expr}))
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

/// $EXPR := $LITERAL | $ID | $LET | $IF | $MATCH | $LIST | $TUPLE | $APPLY
fn expr2typed_expr(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
    match expr {
        parser::Expr::Num(num) => {
            Ok(LangExpr::LitNum(NumNode{num: *num, ast: expr}))
        }
        parser::Expr::Bool(val) => {
            Ok(LangExpr::LitBool(BoolNode{val: *val, ast: expr}))
        }
        parser::Expr::ID(id) => {
            Ok(LangExpr::IDExpr(IDNode{id: id, ast: expr}))
        }
        parser::Expr::List(list) => {
            let mut elms = Vec::new();
            for it in list {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(LangExpr::ListExpr(Exprs{exprs: elms, ast: expr}))
        }
        parser::Expr::Tuple(tuple) => {
            let mut elms = Vec::new();
            for it in tuple {
                elms.push(expr2typed_expr(it)?);
            }
            Ok(LangExpr::TupleExpr(Exprs{exprs: elms, ast: expr}))
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
            Ok(LangExpr::ApplyExpr(Exprs{exprs: elms, ast: expr}))
        }
    }
}

/// $IF := ( if $EXPR $EXPR $EXPR )
fn expr2if(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
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

    Ok(LangExpr::IfExpr(Box::new(IfNode{cond_expr: cond, then_expr: then, else_expr: else_expr, ast: expr})))
}

/// $LET := ( let ( $DEFVAR+ ) $EXPR )
fn expr2let(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
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

    Ok(LangExpr::LetExpr(Box::new(LetNode{def_vars: def_vars, expr: body, ast: expr})))
}

/// $LETPAT := $ID | [ $LETPAT+ ] | ($TID $LETPAT+ )
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
            // [ $LETPAT+ ]
            if tuple.len() == 0 {
                return Err(TypingErr::new("error: require at least one pattern", expr));
            }

            let mut pattern = Vec::new();
            for it in tuple {
                pattern.push(expr2letpat(it)?);
            }

            Ok(LetPat::LetPatTuple(LetPatTupleNode{pattern: pattern, ast: expr}))
        }
        parser::Expr::Apply(exprs) => {
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

            Ok(LetPat::LetPatLabel(LetPatLabelNode{id: tid, pattern: v, ast: expr}))
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

/// $PATTERN := $LITERAL | $ID | $TID | [ $PATTERN+ ] | ( $TID $PATTERN* ) | '()
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
        parser::Expr::List(list) => {
            if list.len() > 0 {
                return Err(TypingErr::new("error: list pattern is not supported", expr));
            }

            Ok(MatchPat::MatchPatElist(expr))
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
fn expr2match(expr: &parser::Expr) -> Result<LangExpr, TypingErr> {
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
            Ok(LangExpr::MatchExpr(Box::new(node)))
        }
        _ => {
            Err(TypingErr::new("error: invalid match", expr))
        }
    }
}