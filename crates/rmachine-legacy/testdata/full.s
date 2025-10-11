_start:
    add a0, a1, a2
    add a3, a4, a5
    add a6, a7, a0
    addi zero, zero, 0
    addi ra, zero, 16
    addi sp, zero, 4
    addi gp, zero, 4
    addi tp, zero, 8
    addi a0, zero, 0
    addi a1, zero, 1
    addi a2, zero, 2
    addi a3, zero, 3
    addi a4, zero, 4
    addi a5, zero, 5
    addi a6, zero, 6
    addi a7, zero, 7
    auipc a0, 42
    auipc a1, 42
    auipc a2, 42
    auipc a3, 42
    auipc a4, 42
    auipc a5, 42
    auipc a6, 42
    auipc a7, 42
    beq t0, zero, end
    beq t1, zero, end
    ecall
    j end
    lb a0, 4(a1)
    lui t2, 42
end:
    addi zero, zero, 0
