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
use alloc::collections::linked_list::LinkedList;
use alloc::vec::Vec;

enum Expr {
    Num(u64),
    ID(String),
    List(LinkedList<Expr>),
    Tuple(Vec<Expr>),
    Apply(Vec<Expr>)
}

pub fn parse_expr(code: &String) -> () {
    driver::uart::puts(code);
    driver::uart::puts("\n");
    for s in code.chars() {
        if s == '(' {
            driver::uart::puts("(\n");
        }
    }
}