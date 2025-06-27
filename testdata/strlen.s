    la     a0, mystr     # Load the address of mystr into a0
    li     t0, 0         # i = 0
loop: # Start of for loop
    add    t1, t0, a0    # Add the byte offset for str[i]
    lb     t1, 0(t1)     # Dereference str[i]
    beq    t1, zero, end # if str[i] == 0, break for loop
    addi   t0, t0, 1     # Add 1 to our iterator
    j      loop          # Jump back to condition (1 backwards)
end: # End of for loop
    ebreak

mystr:
    .ascii "test\0"
