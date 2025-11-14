#!/usr/bin/env bash
set -e

# Commitizen git hook
# This script validates commit messages against conventional commit format

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Skip if commit message is a merge commit
if [[ $COMMIT_MSG =~ ^Merge ]]; then
    exit 0
fi

# Conventional commit regex
# Format: type(scope): subject
CONVENTIONAL_COMMIT_REGEX="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert|release)(\([a-z0-9\-]+\))?: .{1,100}$"

if [[ ! $COMMIT_MSG =~ $CONVENTIONAL_COMMIT_REGEX ]]; then
    echo -e "${RED}❌ Invalid commit message format${NC}"
    echo ""
    echo -e "${YELLOW}Expected format:${NC}"
    echo "  <type>(<scope>): <subject>"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  feat(core): add new pricing algorithm"
    echo "  fix(connectors): resolve websocket connection issue"
    echo "  docs(readme): update installation instructions"
    echo ""
    echo -e "${YELLOW}Valid types:${NC}"
    echo "  feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert, release"
    echo ""
    echo -e "${YELLOW}Your commit message:${NC}"
    echo "  $COMMIT_MSG"
    exit 1
fi

echo -e "${GREEN}✓ Commit message format is valid${NC}"
exit 0
