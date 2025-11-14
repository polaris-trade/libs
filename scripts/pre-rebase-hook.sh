#!/usr/bin/env bash
# Pre-merge hook to prevent merge commits in feature branches

set -e

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m'

BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Rule: No merge commits in feature branches
if [[ "$BRANCH" != "main" ]] && [[ "$BRANCH" != "develop" ]]; then
    echo -e "${RED}❌ Merge commits are not allowed in feature branches!${NC}"
    echo ""
    echo -e "${YELLOW}Use rebase instead of merge:${NC}"
    echo "  Current branch: $BRANCH"
    echo ""
    echo "To update from develop:"
    echo "  git merge --abort                # Cancel this merge"
    echo "  git fetch origin"
    echo "  git rebase origin/develop        # Use rebase instead"
    echo ""
    echo "If you have conflicts during rebase:"
    echo "  1. Resolve conflicts in files"
    echo "  2. git add <resolved-files>"
    echo "  3. git rebase --continue"
    echo ""
    exit 1
fi

# Allow merges in main/develop (from PRs)
echo -e "${GREEN}✓ Merge allowed in $BRANCH${NC}"
exit 0
