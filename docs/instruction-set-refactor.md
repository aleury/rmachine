# Instruction Set Refactoring: Data-Driven Architecture

## Problem Statement

The current implementation has significant code duplication across three major components:

1. **Parser** (parser.rs:99-194) - 95 lines of match arms for 9 instructions
2. **Assembler** (asm.rs:476-685) - 210 lines of match arms for the same 9 instructions
3. **Executor** (machine.rs:161-213) - 52 lines of match arms for execution
4. **Encoder/Decoder** (asm.rs:266-439) - 174 lines of manual bit manipulation

### Current Code Smell

To add a single new instruction (e.g., `SUB`), you must:

1. Add enum variant to `Opcode` (asm.rs:142)
2. Add Display implementation (asm.rs:154)
3. Add encoding logic (asm.rs:189)
4. Add decoding logic (asm.rs:174)
5. Add word-to-opcode conversion (asm.rs:368-438)
6. Add opcode-to-word conversion (asm.rs:189-203)
7. Add parser case (parser.rs:99-194)
8. Add assembler case (asm.rs:476-685)
9. Add execution case (machine.rs:161-213)

**9 different locations** for one instruction! This doesn't scale to the 40+ RV32I instructions.

### Example: Current Duplication

```rust
// In parser.rs
"add" => {
    let rd = self.register()?;
    self.expect(TokenType::Comma)?;
    let rs1 = self.register()?;
    self.expect(TokenType::Comma)?;
    let rs2 = self.register()?;
    Instruction { name, operands: vec![rd, rs1, rs2] }
}

// In asm.rs (assembler)
"add" => {
    assert_eq!(instr.operands.len(), 3, "expected 3 operands for add");
    let Operand::Register(rd) = &instr.operands[0] else { ... };
    let Operand::Register(rs1) = &instr.operands[1] else { ... };
    let Operand::Register(rs2) = &instr.operands[2] else { ... };
    vec![Instruction {
        opcode: Opcode::add,
        rd: Reg::try_from(rd.to_string())?,
        rs1: Reg::try_from(rs1.to_string())?,
        rs2: Reg::try_from(rs2.to_string())?,
        imm: 0,
    }]
}

// In machine.rs (executor)
Opcode::add => {
    self.regs.set(rd, rs1 + rs2);
}
```

All three do essentially the same thing but can't share code.

---

## Proposed Solution: Data-Driven Instruction Set

### Core Concept

Define instructions **once** as declarative data, then use generic code to parse, assemble, and execute them.

### Architecture Overview

```
┌─────────────────────────────────────┐
│   INSTRUCTION_SET (static data)     │
│                                     │
│  ┌─────────────────────────────┐   │
│  │ InstructionSpec {           │   │
│  │   mnemonic: "add"           │   │
│  │   format: Format::R         │   │
│  │   opcode: 0b0110011         │   │
│  │   funct3: Some(0b000)       │   │
│  │   funct7: Some(0b0000000)   │   │
│  │   operands: RdRs1Rs2        │   │
│  │   behavior: AluReg(alu_add) │   │
│  │ }                           │   │
│  └─────────────────────────────┘   │
│           ... 40+ more              │
└─────────────────────────────────────┘
           │
           ├──→ Parser (generic, uses operand pattern)
           ├──→ Assembler (generic, uses format + encoding)
           ├──→ Executor (generic, uses behavior function)
           └──→ Disassembler (generic, uses format + decoding)
```

---

## Detailed Design

### 1. Instruction Format Enumeration

```rust
/// RISC-V instruction format types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Register-register operations
    /// Format: opcode[6:0] | rd[11:7] | funct3[14:12] | rs1[19:15] | rs2[24:20] | funct7[31:25]
    R,

    /// Immediate operations, loads
    /// Format: opcode[6:0] | rd[11:7] | funct3[14:12] | rs1[19:15] | imm[31:20]
    I,

    /// Store operations
    /// Format: opcode[6:0] | imm[4:0][11:7] | funct3[14:12] | rs1[19:15] | rs2[24:20] | imm[11:5][31:25]
    S,

    /// Branch operations
    /// Format: opcode[6:0] | imm[11|4:1][11:7] | funct3[14:12] | rs1[19:15] | rs2[24:20] | imm[12|10:5][31:25]
    B,

    /// Upper immediate operations
    /// Format: opcode[6:0] | rd[11:7] | imm[31:12][31:12]
    U,

    /// Jump operations
    /// Format: opcode[6:0] | rd[11:7] | imm[20|10:1|11|19:12][31:12]
    J,
}
```

### 2. Operand Pattern Enumeration

Defines the syntax expected by the parser:

```rust
/// Operand pattern for parsing and validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandPattern {
    /// Three registers: rd, rs1, rs2
    /// Example: add a0, a1, a2
    RdRs1Rs2,

    /// Destination register, source register, immediate
    /// Example: addi a0, a1, 42
    RdRs1Imm,

    /// Destination register, immediate
    /// Example: lui a0, 0x12345
    RdImm,

    /// Two source registers, symbol/label
    /// Example: beq a0, a1, loop
    Rs1Rs2Symbol,

    /// Destination register, symbol/label
    /// Example: jal ra, function
    RdSymbol,

    /// Destination register, offset(base_register)
    /// Example: lw a0, 8(sp)
    RdOffsetRs1,

    /// Source register, offset(base_register)
    /// Example: sw a0, 8(sp)
    Rs2OffsetRs1,

    /// No operands
    /// Example: ecall, ebreak
    None,
}
```

### 3. Execution Behavior Enumeration

Encodes what the instruction does:

```rust
/// Execution behavior specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecBehavior {
    /// ALU operation with two registers: rd = f(rs1, rs2)
    AluReg(fn(u32, u32) -> u32),

    /// ALU operation with immediate: rd = f(rs1, imm)
    AluImm(fn(u32, u32) -> u32),

    /// Conditional branch: if f(rs1, rs2) then pc += imm
    Branch(fn(u32, u32) -> bool),

    /// Load from memory: rd = mem[rs1 + imm]
    Load(LoadType),

    /// Store to memory: mem[rs1 + imm] = rs2
    Store(StoreType),

    /// Unconditional jump: rd = pc + 4; pc += imm
    Jump,

    /// Register-indirect jump: rd = pc + 4; pc = (rs1 + imm) & ~1
    JumpReg,

    /// Load upper immediate: rd = imm << 12
    UpperImm,

    /// Add PC to upper immediate: rd = pc + (imm << 12)
    AddPcImm,

    /// System call
    Syscall,

    /// Breakpoint
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadType {
    Byte,    // lb - sign extend byte
    Half,    // lh - sign extend halfword
    Word,    // lw - load word
    ByteU,   // lbu - zero extend byte
    HalfU,   // lhu - zero extend halfword
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreType {
    Byte,    // sb - store byte
    Half,    // sh - store halfword
    Word,    // sw - store word
}
```

