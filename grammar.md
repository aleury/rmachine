# Grammar

```
program := line ( '\n' line )*

line := label | instruction | directive

label := identifier ':'

directive := '.globl' identifier | '.section' identifier | '.ascii' string

instruction := identifier operands*

operands := operand ( ',' operand )*

operand := identifier | immediate

identifier := alpha alnum+

immediate := integer

string := '"' ( char | escape )* '"'

char := any character except '"' and '\'

escape := '\' ( '"' | '\' )

integer := digit+

alpha := lower | upper | '_'

alnum := alpha | digit

lower := 'a' | 'b' | ... | 'z'

upper := 'A' | 'B' | ... | 'Z'

digit := '0' | '1' | ... | '9'
```
