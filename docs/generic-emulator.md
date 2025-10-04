# Generic Architecture-Agnostic Emulator Design

## Vision

Create an emulator framework that can simulate **any** instruction set architecture by loading its specification from data files, rather than hardcoding CPU-specific logic. This would allow rmachine to evolve from a RISC-V emulator into a universal emulation platform.

### Goals

1. **Support radically different architectures**: 32-bit RISC-V, 8-bit 6502, ARM, x86, custom/fantasy architectures
2. **Zero code changes for new architectures**: Adding a new CPU = writing a TOML file
3. **Maintain high performance**: Close to native speed for hot paths
4. **Educational value**: Architecture specifications serve as living documentation
5. **Enable experimentation**: Easy to create custom ISAs for teaching or research

### Non-Goals

- Cycle-accurate timing simulation (focus on functional correctness)
- Hardware peripherals modeling (focus on CPU core)
- Full system emulation (focus on instruction execution)

---

## The Challenge: Architectural Diversity

### Comparison of Representative Architectures

| Feature | RISC-V RV32I | MOS 6502 | ARM Cortex-M0 | x86-64 |
|---------|--------------|----------|---------------|--------|
| **Word Size** | 32-bit | 8-bit | 32-bit | 64-bit |
| **Register Count** | 32 general-purpose | 3 special (A, X, Y) | 16 general-purpose | 16 general-purpose |
| **Register Width** | 32-bit uniform | 8-bit (A, X, Y), 16-bit (PC, SP) | 32-bit uniform | 64-bit uniform |
| **Instruction Length** | Fixed 32-bit | Variable 1-3 bytes | Variable 16/32-bit (Thumb) | Variable 1-15 bytes |
| **Instruction Formats** | 6 formats (R, I, S, B, U, J) | 13 addressing modes | Multiple | Hundreds of variations |
| **Endianness** | Little | Little | Little (configurable) | Little |
| **Memory Model** | Load-store only | Memory-memory ops allowed | Load-store only | Memory-memory ops allowed |
| **Status Flags** | None (explicit comparisons) | 7 flags (N, V, Z, C, I, D, B) | 4 flags (N, Z, C, V) | Many flags (ZF, SF, CF, OF, etc.) |
| **PC Increment** | Always +4 | Variable by addressing mode | Variable by instruction | Variable by instruction |
| **Addressing Modes** | Base + offset | 13 modes (immediate, absolute, indexed, indirect, etc.) | Limited | Many complex modes |

### Key Insight

Despite vast differences, all CPUs share core concepts:
- **State**: Registers, flags, memory, PC
- **Fetch-Decode-Execute**: Read instruction, interpret it, update state
- **Data Flow**: Instructions move/transform data between registers and memory

The challenge is to **abstract these commonalities** while allowing **architecture-specific details** to be specified declaratively.

---

## Core Abstraction Layers

To support diverse architectures, we need to parameterize:

### 1. Data Representation
```rust
pub struct ArchitectureSpec {
    /// Fundamental word size in bits (8, 16, 32, 64)
    pub word_size: usize,

    /// Byte ordering
    pub endianness: Endianness,

    /// Address space width in bits
    pub address_width: usize,
}

pub enum Endianness {
    Little,
    Big,
    BiEndian,  // Architecture-configurable (ARM)
}
```

### 2. Register File
```rust
pub struct RegisterSpec {
    /// Number of general-purpose registers (0 for architectures without them)
    pub general_purpose_count: usize,

    /// Width of general-purpose registers in bits
    pub general_purpose_width: usize,

    /// Named special-purpose registers
    pub special: Vec<SpecialRegister>,
}

pub struct SpecialRegister {
    pub name: String,           // "PC", "SP", "A", "X", etc.
    pub width: usize,           // Bits
    pub initial_value: u64,     // Reset value
    pub readable: bool,         // Can be read by instructions
    pub writable: bool,         // Can be written by instructions
}
```

### 3. Flags/Status Register
```rust
pub struct FlagSpec {
    /// Whether architecture has a status/flags register
    pub has_flags: bool,

    /// Individual flag definitions
    pub flags: Vec<FlagDef>,
}

pub struct FlagDef {
    pub name: String,      // "Z", "N", "C", "V", etc.
    pub bit_position: u8,
    pub description: String,
}
```

### 4. Instruction Formats
```rust
pub struct InstructionFormat {
    /// Format name (e.g., "R-Type", "Immediate", "ZeroPage")
    pub name: String,

    /// Total width in bits
    pub width_bits: usize,

    /// Width in bytes (for variable-length instructions)
    pub width_bytes: usize,

    /// Field definitions
    pub fields: Vec<FieldDef>,
}

pub struct FieldDef {
    pub name: String,           // "opcode", "rd", "imm", etc.
    pub bits: Range<usize>,     // Bit range, e.g., 7:0
    pub signed: bool,           // Interpret as signed value
    pub sign_extend: bool,      // Sign-extend to full width
}
```

### 5. Memory Model
```rust
pub struct MemorySpec {
    /// Address space width
    pub address_width: usize,

    /// Whether byte-addressable (vs word-addressable)
    pub byte_addressable: bool,

    /// Alignment requirements
    pub alignment_required: bool,

    /// Memory mapped regions (optional)
    pub regions: Vec<MemoryRegion>,
}

pub struct MemoryRegion {
    pub name: String,
    pub start: u64,
    pub end: u64,
    pub read_only: bool,
}
```

---

## Architecture Definition Language

### TOML Schema

A complete architecture is defined in a TOML file with the following structure:

```toml
[architecture]
name = "Architecture Name"
word_size = 32              # bits
address_width = 32          # bits
endianness = "little"       # or "big" or "bi"
pc_increment = "fixed"      # or "variable"

# For fixed-length instruction architectures
[architecture.fixed_instruction]
size = 32  # bits

# Register specification
[registers]
general_purpose_count = 32
general_purpose_width = 32

[[registers.special]]
name = "pc"
width = 32
initial_value = 0
readable = false
writable = true

[[registers.special]]
name = "sp"
width = 32
initial_value = 0
readable = true
writable = true

# Flags specification (optional)
[flags]
has_flags = true
register_name = "status"

[[flags.bits]]
name = "Z"      # Zero
bit = 1
description = "Set when result is zero"

[[flags.bits]]
name = "N"      # Negative
bit = 7
description = "Set when result is negative"

# Memory model
[memory]
address_width = 32
byte_addressable = true
alignment_required = false

# Instruction formats
[[formats]]
name = "R-Type"
width_bits = 32
width_bytes = 4

[[formats.fields]]
name = "opcode"
bits = "6:0"
signed = false

[[formats.fields]]
name = "rd"
bits = "11:7"
signed = false

[[formats.fields]]
name = "funct3"
bits = "14:12"
signed = false

# ... more fields

# Instructions
[[instruction]]
mnemonic = "add"
format = "R-Type"
encoding = { opcode = 0b0110011, funct3 = 0b000, funct7 = 0b0000000 }
operands = ["rd", "rs1", "rs2"]
behavior = "..."  # See behavior options below
description = "Add two registers"
category = "Arithmetic"
```

