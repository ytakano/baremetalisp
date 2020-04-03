# baremetalisp

## Serial Console

- baud rate: 115200
- no parity
- 1 stop bit

## Semantics of Typed Lisp

### Identifier

- $ID
  - a string whose first character is small

### Type Identifier

- $TID
  - a string whose first character is captal

### Primitive Type

- $PRIM := Int | Bool | $PRIM_LIST | $PRIM_TUPLE
- $PRIM_LIST := '( $PRIM* )
- $PRIM_TUPLE := \[ $PRIM+ \]

### Type

- $TYPE := Int | Bool | $TYPE_LIST | $TYPE_TUPLE | $TYPE_FUN
- $TYPE_LIST := '( $TID )
- $TYPE_TUPLE := \[ $TID+ \]
- $TYPE_FUN := ( $EFFECT ( -> $TYPES $TYPES ) )
- $EFFECT := Pure | IO
- $TYPES := $TYPE | ( $TYPE* )

### Data Type

- $DATA := ( data $DATA_NAME $MEMBER+ )
- $DATA_NAME := $TID | ( $TID $ID* )
- $MEMBER := $TID | ( $TID $PRIM* )

example:
```common-lisp
(data (Maybe t)
    (Just t)
    Nothing)
```

### Function Definition

- $DEFUN := ( $HEAD_DEFUN $ID $TYPE_FUN $EXPRS )
- $HEAD_DEFUN := export defun | defun
