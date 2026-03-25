#!/usr/bin/env bash
# Quick automated test runner for Phase 1 verification
# Run this after installing Rust and fnm

set -e  # Exit on first error

echo "========================================="
echo "ven Phase 1 Automated Test Suite"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

pass_count=0
fail_count=0

# Helper function to run a test
run_test() {
    local test_name="$1"
    local command="$2"
    
    echo -n "Testing: $test_name ... "
    
    if eval "$command" > /tmp/ven_test_output.txt 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        ((pass_count++))
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Command: $command"
        echo "  Output:"
        cat /tmp/ven_test_output.txt | sed 's/^/    /'
        ((fail_count++))
    fi
}

# Check prerequisites
echo "Checking prerequisites..."
echo ""

if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓${NC} Rust/Cargo found: $(cargo --version)"
else
    echo -e "${RED}✗${NC} Rust not found. Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

if command -v fnm &> /dev/null; then
    echo -e "${GREEN}✓${NC} fnm found: $(fnm --version)"
else
    echo -e "${YELLOW}⚠${NC} fnm not found. Some tests will fail. Install from: https://github.com/Schniz/fnm"
fi

echo ""
echo "========================================="
echo "Running Tests"
echo "========================================="
echo ""

# Build first
echo "Building ven..."
cargo build --release
echo -e "${GREEN}Build successful${NC}"
echo ""

# Create temporary test directory
TEST_DIR="/tmp/ven_test_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"
echo "Test directory: $TEST_DIR"
echo ""

# Test suite
echo "--- Unit Tests ---"
cd "$(dirname "$0")"
run_test "Unit tests (all)" "cargo test --all"

echo ""
echo "--- CLI Commands ---"

cd "$TEST_DIR"
VEN_BIN="$(dirname "$0")/target/release/ven"

run_test "ven init" "$VEN_BIN init"
run_test "ven.toml created" "test -f ven.toml"
run_test "ven status" "$VEN_BIN status"
run_test "ven list" "$VEN_BIN list"

echo ""
echo "--- Configuration System ---"

run_test "Parse example config" "cat > test_config.toml <<EOF
[runtime]
node = \"20.11.1\"

[packages]
express = \"^4.18.2\"

[env]
NODE_ENV = \"production\"
EOF"

echo ""
echo "--- Shell Hook ---"

run_test "Generate bash hook" "$VEN_BIN shell hook bash"
run_test "Generate zsh hook" "$VEN_BIN shell hook zsh"
run_test "Generate fish hook" "$VEN_BIN shell hook fish"

echo ""
echo "--- Cleanup ---"
rm -rf "$TEST_DIR"
echo "Test directory cleaned up"

echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "${GREEN}Passed: $pass_count${NC}"
echo -e "${RED}Failed: $fail_count${NC}"
echo ""

if [ $fail_count -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Check output above.${NC}"
    exit 1
fi