### Complete RISC-V RV32I Example

```toml
# arch/riscv32i.toml

[architecture]
name = "RISC-V RV32I Base Integer ISA"
version = "2.2"
word_size = 32
address_width = 32
endianness = "little"
pc_increment = "fixed"

[architecture.fixed_instruction]
size = 32

[registers]
general_purpose_count = 32
general_purpose_width = 32

# x0 is hardwired to zero (handled specially)
[[registers.special]]
name = "pc"
width = 32
initial_value = 0
readable = false
writable = true

[flags]
has_flags = false  # RISC-V doesn't use status flags

[memory]
address_width = 32
byte_addressable = true
alignment_required = false

# R-Type format
[[formats]]
name = "R"
width_bits = 32
width_bytes = 4

[[formats.fields]]
name = "opcode"
bits = "6:0"

[[formats.fields]]
name = "rd"
bits = "11:7"

[[formats.fields]]
name = "funct3"
bits = "14:12"

[[formats.fields]]
name = "rs1"
bits = "19:15"

[[formats.fields]]
name = "rs2"
bits = "24:20"

[[formats.fields]]
name = "funct7"
bits = "31:25"

# I-Type format
[[formats]]
name = "I"
width_bits = 32
width_bytes = 4

[[formats.fields]]
name = "opcode"
bits = "6:0"

[[formats.fields]]
name = "rd"
bits = "11:7"

[[formats.fields]]
name = "funct3"
bits = "14:12"

[[formats.fields]]
name = "rs1"
bits = "19:15"

[[formats.fields]]
name = "imm"
bits = "31:20"
signed = true
sign_extend = true

# Instructions
[[instruction]]
mnemonic = "add"
format = "R"
encoding = { opcode = 0b0110011, funct3 = 0b000, funct7 = 0b0000000 }
pattern = "alu_binary_rrr"
operation = "add"
description = "Add rs1 and rs2, store in rd"

[[instruction]]
mnemonic = "addi"
format = "I"
encoding = { opcode = 0b0010011, funct3 = 0b000 }
pattern = "alu_binary_rri"
operation = "add"
description = "Add rs1 and immediate, store in rd"

# ... more instructions
```

### Complete 6502 Example

```toml
# arch/6502.toml

[architecture]
name = "MOS Technology 6502"
version = "1.0"
word_size = 8
address_width = 16
endianness = "little"
pc_increment = "variable"

[registers]
general_purpose_count = 0  # No general-purpose registers

[[registers.special]]
name = "A"  # Accumulator
width = 8
initial_value = 0
readable = true
writable = true

[[registers.special]]
name = "X"  # Index X
width = 8
initial_value = 0
readable = true
writable = true

[[registers.special]]
name = "Y"  # Index Y
width = 8
initial_value = 0
readable = true
writable = true

[[registers.special]]
name = "SP"  # Stack Pointer (page 1 only, so actually 0x0100 + SP)
width = 8
initial_value = 0xFF
readable = true
writable = true

[[registers.special]]
name = "PC"  # Program Counter
width = 16
initial_value = 0
readable = false
writable = true

[flags]
has_flags = true
register_name = "P"  # Processor Status

[[flags.bits]]
name = "C"
bit = 0
description = "Carry flag"

[[flags.bits]]
name = "Z"
bit = 1
description = "Zero flag"

[[flags.bits]]
name = "I"
bit = 2
description = "Interrupt disable"

[[flags.bits]]
name = "D"
bit = 3
description = "Decimal mode"

[[flags.bits]]
name = "B"
bit = 4
description = "Break command"

[[flags.bits]]
name = "V"
bit = 6
description = "Overflow flag"

[[flags.bits]]
name = "N"
bit = 7
description = "Negative flag"

[memory]
address_width = 16
byte_addressable = true
alignment_required = false

# 6502 Addressing Modes (these are like instruction formats)

[[formats]]
name = "Implied"
width_bits = 8
width_bytes = 1

[[formats.fields]]
name = "opcode"
bits = "7:0"

[[formats]]
name = "Immediate"
width_bits = 16
width_bytes = 2

[[formats.fields]]
name = "opcode"
bits = "7:0"

[[formats.fields]]
name = "operand"
bits = "15:8"

[[formats]]
name = "ZeroPage"
width_bits = 16
width_bytes = 2

[[formats.fields]]
name = "opcode"
bits = "7:0"

[[formats.fields]]
name = "address"
bits = "15:8"

[[formats]]
name = "Absolute"
width_bits = 24
width_bytes = 3

[[formats.fields]]
name = "opcode"
bits = "7:0"

[[formats.fields]]
name = "address_low"
bits = "15:8"

[[formats.fields]]
name = "address_high"
bits = "23:16"

# Instructions

[[instruction]]
mnemonic = "LDA"
format = "Immediate"
encoding = { opcode = 0xA9 }
effects = [
    { write = "A", value = "operand" },
    { flag = "Z", value = "A == 0" },
    { flag = "N", value = "(A & 0x80) != 0" },
    { write = "PC", value = "PC + 2" }
]
description = "Load accumulator with immediate value"

[[instruction]]
mnemonic = "ADC"
format = "Immediate"
encoding = { opcode = 0x69 }
behavior_lang = "simple_expr"
behavior = """
    temp = A + operand + get_flag(C);
    A = temp & 0xFF;
    set_flag(C, temp > 0xFF);
    set_flag(Z, A == 0);
    set_flag(N, (A & 0x80) != 0);
    set_flag(V, (~(A ^ operand) & (A ^ temp) & 0x80) != 0);
    PC = PC + 2;
"""
description = "Add with carry"

# ... more instructions
```

---

## Instruction Behavior Representation: Complete Options

The most critical design decision is how to represent what an instruction **does**. Here are all viable options:

