# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-12-19

### Added
- Initial release of R-Machine CPU emulator and assembler
- 32-bit RISC CPU emulator with 16 registers
- Assembler (`rasm`) that converts assembly code to machine code
- Debugger (`rmon`) for step-by-step execution and running programs
- Disassembler (`rdis`) for examining compiled programs
- Support for 23 instructions covering arithmetic, logic, branching, and memory operations
- Example programs demonstrating various features
- Comprehensive test suite with 46 tests
- Documentation for architecture and instruction set

### Features
- Simple, educational CPU architecture
- Clean instruction encoding (32-bit instructions)
- Terminal-based system interface
- Support for basic system calls (exit, write)
- Step-through debugging with register inspection
- Modern Rust 2024 edition
- Dual MIT/Apache-2.0 licensing

### Developer Experience
- Justfile for common development tasks
- Comprehensive CI/CD with GitHub Actions
- Security audits and nightly builds
- Clean codebase passing clippy pedantic lints

[Unreleased]: https://github.com/aleury/rmachine/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aleury/rmachine/releases/tag/v0.1.0