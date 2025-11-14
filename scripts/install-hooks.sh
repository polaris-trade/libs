#!/usr/bin/env bash
# Install all git hooks for enforcing workflow rules locally

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo -e "${BLUE}Installing Local Git Workflow Enforcement Hooks${NC}"
echo -e "${BLUE}================================================${NC}\n"

# Check if we're in a git repository
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo -e "${RED}Error: Not a git repository. Skipping hook installation.${NC}"
    exit 1
fi

# Make all hook scripts executable
echo "Making hook scripts executable..."
chmod +x "$SCRIPT_DIR"/pre-commit-hook.sh
chmod +x "$SCRIPT_DIR"/pre-checkout-hook.sh
chmod +x "$SCRIPT_DIR"/pre-rebase-hook.sh
chmod +x "$SCRIPT_DIR"/pre-push-hook.sh
chmod +x "$SCRIPT_DIR"/commit-msg-hook.sh

# Install pre-commit hook
echo -e "${GREEN}✓${NC} Installing pre-commit hook"
echo "   • Prevents direct commits to main/develop"
echo "   • Validates branch naming convention"
echo "   • Detects merge commits in feature branches"
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(git rev-parse --show-toplevel)/scripts"
exec "$SCRIPT_DIR/pre-commit-hook.sh" "$@"
EOF
chmod +x "$HOOKS_DIR/pre-commit"

# Install post-checkout hook
echo -e "${GREEN}✓${NC} Installing post-checkout hook"
echo "   • Enforces branch naming when checking out new branches"
echo "   • Warns when checking out protected branches"
cat > "$HOOKS_DIR/post-checkout" << 'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(git rev-parse --show-toplevel)/scripts"
exec "$SCRIPT_DIR/pre-checkout-hook.sh" "$@"
EOF
chmod +x "$HOOKS_DIR/post-checkout"

# Install pre-merge-commit hook
echo -e "${GREEN}✓${NC} Installing pre-merge-commit hook"
echo "   • Prevents merge commits in feature branches"
echo "   • Enforces rebase-only workflow"
cat > "$HOOKS_DIR/pre-merge-commit" << 'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(git rev-parse --show-toplevel)/scripts"
exec "$SCRIPT_DIR/pre-rebase-hook.sh" "$@"
EOF
chmod +x "$HOOKS_DIR/pre-merge-commit"

# Install pre-push hook
echo -e "${GREEN}✓${NC} Installing pre-push hook"
echo "   • Prevents direct push to main/develop"
echo "   • Validates branch names before push"
echo "   • Detects merge commits being pushed"
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(git rev-parse --show-toplevel)/scripts"
exec "$SCRIPT_DIR/pre-push-hook.sh" "$@"
EOF
chmod +x "$HOOKS_DIR/pre-push"

# Install commit-msg hook
echo -e "${GREEN}✓${NC} Installing commit-msg hook"
echo "   • Validates conventional commit message format"
cat > "$HOOKS_DIR/commit-msg" << 'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(git rev-parse --show-toplevel)/scripts"
exec "$SCRIPT_DIR/commit-msg-hook.sh" "$@"
EOF
chmod +x "$HOOKS_DIR/commit-msg"

echo ""
echo -e "${GREEN}✓ All git hooks installed successfully!${NC}"
echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Local Enforcement Rules:${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════${NC}"
echo ""
echo "1. ${BLUE}No direct commits to main/develop${NC}"
echo "   Blocked by: pre-commit hook"
echo ""
echo "2. ${BLUE}Branch naming convention required${NC}"
echo "   Format: feat/*, fix/*, hotfix/*, etc."
echo "   Blocked by: post-checkout, pre-commit, pre-push hooks"
echo ""
echo "3. ${BLUE}No merge commits in feature branches${NC}"
echo "   Only rebase allowed"
echo "   Blocked by: pre-merge-commit, pre-commit, pre-push hooks"
echo ""
echo "4. ${BLUE}No direct push to main/develop${NC}"
echo "   Must use Pull Requests"
echo "   Blocked by: pre-push hook"
echo ""
echo "5. ${BLUE}Conventional commit messages${NC}"
echo "   Format: type(scope): description"
echo "   Blocked by: commit-msg hook"
echo ""
echo -e "${YELLOW}═══════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}To bypass hooks (NOT recommended):${NC}"
echo "  git commit --no-verify"
echo "  git push --no-verify"
echo ""
echo -e "${RED}WARNING: Bypassing hooks may cause CI failures!${NC}"
echo ""
echo -e "${GREEN}Test the hooks:${NC}"
echo "  git checkout main           # Should warn"
echo "  git checkout -b test        # Should fail"
echo "  git checkout -b feat/test   # Should succeed"