### Option 1: Simple Expression DSL

**Concept**: Small imperative scripting language with C-like syntax.

```toml
[[instruction]]
mnemonic = "add"
behavior = """
    rd = rs1 + rs2;
    pc += 4;
"""

[[instruction]]
mnemonic = "beq"
behavior = """
    if (rs1 == rs2) {
        pc += imm;
    } else {
        pc += 4;
    }
"""

[[instruction]]
mnemonic = "adc"  # 6502 add with carry
behavior = """
    temp = A + operand + C;
    A = temp & 0xFF;
    C = (temp > 0xFF);
    Z = (A == 0);
    N = (A & 0x80);
    V = ((~(A ^ operand) & (A ^ temp) & 0x80) != 0);
    pc += 2;
"""
```

**Supported Operations**:
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Bitwise: `&`, `|`, `^`, `<<`, `>>`, `~`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Assignment: `=`, `+=`, `-=`, etc.
- Control flow: `if/else`
- Function calls: `set_flag(name, value)`, `sign_extend(value, bits)`

**Implementation**:
```rust
pub struct SimpleExprInterpreter {
    variables: HashMap<String, u64>,
    machine: MachineState,
}

impl SimpleExprInterpreter {
    pub fn execute(&mut self, script: &str) -> Result<()> {
        let ast = parse_script(script)?;
        self.evaluate_statements(&ast)?;
        Ok(())
    }

    fn evaluate_statements(&mut self, stmts: &[Statement]) -> Result<()> {
        for stmt in stmts {
            match stmt {
                Statement::Assignment(var, expr) => {
                    let value = self.evaluate_expr(expr)?;
                    self.set_variable(var, value)?;
                }
                Statement::If(cond, then_block, else_block) => {
                    if self.evaluate_expr(cond)? != 0 {
                        self.evaluate_statements(then_block)?;
                    } else if let Some(else_stmts) = else_block {
                        self.evaluate_statements(else_stmts)?;
                    }
                }
                // ... more statement types
            }
        }
        Ok(())
    }
}
```

**Pros**:
- Familiar syntax (C/JavaScript-like)
- Expressive enough for most instructions
- Human-readable
- Can express conditionals and complex logic

**Cons**:
- Need to implement parser and interpreter
- Performance overhead (unless JIT compiled)
- Potential for bugs in interpreter
- Limited to imperative style

**Best For**: General-purpose solution, good for educational projects

---

### Option 2: Stack-Based Bytecode

**Concept**: Instructions compile to stack machine bytecode (like JVM, WebAssembly, Forth).

```toml
[[instruction]]
mnemonic = "add"
behavior = [
    { op = "push_reg", reg = "rs1" },
    { op = "push_reg", reg = "rs2" },
    { op = "add" },
    { op = "pop_reg", reg = "rd" },
    { op = "push_const", value = 4 },
    { op = "add_pc" },
]

[[instruction]]
mnemonic = "beq"
behavior = [
    { op = "push_reg", reg = "rs1" },
    { op = "push_reg", reg = "rs2" },
    { op = "eq" },
    { op = "branch_if", target = [
        { op = "push_reg", reg = "pc" },
        { op = "push_operand", operand = "imm" },
        { op = "add" },
        { op = "pop_reg", reg = "pc" },
    ], else_target = [
        { op = "push_reg", reg = "pc" },
        { op = "push_const", value = 4 },
        { op = "add" },
        { op = "pop_reg", reg = "pc" },
    ]},
]
```

Or more compact textual format:
```toml
[[instruction]]
mnemonic = "add"
behavior = "push.rs1 push.rs2 add pop.rd const.4 add_pc"
```

**Implementation**:
```rust
pub enum BytecodeOp {
    PushReg(String),
    PushConst(u64),
    PushOperand(String),
    PopReg(String),
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    Eq,
    Lt,
    BranchIf { then_block: Vec<BytecodeOp>, else_block: Vec<BytecodeOp> },
    AddPc,
    SetFlag(String),
}

pub struct BytecodeInterpreter {
    stack: Vec<u64>,
    machine: MachineState,
}

impl BytecodeInterpreter {
    pub fn execute(&mut self, ops: &[BytecodeOp]) -> Result<()> {
        for op in ops {
            match op {
                BytecodeOp::PushReg(name) => {
                    let value = self.machine.get_reg(name)?;
                    self.stack.push(value);
                }
                BytecodeOp::Add => {
                    let b = self.stack.pop().ok_or(error!("Stack underflow"))?;
                    let a = self.stack.pop().ok_or(error!("Stack underflow"))?;
                    self.stack.push(a.wrapping_add(b));
                }
                // ... more operations
            }
        }
        Ok(())
    }
}
```

**Pros**:
- Very simple interpreter
- Fast execution (small instruction set)
- Well-understood execution model
- Easy to optimize (can inline, JIT compile)
- Compact representation

**Cons**:
- Verbose for complex operations
- Stack management can be confusing
- Less human-readable
- Harder to write by hand

**Best For**: Performance-critical emulators, when behavior can be generated programmatically

---

### Option 3: Register Transfer Language (RTL)

**Concept**: Declarative hardware description language (like Verilog/VHDL).

```toml
[[instruction]]
mnemonic = "add"
behavior = """
    rd <- rs1 + rs2
    pc <- pc + 4
"""

[[instruction]]
mnemonic = "beq"
behavior = """
    cond <- (rs1 == rs2)
    pc <- if cond then pc + imm else pc + 4
"""

[[instruction]]
mnemonic = "adc"
behavior = """
    temp <- A + operand + C
    A <- temp[7:0]
    C <- temp[8]
    Z <- (A == 0)
    N <- A[7]
    V <- (~(A[7] ^ operand[7]) & (A[7] ^ temp[7]))
    pc <- pc + 2
"""
```

**Syntax**:
- `<-` for assignment (data transfer)
- `[n:m]` for bit slicing
- `if cond then expr else expr` for conditionals
- All assignments happen "simultaneously" (like hardware)

**Implementation**:
```rust
pub struct RTLInterpreter {
    assignments: Vec<(String, Expr)>,
    machine: MachineState,
}

impl RTLInterpreter {
    pub fn execute(&mut self, rtl: &str) -> Result<()> {
        let statements = parse_rtl(rtl)?;

        // Evaluate all right-hand sides first (parallel semantics)
        let mut values = Vec::new();
        for (target, expr) in &statements {
            let value = self.evaluate_expr(expr)?;
            values.push((target.clone(), value));
        }

        // Then commit all writes (avoids ordering issues)
        for (target, value) in values {
            self.machine.set_value(&target, value)?;
        }

        Ok(())
    }
}
```

