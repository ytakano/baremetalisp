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

use alloc::string::{String, ToString};
use alloc::collections::linked_list::LinkedList;

#[derive(Debug)]
pub struct SyntaxErr {
    line: usize,
    column: usize,
    msg: &'static str
}

pub struct Parser<'a> {
    line: usize,
    column: usize,
    remain: &'a str,
}

#[derive(Debug)]
pub enum Expr {
    Num(i64),
    ID(String),
    Bool(bool),
    List(LinkedList<Expr>),
    Tuple(LinkedList<Expr>),
    Apply(LinkedList<Expr>)
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Parser<'a> {
        Parser{line: 0, column:0, remain: code}
    }

    pub fn parse(&mut self) -> Result<LinkedList<Expr>, SyntaxErr> {
        let mut exprs = LinkedList::new();

        loop {
            self.skip_spaces();
            if self.remain.len() == 0 {
                return Ok(exprs);
            }

            exprs.push_back(self.parse_expr()?);
        }
    }

    fn parse_id_bool(&mut self) -> Result<Expr, SyntaxErr> {
        let mut i = 0;

        for s in self.remain.chars() {
            if is_paren(s) || is_space(s) {
                break;
            }
            i += 1;
        }

        if i == 0 {
            Err(SyntaxErr{line: self.line, column: self.column, msg: "unexpected EOF"})
        } else {
            let c = self.remain[..i].to_string();
            self.remain = &self.remain[i..];
            self.column += i;

            if c == "true" {
                Ok(Expr::Bool(true))
            } else if c == "false" {
                Ok(Expr::Bool(false))
            } else {
                Ok(Expr::ID(c))
            }
        }
    }

    fn parse_num(&mut self) -> Result<Expr, SyntaxErr> {
        let mut i = 0;

        let c = if self.remain.chars().nth(0) == Some('-') {
            i += 1;
            &self.remain[1..]
        } else {
            self.remain
        };

        for a in c.chars() {
            if '0' <= a && a <= '9' {
                i += 1;
            } else {
                break;
            }
        }

        let expr;

        match self.remain[0..i].parse::<i64>() {
            Ok(num) => {
                expr = Ok(Expr::Num(num));
            }
            Err(_msg) => {
                return Err(SyntaxErr{line: self.line, column: self.column, msg: "failed to parse number"})
            }
        };

        self.column += i;
        self.remain = &self.remain[i..];

        if self.remain.len() == 0 {
            return expr;
        }

        match self.remain.chars().nth(0) {
            Some(c0) => {
                if is_paren(c0) || is_space(c0) {
                    expr
                } else {
                    Err(SyntaxErr{line: self.line, column: self.column, msg: "expected '(', ')', '[', ']' or space"})
                }
            }
            None => {
                Err(SyntaxErr{line: self.line, column: self.column, msg: "unexpected EOF"})
            }
        }
    }


    fn skip_spaces(&mut self) {
        let mut i = 0;
        let mut prev = ' ';
        for s in self.remain.chars() {
            if is_space(s) {
                if s == '\r' || (s == '\n' && prev != '\r') {
                    self.line += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
                i += 1;
                prev = s;
            } else {
                break;
            }
        }
        self.remain = &self.remain[i..]
    }

    fn parse_exprs(&mut self) -> Result<LinkedList<Expr>, SyntaxErr> {
        let mut exprs = LinkedList::<Expr>::new();
        self.skip_spaces();

        loop {
            exprs.push_back(self.parse_expr()?);

            self.skip_spaces();
            let c0 = self.remain.chars().nth(0);
            if self.remain.len() == 0 || c0 == Some(')') || c0 == Some(']') {
                break;
            }
        }

        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<Expr, SyntaxErr> {
        self.skip_spaces();
        match self.remain.chars().nth(0) {
            Some('(') => {
                self.parse_apply()
            }
            Some('\'') => {
                self.parse_list()
            }
            Some('[') => {
                self.parse_tuple()
            }
            Some(a) => {
                if '0' <= a && a <= '9' {
                    self.parse_num()
                } else if a == '-' {
                    match self.remain.chars().nth(1) {
                        Some(b) => {
                            if '0' <= b && b <= '9' {
                                self.parse_num()
                            } else {
                                self.parse_id_bool()
                            }
                        }
                        _ => {
                            self.parse_id_bool()
                        }
                    }
                } else {
                    self.parse_id_bool()
                }
            }
            _ => {
                Err(SyntaxErr{line: self.line, column: self.column, msg: "unexpected character"})
            }
        }
    }

    fn parse_apply(&mut self) -> Result<Expr, SyntaxErr> {
        self.remain = &self.remain[1..]; // skip '('
        self.column += 1;

        let exprs = self.parse_exprs()?;
        if self.remain.chars().nth(0) == Some(')') {
            self.remain = &self.remain[1..];
            self.column += 1;
            Ok(Expr::Apply(exprs))
        } else {
            Err(SyntaxErr{line: self.line, column: self.column, msg: "expected ')'"})
        }
    }

    fn parse_list(&mut self) -> Result<Expr, SyntaxErr> {
        let c = &self.remain[1..]; // skip '\''
        self.column += 1;

        match c.chars().nth(0) {
            Some('(') => {
                self.remain = &c[1..];
                let exprs = self.parse_exprs()?;
                if self.remain.chars().nth(0) == Some(')') {
                    self.remain = &self.remain[1..];
                    self.column += 1;
                    Ok(Expr::List(exprs))
                } else {
                    Err(SyntaxErr{line: self.line, column: self.column, msg: "expected ')'"})
                }
            }
            _ => {
                Err(SyntaxErr{line: self.line, column: self.column, msg: "expected '('"})
            }
        }
    }

    fn parse_tuple(&mut self) -> Result<Expr, SyntaxErr> {
        self.remain = &self.remain[1..]; // skip '['
        self.column += 1;

        let exprs = self.parse_exprs()?;
        if self.remain.chars().nth(0) == Some(']') {
            self.remain = &self.remain[1..];
            self.column += 1;
            Ok(Expr::Tuple(exprs))
        } else {
            Err(SyntaxErr{line: self.line, column: self.column, msg: "expected ']'"})
        }
    }
}

fn is_space(c: char) -> bool {
    c == ' ' || c == '\r' || c == '\n' || c == '\t'
}

fn is_paren(c: char) -> bool {
    c == '(' || c == ')' || c == '[' || c == ']'
}