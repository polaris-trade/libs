#!/usr/bin/env bash
# Sync Configuration Across Repos
# This script helps sync common configuration files across all quantforge repos

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CURRENT_REPO=$(pwd)
REPOS_ROOT="${REPOS_ROOT:-$(dirname "$CURRENT_REPO")}"

# Files to sync
COMMON_FILES=(
    "dependency-versions.toml"
    ".commitlintrc.json"
    ".cz.json"
    "rust-toolchain.toml"
    ".cargo/config.toml"
    ".github/workflows/version-check.yml"
    ".github/workflows/ci.yml"
    ".github/workflows/commitlint.yml"
    ".github/workflows/release.yml"
    "scripts/check-versions.py"
    "scripts/commit-msg-hook.sh"
)

# Target repos (relative to REPOS_ROOT)
TARGET_REPOS=(
    "quantforge-connectors"
    "quantforge-engines"
    "quantforge-infrastructure"
)

echo -e "${BLUE}QuantForge Configuration Sync${NC}"
echo -e "${BLUE}================================${NC}\n"

# Check if we're in the source repo
if [[ ! -f "dependency-versions.toml" ]]; then
    echo -e "${RED}Error: Must be run from the core/libs repository${NC}"
    exit 1
fi

echo -e "${YELLOW}Source repo:${NC} $CURRENT_REPO"
echo -e "${YELLOW}Repos root:${NC} $REPOS_ROOT\n"

# Parse command line arguments
DRY_RUN=false
FORCE=false
SPECIFIC_REPO=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --repo)
            SPECIFIC_REPO="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

if [[ "$DRY_RUN" == true ]]; then
    echo -e "${YELLOW}DRY RUN MODE - No files will be modified${NC}\n"
fi

# Function to sync files to a repo
sync_to_repo() {
    local repo_name=$1
    local repo_path="${REPOS_ROOT}/${repo_name}"
    
    echo -e "${BLUE}Syncing to: ${repo_name}${NC}"
    
    if [[ ! -d "$repo_path" ]]; then
        echo -e "${YELLOW}  ⚠ Repo not found, skipping: $repo_path${NC}"
        return
    fi
    
    local synced=0
    local skipped=0
    local errors=0
    
    for file in "${COMMON_FILES[@]}"; do
        local source_file="${CURRENT_REPO}/${file}"
        local target_file="${repo_path}/${file}"
        
        if [[ ! -f "$source_file" ]]; then
            echo -e "${YELLOW}  ⚠ Source file not found: $file${NC}"
            ((skipped++))
            continue
        fi
        
        # Create target directory if needed
        local target_dir=$(dirname "$target_file")
        if [[ ! -d "$target_dir" ]]; then
            if [[ "$DRY_RUN" == false ]]; then
                mkdir -p "$target_dir"
            fi
            echo -e "    ${BLUE}Creating directory: $(dirname "$file")${NC}"
        fi
        
        # Check if file exists and is different
        if [[ -f "$target_file" ]]; then
            if diff -q "$source_file" "$target_file" > /dev/null 2>&1; then
                echo -e "    ${GREEN}✓${NC} $file (unchanged)"
                ((skipped++))
                continue
            else
                if [[ "$FORCE" == false ]]; then
                    echo -e "    ${YELLOW}△${NC} $file (modified, use --force to overwrite)"
                    ((skipped++))
                    continue
                fi
            fi
        fi
        
        # Copy file
        if [[ "$DRY_RUN" == false ]]; then
            if cp "$source_file" "$target_file"; then
                # Make scripts executable
                if [[ "$file" == scripts/*.sh ]] || [[ "$file" == scripts/*.py ]]; then
                    chmod +x "$target_file"
                fi
                echo -e "    ${GREEN}✓${NC} $file (synced)"
                ((synced++))
            else
                echo -e "    ${RED}✗${NC} $file (error)"
                ((errors++))
            fi
        else
            echo -e "    ${BLUE}→${NC} $file (would sync)"
            ((synced++))
        fi
    done
    
    echo -e "  ${GREEN}Synced: $synced${NC} | ${YELLOW}Skipped: $skipped${NC} | ${RED}Errors: $errors${NC}\n"
}

# Sync to all repos or specific repo
if [[ -n "$SPECIFIC_REPO" ]]; then
    sync_to_repo "$SPECIFIC_REPO"
else
    for repo in "${TARGET_REPOS[@]}"; do
        sync_to_repo "$repo"
    done
fi

echo -e "${GREEN}Sync complete!${NC}\n"

if [[ "$DRY_RUN" == false ]]; then
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Review changes in each repo"
    echo "  2. Run 'make check-versions' in each repo"
    echo "  3. Commit changes with: git commit -m 'build(deps): sync configuration'"
    echo "  4. Push to remote"
else
    echo -e "${YELLOW}Run without --dry-run to actually sync files${NC}"
fi