### 4. Complete Instruction Specification

```rust
/// Complete specification for a single instruction
pub struct InstructionSpec {
    /// Assembly mnemonic (e.g., "add", "lw", "beq")
    pub mnemonic: &'static str,

    /// 7-bit opcode field
    pub opcode: u32,

    /// 3-bit funct3 field (if applicable)
    pub funct3: Option<u32>,

    /// 7-bit funct7 field (if applicable)
    pub funct7: Option<u32>,

    /// Instruction encoding format
    pub format: Format,

    /// Expected operand pattern for parsing
    pub operands: OperandPattern,

    /// Execution behavior
    pub behavior: ExecBehavior,

    /// Whether this is a pseudo-instruction
    pub is_pseudo: bool,
}
```

### 5. The Instruction Set Table

```rust
/// ALU operations
fn alu_add(a: u32, b: u32) -> u32 { a.wrapping_add(b) }
fn alu_sub(a: u32, b: u32) -> u32 { a.wrapping_sub(b) }
fn alu_and(a: u32, b: u32) -> u32 { a & b }
fn alu_or(a: u32, b: u32) -> u32 { a | b }
fn alu_xor(a: u32, b: u32) -> u32 { a ^ b }
fn alu_sll(a: u32, b: u32) -> u32 { a << (b & 0x1f) }
fn alu_srl(a: u32, b: u32) -> u32 { a >> (b & 0x1f) }
fn alu_sra(a: u32, b: u32) -> u32 { ((a as i32) >> (b & 0x1f)) as u32 }
fn alu_slt(a: u32, b: u32) -> u32 { if (a as i32) < (b as i32) { 1 } else { 0 } }
fn alu_sltu(a: u32, b: u32) -> u32 { if a < b { 1 } else { 0 } }

/// Branch conditions
fn branch_eq(a: u32, b: u32) -> bool { a == b }
fn branch_ne(a: u32, b: u32) -> bool { a != b }
fn branch_lt(a: u32, b: u32) -> bool { (a as i32) < (b as i32) }
fn branch_ge(a: u32, b: u32) -> bool { (a as i32) >= (b as i32) }
fn branch_ltu(a: u32, b: u32) -> bool { a < b }
fn branch_geu(a: u32, b: u32) -> bool { a >= b }

/// Macro for cleaner instruction definitions
macro_rules! r_type {
    ($mnem:expr, $f3:expr, $f7:expr, $behavior:expr) => {
        InstructionSpec {
            mnemonic: $mnem,
            opcode: 0b0110011,
            funct3: Some($f3),
            funct7: Some($f7),
            format: Format::R,
            operands: OperandPattern::RdRs1Rs2,
            behavior: $behavior,
            is_pseudo: false,
        }
    };
}

macro_rules! i_type {
    ($mnem:expr, $opcode:expr, $f3:expr, $operands:expr, $behavior:expr) => {
        InstructionSpec {
            mnemonic: $mnem,
            opcode: $opcode,
            funct3: Some($f3),
            funct7: None,
            format: Format::I,
            operands: $operands,
            behavior: $behavior,
            is_pseudo: false,
        }
    };
}

macro_rules! b_type {
    ($mnem:expr, $f3:expr, $behavior:expr) => {
        InstructionSpec {
            mnemonic: $mnem,
            opcode: 0b1100011,
            funct3: Some($f3),
            funct7: None,
            format: Format::B,
            operands: OperandPattern::Rs1Rs2Symbol,
            behavior: $behavior,
            is_pseudo: false,
        }
    };
}

/// The complete RV32I instruction set (40+ instructions)
pub static INSTRUCTION_SET: &[InstructionSpec] = &[
    // ===== Integer Computational Instructions =====

    // Register-Register Operations (R-type)
    r_type!("add",  0b000, 0b0000000, ExecBehavior::AluReg(alu_add)),
    r_type!("sub",  0b000, 0b0100000, ExecBehavior::AluReg(alu_sub)),
    r_type!("sll",  0b001, 0b0000000, ExecBehavior::AluReg(alu_sll)),
    r_type!("slt",  0b010, 0b0000000, ExecBehavior::AluReg(alu_slt)),
    r_type!("sltu", 0b011, 0b0000000, ExecBehavior::AluReg(alu_sltu)),
    r_type!("xor",  0b100, 0b0000000, ExecBehavior::AluReg(alu_xor)),
    r_type!("srl",  0b101, 0b0000000, ExecBehavior::AluReg(alu_srl)),
    r_type!("sra",  0b101, 0b0100000, ExecBehavior::AluReg(alu_sra)),
    r_type!("or",   0b110, 0b0000000, ExecBehavior::AluReg(alu_or)),
    r_type!("and",  0b111, 0b0000000, ExecBehavior::AluReg(alu_and)),

    // Register-Immediate Operations (I-type)
    i_type!("addi",  0b0010011, 0b000, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_add)),
    i_type!("slti",  0b0010011, 0b010, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_slt)),
    i_type!("sltiu", 0b0010011, 0b011, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_sltu)),
    i_type!("xori",  0b0010011, 0b100, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_xor)),
    i_type!("ori",   0b0010011, 0b110, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_or)),
    i_type!("andi",  0b0010011, 0b111, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_and)),
    i_type!("slli",  0b0010011, 0b001, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_sll)),
    i_type!("srli",  0b0010011, 0b101, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_srl)),
    i_type!("srai",  0b0010011, 0b101, OperandPattern::RdRs1Imm, ExecBehavior::AluImm(alu_sra)),

    // Upper Immediate (U-type)
    InstructionSpec {
        mnemonic: "lui",
        opcode: 0b0110111,
        funct3: None,
        funct7: None,
        format: Format::U,
        operands: OperandPattern::RdImm,
        behavior: ExecBehavior::UpperImm,
        is_pseudo: false,
    },
    InstructionSpec {
        mnemonic: "auipc",
        opcode: 0b0010111,
        funct3: None,
        funct7: None,
        format: Format::U,
        operands: OperandPattern::RdImm,
        behavior: ExecBehavior::AddPcImm,
        is_pseudo: false,
    },

    // ===== Control Transfer Instructions =====

    // Unconditional Jump (J-type)
    InstructionSpec {
        mnemonic: "jal",
        opcode: 0b1101111,
        funct3: None,
        funct7: None,
        format: Format::J,
        operands: OperandPattern::RdSymbol,
        behavior: ExecBehavior::Jump,
        is_pseudo: false,
    },

    // Register-Indirect Jump (I-type)
    i_type!("jalr", 0b1100111, 0b000, OperandPattern::RdRs1Imm, ExecBehavior::JumpReg),

    // Conditional Branches (B-type)
    b_type!("beq",  0b000, ExecBehavior::Branch(branch_eq)),
    b_type!("bne",  0b001, ExecBehavior::Branch(branch_ne)),
    b_type!("blt",  0b100, ExecBehavior::Branch(branch_lt)),
    b_type!("bge",  0b101, ExecBehavior::Branch(branch_ge)),
    b_type!("bltu", 0b110, ExecBehavior::Branch(branch_ltu)),
    b_type!("bgeu", 0b111, ExecBehavior::Branch(branch_geu)),

    // ===== Load Instructions (I-type) =====
    i_type!("lb",  0b0000011, 0b000, OperandPattern::RdOffsetRs1, ExecBehavior::Load(LoadType::Byte)),
    i_type!("lh",  0b0000011, 0b001, OperandPattern::RdOffsetRs1, ExecBehavior::Load(LoadType::Half)),
    i_type!("lw",  0b0000011, 0b010, OperandPattern::RdOffsetRs1, ExecBehavior::Load(LoadType::Word)),
    i_type!("lbu", 0b0000011, 0b100, OperandPattern::RdOffsetRs1, ExecBehavior::Load(LoadType::ByteU)),
    i_type!("lhu", 0b0000011, 0b101, OperandPattern::RdOffsetRs1, ExecBehavior::Load(LoadType::HalfU)),

    // ===== Store Instructions (S-type) =====
    InstructionSpec {
        mnemonic: "sb",
        opcode: 0b0100011,
        funct3: Some(0b000),
        funct7: None,
        format: Format::S,
        operands: OperandPattern::Rs2OffsetRs1,
        behavior: ExecBehavior::Store(StoreType::Byte),
        is_pseudo: false,
    },
    InstructionSpec {
        mnemonic: "sh",
        opcode: 0b0100011,
        funct3: Some(0b001),
        funct7: None,
        format: Format::S,
        operands: OperandPattern::Rs2OffsetRs1,
        behavior: ExecBehavior::Store(StoreType::Half),
        is_pseudo: false,
    },
    InstructionSpec {
        mnemonic: "sw",
        opcode: 0b0100011,
        funct3: Some(0b010),
        funct7: None,
        format: Format::S,
        operands: OperandPattern::Rs2OffsetRs1,
        behavior: ExecBehavior::Store(StoreType::Word),
        is_pseudo: false,
    },

    // ===== System Instructions =====
    InstructionSpec {
        mnemonic: "ecall",
        opcode: 0b1110011,
        funct3: Some(0b000),
        funct7: None,
        format: Format::I,
        operands: OperandPattern::None,
        behavior: ExecBehavior::Syscall,
        is_pseudo: false,
    },
    InstructionSpec {
        mnemonic: "ebreak",
        opcode: 0b1110011,
        funct3: Some(0b000),
        funct7: None,
        format: Format::I,
        operands: OperandPattern::None,
        behavior: ExecBehavior::Break,
        is_pseudo: false,
    },
];

/// Lookup instruction specification by mnemonic
pub fn lookup_by_mnemonic(name: &str) -> Option<&'static InstructionSpec> {
    INSTRUCTION_SET.iter().find(|spec| spec.mnemonic == name)
}

/// Lookup instruction specification by encoded bits
pub fn lookup_by_encoding(word: u32) -> Option<&'static InstructionSpec> {
    let opcode = word & 0x7f;
    let funct3 = (word >> 12) & 0x7;
    let funct7 = (word >> 25) & 0x7f;

    INSTRUCTION_SET.iter().find(|spec| {
        spec.opcode == opcode &&
        spec.funct3.map_or(true, |f3| f3 == funct3) &&
        spec.funct7.map_or(true, |f7| f7 == funct7)
    })
}
```

