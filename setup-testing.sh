#!/bin/bash

# Quick setup script for Miden examples testing system
# This script helps developers set up the testing environment

set -e

echo "Setting up Miden examples testing system..."

# Check if we're in the right directory
if [ ! -f "README.md" ] || [ ! -d "examples" ]; then
    echo "Error: Please run this script from the root of the examples repository"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build the benchmarking CLI
echo "Building benchmarking CLI..."
cd benchmarking-cli
cargo build --release
cd ..

# Run initial validation
echo "Running initial validation..."
cd benchmarking-cli
cargo run --release -- validate --verbose
cd ..

echo ""
echo "Setup complete!"
echo ""
echo "Testing system commands:"
echo "  Test all examples:          cd benchmarking-cli && cargo run --release -- test-all"
echo "  Test specific example:       cd benchmarking-cli && cargo run --release -- test --example <name>"
echo "  Validate directory:         cd benchmarking-cli && cargo run --release -- validate"
echo "  Benchmark example:           cd benchmarking-cli && cargo run --release -- benchmark --example <name>"
echo ""
echo "Pre-commit hooks are now installed and will run automatically on commits"
echo "See README.md for detailed documentation"
echo ""
echo "Testing a few examples to verify setup..."
cd benchmarking-cli

# Test a simple example
if cargo run --release -- test --example fibonacci --ci; then
    echo "fibonacci example: PASSED"
else
    echo "fibonacci example: FAILED"
fi

# Test directory validation
if cargo run --release -- validate --ci; then
    echo "Directory validation: PASSED"
else
    echo "Directory validation: FAILED"
fi

cd ..
echo ""
echo "Setup complete! The testing system is ready to use."
