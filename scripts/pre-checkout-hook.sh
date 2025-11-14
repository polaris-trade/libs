#!/usr/bin/env bash
# Post-checkout hook to enforce branch naming convention and prevent direct checkout of protected branches

set -e

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Hook arguments
PREV_HEAD="$1"
NEW_HEAD="$2"
IS_BRANCH_CHECKOUT="$3"

# Only check on branch checkout (not file checkout)
if [ "$IS_BRANCH_CHECKOUT" != "1" ]; then
    exit 0
fi

# Get the current branch
BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)

# Skip if HEAD is detached
if [[ "$BRANCH_NAME" == "HEAD" ]]; then
    exit 0
fi

# Rule 1: Warn when checking out protected branches (but don't block)
if [[ "$BRANCH_NAME" == "main" ]] || [[ "$BRANCH_NAME" == "develop" ]]; then
    echo -e "${YELLOW}⚠️  Checked out protected branch: $BRANCH_NAME${NC}"
    echo -e "${BLUE}Remember: No direct commits allowed on this branch${NC}"
    echo "Create a feature branch before making changes:"
    echo "  git checkout -b feat/your-feature"
    echo ""
    exit 0
fi

# Rule 2: Validate branch naming convention for feature branches
VALID_PREFIXES=("feat/" "fix/" "hotfix/" "refactor/" "perf/" "docs/" "test/" "chore/" "ci/" "build/" "release/")
VALID=false

for prefix in "${VALID_PREFIXES[@]}"; do
    if [[ "$BRANCH_NAME" == $prefix* ]]; then
        VALID=true
        break
    fi
done

if [ "$VALID" = false ]; then
    # Check if this is an existing remote branch (allow it but warn)
    if git show-ref --verify --quiet "refs/remotes/origin/$BRANCH_NAME"; then
        echo -e "${YELLOW}⚠️  Warning: Branch name doesn't follow convention: $BRANCH_NAME${NC}"
        echo -e "${BLUE}This is an existing remote branch, but please use conventional naming for new branches.${NC}"
        echo ""
        exit 0
    fi
    
    echo -e "${RED}❌ Invalid branch name: $BRANCH_NAME${NC}"
    echo ""
    echo -e "${YELLOW}Branch names must follow conventional format:${NC}"
    echo "  feat/<description>     - New features"
    echo "  fix/<description>      - Bug fixes"
    echo "  hotfix/<description>   - Critical fixes"
    echo "  refactor/<description> - Code refactoring"
    echo "  perf/<description>     - Performance improvements"
    echo "  docs/<description>     - Documentation"
    echo "  test/<description>     - Tests"
    echo "  chore/<description>    - Maintenance"
    echo "  ci/<description>       - CI/CD changes"
    echo ""
    echo -e "${YELLOW}To fix this branch:${NC}"
    echo "  git branch -m feat/$(echo $BRANCH_NAME | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g')"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Checked out valid branch: $BRANCH_NAME${NC}"
exit 0