---

## Generic Implementation Examples

### Parser (After Refactoring)

```rust
fn instruction(&mut self, name: String) -> Result<Line> {
    let spec = lookup_by_mnemonic(&name)
        .ok_or(anyhow!("unknown instruction: {}", name))?;

    let operands = match spec.operands {
        OperandPattern::RdRs1Rs2 => vec![
            self.register()?,
            self.expect_comma()?,
            self.register()?,
            self.expect_comma()?,
            self.register()?,
        ],
        OperandPattern::RdRs1Imm => vec![
            self.register()?,
            self.expect_comma()?,
            self.register()?,
            self.expect_comma()?,
            self.immediate()?,
        ],
        OperandPattern::Rs1Rs2Symbol => vec![
            self.register()?,
            self.expect_comma()?,
            self.register()?,
            self.expect_comma()?,
            self.symbol()?,
        ],
        // ... other patterns
    };

    Ok(Line::Instruction(Instruction { name, operands }))
}
```

**Before: 95 lines of match arms**
**After: ~20 lines of generic code**

### Assembler (After Refactoring)

```rust
fn assemble_instruction(
    instr: ast::Instruction,
    address: Address,
    refs: &mut Vec<Ref>,
) -> Result<Vec<MachineInstruction>> {
    let spec = lookup_by_mnemonic(&instr.name)?;

    // Extract operands based on pattern
    let (rd, rs1, rs2, imm) = extract_operands(&instr.operands, spec.operands)?;

    // Handle symbol references
    if needs_reference(spec.operands) {
        refs.push(create_ref(&instr, address, spec));
    }

    // Encode based on format
    let encoded = encode_instruction(spec, rd, rs1, rs2, imm)?;
    Ok(vec![encoded])
}

fn encode_instruction(
    spec: &InstructionSpec,
    rd: Reg,
    rs1: Reg,
    rs2: Reg,
    imm: u32,
) -> Result<MachineInstruction> {
    let opcode: u32 = spec.opcode;
    let rd_bits: u32 = rd.into();
    let rs1_bits: u32 = rs1.into();
    let rs2_bits: u32 = rs2.into();

    let word = match spec.format {
        Format::R => {
            let funct3 = spec.funct3.unwrap();
            let funct7 = spec.funct7.unwrap();
            opcode | (rd_bits << 7) | (funct3 << 12) | (rs1_bits << 15)
                   | (rs2_bits << 20) | (funct7 << 25)
        }
        Format::I => {
            let funct3 = spec.funct3.unwrap();
            opcode | (rd_bits << 7) | (funct3 << 12) | (rs1_bits << 15) | (imm << 20)
        }
        Format::S => {
            let funct3 = spec.funct3.unwrap();
            let imm_low = imm & 0x1f;
            let imm_high = (imm >> 5) & 0x7f;
            opcode | (imm_low << 7) | (funct3 << 12) | (rs1_bits << 15)
                   | (rs2_bits << 20) | (imm_high << 25)
        }
        Format::B => encode_b_type(opcode, spec.funct3.unwrap(), rs1_bits, rs2_bits, imm),
        Format::U => opcode | (rd_bits << 7) | (imm << 12),
        Format::J => encode_j_type(opcode, rd_bits, imm),
    };

    Ok(MachineInstruction::from(word))
}
```

**Before: 210 lines of repetitive match arms**
**After: ~50 lines of generic encoding logic**

### Executor (After Refactoring)

```rust
pub fn execute_next(&mut self, sys: &mut impl Sys) -> Result<()> {
    let pc = self.pc;
    let word = self.mem.get(pc);
    let instruction = decode_instruction(word)?;

    self.pc += 4; // Default: advance to next instruction

    let spec = lookup_by_encoding(word)
        .ok_or(anyhow!("illegal instruction at pc={pc:04x}"))?;

    // Decode operands from the instruction word
    let rd = instruction.rd;
    let rs1_val = self.regs.get(instruction.rs1);
    let rs2_val = self.regs.get(instruction.rs2);
    let imm = instruction.imm;

    match spec.behavior {
        ExecBehavior::AluReg(f) => {
            let result = f(rs1_val, rs2_val);
            self.regs.set(rd, result);
        }
        ExecBehavior::AluImm(f) => {
            let result = f(rs1_val, imm);
            self.regs.set(rd, result);
        }
        ExecBehavior::Branch(condition) => {
            if condition(rs1_val, rs2_val) {
                self.pc = pc.wrapping_add(imm);
            }
        }
        ExecBehavior::Load(load_type) => {
            let addr = rs1_val.wrapping_add(imm);
            let value = match load_type {
                LoadType::Byte => sign_extend_8(self.mem.get_byte(addr)),
                LoadType::Half => sign_extend_16(self.mem.get_half(addr)),
                LoadType::Word => self.mem.get_word(addr),
                LoadType::ByteU => self.mem.get_byte(addr) as u32,
                LoadType::HalfU => self.mem.get_half(addr) as u32,
            };
            self.regs.set(rd, value);
        }
        ExecBehavior::Store(store_type) => {
            let addr = rs1_val.wrapping_add(imm);
            match store_type {
                StoreType::Byte => self.mem.set_byte(addr, rs2_val as u8),
                StoreType::Half => self.mem.set_half(addr, rs2_val as u16),
                StoreType::Word => self.mem.set_word(addr, rs2_val),
            }
        }
        ExecBehavior::Jump => {
            self.regs.set(rd, pc + 4);
            self.pc = pc.wrapping_add(imm);
        }
        ExecBehavior::JumpReg => {
            self.regs.set(rd, pc + 4);
            self.pc = (rs1_val.wrapping_add(imm)) & !1;
        }
        ExecBehavior::UpperImm => {
            self.regs.set(rd, imm << 12);
        }
        ExecBehavior::AddPcImm => {
            self.regs.set(rd, pc + (imm << 12));
        }
        ExecBehavior::Syscall => {
            self.handle_syscall(sys)?;
        }
        ExecBehavior::Break => {
            bail!("breakpoint at pc={pc:04x}");
        }
    }

    Ok(())
}
```

**Before: 52 lines of instruction-specific match arms**
**After: ~60 lines of generic execution logic (handles ALL instructions)**

---

## Benefits

### 1. **Maintainability**
- Single source of truth for instruction definitions
- Adding new instruction: change 1 line instead of 9 locations
- Less code to maintain (~400 lines → ~200 lines)

### 2. **Correctness**
- Harder to make mistakes (encode/decode mismatch, etc.)
- Format specifications prevent invalid combinations
- Compile-time enforcement of structure

### 3. **Testability**
- Can iterate over entire instruction set programmatically
- Easy to write property-based tests
- Can auto-generate test cases from the table

```rust
#[test]
fn all_instructions_roundtrip() {
    for spec in INSTRUCTION_SET {
        let encoded = encode_test_instruction(spec);
        let decoded = decode_instruction(encoded).unwrap();
        let re_encoded = encode_instruction(decoded).unwrap();
        assert_eq!(encoded, re_encoded, "Failed for {}", spec.mnemonic);
    }
}
```

### 4. **Documentation**
- The instruction table IS the documentation
- Easy to see all supported instructions at a glance
- Clear mapping to RISC-V spec

### 5. **Extensibility**
- Easy to add new instruction sets (RV64I, M extension, etc.)
- Can add metadata (cycle counts, instruction category, etc.)
- Could generate documentation from the table

### 6. **Performance**
- Lookup by mnemonic can use HashMap for O(1)
- Lookup by encoding can use decision tree or perfect hash
- Current linear scan is fine for 40 instructions

---

## Tradeoffs

### Potential Downsides

1. **Function Pointers**: Slight performance overhead vs inline match
   - **Mitigation**: Modern CPUs handle this well, likely negligible
   - **Alternative**: Could use macros to generate match arms from table

2. **Indirection**: Less obvious what each instruction does
   - **Mitigation**: Well-named behavior functions, good documentation
   - **Upside**: Forces separation of concerns

3. **Initial Complexity**: More upfront design
   - **Upside**: Pays dividends as instruction count grows

4. **Learning Curve**: New contributors need to understand the system
   - **Mitigation**: This document + code comments
   - **Upside**: Once understood, contributions are easier

---

## Migration Strategy

### Phase 1: Infrastructure (No Breaking Changes)
1. Create `src/instruction_set.rs` module
2. Define core types: `Format`, `OperandPattern`, `ExecBehavior`
3. Define `InstructionSpec` struct
4. Create `INSTRUCTION_SET` table with current 9 instructions
5. Add lookup functions
6. Add tests for the table itself

### Phase 2: Refactor Components (One at a Time)
1. **Executor First** (easiest, most localized)
   - Rewrite `Machine::execute_next` to use behavior functions
   - Keep old Opcode match as fallback
   - Test thoroughly
   - Remove fallback once confident

2. **Parser Second**
   - Add generic operand parsing
   - Use `OperandPattern` to drive parsing
   - Keep old match as fallback initially
   - Test thoroughly
   - Remove fallback

3. **Assembler Third** (most complex)
   - Add generic encoding functions for each Format
   - Rewrite `assemble_instruction` to use specs
   - Test thoroughly with existing test suite
   - Remove old code

4. **Encoder/Decoder Fourth**
   - Extract encoding/decoding to format-based functions
   - Use Format enum to drive logic
   - Test roundtrip encoding

### Phase 3: Expand Instruction Set
1. Add remaining RV32I instructions to table (30+ more)
2. Test each addition with isolated tests
3. Add example programs using new instructions
4. Update documentation

### Phase 4: Advanced Features
1. Add pseudo-instruction support (li, la, mv, etc.)
   - Mark with `is_pseudo: true`
   - Add expansion logic in assembler
2. Add instruction metadata (description, category, etc.)
3. Consider auto-generating parts of docs from table

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn instruction_table_is_valid() {
    for spec in INSTRUCTION_SET {
        // Ensure opcode is 7 bits
        assert!(spec.opcode <= 0x7f);

        // Ensure funct3 is 3 bits if present
        if let Some(f3) = spec.funct3 {
            assert!(f3 <= 0x7);
        }

        // Ensure funct7 is 7 bits if present
        if let Some(f7) = spec.funct7 {
            assert!(f7 <= 0x7f);
        }

        // Ensure format matches operand pattern
        match spec.format {
            Format::R => assert_eq!(spec.operands, OperandPattern::RdRs1Rs2),
            // ... other validations
        }
    }
}

#[test]
fn no_duplicate_mnemonics() {
    let mut seen = HashSet::new();
    for spec in INSTRUCTION_SET {
        assert!(seen.insert(spec.mnemonic),
                "Duplicate mnemonic: {}", spec.mnemonic);
    }
}

#[test]
fn all_instructions_have_unique_encoding() {
    let mut encodings = HashSet::new();
    for spec in INSTRUCTION_SET {
        let key = (spec.opcode, spec.funct3, spec.funct7);
        assert!(encodings.insert(key),
                "Duplicate encoding for: {}", spec.mnemonic);
    }
}
```

### Integration Tests
```rust
#[test]
fn data_driven_parsing() {
    for spec in INSTRUCTION_SET {
        if spec.is_pseudo { continue; }

        let asm = generate_asm_example(spec);
        let tokens = tokenize(&asm);
        let parsed = parse(tokens).unwrap();

        // Verify it parsed correctly
        assert_instruction_matches(parsed, spec);
    }
}

#[test]
fn data_driven_execution() {
    for spec in INSTRUCTION_SET {
        if spec.is_pseudo { continue; }

        let mut machine = Machine::new();
        let instruction = create_test_instruction(spec);
        machine.execute(instruction).unwrap();

        // Verify behavior
        verify_execution(machine, spec);
    }
}
```

---

## Future Extensions

### 1. Additional Instruction Sets
```rust
pub static M_EXTENSION: &[InstructionSpec] = &[
    // MUL, MULH, MULHSU, MULHU, DIV, DIVU, REM, REMU
];

pub static A_EXTENSION: &[InstructionSpec] = &[
    // LR.W, SC.W, AMOSWAP.W, AMOADD.W, ...
];

pub static F_EXTENSION: &[InstructionSpec] = &[
    // FLW, FSW, FADD.S, FSUB.S, ...
];
```

### 2. Rich Metadata
```rust
pub struct InstructionSpec {
    // ... existing fields ...
    pub description: &'static str,
    pub category: Category,
    pub cycles: u32,  // For performance simulation
    pub privilege: PrivilegeLevel,
}

pub enum Category {
    Arithmetic,
    Logical,
    Shift,
    Compare,
    Branch,
    Jump,
    Load,
    Store,
    System,
}
```

### 3. Automatic Documentation Generation
```rust
pub fn generate_instruction_reference() -> String {
    let mut doc = String::from("# Instruction Reference\n\n");

    for spec in INSTRUCTION_SET {
        doc.push_str(&format!("## {}\n", spec.mnemonic));
        doc.push_str(&format!("**Format**: {:?}\n", spec.format));
        doc.push_str(&format!("**Syntax**: {}\n", syntax_from_pattern(spec.operands)));
        doc.push_str(&format!("**Description**: {}\n\n", spec.description));
    }

    doc
}
```

### 4. Disassembler Improvements
```rust
pub fn disassemble_word(word: u32) -> String {
    if let Some(spec) = lookup_by_encoding(word) {
        let (rd, rs1, rs2, imm) = decode_operands(word, spec.format);
        format_instruction(spec, rd, rs1, rs2, imm)
    } else {
        format!(".word 0x{:08x}", word)
    }
}
```

---

## Alternative Approach: Askama-Style Attribute Macro + TOML

### The Ultimate Developer Experience

While the pure Rust table approach works well, we can achieve an even better experience by combining:
1. **TOML files** for instruction definitions (easy to edit, external tooling)
2. **Procedural macros** for compile-time code generation (zero runtime cost)
3. **Attribute macro syntax** like Askama (ergonomic, familiar)

### Architecture Overview

```
┌─────────────────────────────┐
│  instructions/rv32i.toml    │  ← Human-editable TOML
│  [[instruction]]            │
│  mnemonic = "add"           │
│  format = "R"               │
│  opcode = 0b0110011         │
│  ...                        │
└──────────────┬──────────────┘
               │
               │ Proc macro reads at compile time
               ↓
┌─────────────────────────────┐
│  #[derive(InstructionSet)]  │  ← User code
│  #[instruction_set(         │
│    path = "rv32i.toml"      │
│  )]                         │
│  pub struct RV32I;          │
└──────────────┬──────────────┘
               │
               │ Macro generates impl
               ↓
┌─────────────────────────────┐
│  impl RV32I {               │  ← Generated Rust code
│    pub fn lookup(...) {...} │
│    pub fn all() {...}       │
│  }                          │
└─────────────────────────────┘
```

### User-Facing API

```rust
use rmachine_macros::InstructionSet;

/// RV32I Base Integer Instruction Set
#[derive(InstructionSet)]
#[instruction_set(path = "instructions/rv32i.toml")]
pub struct RV32I;

/// RV32M Multiply/Divide Extension
#[derive(InstructionSet)]
#[instruction_set(path = "instructions/m_extension.toml")]
pub struct MExtension;

