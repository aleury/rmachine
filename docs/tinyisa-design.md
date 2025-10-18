# TinyISA Design Document

## Overview

TinyISA is a simplified 6502-like instruction set designed specifically for:

- **Testing and development** of the rmachine-core architecture
- **Demonstrating** the trait-based CPU emulator design
- **Building tooling** (assembler, disassembler, monitor) before tackling complex ISAs
- **Dogfooding** the rmachine abstractions with a real, usable ISA

TinyISA is intentionally minimal but complete enough to write real programs and stress-test all aspects of the emulator framework.

## Architecture

### Fundamental Characteristics

- **Data bus**: 8-bit
- **Address bus**: 16-bit (64KB address space)
- **Instruction length**: Variable (1-3 bytes)
- **Byte order**: Little-endian
- **Instruction encoding**: Single byte opcode + optional operands

### Memory Map

```
$0000 - $00FF   Zero Page (256 bytes, fast access)
$0100 - $01FF   Stack (256 bytes)
$0200 - $FFFF   General purpose memory
```

### Registers

| Register | Size | Description |
|----------|------|-------------|
| **A** | 8-bit | Accumulator - primary data register |
| **X** | 8-bit | Index register - counters, offsets, general purpose |
| **PC** | 16-bit | Program Counter - points to next instruction |
| **SP** | 8-bit | Stack Pointer - points to top of stack ($0100-$01FF) |
| **P** | 8-bit | Processor Status - flags register (NV-BDIZC) |

### Status Flags (P Register)

```
7  6  5  4  3  2  1  0
N  V  -  B  D  I  Z  C

N: Negative   - Set if bit 7 of result is 1
V: Overflow   - Set if signed overflow occurred
-: Unused     - Always 1
B: Break      - Set if BRK instruction caused interrupt
D: Decimal    - Decimal mode flag (not implemented initially)
I: Interrupt  - Interrupt disable flag
Z: Zero       - Set if result is zero
C: Carry      - Carry or borrow from bit 7
```

**Initial implementation**: Only N, Z, C flags are required.

## Addressing Modes

TinyISA supports multiple addressing modes to enable flexible memory access patterns.

### Mode Descriptions

| Mode | Syntax | Description | Example | Bytes |
|------|--------|-------------|---------|-------|
| **Implied** | - | No operand, operates on register | `INX` | 1 |
| **Immediate** | #$nn | Operand is the next byte | `LDA #$42` | 2 |
| **Zero Page** | $nn | Address is in zero page ($00-$FF) | `STA $20` | 2 |
| **Absolute** | $nnnn | Full 16-bit address | `LDA $2000` | 3 |
| **Zero Page,X** | $nn,X | Zero page address + X register | `STA $20,X` | 2 |
| **Absolute,X** | $nnnn,X | Absolute address + X register | `LDA $2000,X` | 3 |
| **Relative** | label | PC-relative offset for branches | `BEQ loop` | 2 |

## Instruction Set

### Current Implementation Status

**Implemented and Tested:**

| Mnemonic | Opcode | Bytes | Mode | Description | Status |
|----------|--------|-------|------|-------------|--------|
| `NOP` | $00 | 1 | Implied | No operation | ✅ Complete |
| `LDA #nn` | $01 | 2 | Immediate | Load accumulator immediate | ✅ Complete |
| `LDA $nn` | $02 | 2 | Zero Page | Load accumulator from zero page | ✅ Complete |
| `STA $nn` | $03 | 2 | Zero Page | Store accumulator to zero page | ✅ Complete |

**Implementation Notes:**
- LDA has two addressing modes (immediate and zero page), demonstrating addressing mode diversity
- All instructions have comprehensive unit tests
- PC advancement verified for all instructions
- Memory read/write operations tested

**Reserved opcodes**:
- `$FF`: Illegal instruction (used for test program termination)

**Example programs you can write now:**
```asm
LDA #42     ; Load 42 into A
STA $20     ; Store to zero page address $20
LDA $20     ; Load back from $20
STA $21     ; Store to $21
```

