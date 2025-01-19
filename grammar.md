# Grammar

```
program := line ( '\n' line )*

line := label | instruction

label := identifier ':'

instruction := identifier operands*

operands := operand ( ',' operand )*

operand := register | immediate

register := identifier

immediate := integer

identifier := alpha alnum+

integer := digit+

alpha := lower | upper | '_'

alnum := alpha | digit

lower := 'a' | 'b' | ... | 'z'

upper := 'A' | 'B' | ... | 'Z'

digit := '0' | '1' | ... | '9'
```
