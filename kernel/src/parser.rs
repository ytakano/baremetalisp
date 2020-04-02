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

use alloc::string::{String, ToString};
use alloc::collections::linked_list::LinkedList;

#[derive(Debug)]
pub enum Expr {
    Num(i64),
    ID(String),
    Bool(bool),
    List(LinkedList<Expr>),
    Tuple(LinkedList<Expr>),
    Apply(LinkedList<Expr>)
}

pub fn parse(code: &str) -> Option<Expr> {
    driver::uart::puts(code);
    driver::uart::puts("\n");

    match parse_expr(code) {
        Some((e, _c)) => {
            let msg = format!("AST: {:?}\n", e);
            driver::uart::puts(&msg);
            Some(e)
        }
        None => { None }
    }
}

fn parse_expr(code: &str) -> Option<(Expr, &str)> {
    let c = skip_spaces(code);
    match (*c).chars().nth(0) {
        Some('(') => {
            parse_apply(c)
        }
        Some('\'') => {
            parse_list(c)
        }
        Some('[') => {
            parse_tuple(c)
        }
        Some(a) => {
            if '0' <= a && a <= '9' || a == '-' {
                parse_num(c)
            } else {
                match parse_id_bool(c) {
                    None => { None }
                    ret => { ret }
                }
            }
        }
        _ => { None }
    }
}

fn skip_spaces(code: &str) -> &str {
    let mut i = 0;
    for s in code.chars() {
        if is_space(s) {
            i += 1;
            continue;
        } else {
            break;
        }
    }
    &code[i..]
}

fn parse_id_bool(code: &str) -> Option<(Expr, &str)> {
    let mut i = 0;
    for s in code.chars() {
        if is_paren(s) || is_space(s) {
            break;
        }
        i += 1;
    }

    if i == 0 {
        None
    } else {
        let c = code[..i].to_string();
        if c == "true" {
            Some((Expr::Bool(true), &code[i..]))
        } else if c == "false" {
            Some((Expr::Bool(false), &code[i..]))
        } else {
            Some((Expr::ID(c), &code[i..]))
        }
    }
}

fn parse_apply(code: &str) -> Option<(Expr, &str)> {
    let c = &code[1..]; // skip '('

    match parse_exprs(c) {
        Some((list, c)) => {
            if c.chars().nth(0) == Some(')') {
                Some((Expr::Apply(list), &c[1..]))
            } else {
                None
            }
        }
        None => { None }
    }
}

fn parse_exprs(code: &str) -> Option<(LinkedList<Expr>, &str)> {
    let mut exprs = LinkedList::<Expr>::new();
    let mut c = skip_spaces(code);

    loop {
        match parse_expr(c) {
            Some((e, c2)) => {
                exprs.push_back(e);
                c = c2;
            }
            None => {
                return None;
            }
        }

        c = skip_spaces(c);
        let c0 = c.chars().nth(0);
        if c.len() == 0 || c0 == Some(')') || c0 == Some(']') {
            break;
        }
    }

    Some((exprs, c))
}

fn parse_num(code: &str) -> Option<(Expr, &str)> {
    let mut i = 0;

    let c = if code.chars().nth(0) == Some('-') {
        i += 1;
        &code[1..]
    } else {
        code
    };

    for a in c.chars() {
        if '0' <= a && a <= '9' {
            i += 1;
        } else {
            break;
        }
    }

    let c = &code[i..];

    let fun = || {
        match code[0..i].parse::<i64>() {
            Ok(num) => {
                Some((Expr::Num(num), c))
            }
            Err(_msg) => { None }
        }
    };

    if c.len() == 0 {
        return fun();
    }

    match c.chars().nth(0) {
        Some(c0) => {
            if is_paren(c0) || is_space(c0) {
                fun()
            } else {
                None
            }
        }
        None => { None }
    }
}

fn is_space(c: char) -> bool {
    c == ' ' || c == '\r' || c == '\n' || c == '\t'
}

fn is_paren(c: char) -> bool {
    c == '(' || c == ')' || c == '[' || c == ']'
}

fn parse_list(code: &str) -> Option<(Expr, &str)> {
    let c = &code[1..]; // skip '\''

    match c.chars().nth(0) {
        Some('(') => {
            let c = &c[1..];
            match parse_exprs(c) {
                Some((list, c)) => {
                    if c.chars().nth(0) == Some(')') {
                        Some((Expr::List(list), c))
                    } else {
                        None
                    }
                }
                None => { None }
            }
        }
        _ => { None }
    }
}

fn parse_tuple(code: &str) -> Option<(Expr, &str)> {
    let c = &code[1..]; // skip '['

    match parse_exprs(c) {
        Some((list, c)) => {
            if c.chars().nth(0) == Some(']') {
                Some((Expr::Tuple(list), &c[1..]))
            } else {
                None
            }
        }
        None => { None }
    }
}