**Pros**:
- Very declarative and clear
- Natural for hardware description
- No ordering ambiguity (parallel semantics)
- Standard in CPU design
- Easy to analyze and optimize

**Cons**:
- Less familiar to software developers
- Need custom parser
- Parallel semantics can be confusing
- Harder to express imperative algorithms

**Best For**: When targeting hardware designers, formal verification, high-level ISA specification

---

### Option 4: Functional/Declarative Effects

**Concept**: Describe instruction as a list of effects on machine state.

```toml
[[instruction]]
mnemonic = "add"
effects = [
    { register = "rd", formula = "rs1 + rs2" },
    { register = "pc", formula = "pc + 4" }
]

[[instruction]]
mnemonic = "lw"
effects = [
    { register = "rd", formula = "mem[rs1 + imm]" },
    { register = "pc", formula = "pc + 4" }
]

[[instruction]]
mnemonic = "beq"
effects = [
    {
        register = "pc",
        formula = "if (rs1 == rs2) then (pc + imm) else (pc + 4)"
    }
]

[[instruction]]
mnemonic = "adc"
effects = [
    { register = "A", formula = "(A + operand + C) & 0xFF" },
    { flag = "C", formula = "(A + operand + C) > 0xFF" },
    { flag = "Z", formula = "A == 0" },
    { flag = "N", formula = "(A & 0x80) != 0" },
    { register = "pc", formula = "pc + 2" }
]
```

**More complex example with temporary values**:
```toml
[[instruction]]
mnemonic = "adc"
temps = [
    { name = "sum", formula = "A + operand + C" },
    { name = "result", formula = "sum & 0xFF" }
]
effects = [
    { register = "A", formula = "result" },
    { flag = "C", formula = "sum > 0xFF" },
    { flag = "Z", formula = "result == 0" },
    { flag = "N", formula = "(result & 0x80) != 0" },
    { flag = "V", formula = "overflow_check(A, operand, result)" },
    { register = "pc", formula = "pc + 2" }
]
```

**Implementation**:
```rust
pub struct Effect {
    pub target: EffectTarget,
    pub formula: Expr,
}

pub enum EffectTarget {
    Register(String),
    Flag(String),
    Memory { address: Expr, width: usize },
}

pub struct EffectInterpreter {
    machine: MachineState,
    operands: HashMap<String, u64>,
}

impl EffectInterpreter {
    pub fn execute(&mut self, effects: &[Effect], temps: &[(String, Expr)]) -> Result<()> {
        // Evaluate temporaries first
        let mut temp_values = HashMap::new();
        for (name, expr) in temps {
            let value = self.evaluate_expr(expr)?;
            temp_values.insert(name.clone(), value);
        }

        // Then apply effects
        for effect in effects {
            let value = self.evaluate_expr_with_temps(&effect.formula, &temp_values)?;
            match &effect.target {
                EffectTarget::Register(name) => {
                    self.machine.set_reg(name, value)?;
                }
                EffectTarget::Flag(name) => {
                    self.machine.set_flag(name, value != 0)?;
                }
                EffectTarget::Memory { address, width } => {
                    let addr = self.evaluate_expr_with_temps(address, &temp_values)?;
                    self.machine.write_mem(addr, value, *width)?;
                }
            }
        }

        Ok(())
    }
}
```

**Pros**:
- Very declarative and composable
- Easy to validate
- Clear data dependencies
- Can be optimized/parallelized
- Natural for functional programming

**Cons**:
- Verbose for complex operations
- Hard to express control flow
- Need expression evaluator
- Temporary values add complexity

**Best For**: When clarity and correctness are paramount, formal verification

---

### Option 5: Embedded Scripting Language (Lua/Rhai/Wasm)

**Concept**: Use a mature scripting language for full programmability.

**Lua Example**:
```toml
[[instruction]]
mnemonic = "add"
behavior_lang = "lua"
behavior = """
function execute(ctx)
    ctx.rd = ctx.rs1 + ctx.rs2
    return ctx.pc + 4
end
"""

[[instruction]]
mnemonic = "adc"
behavior_lang = "lua"
behavior = """
function execute(ctx)
    local sum = ctx.A + ctx.operand + ctx:get_flag("C")
    ctx.A = sum & 0xFF
    ctx:set_flag("C", sum > 0xFF)
    ctx:set_flag("Z", ctx.A == 0)
    ctx:set_flag("N", (ctx.A & 0x80) ~= 0)
    -- Overflow: sign(A) == sign(operand) && sign(A) != sign(result)
    local v = (bit.band(bit.bnot(bit.bxor(ctx.A, ctx.operand)), bit.bxor(ctx.A, sum)) & 0x80) ~= 0
    ctx:set_flag("V", v)
    return ctx.pc + 2
end
"""
```

**Rhai Example** (Rust-like syntax):
```toml
[[instruction]]
mnemonic = "add"
behavior_lang = "rhai"
behavior = """
fn execute(ctx) {
    ctx.rd = ctx.rs1 + ctx.rs2;
    ctx.pc + 4
}
"""
```

**Implementation**:
```rust
use rlua::{Lua, Context};

pub struct LuaInterpreter {
    lua: Lua,
}

impl LuaInterpreter {
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    pub fn execute(&self, script: &str, machine: &mut MachineState, operands: &HashMap<String, u64>) -> Result<()> {
        self.lua.context(|lua_ctx| {
            // Create context object
            let ctx = lua_ctx.create_table()?;

            // Add operands
            for (name, value) in operands {
                ctx.set(name.as_str(), *value)?;
            }

            // Add machine state accessors
            ctx.set("get_flag", lua_ctx.create_function(|_, flag: String| {
                // Access machine state
                Ok(machine.get_flag(&flag))
            })?)?;

            // Load and run script
            lua_ctx.load(script).exec()?;

            Ok(())
        })
    }
}
```

**Pros**:
- Full programming language features
- Mature, battle-tested implementations
- Great debugging tools
- Can express anything
- Good performance (LuaJIT is very fast)
- Large ecosystem

**Cons**:
- Large dependency
- Slower than native code
- Security concerns (need sandboxing)
- More complex integration
- Harder to analyze statically

**Best For**: When maximum flexibility is needed, complex instructions, when embedding is acceptable

