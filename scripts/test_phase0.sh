#!/bin/bash
# Phase 0 Test Script - Shell Integration & Auto-Switching
# Run this to verify the complete cd hook workflow

echo -e "\n========================================"
echo -e "  PHASE 0: Foundation Fixes Test"
echo -e "========================================\n"

TEST_DIR="$PWD/test-phase0"
PROJECT_A="$TEST_DIR/project-a"
PROJECT_B="$TEST_DIR/project-b"

# Cleanup previous tests
if [ -d "$TEST_DIR" ]; then
    echo -e "[CLEANUP] Removing old test directory..."
    rm -rf "$TEST_DIR"
fi

# Create test structure
echo -e "[SETUP] Creating test projects..."
mkdir -p "$PROJECT_A" "$PROJECT_B"

# Project A: Node 20
cat > "$PROJECT_A/ven.toml" << 'EOF'
[runtime]
node = "20"

[packages]
express = "^4.18.2"

[env]
NODE_ENV = "development"
PORT = "3000"
EOF

# Project B: Node 18
cat > "$PROJECT_B/ven.toml" << 'EOF'
[runtime]
node = "18"

[env]
NODE_ENV = "production"
EOF

echo -e "  ✓ Created project-a (Node 20)"
echo -e "  ✓ Created project-b (Node 18)"

# Test 1: ven.toml parsing
echo -e "\n[TEST 1] ven.toml parsing..."
echo -e "  Running: ven status --verbose"
ven status --verbose
if [ $? -eq 0 ]; then
    echo -e "  ✓ ven status works"
else
    echo -e "  ✗ ven status failed"
fi

# Test 2: Shell hook generation
echo -e "\n[TEST 2] Shell hook generation..."
echo -e "  Running: ven shell hook bash"
HOOK=$(ven shell hook bash)
if echo "$HOOK" | grep -q "__ven_activate"; then
    echo -e "  ✓ Bash hook generated"
else
    echo -e "  ✗ Bash hook invalid"
fi

# Test 3: Shell activate (PATH output)
echo -e "\n[TEST 3] ven shell activate PATH output..."
cd "$PROJECT_A"
echo -e "  Running: ven shell activate '$PROJECT_A'"
EXPORTS=$(ven shell activate "$PROJECT_A")
if [ -n "$EXPORTS" ]; then
    echo -e "  Exports returned:"
    echo "$EXPORTS" | sed 's/^/    /'
    
    if echo "$EXPORTS" | grep -q "VEN_NODE_VERSION" && echo "$EXPORTS" | grep -q "PATH"; then
        echo -e "  ✓ PATH exports generated correctly"
    else
        echo -e "  ✗ Missing required exports"
    fi
else
    echo -e "  ✗ No exports returned"
fi

# Test 4: Test with Project B
echo -e "\n[TEST 4] Switch to project-b..."
cd "$PROJECT_B"
EXPORTS_B=$(ven shell activate "$PROJECT_B")
if echo "$EXPORTS_B" | grep -q "18"; then
    echo -e "  ✓ Project B activates Node 18"
else
    echo -e "  ✗ Project B activation failed"
fi

# Test 5: No ven.toml scenario
echo -e "\n[TEST 5] Directory without ven.toml..."
cd "$TEST_DIR"
NO_EXPORTS=$(ven shell activate "$TEST_DIR")
if [ -z "$NO_EXPORTS" ]; then
    echo -e "  ✓ No exports when ven.toml missing (correct)"
else
    echo -e "  ✗ Should return empty when no ven.toml"
fi

# Cleanup
echo -e "\n[CLEANUP] Removing test directory..."
cd "$PWD"
rm -rf "$TEST_DIR"

echo -e "\n========================================"
echo -e "  Phase 0 Test Complete!"
echo -e "========================================\n"

echo -e "Next Steps:"
echo -e "  1. Run 'ven setup' to install shell hooks"
echo -e "  2. Restart shell: source ~/.bashrc (or ~/.zshrc)"
echo -e "  3. Manually test: cd into project with ven.toml"
echo -e "  4. Verify: node --version matches ven.toml"
echo ""