// Now you can use them like this:
fn example() {
    // Lookup by mnemonic
    let add = RV32I::lookup("add").unwrap();
    assert_eq!(add.mnemonic, "add");
    assert_eq!(add.format, Format::R);

    // Get all instructions
    let all = RV32I::all();
    assert_eq!(all.len(), RV32I::COUNT);

    // Lookup by encoding
    let spec = RV32I::lookup_encoding(0x00A50533).unwrap();

    // Iterate over instructions
    for instr in RV32I::iter() {
        println!("{}: {:?}", instr.mnemonic, instr.format);
    }

    // Check support
    assert!(RV32I::supports("add"));
    assert!(!RV32I::supports("mul")); // M extension
}
```

### TOML Instruction Definition Format

```toml
# instructions/rv32i.toml

version = "2.2"
description = "RISC-V 32-bit Base Integer Instructions"

# R-Type Instructions (Register-Register)
[[instruction]]
mnemonic = "add"
format = "R"
opcode = 0b0110011
funct3 = 0b000
funct7 = 0b0000000
operands = "RdRs1Rs2"
behavior = "AluReg"
alu_op = "add"
description = "Addition: rd = rs1 + rs2"
category = "Arithmetic"

[[instruction]]
mnemonic = "sub"
format = "R"
opcode = 0b0110011
funct3 = 0b000
funct7 = 0b0100000
operands = "RdRs1Rs2"
behavior = "AluReg"
alu_op = "sub"
description = "Subtraction: rd = rs1 - rs2"
category = "Arithmetic"

[[instruction]]
mnemonic = "and"
format = "R"
opcode = 0b0110011
funct3 = 0b111
funct7 = 0b0000000
operands = "RdRs1Rs2"
behavior = "AluReg"
alu_op = "and"
description = "Bitwise AND: rd = rs1 & rs2"
category = "Logical"

# I-Type Instructions (Immediate)
[[instruction]]
mnemonic = "addi"
format = "I"
opcode = 0b0010011
funct3 = 0b000
operands = "RdRs1Imm"
behavior = "AluImm"
alu_op = "add"
description = "Add immediate: rd = rs1 + imm"
category = "Arithmetic"

[[instruction]]
mnemonic = "lb"
format = "I"
opcode = 0b0000011
funct3 = 0b000
operands = "RdOffsetRs1"
behavior = "Load"
load_type = "Byte"
description = "Load byte (sign-extended)"
category = "Memory"

# B-Type Instructions (Branch)
[[instruction]]
mnemonic = "beq"
format = "B"
opcode = 0b1100011
funct3 = 0b000
operands = "Rs1Rs2Symbol"
behavior = "Branch"
branch_cond = "eq"
description = "Branch if equal"
category = "Control"

[[instruction]]
mnemonic = "bne"
format = "B"
opcode = 0b1100011
funct3 = 0b001
operands = "Rs1Rs2Symbol"
behavior = "Branch"
branch_cond = "ne"
description = "Branch if not equal"
category = "Control"

# U-Type Instructions (Upper Immediate)
[[instruction]]
mnemonic = "lui"
format = "U"
opcode = 0b0110111
operands = "RdImm"
behavior = "UpperImm"
description = "Load upper immediate: rd = imm << 12"
category = "Arithmetic"

# J-Type Instructions (Jump)
[[instruction]]
mnemonic = "jal"
format = "J"
opcode = 0b1101111
operands = "RdSymbol"
behavior = "Jump"
description = "Jump and link: rd = pc + 4; pc += imm"
category = "Control"

# System Instructions
[[instruction]]
mnemonic = "ecall"
format = "I"
opcode = 0b1110011
funct3 = 0b000
operands = "None"
behavior = "Syscall"
description = "Environment call (system call)"
category = "System"
```

### Procedural Macro Implementation

#### Project Structure

```
rmachine/
├── Cargo.toml
├── instructions/
│   ├── rv32i.toml
│   ├── m_extension.toml
│   └── a_extension.toml
├── src/
│   └── instruction_set.rs
└── rmachine-macros/          ← Separate proc macro crate
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

#### `rmachine-macros/Cargo.toml`

