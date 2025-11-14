#!/usr/bin/env bash
# Pre-push hook to enforce Git workflow rules

set -e

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

# Read stdin for refs being pushed
while read local_ref local_sha remote_ref remote_sha; do
    # Extract branch name from ref
    if [[ "$remote_ref" =~ refs/heads/(.+) ]]; then
        PUSH_BRANCH="${BASH_REMATCH[1]}"
        
        # Rule 1: Prevent direct push to main/develop
        if [[ "$PUSH_BRANCH" == "main" ]] || [[ "$PUSH_BRANCH" == "develop" ]]; then
            echo -e "${RED}❌ Direct push to '$PUSH_BRANCH' is not allowed!${NC}"
            echo ""
            echo -e "${YELLOW}Protected branches cannot be pushed to directly.${NC}"
            echo ""
            echo "Workflow:"
            echo "  1. Push your feature branch: git push origin $CURRENT_BRANCH"
            echo "  2. Create a Pull Request on GitHub"
            echo "  3. After approval, merge via GitHub UI"
            echo ""
            echo -e "${BLUE}Why this rule exists:${NC}"
            echo "  • Ensures code review"
            echo "  • Runs CI/CD checks"
            echo "  • Maintains linear history"
            echo "  • Prevents accidental breaking changes"
            echo ""
            exit 1
        fi
        
        # Rule 2: Validate branch naming for feature branches
        VALID_PREFIXES=("feat/" "fix/" "hotfix/" "refactor/" "perf/" "docs/" "test/" "chore/" "ci/" "build/" "release/")
        VALID=false
        
        for prefix in "${VALID_PREFIXES[@]}"; do
            if [[ "$PUSH_BRANCH" == $prefix* ]]; then
                VALID=true
                break
            fi
        done
        
        if [ "$VALID" = false ]; then
            echo -e "${RED}❌ Invalid branch name: $PUSH_BRANCH${NC}"
            echo ""
            echo -e "${YELLOW}Branch names must follow conventional format:${NC}"
            echo "  feat/<description>     - New features"
            echo "  fix/<description>      - Bug fixes"
            echo "  hotfix/<description>   - Critical fixes"
            echo "  etc."
            echo ""
            echo "To fix:"
            echo "  1. Rename branch: git branch -m feat/your-description"
            echo "  2. Push again: git push origin feat/your-description"
            echo ""
            exit 1
        fi
        
        # Rule 3: Check for merge commits in feature branch commits
        if [[ "$PUSH_BRANCH" != "main" ]] && [[ "$PUSH_BRANCH" != "develop" ]]; then
            # Get commits being pushed
            if [ "$remote_sha" = "0000000000000000000000000000000000000000" ]; then
                # New branch, check all commits
                RANGE="$local_sha"
            else
                # Existing branch, check new commits
                RANGE="$remote_sha..$local_sha"
            fi
            
            # Check for merge commits
            MERGE_COMMITS=$(git log --merges --oneline $RANGE 2>/dev/null || echo "")
            if [ -n "$MERGE_COMMITS" ]; then
                echo -e "${RED}❌ Merge commits detected in feature branch!${NC}"
                echo ""
                echo "Merge commits found:"
                echo "$MERGE_COMMITS"
                echo ""
                echo -e "${YELLOW}Feature branches must use rebase, not merge.${NC}"
                echo ""
                echo "To fix:"
                echo "  1. Reset to remote: git reset --hard origin/$PUSH_BRANCH"
                echo "  2. Rebase instead: git rebase origin/develop"
                echo "  3. Force push: git push --force-with-lease origin $PUSH_BRANCH"
                echo ""
                exit 1
            fi
        fi
        
        echo -e "${GREEN}✓ Push validation passed for $PUSH_BRANCH${NC}"
    fi
done

exit 0
