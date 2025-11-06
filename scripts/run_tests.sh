#!/bin/bash
# scripts/run_tests.sh
# Comprehensive test script for local development

set -e

echo "======================================"
echo "Running Tower Defense Game Test Suite"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[ℹ]${NC} $1"
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed. Please install Rust."
    exit 1
fi

print_status "Cargo found"

# 1. Format check
print_info "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_status "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# 2. Clippy (linting)
print_info "Running Clippy linter..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_status "No linting issues found"
else
    print_error "Clippy found issues"
    exit 1
fi

# 3. Build check
print_info "Building project..."
if cargo build; then
    print_status "Build successful"
else
    print_error "Build failed"
    exit 1
fi

# 4. Unit tests
print_info "Running unit tests..."
if cargo test --lib; then
    print_status "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# 5. Integration tests
print_info "Running integration tests..."
if cargo test --test '*'; then
    print_status "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# 6. Doc tests
print_info "Running documentation tests..."
if cargo test --doc; then
    print_status "Documentation tests passed"
else
    print_error "Documentation tests failed"
    exit 1
fi

# 7. All tests with verbose output
print_info "Running all tests with verbose output..."
cargo test --verbose --all-features

# 8. Benchmarks (optional - only if criterion is set up)
if grep -q "criterion" Cargo.toml; then
    print_info "Running benchmarks..."
    cargo bench --no-fail-fast
    print_status "Benchmarks completed"
fi

echo ""
echo "======================================"
print_status "All tests passed successfully!"
echo "======================================"