### Planned Phases

### Phase 1: X Register Operations (~3 more instructions)

**Goal**: Add X register for indexing and counting operations.

**CPU Changes Required:**
- Add `x: u8` field to TinyCpu
- Update `reset()` to clear X register

| Mnemonic | Opcode | Bytes | Cycles | Description | Status |
|----------|--------|-------|--------|-------------|--------|
| `LDX #nn` | $04 | 2 | 2 | Load X register immediate | ⏸️ Planned |
| `INX` | $05 | 1 | 2 | Increment X register | ⏸️ Planned |
| `STX $nn` | $06 | 2 | 3 | Store X register to zero page | ⏸️ Planned |

**Example program after Phase 1**:
```asm
LDA #42     ; A = 42
STA $20     ; mem[$20] = 42
LDX #0      ; X = 0
INX         ; X = 1
STX $21     ; mem[$21] = 1
```

**Capabilities unlocked**:
- Memory writes from instructions ✓
- Multi-register programs ✓
- Simple data movement ✓
- Counter operations ✓

### Phase 2: Status Flags + Arithmetic (~6 instructions)

**Goal**: Add flag management and arithmetic operations.

**CPU Changes**:
- Add `status: u8` field to TinyCpu
- Implement `update_nz_flags()`, `set_flag()`, `get_flag()` helpers
- Update LDA and LDX to set N/Z flags

| Mnemonic | Opcode | Bytes | Cycles | Description | Flags | Status |
|----------|--------|-------|--------|-------------|-------|--------|
| `ADC #nn` | $07 | 2 | 2 | Add with carry | N Z C V | ⏸️ Planned |
| `SBC #nn` | $08 | 2 | 2 | Subtract with carry | N Z C V | ⏸️ Planned |
| `AND #nn` | $09 | 2 | 2 | Logical AND | N Z | ⏸️ Planned |
| `ORA #nn` | $0A | 2 | 2 | Logical OR | N Z | ⏸️ Planned |
| `EOR #nn` | $0B | 2 | 2 | Exclusive OR | N Z | ⏸️ Planned |
| `CMP #nn` | $0C | 2 | 2 | Compare accumulator | N Z C | ⏸️ Planned |

**Example program after Phase 2**:
```asm
LDA #10     ; A = 10, Z=0 N=0
ADC #32     ; A = 42, Z=0 N=0 C=0
CMP #42     ; Compare A with 42, Z=1 (equal)
```

**Capabilities unlocked**:
- Arithmetic operations ✓
- Logic operations ✓
- Flag-based comparisons ✓
- Foundation for conditional branching ✓

### Phase 3: Branches (~4 instructions)

**Goal**: Enable conditional execution and loops.

| Mnemonic | Opcode | Bytes | Cycles | Description | Status |
|----------|--------|-------|--------|-------------|--------|
| `BEQ rel` | $0D | 2 | 2/3 | Branch if equal (Z=1) | ⏸️ Planned |
| `BNE rel` | $0E | 2 | 2/3 | Branch if not equal (Z=0) | ⏸️ Planned |
| `BCC rel` | $0F | 2 | 2/3 | Branch if carry clear (C=0) | ⏸️ Planned |
| `BCS rel` | $10 | 2 | 2/3 | Branch if carry set (C=1) | ⏸️ Planned |

**Branch mechanics**:
- Operand is signed 8-bit offset (-128 to +127)
- Offset is relative to PC after instruction fetch
- Takes 3 cycles if branch taken, 2 if not

**Example program after Phase 3**:
```asm
      LDX #0      ; X = 0
loop: INX         ; X++
      CPX #10     ; Compare X with 10
      BNE loop    ; Loop if not equal
      ; X = 10 here
```

**Capabilities unlocked**:
- Loops ✓
- Conditional execution ✓
- Real control flow ✓
- Useful programs ✓

### Phase 4: Extended Addressing Modes (~8 instructions)

