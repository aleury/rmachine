# R-Machine

A simple 32-bit RISC CPU.

# Architecture

The 32-bit version of the R-machine has the following 16 32-bit registers:

| ID | Name | Purpose |
| --- | ---- | -------|
| 0000 | x0 | Hardwired to zero |
| 0001 - 1101 | a0 - a12 | General purpose registers |
| 1110 | ra | Return address |
| 1111 | sp | Stack pointer |

# Instruction Encoding

Each instruction is 32-bits in length and is encoded as follows:

| 31 - 17 | 16 - 13 | 12 - 9 | 8 - 5 | 4 - 0 |
| ------- | ------- | ------ | ----- | ----- |
| imm | rs2 | rs1 | rd | opcode |

- The opcode field is 5-bits in length and specifies the operation to be performed.
- The rd, rs1, and rs2 fields are 4-bits in length and specify the destination register and source registers respectively.
- The imm field is 15-bits in length and specifies an immediate value.

## Instruction Set

| Opcode | Mnemonic | Description |
| ------ | -------- | ----------- |
| 00000 | - | Invalid instruction  |
| 00001 | LI | Load Immediate; rd = imm |
| 00010 | ADD | Add; rd = rs1 + rs2 + imm |
| 00011 | AND | Bitwise And; rd = rs1 & rs2 |
| 00100 | ANDI | Bitwise And Immediate; rd = rs1 & imm |
| 00101 | OR | Bitwise Or; rd = rs1 \| rs2 |
| 00110 | ORI | Bitwise Or Immediate; rd = rs1 \| imm |
| 00111 | XOR | Bitwise Xor; rd = rs1 ^ rs2 |
| 01000 | XORI | Bitwise Xor Immediate; rd = rs1 ^ imm |
| 01001 | SUB | Subtract; rd = rs1 - (rs2 + imm) |
| 01010 | SHL | Shift Left; rd = rs1 << (rs2 + imm) |
| 01011 | SHR | Shift Right; rd = rs1 >> (rs2 + imm) |
| 01100 | JMP | Unconditional Jump; pc = imm |
| 01101 | JMPL | Unconditional Jump to 32-bit address in next word |
| 01110 | RET | Return to address saved in ra from previous jump |
| 01111 | BEQ | Branch if Equal; pc += imm if rs1 == rs2 |
| 10000 | BNE | Branch if Not Equal; pc += imm if rs1 != rs2 |
| 10001 | BLT | Branch if Less Than; pc += imm if rs1 < rs2 |
| 10010 | BGE | Branch if Greater Than or Equal; pc += imm if rs1 >= rs2 |
| 10011 | PUSH | Push value in rs1 to stack, adjusting sp |
| 10100 | POP | Pop value from stack to rd, adjusting sp |
| 10101 | LOAD | Copy value from memory address rd = rs1 + rs2 + imm |
| 10110 | STORE | Copy value from rs2 to memory address rs1 + rs2 + imm |
| 10111 | ECALL | Make a call to surrounding execution environment |
| 11000 | EBREAK | Transfer control back to debugging environment |
| - | - | Unused |

# Notes

https://github.com/bitfield/rmachine

https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf
