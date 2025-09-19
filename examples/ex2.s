_start:
    li a0, 5    # let mut num = 5;
    li a1, 1    # let mut result = 1;
_loop:
    beq a0, 1, _done
    mul a1, a1, a0
    sub a0, a0, 1
    j _loop
_done:
    nop