---

### Option 6: WebAssembly (Wasm)

**Concept**: Compile behavior to WebAssembly for safe, fast execution.

**WebAssembly Text (WAT) Format**:
```toml
[[instruction]]
mnemonic = "add"
behavior_lang = "wasm"
behavior = """
(module
  (func $execute (param $rs1 i32) (param $rs2 i32) (result i32)
    local.get $rs1
    local.get $rs2
    i32.add
  )
)
"""
```

Or write in Rust, compile to Wasm:
```rust
// behavior/add.rs
#[no_mangle]
pub extern "C" fn execute_add(rs1: u32, rs2: u32) -> u32 {
    rs1.wrapping_add(rs2)
}
```

**Implementation**:
```rust
use wasmer::{Store, Module, Instance};

pub struct WasmInterpreter {
    store: Store,
}

impl WasmInterpreter {
    pub fn execute(&self, wasm_module: &[u8], operands: &HashMap<String, u64>) -> Result<u64> {
        let module = Module::new(&self.store, wasm_module)?;
        let instance = Instance::new(&module, &imports)?;

        let execute_fn = instance.exports.get_function("execute")?;
        let result = execute_fn.call(&[/* operands */])?;

        Ok(result[0].unwrap_i64() as u64)
    }
}
```

**Pros**:
- Standardized bytecode format
- Very fast (near-native, JIT compiled)
- Sandboxed by design (safe)
- Can write in multiple languages (Rust, C, AssemblyScript)
- Growing ecosystem
- Future-proof

**Cons**:
- Complex setup
- Binary format (less inspectable)
- Overkill for simple operations
- Larger runtime dependency
- Steeper learning curve

**Best For**: Performance-critical emulators, when behaviors are compiled not hand-written

---

### Option 7: Direct Rust Code Generation

**Concept**: Generate Rust functions at compile time from behavior descriptions.

```toml
[[instruction]]
mnemonic = "add"
behavior = """
    rd = rs1 + rs2;
    pc += 4;
"""
```

Procedural macro generates:
```rust
// Generated code
fn execute_add(&mut self, rd: usize, rs1: usize, rs2: usize) {
    let rs1_val = self.regs[rs1];
    let rs2_val = self.regs[rs2];
    self.regs[rd] = rs1_val.wrapping_add(rs2_val);
    self.pc += 4;
}

// In the main execute loop:
match instruction.mnemonic {
    "add" => self.execute_add(rd, rs1, rs2),
    // ...
}
```

**Implementation**:
```rust
// In proc macro
fn generate_behavior_function(instr: &InstructionDef) -> TokenStream {
    let behavior_ast = parse_behavior(&instr.behavior)?;
    let rust_code = codegen_rust(&behavior_ast)?;

    quote! {
        fn #fn_name(&mut self, #params) {
            #rust_code
        }
    }
}
```

**Pros**:
- Zero runtime overhead
- Full Rust optimization
- Type-safe
- Native speed
- No interpreter needed

**Cons**:
- Can't load architectures at runtime
- Compile-time only
- Longer compile times
- Harder to debug generated code
- Less flexible

**Best For**: When maximum performance is critical, production emulators

---

### Option 8: Pattern-Based Templates

**Concept**: Most instructions follow common patterns. Define templates for these.

```toml
# Define templates in Rust
# pub enum InstructionTemplate {
#     AluBinaryRRR,
#     AluBinaryRRI,
#     Load,
#     Store,
#     BranchConditional,
#     Jump,
#     Custom,
# }

[[instruction]]
mnemonic = "add"
template = "alu_binary_rrr"
operation = "add"
dest = "rd"
src1 = "rs1"
src2 = "rs2"

[[instruction]]
mnemonic = "addi"
template = "alu_binary_rri"
operation = "add"
dest = "rd"
src1 = "rs1"
immediate = "imm"

[[instruction]]
mnemonic = "lw"
template = "load"
width = 32
dest = "rd"
base = "rs1"
offset = "imm"

[[instruction]]
mnemonic = "beq"
template = "branch_conditional"
condition = "equal"
src1 = "rs1"
src2 = "rs2"
offset = "imm"

[[instruction]]
mnemonic = "csrrw"  # Complex instruction
template = "custom"
behavior = "/* custom code */"
```

**Template Implementations**:
```rust
pub trait InstructionTemplate {
    fn execute(&self, ctx: &mut ExecutionContext, params: &TemplateParams) -> Result<()>;
}

pub struct AluBinaryRRR;

impl InstructionTemplate for AluBinaryRRR {
    fn execute(&self, ctx: &mut ExecutionContext, params: &TemplateParams) -> Result<()> {
        let a = ctx.get_reg(&params.get("src1")?)?;
        let b = ctx.get_reg(&params.get("src2")?)?;

        let result = match params.operation {
            AluOp::Add => a.wrapping_add(b),
            AluOp::Sub => a.wrapping_sub(b),
            AluOp::And => a & b,
            AluOp::Or => a | b,
            AluOp::Xor => a ^ b,
            // ...
        };

        ctx.set_reg(&params.get("dest")?, result)?;
        ctx.advance_pc(4)?;

        Ok(())
    }
}
```

**Pros**:
- Very concise for common patterns
- Type-safe (templates are Rust code)
- Fast (native code)
- Easy to understand
- Covers 80-90% of instructions

**Cons**:
- Limited to predefined templates
- Need custom behavior for unusual instructions
- Hybrid system (templates + custom)
- Template proliferation for edge cases

**Best For**: Pragmatic solution, balancing simplicity and power

---

### Option 9: State Transition Description

**Concept**: Describe instructions as state transformations.

```toml
[[instruction]]
mnemonic = "add"
preconditions = []
effects = [
    {
        type = "register_write",
        target = { type = "operand", name = "rd" },
        value = {
            type = "binary_op",
            op = "add",
            left = { type = "register_read", source = { type = "operand", name = "rs1" } },
            right = { type = "register_read", source = { type = "operand", name = "rs2" } }
        }
    },
    {
        type = "pc_increment",
        amount = 4
    }
]
postconditions = []

[[instruction]]
mnemonic = "lw"
preconditions = []
effects = [
    {
        type = "register_write",
        target = { type = "operand", name = "rd" },
        value = {
            type = "memory_read",
            address = {
                type = "binary_op",
                op = "add",
                left = { type = "register_read", source = { type = "operand", name = "rs1" } },
                right = { type = "operand_value", name = "imm" }
            },
            width = 32
        }
    },
    { type = "pc_increment", amount = 4 }
]
```

