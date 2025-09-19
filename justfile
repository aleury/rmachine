# List available commands
default:
    @just --list

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run tests with documentation-style output
testdox:
    cargo testdox

# Run clippy with pedantic lints
lint:
    cargo clippy --all-targets -- -W clippy::pedantic

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Generate documentation
doc:
    cargo doc --no-deps --open

# Run cargo-machete to check for unused dependencies
check-deps:
    cargo machete

# Run cargo deny checks
check-deny:
    cargo deny check

# Clean build artifacts
clean:
    cargo clean
    rm -rf ex1 ex1.o ex2 ex2.o

# Build RISC-V example 1
build-riscv-ex1:
    rm -rf ex1 ex1.o
    riscv64-linux-gnu-as ./examples/ex1.s -o ex1.o
    riscv64-linux-gnu-gcc -o ex1 ex1.o -nostdlib -static

# Build RISC-V example 2
build-riscv-ex2:
    rm -rf ex2 ex2.o
    riscv64-linux-gnu-as ./examples/ex2.s -o ex2.o
    riscv64-linux-gnu-gcc -o ex2 ex2.o -nostdlib -static

# Build all RISC-V examples
build-riscv-examples: build-riscv-ex1 build-riscv-ex2

# Run example with rasm and rmon
run-example file:
    cargo run --bin rasm -- examples/{{file}}.s
    cargo run --bin rmon -- examples/{{file}}

# Publish dry run
publish-dry:
    cargo publish --dry-run

# Run all checks before publishing
pre-publish: fmt-check lint test check-deps
    @echo "All checks passed! Ready to publish."

# Install the binaries locally
install:
    cargo install --path .

# Uninstall the binaries
uninstall:
    cargo uninstall rmachine
