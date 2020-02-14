extern crate combine;
use combine::{many1, sep_by, Parser};
use combine::parser::char::{letter, space};

/*
 * $NUM  := [1-9][0-9]*
 * $BOOL := true | false
 * $EXP  := $NUM | ( $STR $EXPS )
 * $EXPS := $EXP | $EXP $EXPS
 * $LIST := '() | '( $EXPS )
 * $STR is some string
 */

pub fn parse_exp(s : &str) -> Result<(Option<String>, &str), combine::error::StringStreamError> {
    let word = many1(letter());
    sep_by(word, space())
    // Combine can collect into any type implementing `Default + Extend` so we need to assist rustc
    // by telling it that `sep_by` should collect into a `Vec` and `many1` should collect to a `String`
    .map(|mut words: Vec<String>| words.pop())
    .parse(s)
}