**Implementation**:
```rust
pub enum Effect {
    RegisterWrite {
        target: Target,
        value: Value,
    },
    MemoryWrite {
        address: Value,
        value: Value,
        width: usize,
    },
    FlagUpdate {
        flag: String,
        value: Value,
    },
    PcIncrement {
        amount: u64,
    },
    PcSet {
        value: Value,
    },
}

pub enum Value {
    Constant(u64),
    RegisterRead { source: Target },
    OperandValue { name: String },
    MemoryRead { address: Box<Value>, width: usize },
    BinaryOp { op: BinaryOp, left: Box<Value>, right: Box<Value> },
    UnaryOp { op: UnaryOp, operand: Box<Value> },
    Conditional { cond: Box<Value>, then_val: Box<Value>, else_val: Box<Value> },
}
```

**Pros**:
- Very declarative
- Easy to validate correctness
- Can verify properties (e.g., "does instruction write memory?")
- Good for formal verification
- Machine-readable
- Can generate visualizations

**Cons**:
- Extremely verbose
- Hard to write by hand
- Complex data structures
- Overkill for simple instructions

**Best For**: Tool-generated specifications, formal methods, when properties need to be proven

---

### Option 10: Hybrid Three-Tier System (RECOMMENDED)

**Concept**: Combine the best approaches - templates for common cases, effects for medium complexity, scripts for edge cases.

```toml
# Tier 1: Pattern Template (80% of instructions)
[[instruction]]
mnemonic = "add"
tier = "template"
template = "alu_binary_rrr"
operation = "add"

[[instruction]]
mnemonic = "lw"
tier = "template"
template = "load"
width = 32
base = "rs1"
offset = "imm"

# Tier 2: Declarative Effects (15% of instructions)
[[instruction]]
mnemonic = "lui"
tier = "effects"
effects = [
    { write = "rd", value = "imm << 12" },
    { write = "pc", value = "pc + 4" }
]

[[instruction]]
mnemonic = "adc"
tier = "effects"
temps = [
    { name = "sum", value = "A + operand + C" }
]
effects = [
    { write = "A", value = "sum & 0xFF" },
    { flag = "C", value = "sum > 0xFF" },
    { flag = "Z", value = "A == 0" },
    { flag = "N", value = "(A & 0x80) != 0" },
    { write = "pc", value = "pc + 2" }
]

# Tier 3: Behavior Script (5% of instructions)
[[instruction]]
mnemonic = "csrrw"
tier = "script"
behavior_lang = "simple_expr"
behavior = """
    temp = read_csr(csr_num);
    write_csr(csr_num, rs1);
    rd = temp;
    pc += 4;
"""
```

**Implementation**:
```rust
pub enum InstructionBehavior {
    /// Tier 1: Use predefined template
    Template {
        name: String,
        params: HashMap<String, String>,
    },

    /// Tier 2: Declarative effects
    Effects {
        temps: Vec<(String, Expr)>,
        effects: Vec<Effect>,
    },

    /// Tier 3: Behavior script
    Script {
        lang: BehaviorLang,
        code: String,
    },
}

impl InstructionBehavior {
    pub fn execute(&self, ctx: &mut ExecutionContext) -> Result<()> {
        match self {
            Self::Template { name, params } => {
                let template = TEMPLATES.get(name)?;
                template.execute(ctx, params)
            }
            Self::Effects { temps, effects } => {
                EffectInterpreter::new(ctx).execute(temps, effects)
            }
            Self::Script { lang, code } => {
                match lang {
                    BehaviorLang::SimpleExpr => {
                        SimpleExprInterpreter::new(ctx).execute(code)
                    }
                    BehaviorLang::Lua => {
                        LuaInterpreter::new().execute(code, ctx)
                    }
                }
            }
        }
    }
}
```

**Why This Works**:

1. **Most instructions are simple**: ALU operations, loads, stores → templates handle them efficiently
2. **Some need flexibility**: Flag updates, complex addressing → declarative effects provide clarity
3. **Few need full power**: System instructions, CSRs, interrupts → scripting provides escape hatch

**Distribution** (based on RISC-V and 6502 analysis):
- **Tier 1 (Templates)**: ~80% of instructions
  - All R-type ALU operations
  - All I-type immediate operations
  - Simple loads and stores
  - Simple branches

- **Tier 2 (Effects)**: ~15% of instructions
  - Instructions with flag updates
  - Complex addressing modes
  - Multi-step operations

- **Tier 3 (Scripts)**: ~5% of instructions
  - System/control instructions
  - CSR operations
  - Interrupts and exceptions
  - Architecture-specific edge cases

**Pros**:
- Optimal for each complexity level
- Simple cases stay simple
- Complex cases have full power
- Good performance overall
- Pragmatic and flexible

**Cons**:
- More complex implementation
- Need to maintain three systems
- Learning curve for contributors

**Best For**: Real-world emulators that need both simplicity and power

---

## Generic Emulator Core Implementation

### Machine State

```rust
pub struct GenericMachine {
    /// Architecture specification
    arch: ArchSpec,

    /// Program counter (width determined by architecture)
    pc: u64,

    /// General-purpose registers (if architecture has them)
    /// x0 hardwired to zero handled specially
    general_regs: Vec<u64>,

    /// Named special registers (A, X, Y, SP, etc.)
    special_regs: HashMap<String, u64>,

    /// Processor status flags (if architecture has them)
    flags: Option<u8>,

    /// Memory (byte-addressed)
    memory: Vec<u8>,

    /// Execution trace (for debugging)
    trace: Vec<TraceEntry>,

    /// Instruction cache for decoded instructions
    instruction_cache: HashMap<u64, CachedInstruction>,
}

pub struct CachedInstruction {
    pub definition: &'static InstructionDef,
    pub operands: HashMap<String, u64>,
    pub bytes: Vec<u8>,
}
```

### Fetch-Decode-Execute Loop

