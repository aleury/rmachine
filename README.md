[![CI](https://github.com/aleury/rmachine/actions/workflows/ci.yaml/badge.svg)](https://github.com/aleury/rmachine/actions/workflows/ci.yaml)
[![Nightly](https://github.com/aleury/rmachine/actions/workflows/nightly.yaml/badge.svg)](https://github.com/aleury/rmachine/actions/workflows/nightly.yaml)
[![Security audit](https://github.com/aleury/rmachine/actions/workflows/audit.yaml/badge.svg)](https://github.com/aleury/rmachine/actions/workflows/audit.yaml)

# R-Machine

A simple 32-bit RISC CPU emulator and assembler written in Rust.

## Overview

R-Machine is an educational project that implements a minimal RISC (Reduced Instruction Set Computer) architecture. It includes:

- A 32-bit CPU emulator with 16 registers
- An assembler that converts assembly code to machine code
- A debugger for step-by-step execution
- Support for basic arithmetic, logical, and control flow operations

## Features

- **Simple Architecture**: 16 32-bit registers with clear purposes
- **Rich Instruction Set**: 23 instructions covering arithmetic, logic, branching, and memory operations
- **Development Tools**: Includes assembler (`rasm`), debugger (`rmon`), and disassembler (`rdis`)
- **Educational Focus**: Clean, understandable implementation ideal for learning about CPU design

## Installation

```bash
cargo install rmachine
```

Or build from source:

```bash
git clone https://github.com/aleury/rmachine
cd rmachine
cargo build --release
```

## Usage

### Assembler (rasm)

Assemble source code into executable format:

```bash
rasm input.s    # Creates input (executable without extension)
```

### Debugger (rmon)

Run programs with debugging support:

```bash
rmon program.rmx       # Run with debugger
rmon program.rmx -d    # Start in debug mode
```

### Disassembler (rdis)

Disassemble executable files:

```bash
rdis program.rmx
```

## Example

```asm
; Simple counter program
    li a0, 0        ; Initialize counter
loop:
    addi a0, a0, 1  ; Increment
    li a1, 10       ; Load limit
    blt a0, a1, loop ; Loop if counter < 10
    ecall           ; Exit
```

## Examples

The `examples/` directory contains sample R-Machine assembly programs:

- `ex1.s` - Hello World program demonstrating system calls
- `ex2.s` - Example showcasing arithmetic operations
- `strlen.s` - String length calculation implementation

Run an example:

```bash
rasm examples/ex1.s
rmon examples/ex1
```

## Documentation

- [Architecture and Instruction Set](docs/design.md)
- [Todo List](docs/todos.md)

## License

This project is dual-licensed under MIT OR Apache-2.0.