```toml
[package]
name = "rmachine-macros"
version = "0.1.0"
edition = "2024"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

#### `rmachine-macros/src/lib.rs`

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta, Lit};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct InstructionSetSpec {
    version: String,
    description: String,
    instruction: Vec<InstructionDef>,
}

#[derive(Debug, Deserialize)]
struct InstructionDef {
    mnemonic: String,
    format: String,
    opcode: u32,
    funct3: Option<u32>,
    funct7: Option<u32>,
    operands: String,
    behavior: String,

    // Optional behavior-specific fields
    alu_op: Option<String>,
    branch_cond: Option<String>,
    load_type: Option<String>,
    store_type: Option<String>,

    description: String,
    category: String,
}

#[proc_macro_derive(InstructionSet, attributes(instruction_set))]
pub fn derive_instruction_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the struct name
    let struct_name = &input.ident;

    // Find the #[instruction_set(path = "...")] attribute
    let path = extract_path_from_attrs(&input.attrs)
        .expect("Missing #[instruction_set(path = \"...\")] attribute");

    // Read and parse TOML
    let toml_content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));

    let spec: InstructionSetSpec = toml::from_str(&toml_content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e));

    // Validate instruction set
    validate_instruction_set(&spec);

    // Generate the implementation
    generate_impl(struct_name, &spec)
}

fn extract_path_from_attrs(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("instruction_set") {
            if let Ok(Meta::NameValue(nv)) = attr.parse_args::<Meta>() {
                if nv.path.is_ident("path") {
                    if let syn::Expr::Lit(lit) = &nv.value {
                        if let Lit::Str(s) = &lit.lit {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
    }
    None
}

fn validate_instruction_set(spec: &InstructionSetSpec) {
    for instr in &spec.instruction {
        // Validate opcode is 7 bits
        assert!(instr.opcode <= 0x7f,
                "Invalid opcode for {}: must be 7 bits", instr.mnemonic);

        // Validate funct3 is 3 bits if present
        if let Some(f3) = instr.funct3 {
            assert!(f3 <= 0x7,
                    "Invalid funct3 for {}: must be 3 bits", instr.mnemonic);
        }

        // Validate funct7 is 7 bits if present
        if let Some(f7) = instr.funct7 {
            assert!(f7 <= 0x7f,
                    "Invalid funct7 for {}: must be 7 bits", instr.mnemonic);
        }

        // Validate behavior has required fields
        match instr.behavior.as_str() {
            "AluReg" | "AluImm" => {
                assert!(instr.alu_op.is_some(),
                        "{} requires alu_op field", instr.mnemonic);
            }
            "Branch" => {
                assert!(instr.branch_cond.is_some(),
                        "{} requires branch_cond field", instr.mnemonic);
            }
            "Load" => {
                assert!(instr.load_type.is_some(),
                        "{} requires load_type field", instr.mnemonic);
            }
            "Store" => {
                assert!(instr.store_type.is_some(),
                        "{} requires store_type field", instr.mnemonic);
            }
            _ => {}
        }
    }
}

fn generate_impl(struct_name: &syn::Ident, spec: &InstructionSetSpec) -> TokenStream {
    let instruction_specs = spec.instruction.iter().map(|instr| {
        let mnemonic = &instr.mnemonic;
        let opcode = instr.opcode;
        let format_ident = format_ident(&instr.format);
        let operands_ident = format_ident(&instr.operands);

        let funct3 = match instr.funct3 {
            Some(f3) => quote! { Some(#f3) },
            None => quote! { None },
        };

        let funct7 = match instr.funct7 {
            Some(f7) => quote! { Some(#f7) },
            None => quote! { None },
        };

        let behavior = generate_behavior(instr);

        quote! {
            InstructionSpec {
                mnemonic: #mnemonic,
                opcode: #opcode,
                funct3: #funct3,
                funct7: #funct7,
                format: Format::#format_ident,
                operands: OperandPattern::#operands_ident,
                behavior: #behavior,
                is_pseudo: false,
            }
        }
    });

    let count = spec.instruction.len();

    // Generate a constant array name based on struct name
    let const_name = format_ident!("{}_INSTRUCTIONS",
                                   struct_name.to_string().to_uppercase());

    let expanded = quote! {
        // Private constant holding the instruction set
        const #const_name: &[InstructionSpec] = &[
            #(#instruction_specs),*
        ];

        impl #struct_name {
            /// Total number of instructions in this set
            pub const COUNT: usize = #count;

            /// Get all instructions in this set
            pub const fn all() -> &'static [InstructionSpec] {
                #const_name
            }

            /// Lookup instruction by mnemonic
            pub fn lookup(mnemonic: &str) -> Option<&'static InstructionSpec> {
                #const_name.iter().find(|spec| spec.mnemonic == mnemonic)
            }

            /// Lookup instruction by encoded word
            pub fn lookup_encoding(word: u32) -> Option<&'static InstructionSpec> {
                let opcode = word & 0x7f;
                let funct3 = (word >> 12) & 0x7;
                let funct7 = (word >> 25) & 0x7f;

                #const_name.iter().find(|spec| {
                    spec.opcode == opcode &&
                    spec.funct3.map_or(true, |f3| f3 == funct3) &&
                    spec.funct7.map_or(true, |f7| f7 == funct7)
                })
            }

            /// Get instruction by index
            pub fn get(index: usize) -> Option<&'static InstructionSpec> {
                #const_name.get(index)
            }

            /// Iterate over all instructions
            pub fn iter() -> impl Iterator<Item = &'static InstructionSpec> {
                #const_name.iter()
            }

            /// Check if a mnemonic is supported
            pub fn supports(mnemonic: &str) -> bool {
                Self::lookup(mnemonic).is_some()
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_behavior(instr: &InstructionDef) -> proc_macro2::TokenStream {
    match instr.behavior.as_str() {
        "AluReg" => {
            let op = format_ident(instr.alu_op.as_ref().unwrap());
            quote! { ExecBehavior::AluReg(alu_#op) }
        }
        "AluImm" => {
            let op = format_ident(instr.alu_op.as_ref().unwrap());
            quote! { ExecBehavior::AluImm(alu_#op) }
        }
        "Branch" => {
            let cond = format_ident(instr.branch_cond.as_ref().unwrap());
            quote! { ExecBehavior::Branch(branch_#cond) }
        }
        "Load" => {
            let load_type = format_ident(instr.load_type.as_ref().unwrap());
            quote! { ExecBehavior::Load(LoadType::#load_type) }
        }
        "Store" => {
            let store_type = format_ident(instr.store_type.as_ref().unwrap());
            quote! { ExecBehavior::Store(StoreType::#store_type) }
        }
        "Jump" => quote! { ExecBehavior::Jump },
        "JumpReg" => quote! { ExecBehavior::JumpReg },
        "UpperImm" => quote! { ExecBehavior::UpperImm },
        "AddPcImm" => quote! { ExecBehavior::AddPcImm },
        "Syscall" => quote! { ExecBehavior::Syscall },
        other => panic!("Unknown behavior: {}", other),
    }
}

fn format_ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, proc_macro2::Span::call_site())
}
```

#### `src/instruction_set.rs` (User Code)

```rust
use rmachine_macros::InstructionSet;

// All the type definitions (same as pure Rust approach)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    R, I, S, B, U, J,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandPattern {
    RdRs1Rs2,
    RdRs1Imm,
    RdImm,
    Rs1Rs2Symbol,
    RdSymbol,
    RdOffsetRs1,
    Rs2OffsetRs1,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecBehavior {
    AluReg(fn(u32, u32) -> u32),
    AluImm(fn(u32, u32) -> u32),
    Branch(fn(u32, u32) -> bool),
    Load(LoadType),
    Store(StoreType),
    Jump,
    JumpReg,
    UpperImm,
    AddPcImm,
    Syscall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadType { Byte, Half, Word, ByteU, HalfU }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreType { Byte, Half, Word }

pub struct InstructionSpec {
    pub mnemonic: &'static str,
    pub opcode: u32,
    pub funct3: Option<u32>,
    pub funct7: Option<u32>,
    pub format: Format,
    pub operands: OperandPattern,
    pub behavior: ExecBehavior,
    pub is_pseudo: bool,
}

// ALU operations (can't be in TOML - must be Rust code)
fn alu_add(a: u32, b: u32) -> u32 { a.wrapping_add(b) }
fn alu_sub(a: u32, b: u32) -> u32 { a.wrapping_sub(b) }
fn alu_and(a: u32, b: u32) -> u32 { a & b }
fn alu_or(a: u32, b: u32) -> u32 { a | b }
fn alu_xor(a: u32, b: u32) -> u32 { a ^ b }
fn alu_sll(a: u32, b: u32) -> u32 { a << (b & 0x1f) }
fn alu_srl(a: u32, b: u32) -> u32 { a >> (b & 0x1f) }
fn alu_sra(a: u32, b: u32) -> u32 { ((a as i32) >> (b & 0x1f)) as u32 }
fn alu_slt(a: u32, b: u32) -> u32 { if (a as i32) < (b as i32) { 1 } else { 0 } }
fn alu_sltu(a: u32, b: u32) -> u32 { if a < b { 1 } else { 0 } }

// Branch conditions
fn branch_eq(a: u32, b: u32) -> bool { a == b }
fn branch_ne(a: u32, b: u32) -> bool { a != b }
fn branch_lt(a: u32, b: u32) -> bool { (a as i32) < (b as i32) }
fn branch_ge(a: u32, b: u32) -> bool { (a as i32) >= (b as i32) }
fn branch_ltu(a: u32, b: u32) -> bool { a < b }
fn branch_geu(a: u32, b: u32) -> bool { a >= b }

// ============================================
// Define instruction sets using the macro
// ============================================

/// RV32I Base Integer Instruction Set
#[derive(InstructionSet)]
#[instruction_set(path = "instructions/rv32i.toml")]
pub struct RV32I;

/// RV32M Multiply/Divide Extension (future)
#[derive(InstructionSet)]
#[instruction_set(path = "instructions/m_extension.toml")]
pub struct MExtension;

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rv32i_has_instructions() {
        assert!(RV32I::COUNT > 0);
        assert_eq!(RV32I::all().len(), RV32I::COUNT);
    }

    #[test]
    fn can_lookup_add() {
        let add = RV32I::lookup("add").unwrap();
        assert_eq!(add.mnemonic, "add");
        assert_eq!(add.format, Format::R);
    }

    #[test]
    fn supports_check() {
        assert!(RV32I::supports("add"));
        assert!(!RV32I::supports("invalid"));
    }
}
```

