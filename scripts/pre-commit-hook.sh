#!/usr/bin/env bash
# Pre-commit hook to enforce Git workflow rules locally

set -e

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Rule 1: Prevent direct commits to protected branches
if [[ "$BRANCH" == "main" ]] || [[ "$BRANCH" == "develop" ]]; then
    echo -e "${RED}❌ Direct commits to '$BRANCH' are not allowed!${NC}"
    echo ""
    echo -e "${YELLOW}Please create a feature branch:${NC}"
    echo "  git checkout -b feat/your-feature"
    echo ""
    echo "Then create a Pull Request to merge into $BRANCH"
    echo ""
    exit 1
fi

# Rule 2: Validate branch naming convention
VALID_PREFIXES=("feat/" "fix/" "hotfix/" "refactor/" "perf/" "docs/" "test/" "chore/" "ci/" "build/")
VALID=false

for prefix in "${VALID_PREFIXES[@]}"; do
    if [[ "$BRANCH" == $prefix* ]]; then
        VALID=true
        break
    fi
done

if [ "$VALID" = false ]; then
    echo -e "${RED}❌ Invalid branch name: $BRANCH${NC}"
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
    echo "To fix: git checkout -b feat/$(echo $BRANCH | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g')"
    exit 1
fi

# Rule 3: Auto-format Rust code
echo -e "${YELLOW}Running cargo fmt...${NC}"
if ! cargo fmt --all -- --check &> /dev/null; then
    echo -e "${YELLOW}⚠️  Code needs formatting. Auto-formatting now...${NC}"
    cargo fmt --all
    echo -e "${GREEN}✓ Code formatted. Please review and add changes.${NC}"
    echo ""
    echo "Staged files have been formatted. Review with:"
    echo "  git diff"
    echo ""
    echo "Add formatted files:"
    echo "  git add -u"
    echo "  git commit"
    exit 1
fi

# Rule 4: Check for merge commits in feature branches
if git rev-parse -q --verify MERGE_HEAD > /dev/null; then
    echo -e "${RED}❌ Merge commits are not allowed in feature branches!${NC}"
    echo ""
    echo -e "${YELLOW}Use rebase instead:${NC}"
    echo "  git merge --abort"
    echo "  git rebase develop"
    echo ""
    echo "To update your branch:"
    echo "  git fetch origin"
    echo "  git rebase origin/develop"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓ Pre-commit checks passed${NC}"
exit 0