**Goal**: Add absolute and indexed addressing for full memory access.

| Mnemonic | Opcode | Bytes | Cycles | Description | Status |
|----------|--------|-------|--------|-------------|--------|
| `LDA $nnnn` | $11 | 3 | 4 | Load accumulator absolute | ⏸️ Planned |
| `STA $nnnn` | $12 | 3 | 4 | Store accumulator absolute | ⏸️ Planned |
| `LDA $nn,X` | $13 | 2 | 4 | Load accumulator zero page,X | ⏸️ Planned |
| `STA $nn,X` | $14 | 2 | 4 | Store accumulator zero page,X | ⏸️ Planned |
| `LDA $nnnn,X` | $15 | 3 | 4/5 | Load accumulator absolute,X | ⏸️ Planned |
| `STA $nnnn,X` | $16 | 3 | 5 | Store accumulator absolute,X | ⏸️ Planned |
| `INC $nn` | $17 | 2 | 5 | Increment memory zero page | ⏸️ Planned |
| `DEC $nn` | $18 | 2 | 5 | Decrement memory zero page | ⏸️ Planned |

**Example program after Phase 4**:
```asm
      LDX #0          ; Index = 0
loop: LDA table,X     ; Load from table[X]
      STA $2000,X     ; Store to destination[X]
      INX             ; Index++
      CPX #10         ; Check if done
      BNE loop        ; Continue if not

table: .byte 1,2,3,4,5,6,7,8,9,10
```

**Capabilities unlocked**:
- Full 64KB address space access ✓
- Array/table operations ✓
- Memory-to-memory operations ✓
- Complex data structures ✓

### Phase 5: Stack Operations (Future)

**Goal**: Enable subroutines and complex control flow.

| Mnemonic | Opcode | Bytes | Cycles | Description | Status |
|----------|--------|-------|--------|-------------|--------|
| `JSR $nnnn` | $19 | 3 | 6 | Jump to subroutine | ⏸️ Future |
| `RTS` | $1A | 1 | 6 | Return from subroutine | ⏸️ Future |
| `PHA` | $1B | 1 | 3 | Push accumulator to stack | ⏸️ Future |
| `PLA` | $1C | 1 | 4 | Pull accumulator from stack | ⏸️ Future |
| `PHP` | $1D | 1 | 3 | Push processor status to stack | ⏸️ Future |
| `PLP` | $1E | 1 | 4 | Pull processor status from stack | ⏸️ Future |

## Implementation Strategy

### Phased Development

1. **Phase 1** (~40 min): Add STA, LDX, INX, STX
   - Stop and build simple assembler
   - Test with hand-written programs

2. **Phase 2** (~60 min): Add status flags and arithmetic
   - Update existing instructions to set flags
   - Add ADC, SBC, AND, ORA, EOR, CMP
   - Test flag behavior thoroughly

3. **Phase 3** (~45 min): Add branches
   - Implement PC-relative addressing
   - Add BEQ, BNE, BCC, BCS
   - Test loops and conditionals

4. **Phase 4** (~90 min): Extended addressing modes
   - Add absolute addressing
   - Add indexed addressing
   - Test with array/table operations

5. **Phase 5** (Later): Stack and subroutines
   - Add SP register management
   - Implement JSR/RTS
   - Add stack instructions

### Stop Points

**After each phase**:
1. Run all tests
2. Build/update tooling (assembler, disassembler)
3. Write example programs
4. Document any design issues

**Do NOT continue adding instructions without stopping to build tooling**. Having 6-10 instructions is enough to validate tooling design.

## Instruction Encoding Format

All TinyISA instructions follow these encoding rules:

### Opcode Byte (First byte)
```
7  6  5  4  3  2  1  0
[  Opcode Value      ]
```

Opcodes are organized by category:
- `$00-$0F`: Control flow and special (NOP, branches)
- `$10-$1F`: Memory operations with extended addressing
- `$20-$2F`: Stack operations (future)
- `$F0-$FE`: Reserved for future use
- `$FF`: Illegal instruction (test termination)