```rust
impl GenericMachine {
    pub fn step(&mut self) -> Result<StepResult> {
        // Fetch instruction bytes
        let instruction_bytes = self.fetch()?;

        // Decode instruction
        let (instr_def, operands) = self.decode(&instruction_bytes)?;

        // Execute instruction
        self.execute(instr_def, operands)?;

        Ok(StepResult::Continue)
    }

    fn fetch(&self) -> Result<Vec<u8>> {
        // Determine max instruction size for this architecture
        let max_bytes = self.arch.max_instruction_bytes();

        let mut bytes = Vec::with_capacity(max_bytes);
        for i in 0..max_bytes {
            let addr = self.pc + i as u64;
            if addr >= self.memory.len() as u64 {
                break;
            }
            bytes.push(self.memory[addr as usize]);
        }

        Ok(bytes)
    }

    fn decode(&self, bytes: &[u8]) -> Result<(&InstructionDef, HashMap<String, u64>)> {
        // Try each instruction definition until one matches
        for instr_def in &self.arch.instructions {
            if let Some(operands) = self.try_match(instr_def, bytes) {
                return Ok((instr_def, operands));
            }
        }

        bail!("Unknown instruction at PC={:#x}: {:02x?}", self.pc, bytes);
    }

    fn try_match(&self, instr_def: &InstructionDef, bytes: &[u8]) -> Option<HashMap<String, u64>> {
        // Get instruction format
        let format = self.arch.get_format(&instr_def.format)?;

        // Check if we have enough bytes
        if bytes.len() < format.width_bytes {
            return None;
        }

        // Extract all fields from instruction bytes
        let fields = self.extract_fields(format, bytes);

        // Check if fixed fields match encoding pattern
        for (field_name, expected_value) in &instr_def.encoding {
            if fields.get(field_name) != Some(expected_value) {
                return None;  // Encoding mismatch
            }
        }

        // Extract operand values
        let mut operands = HashMap::new();
        for operand_name in &instr_def.operands {
            if let Some(&value) = fields.get(operand_name) {
                operands.insert(operand_name.clone(), value);
            }
        }

        Some(operands)
    }

    fn extract_fields(&self, format: &InstructionFormat, bytes: &[u8]) -> HashMap<String, u64> {
        let mut fields = HashMap::new();

        // Assemble bytes into a word (respecting endianness)
        let word = self.bytes_to_word(bytes);

        // Extract each field
        for field_def in &format.fields {
            let value = self.extract_bitfield(word, &field_def.bits);

            // Apply sign extension if needed
            let value = if field_def.sign_extend {
                self.sign_extend(value, field_def.bits.end - field_def.bits.start + 1)
            } else {
                value
            };

            fields.insert(field_def.name.clone(), value);
        }

        fields
    }

    fn execute(&mut self, instr_def: &InstructionDef, operands: HashMap<String, u64>) -> Result<()> {
        let mut ctx = ExecutionContext {
            machine: self,
            operands,
            temps: HashMap::new(),
        };

        // Execute based on behavior type
        instr_def.behavior.execute(&mut ctx)?;

        Ok(())
    }
}
```

### Execution Context

```rust
pub struct ExecutionContext<'a> {
    machine: &'a mut GenericMachine,
    operands: HashMap<String, u64>,
    temps: HashMap<String, u64>,
}

impl ExecutionContext<'_> {
    pub fn get_operand(&self, name: &str) -> Result<u64> {
        self.operands.get(name)
            .copied()
            .ok_or_else(|| anyhow!("Unknown operand: {}", name))
    }

    pub fn get_reg(&self, name: &str) -> Result<u64> {
        // Check if it's an operand that refers to a register
        if let Ok(reg_num) = self.get_operand(name) {
            return Ok(self.machine.general_regs[reg_num as usize]);
        }

        // Check special registers
        if let Some(&value) = self.machine.special_regs.get(name) {
            return Ok(value);
        }

        bail!("Unknown register: {}", name);
    }

    pub fn set_reg(&mut self, name: &str, value: u64) -> Result<()> {
        // Handle x0 specially (hardwired to zero)
        if name == "zero" || name == "x0" {
            return Ok(());  // Writes to x0 are ignored
        }

        // Check if it's an operand that refers to a register
        if let Ok(reg_num) = self.get_operand(name) {
            self.machine.general_regs[reg_num as usize] = value;
            return Ok(());
        }

        // Check special registers
        if self.machine.special_regs.contains_key(name) {
            self.machine.special_regs.insert(name.to_string(), value);
            return Ok(());
        }

        bail!("Unknown register: {}", name);
    }

    pub fn get_flag(&self, name: &str) -> Result<bool> {
        let flags = self.machine.flags
            .ok_or_else(|| anyhow!("Architecture has no flags"))?;

        let flag_def = self.machine.arch.flags.as_ref()
            .and_then(|f| f.flags.iter().find(|fd| fd.name == name))
            .ok_or_else(|| anyhow!("Unknown flag: {}", name))?;

        Ok((flags & (1 << flag_def.bit_position)) != 0)
    }

    pub fn set_flag(&mut self, name: &str, value: bool) -> Result<()> {
        let flags = self.machine.flags
            .as_mut()
            .ok_or_else(|| anyhow!("Architecture has no flags"))?;

        let flag_def = self.machine.arch.flags.as_ref()
            .and_then(|f| f.flags.iter().find(|fd| fd.name == name))
            .ok_or_else(|| anyhow!("Unknown flag: {}", name))?;

        if value {
            *flags |= 1 << flag_def.bit_position;
        } else {
            *flags &= !(1 << flag_def.bit_position);
        }

        Ok(())
    }

    pub fn read_memory(&self, address: u64, width: usize) -> Result<u64> {
        let mut value = 0u64;
        for i in 0..(width / 8) {
            let byte = self.machine.memory[(address + i as u64) as usize];
            value |= (byte as u64) << (i * 8);
        }
        Ok(value)
    }

    pub fn write_memory(&mut self, address: u64, value: u64, width: usize) -> Result<()> {
        for i in 0..(width / 8) {
            let byte = ((value >> (i * 8)) & 0xFF) as u8;
            self.machine.memory[(address + i as u64) as usize] = byte;
        }
        Ok(())
    }

    pub fn advance_pc(&mut self, bytes: u64) -> Result<()> {
        self.machine.pc += bytes;
        Ok(())
    }
}
```

---

## Example Usage

### Running RISC-V Program

```rust
use rmachine::generic_emulator::GenericMachine;

fn main() -> Result<()> {
    // Load RISC-V architecture
    let mut machine = GenericMachine::load("arch/riscv32i.toml")?;

    // Load program
    let program = std::fs::read("program.bin")?;
    machine.load_program(&program, 0x0000)?;

    // Run until halt
    loop {
        match machine.step()? {
            StepResult::Continue => {}
            StepResult::Halt => break,
        }
    }

    Ok(())
}
```

