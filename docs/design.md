# RISC-V Architecture Design Documentation

## Overview

This project implements a subset of the RISC-V RV32I (32-bit integer) instruction set architecture. RISC-V is an open-source instruction set architecture (ISA) based on established reduced instruction set computer (RISC) principles.

## Registers

The RISC-V RV32I architecture defines 32 general-purpose registers, each 32 bits wide. Currently, we implement 18 of these registers:

| Register | ABI Name | Description | Preserved across calls? |
| -------- | -------- | ----------- | ---------------------- |
| x0 | zero | Hardwired to zero | N/A |
| x1 | ra | Return address | No |
| x2 | sp | Stack pointer | Yes |
| x3 | gp | Global pointer | N/A |
| x4 | tp | Thread pointer | N/A |
| x5 | t0 | Temporary register 0 | No |
| x6 | t1 | Temporary register 1 | No |
| x7 | t2 | Temporary register 2 | No |
| x8 | s0/fp | Saved register 0 / Frame pointer | Yes |
| x9 | s1 | Saved register 1 | Yes |
| x10 | a0 | Function argument 0 / Return value 0 | No |
| x11 | a1 | Function argument 1 / Return value 1 | No |
| x12 | a2 | Function argument 2 | No |
| x13 | a3 | Function argument 3 | No |
| x14 | a4 | Function argument 4 | No |
| x15 | a5 | Function argument 5 | No |
| x16 | a6 | Function argument 6 | No |
| x17 | a7 | Function argument 7 | No |

### Register Conventions

- **zero (x0)**: Always contains the value 0. Writes to this register are ignored.
- **ra (x1)**: Used by the JAL and JALR instructions to save the return address.
- **sp (x2)**: Points to the top of the stack, grows downward.
- **gp (x3)**: Used for position-independent code.
- **tp (x4)**: Used for thread-local storage.
- **t0-t2**: Temporary registers that can be used freely by functions.
- **s0-s1**: Saved registers that must be preserved across function calls.
- **a0-a7**: Used for function arguments and return values. a0 and a1 also hold return values.

## Instruction Encoding

RISC-V uses several instruction formats. The base RV32I ISA uses 32-bit fixed-length instructions aligned on 32-bit boundaries.

### Instruction Formats

#### R-Type (Register)
Used for register-register operations.

| 31-25 | 24-20 | 19-15 | 14-12 | 11-7 | 6-0 |
|-------|-------|-------|-------|------|-----|
| funct7 | rs2 | rs1 | funct3 | rd | opcode |

#### I-Type (Immediate)
Used for immediate operations, loads, and system instructions.

| 31-20 | 19-15 | 14-12 | 11-7 | 6-0 |
|-------|-------|-------|------|-----|
| imm[11:0] | rs1 | funct3 | rd | opcode |

#### S-Type (Store)
Used for store instructions.

| 31-25 | 24-20 | 19-15 | 14-12 | 11-7 | 6-0 |
|-------|-------|-------|-------|------|-----|
| imm[11:5] | rs2 | rs1 | funct3 | imm[4:0] | opcode |

#### B-Type (Branch)
Used for conditional branch instructions.

| 31-25 | 24-20 | 19-15 | 14-12 | 11-7 | 6-0 |
|-------|-------|-------|-------|------|-----|
| imm[12,10:5] | rs2 | rs1 | funct3 | imm[4:1,11] | opcode |

#### U-Type (Upper immediate)
Used for LUI and AUIPC instructions.

| 31-12 | 11-7 | 6-0 |
|-------|------|-----|
| imm[31:12] | rd | opcode |

#### J-Type (Jump)
Used for JAL instruction.

| 31-12 | 11-7 | 6-0 |
|-------|------|-----|
| imm[20,10:1,11,19:12] | rd | opcode |

## Currently Implemented Instructions

The following subset of RV32I instructions are currently implemented:

| Instruction | Type | Opcode | Description |
|-------------|------|--------|-------------|
| ADD | R | 0110011 | rd = rs1 + rs2 |
| ADDI | I | 0010011 | rd = rs1 + sign_extend(imm) |
| AUIPC | U | 0010111 | rd = pc + (imm << 12) |
| BEQ | B | 1100011 | if (rs1 == rs2) pc += sign_extend(imm) |
| ECALL | I | 1110011 | System call |
| JAL | J | 1101111 | rd = pc + 4; pc += sign_extend(imm) |
| LB | I | 0000011 | rd = sign_extend(M[rs1 + imm][7:0]) |
| LUI | U | 0110111 | rd = imm << 12 |
| UNIMP | - | - | Unimplemented instruction trap |

### System Calls (ECALL)

The ECALL instruction is used to make system calls. The system call number is passed in register a7, with arguments in a0-a5.

Currently implemented system calls:

| Number | Name | Arguments | Description |
|--------|------|-----------|-------------|
| 64 | write | a0=fd, a1=buffer, a2=count | Write to file descriptor |

## Memory Model

- Memory is byte-addressable
- Little-endian byte ordering
- 32-bit addresses
- Load/store instructions are the only way to access memory

## Program Counter (PC)

- 32-bit program counter
- Incremented by 4 after each instruction (except for branches and jumps)
- Must be aligned on 4-byte boundaries

## Future Implementation Goals

To achieve full RV32I compliance, the following instructions need to be implemented:

### Arithmetic and Logical
- SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND
- SLLI, SLTI, SLTIU, XORI, SRLI, SRAI, ORI, ANDI

### Load and Store
- LH, LW, LBU, LHU
- SB, SH, SW

### Branches
- BNE, BLT, BGE, BLTU, BGEU

### Jumps
- JALR

### Memory Ordering
- FENCE

## References

- https://github.com/bitfield/rmachine
- [RISC-V Specification v2.2](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf)
- [RISC-V Unprivileged ISA Specification](https://github.com/riscv/riscv-isa-manual/releases/download/Ratified-IMAFDQC/riscv-spec-20191213.pdf)
- [RISC-V ABI Documentation](https://github.com/riscv/riscv-elf-psabi-doc)
