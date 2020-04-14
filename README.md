# baremetalisp

## Serial Console

- baud rate: 115200
- no parity
- 1 stop bit

# Semantics of Typed Lisp

## Literal

- $LITERAL := $DECIMAL | $BOOL
- $DECIMAL
  - decimal number
  - examples: 0, 100, 224, -130, 4457
- $BOOL := true | false

## Identifier

- $ID
  - a string whose first character is not captal (not 'A' to 'Z')
  - excludes "true" and "false"

## Type Identifier

- $TID
  - a string whose first character is captal ('A' to 'Z')

## Primitive Type

- $PRIM := Int | Bool | $PRIM_LIST | $PRIM_TUPLE
- $PRIM_LIST := '( $PRIM )
- $PRIM_TUPLE := \[ $PRIM+ \]

## Type

- $TYPE := Int | Bool | $TYPE_LIST | $TYPE_TUPLE | $TYPE_FUN | $TYPE_DATA
- $TYPE_LIST := '( $TYPE )
- $TYPE_TUPLE := \[ $TYPE+ \]
- $TYPE_DATA := $TID | ( $TID $PRIM* )
- $TYPE_FUN := ( $EFFECT ( -> $TYPES $TYPE ) )
- $EFFECT := Pure | IO
- $TYPES := $TYPE | ( $TYPE* )

examples:
```common-lisp
'(Int)
[Int Bool]
(Pure (-> (Int INT) Bool))
'('(Int Bool))
[Int Int '([Int Bool])]
```

## Data Type

- $DATA := ( data $DATA_NAME $MEMBER* )
- $DATA_NAME := $TID | ( $TID $ID* )
- $MEMBER := $TID | ( $TID $PRIM* )

examples:
```common-lisp
(data Dim2
  (X Int)
  (Y Int))

(data (Maybe t)
    (Just t)
    Nothing)
```

## Function Definition

- $DEFUN := ( $HEAD_DEFUN $ID ( $ID* ) $TYPE_FUN $EXPR )
- $HEAD_DEFUN := export | defun

## Expression

- $EXPR := $LITERAL | $ID | $LET | $IF | $MATCH | $LIST | $TUPLE | $APPLY

### Let Expression

- $LET := ( let ( $DEFVAR+ ) $EXPR )
- $DEFVAR := ( $LETPAT $EXPR )
- $LETPAT := $ID | [ $LETPAT ]

### If Expression

- $IF := ( if $EXPR $EXPR $EXPR )

### List Expression

- $LIST := '( $EXPR* )

### Tuple Expression

- $TUPLE := [ $EXPR+ ]

### Match Expression

- $MATCH := ( match $EXPR $CASE+ )
- $CASE := ( $PATTERN $EXPR )
- $PATTERN := $LITERAL | $ID | $TID | \[ $PATTERN+ \] | ( $TID $PATTERN* )

## Built-in Functions

- cons: (-> (T '(T) '(T))
- car: (-> '(T) (Maybe T))
- cdr: (-> '(T) (Maybe '(T)))
- nth: (-> (Int \[T\]) (Maybe T))
- nth: (-> (Int '(T)) (Maybe T))