/*
 * $NUM  := [1-9][0-9]*
 * $BOOL := true | false
 * $EXP  := $NUM | ( $STR $EXPS )
 * $EXPS := $EXP | $EXP $EXPS
 * $LIST := '() | '( $EXPS )
 * $STR is some string
 */

pub fn test(a: u8) -> u8 {
    a
}