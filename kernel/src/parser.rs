/*
 * $NUM   := [1-9][0-9]*
 * $BOOL  := true | false
 * $ID    := string
 * $LIST  := '() | '( $EXPS )
 * $TUPLE := [ $EXPRS ]
 * $APPLY := ( $EXPRS )
 * $EXP   := $NUM | $BOOL | $ID | $LIST | $TUPLE | $APPLY
 * $EXPS  := $EXP | $EXP $EXPS
 */

use crate::driver;

use alloc::string::String;
use alloc::string::ToString;
use alloc::collections::linked_list::LinkedList;
use alloc::vec::Vec;

#[derive(Debug)]
pub enum Expr {
    Num(u64),
    ID(String),
    List(LinkedList<Expr>),
    Tuple(Vec<Expr>),
    Apply(Vec<Expr>),
    NoExpr
}

pub fn parse(code: &str) -> Expr {
    let (e, _c) = parse_expr(code);
    let msg = format!("AST: {:?}\n", e);
    driver::uart::puts(&msg);
    e
}

pub fn parse_expr(code: &str) -> (Expr, &str) {
    driver::uart::puts(code);
    driver::uart::puts("\n");

    let c = skip_spaces(code);
    match (*c).chars().nth(0) {
        Some('(') => { (Expr::NoExpr, code) }
        Some(_a) => {
            parse_id(c)
        }
        _ => { (Expr::NoExpr, code) }
    }
}

fn skip_spaces(code: &str) -> &str {
    let mut i = 0;
    for s in code.chars() {
        if s == ' ' || s == '\r' || s == '\n' || s == '\t' {
            i += 1;
            continue;
        } else {
            break;
        }
    }
    &code[i..]
}

fn parse_id(code: &str) -> (Expr, &str) {
    let mut i = 0;
    for s in code.chars() {
        if s == ' ' || s == '\r' || s == '\n' || s == '\t' {
            break;
        }
        i += 1;
    }

    (Expr::ID(code[..i].to_string()), &code[i..])
}

fn parse_apply(code: &str) -> () {
    let v: Vec<Expr>;

    let c = &code[1..];
    let e = parse_expr(c);

    driver::uart::puts("(\n");
}