### Running 6502 Program

```rust
fn main() -> Result<()> {
    // Load 6502 architecture
    let mut machine = GenericMachine::load("arch/6502.toml")?;

    // Load ROM at typical NES location
    let rom = std::fs::read("game.nes")?;
    machine.load_program(&rom, 0xC000)?;

    // Set PC to reset vector
    machine.set_pc(0xFFFC)?;

    // Run
    loop {
        machine.step()?;
    }
}
```

### Creating Custom Architecture

```toml
# arch/fantasy8.toml - Simple 8-bit fantasy console

[architecture]
name = "Fantasy-8"
word_size = 8
address_width = 16
endianness = "little"
pc_increment = "variable"

[registers]
general_purpose_count = 8
general_purpose_width = 8

[[registers.special]]
name = "pc"
width = 16
initial_value = 0

[flags]
has_flags = true

[[flags.bits]]
name = "Z"
bit = 0

[[flags.bits]]
name = "N"
bit = 1

# Simple instruction set
[[formats]]
name = "RegReg"
width_bytes = 1
[[formats.fields]]
name = "opcode"
bits = "7:4"
[[formats.fields]]
name = "rd"
bits = "3:2"
[[formats.fields]]
name = "rs"
bits = "1:0"

[[instruction]]
mnemonic = "MOV"
format = "RegReg"
encoding = { opcode = 0x0 }
template = "register_copy"

[[instruction]]
mnemonic = "ADD"
format = "RegReg"
encoding = { opcode = 0x1 }
template = "alu_binary_rr"
operation = "add"
```

---

## Migration Path for rmachine

### Phase 1: Extract Architecture Spec
1. Create `arch/riscv32i.toml` from current hardcoded logic
2. Keep existing implementation working
3. Add tests that verify spec matches current behavior

### Phase 2: Implement Generic Core
1. Build `GenericMachine` alongside current `Machine`
2. Implement behavior tier 1 (templates) only
3. Verify functional equivalence with tests

### Phase 3: Add Behavior Tiers
1. Implement tier 2 (declarative effects)
2. Implement tier 3 (simple expression DSL)
3. Migrate complex instructions to appropriate tiers

### Phase 4: Generalize
1. Replace current `Machine` with `GenericMachine`
2. Add support for second architecture (6502) to validate abstraction
3. Document architecture specification format

### Phase 5: Extend
1. Add more architectures (ARM Thumb, MIPS, custom)
2. Optimize hot paths (JIT compilation, caching)
3. Add tooling (architecture validator, behavior debugger)

---

## Performance Considerations

### Optimization Strategies

1. **Instruction Caching**: Cache decoded instructions
   ```rust
   if let Some(cached) = self.instruction_cache.get(&self.pc) {
       return self.execute_cached(cached);
   }
   ```

2. **Direct Threading**: Generate jump table for hot loops
   ```rust
   // Instead of match on every instruction
   let fn_ptr = self.instruction_dispatch[opcode];
   fn_ptr(self, operands);
   ```

3. **JIT Compilation**: Compile behavior scripts to native code
   - Use `cranelift` or `llvm` to compile hot paths
   - Fall back to interpreter for cold code

4. **Template Inlining**: Inline template code for common instructions
   ```rust
   #[inline(always)]
   fn execute_alu_binary_rrr(&mut self, op: AluOp, rd: u8, rs1: u8, rs2: u8) {
       // Inlined for performance
   }
   ```

5. **Batch Execution**: Execute multiple instructions before checking interrupts
   ```rust
   pub fn run_batch(&mut self, count: usize) -> Result<()> {
       for _ in 0..count {
           self.step()?;
       }
       Ok(())
   }
   ```

### Performance Targets

- **Interpreter mode**: 10-50 MIPS on modern hardware
- **JIT mode**: 100-500 MIPS (with tiered compilation)
- **Overhead**: < 10% compared to hand-written emulator for hot paths

---

## Tradeoffs and Limitations

### What Works Well

✅ **Functional ISAs**: Instructions that compute values (ALU, loads, stores)
✅ **Simple control flow**: Branches, jumps, calls
✅ **Flags and status**: Condition codes, processor modes
✅ **Multiple architectures**: RISC-V, 6502, ARM, simple custom ISAs

### What's Challenging

⚠️ **Precise timing**: Cycle-accurate emulation needs per-instruction timing
⚠️ **Interrupts/Exceptions**: Asynchronous events need special handling
⚠️ **Memory-mapped I/O**: Peripheral emulation beyond CPU scope
⚠️ **Cache simulation**: Hardware caches need separate modeling
⚠️ **Complex instructions**: x86 string operations, SIMD need special treatment

### Fundamental Limitations

❌ **Micro-architecture details**: Pipeline, branch prediction, out-of-order execution
❌ **Hardware peripherals**: GPUs, timers, DMA controllers
❌ **System-level features**: MMU, virtual memory, protection rings
❌ **Debugging features**: Hardware breakpoints, watchpoints, trace buffers

These could be added as extensions, but are beyond the core ISA emulation scope.

---

## Future Work

### Short Term
- Implement all three behavior tiers
- Add 6502 as validation
- Optimize instruction dispatch
- Add comprehensive tests

### Medium Term
- JIT compilation for hot paths
- More architectures (ARM Thumb, MIPS, AVR)
- Debugging support (breakpoints, single-step, register inspection)
- Performance profiling

### Long Term
- Cycle-accurate mode (optional timing)
- Interrupt and exception handling framework
- Memory management unit simulation
- Visual architecture editor (GUI for creating arch specs)
- Architecture test suite generator

---

## Conclusion

A generic, data-driven emulator is achievable and valuable. The key insights:

1. **Abstract common patterns**: Fetch-decode-execute, registers, memory, PC
2. **Parameterize differences**: Word size, instruction formats, addressing modes
3. **Layered behavior representation**: Templates for simple, scripts for complex
4. **Performance through pragmatism**: Optimize common cases, allow flexibility for edge cases

**For rmachine specifically**, this represents an evolution from a RISC-V emulator to a universal emulation platform - a tool for teaching, experimentation, and understanding CPU architecture at a fundamental level.

The three-tier behavior system (templates + effects + scripts) provides the right balance of:
- **Simplicity** for common instructions
- **Clarity** for medium complexity
- **Power** for edge cases

This design makes adding new architectures as simple as writing a TOML file, while maintaining performance competitive with hand-written emulators.