### Usage in Parser/Assembler/Executor

```rust
// In parser.rs
fn instruction(&mut self, name: String) -> Result<Line> {
    let spec = RV32I::lookup(&name)
        .ok_or(anyhow!("unknown instruction: {}", name))?;

    let operands = self.parse_operands(spec.operands)?;
    Ok(Line::Instruction(Instruction { name, operands }))
}

// In assembler.rs
fn assemble_instruction(instr: ast::Instruction) -> Result<Vec<Word>> {
    let spec = RV32I::lookup(&instr.name)?;
    encode_by_format(spec.format, spec.opcode, &instr.operands)
}

// In machine.rs (executor)
pub fn execute(&mut self, word: u32) -> Result<()> {
    let spec = RV32I::lookup_encoding(word)?;

    match spec.behavior {
        ExecBehavior::AluReg(f) => { /* ... */ }
        ExecBehavior::Branch(cond) => { /* ... */ }
        // ...
    }
}
```

### Advanced: Composing Multiple ISAs

```rust
/// Lookup across multiple instruction sets
pub fn lookup_any(mnemonic: &str) -> Option<&'static InstructionSpec> {
    RV32I::lookup(mnemonic)
        .or_else(|| MExtension::lookup(mnemonic))
        .or_else(|| AExtension::lookup(mnemonic))
}

/// Combined instruction iterator
pub fn all_instructions() -> impl Iterator<Item = &'static InstructionSpec> {
    RV32I::iter()
        .chain(MExtension::iter())
        .chain(AExtension::iter())
}
```

### Comparison: All Approaches

| Feature | Pure Rust | build.rs + TOML | Proc Macro + TOML |
|---------|-----------|-----------------|-------------------|
| **Ease of editing** | Medium (Rust code) | High (TOML) | High (TOML) |
| **Compile-time safety** | ✅ Yes | ✅ Yes | ✅ Yes |
| **Runtime cost** | ✅ Zero | ✅ Zero | ✅ Zero |
| **IDE support** | ✅ Excellent | ⚠️ Medium | ✅ Excellent |
| **Error messages** | ✅ Great | ⚠️ Build script errors | ✅ Great |
| **External tooling** | ❌ No | ✅ Yes (TOML) | ✅ Yes (TOML) |
| **Code generation** | ❌ Manual | ✅ Automated | ✅ Automated |
| **Multiple ISAs** | Verbose | Clean | ✅ Very clean |
| **cargo expand** | N/A | ❌ No | ✅ Yes |
| **Build complexity** | ✅ Simple | ⚠️ build.rs | ⚠️ Proc macro crate |
| **Learning curve** | ✅ Low | Medium | Medium |

### Advantages of Askama-Style Approach

1. **Best Developer Experience**
   - Familiar syntax (like `#[derive(Debug)]`)
   - Clean separation: TOML for data, Rust for behavior
   - Easy to add new ISA: just create new TOML + one struct

2. **Compile-Time Everything**
   - TOML parsing happens during compilation
   - Invalid TOML = build failure with clear errors
   - Generated code is optimized by rustc

3. **Type-Safe and Namespaced**
   - Each ISA is its own type
   - `RV32I::lookup()` vs `MExtension::lookup()`
   - Can't mix up instruction sets

4. **Excellent Tooling**
   - `cargo expand` shows generated code
   - rust-analyzer understands proc macros
   - Error messages point to correct locations

5. **External Tooling Friendly**
   - TOML can be validated by external tools
   - Can generate docs from TOML
   - Could share TOML specs with other projects

6. **Testable**
   - Each ISA independently testable
   - Can verify TOML validity at compile time
   - Property-based tests across all ISAs

### When to Use Each Approach

**Pure Rust Table:**
- Small projects (< 20 instructions)
- Want simplest possible setup
- No need for external tooling

**build.rs + TOML:**
- Need maximum control over generation
- Want to generate multiple outputs from TOML
- Already have build.rs for other reasons

**Proc Macro + TOML (Recommended):**
- Growing instruction set (40+ instructions)
- Multiple ISAs or extensions
- Want best developer experience
- Educational project showing modern Rust

---

## Updated Recommendation

For the rmachine project, we recommend the **Askama-style attribute macro + TOML** approach because:

1. **Educational Value**: Shows real-world proc macro usage, aligning with project goals
2. **Scalability**: Easily add 30+ RV32I instructions, then M/A/F extensions
3. **Clarity**: TOML is self-documenting and easy for contributors to understand
4. **Industry Standard**: Same pattern as Askama, serde, many popular crates
5. **Future-Proof**: Easy to extend with metadata, validation, doc generation

The upfront cost of creating the proc macro crate pays off immediately when adding the remaining RV32I instructions, and continues to provide value as the project grows.

---

## Conclusion

This data-driven architecture transforms instruction handling from **scattered procedural code** (9 locations per instruction) to **declarative data** (1 line per instruction).

**Current state:**
- 9 instructions × 9 locations = 81 points of maintenance
- ~500 lines of repetitive code

**After refactoring (with proc macro):**
- 40+ instructions × 1 TOML entry = 40 points of maintenance
- ~200 lines of generic code + ~100 lines proc macro + TOML data
- **Vastly easier to extend, test, and maintain**

The upfront investment in designing this system pays off immediately when adding the remaining 30+ RV32I instructions, and continues to pay dividends with every future extension.