### Operand Bytes

**Immediate**: 8-bit value
```
[Opcode] [Value]
```

**Zero Page**: 8-bit address (implies $00-$FF)
```
[Opcode] [Address]
```

**Absolute**: 16-bit address (little-endian)
```
[Opcode] [Address Lo] [Address Hi]
```

**Relative**: 8-bit signed offset
```
[Opcode] [Offset]
```

## Design Rationale

### Why Variable-Length Instructions?

TinyISA uses variable-length instructions (like 6502) rather than fixed-length (like RISC-V) to:

1. **Test variable-length handling** - More complex than fixed-length, validates architecture
2. **Match 6502 reality** - TinyISA models real 8-bit CPU characteristics
3. **Enable on-demand operand fetching** - Instructions fetch bytes as needed via `next()`
4. **Demonstrate addressing mode diversity** - Different modes naturally have different lengths

### Why No HALT Instruction?

Real 6502 CPUs have no HALT instruction. Programs either:
- Loop forever (`JMP *`)
- Return to monitor/OS
- Just keep running

TinyISA matches this reality. For testing, programs terminate by:
- Executing illegal instruction ($FF)
- Running past memory bounds
- Meeting predicate condition (`run_until`)

This is more realistic than synthetic HALT and matches real CPU test suite patterns.

### Why These Instructions First?

The phased approach prioritizes:

1. **Data movement** (LDA, STA, LDX, STX) - Foundation for everything
2. **Arithmetic** (ADC, SBC, CMP) - Real computation
3. **Branches** (BEQ, BNE, etc.) - Control flow
4. **Addressing modes** - Full memory access

This order enables writing increasingly complex programs at each phase while keeping the implementation manageable.

## Testing Strategy

### Unit Tests

Each instruction should have tests covering:
- Basic operation (correct result)
- Flag updates (N, Z, C, V as applicable)
- PC advancement (correct number of bytes)
- Edge cases (zero, negative, overflow)

### Integration Tests

Test programs that combine multiple instructions:
- Simple arithmetic sequences
- Loops with counters
- Array operations
- Conditional logic

### Example Test Pattern

```rust
#[test]
fn adc_sets_carry_flag_on_overflow() {
    let mut machine = Machine::<TinyISA>::new(256);
    machine.load(0, &[
        0x01, 0xFF,  // LDA #$FF
        0x07, 0x01,  // ADC #$01 (opcode updated to $07)
        0xFF,        // Illegal instruction (terminate)
    ]).unwrap();

    machine.run().unwrap_err();

    assert_eq!(machine.cpu.a, 0x00);           // $FF + $01 = $00 (wrapped)
    assert!(machine.cpu.get_flag(Carry));       // Carry flag set
    assert!(machine.cpu.get_flag(Zero));        // Zero flag set
    assert!(!machine.cpu.get_flag(Negative));   // Negative flag clear
}
```

## Future Extensions

### Possible Future Additions

1. **Indirect addressing** - `($nn)`, `($nn,X)`, `($nn),Y`
2. **More branches** - BPL, BMI, BVC, BVS
3. **Decimal mode** - BCD arithmetic (D flag)
4. **Interrupts** - IRQ, NMI handling
5. **Y register** - Second index register
6. **More memory ops** - INC/DEC absolute, shifts/rotates

### Extension Philosophy

TinyISA should remain **minimal but useful**. Extensions should:
- Add genuine capability (not just variations)
- Be needed for real programs
- Justify the complexity they add

Resist the temptation to implement every 6502 instruction. TinyISA is a teaching ISA, not a faithful reproduction.

## Summary

TinyISA strikes a balance between:
- **Simple enough** to implement quickly and understand completely
- **Complex enough** to stress-test the emulator architecture
- **Realistic enough** to model real 8-bit CPU characteristics
- **Useful enough** to write actual programs and build tooling

By following the phased development approach, TinyISA serves as both a development platform for rmachine-core and a reference implementation for variable-length ISAs.
