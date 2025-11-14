#!/usr/bin/env python3
"""
Smart Version Bumper

Analyzes conventional commits since last tag and suggests version bump.
Can optionally auto-bump versions in Cargo.toml files.

Usage:
    python scripts/bump-version.py --crate data_types [--apply] [--bump major|minor|patch]
    python scripts/bump-version.py --workspace [--apply] [--bump major|minor|patch]
"""

import subprocess
import sys
import re
from pathlib import Path
from typing import Optional, Tuple, List
import argparse
import tomllib
import tomli_w


class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    END = '\033[0m'


def get_git_tags() -> List[str]:
    """Get all git tags sorted by version."""
    try:
        result = subprocess.run(
            ['git', 'tag', '--sort=-version:refname'],
            capture_output=True,
            text=True,
            check=True
        )
        return [tag.strip() for tag in result.stdout.split('\n') if tag.strip()]
    except subprocess.CalledProcessError:
        return []


def get_latest_tag_for_crate(crate_name: Optional[str] = None) -> Optional[str]:
    """Get the latest tag for a specific crate or workspace."""
    tags = get_git_tags()
    
    if crate_name:
        # Look for crate-specific tags like "data_types/v1.0.0"
        pattern = f"{crate_name}/v"
        for tag in tags:
            if tag.startswith(pattern):
                return tag
    
    # Look for workspace tags like "v1.0.0"
    for tag in tags:
        if tag.startswith('v') and '/' not in tag:
            return tag
    
    return None


def get_commits_since_tag(tag: Optional[str], crate_path: Optional[str] = None) -> List[str]:
    """Get commit messages since the given tag."""
    if tag:
        ref_range = f"{tag}..HEAD"
    else:
        ref_range = "HEAD"
    
    cmd = ['git', 'log', '--pretty=format:%s', ref_range]
    
    if crate_path:
        cmd.extend(['--', crate_path])
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return [line.strip() for line in result.stdout.split('\n') if line.strip()]
    except subprocess.CalledProcessError:
        return []


def analyze_commits(commits: List[str]) -> Tuple[str, dict]:
    """
    Analyze commits and determine version bump level.
    
    Returns:
        Tuple of (bump_type, commit_breakdown)
        bump_type: 'major', 'minor', 'patch', or 'none'
    """
    breaking_changes = []
    features = []
    fixes = []
    others = []
    
    # Regex patterns for conventional commits
    breaking_pattern = re.compile(r'^(\w+)(?:\([^)]+\))?!:|BREAKING CHANGE:', re.IGNORECASE)
    feat_pattern = re.compile(r'^feat(?:\([^)]+\))?:', re.IGNORECASE)
    fix_pattern = re.compile(r'^fix(?:\([^)]+\))?:', re.IGNORECASE)
    
    for commit in commits:
        if breaking_pattern.search(commit):
            breaking_changes.append(commit)
        elif feat_pattern.match(commit):
            features.append(commit)
        elif fix_pattern.match(commit):
            fixes.append(commit)
        else:
            others.append(commit)
    
    # Determine bump type
    if breaking_changes:
        bump_type = 'major'
    elif features:
        bump_type = 'minor'
    elif fixes:
        bump_type = 'patch'
    else:
        bump_type = 'none'
    
    breakdown = {
        'breaking': breaking_changes,
        'features': features,
        'fixes': fixes,
        'others': others
    }
    
    return bump_type, breakdown


def parse_version(version_str: str) -> Tuple[int, int, int]:
    """Parse semantic version string."""
    match = re.match(r'v?(\d+)\.(\d+)\.(\d+)', version_str)
    if not match:
        raise ValueError(f"Invalid version format: {version_str}")
    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def bump_version(version: str, bump_type: str) -> str:
    """Bump version according to semver rules."""
    major, minor, patch = parse_version(version)
    
    if bump_type == 'major':
        return f"{major + 1}.0.0"
    elif bump_type == 'minor':
        return f"{major}.{minor + 1}.0"
    elif bump_type == 'patch':
        return f"{major}.{minor}.{patch + 1}"
    else:
        return f"{major}.{minor}.{patch}"


def get_current_version(cargo_toml_path: Path) -> str:
    """Get current version from Cargo.toml."""
    with open(cargo_toml_path, 'rb') as f:
        cargo = tomllib.load(f)
    
    if 'package' in cargo and 'version' in cargo['package']:
        return cargo['package']['version']
    
    raise ValueError(f"No version found in {cargo_toml_path}")


def update_cargo_version(cargo_toml_path: Path, new_version: str):
    """Update version in Cargo.toml."""
    with open(cargo_toml_path, 'rb') as f:
        cargo = tomllib.load(f)
    
    if 'package' not in cargo:
        raise ValueError(f"No [package] section in {cargo_toml_path}")
    
    cargo['package']['version'] = new_version
    
    # Read original content to preserve formatting
    with open(cargo_toml_path, 'r') as f:
        content = f.read()
    
    # Simple regex replacement to preserve formatting
    old_version = get_current_version(cargo_toml_path)
    pattern = rf'(version\s*=\s*["\']){re.escape(old_version)}(["\'])'
    new_content = re.sub(pattern, rf'\g<1>{new_version}\g<2>', content, count=1)
    
    with open(cargo_toml_path, 'w') as f:
        f.write(new_content)


def print_breakdown(breakdown: dict):
    """Print commit breakdown."""
    if breakdown['breaking']:
        print(f"\n{Colors.RED}💥 Breaking Changes ({len(breakdown['breaking'])}){Colors.END}")
        for commit in breakdown['breaking'][:5]:
            print(f"  - {commit}")
        if len(breakdown['breaking']) > 5:
            print(f"  ... and {len(breakdown['breaking']) - 5} more")
    
    if breakdown['features']:
        print(f"\n{Colors.GREEN}✨ Features ({len(breakdown['features'])}){Colors.END}")
        for commit in breakdown['features'][:5]:
            print(f"  - {commit}")
        if len(breakdown['features']) > 5:
            print(f"  ... and {len(breakdown['features']) - 5} more")
    
    if breakdown['fixes']:
        print(f"\n{Colors.BLUE}🐛 Bug Fixes ({len(breakdown['fixes'])}){Colors.END}")
        for commit in breakdown['fixes'][:5]:
            print(f"  - {commit}")
        if len(breakdown['fixes']) > 5:
            print(f"  ... and {len(breakdown['fixes']) - 5} more")
    
    if breakdown['others']:
        print(f"\n{Colors.YELLOW}📦 Other Changes ({len(breakdown['others'])}){Colors.END}")
        for commit in breakdown['others'][:3]:
            print(f"  - {commit}")
        if len(breakdown['others']) > 3:
            print(f"  ... and {len(breakdown['others']) - 3} more")


def main():
    parser = argparse.ArgumentParser(description='Smart version bumper for Rust crates')
    parser.add_argument('--crate', type=str, help='Crate name to bump (e.g., data_types)')
    parser.add_argument('--workspace', action='store_true', help='Bump entire workspace')
    parser.add_argument('--apply', action='store_true', help='Apply version bump to Cargo.toml')
    parser.add_argument('--bump', choices=['major', 'minor', 'patch'], help='Force specific bump type')
    parser.add_argument('--root', type=str, default='.', help='Repository root directory')
    args = parser.parse_args()
    
    root = Path(args.root).resolve()
    
    if not args.crate and not args.workspace:
        print(f"{Colors.RED}Error: Specify either --crate or --workspace{Colors.END}")
        sys.exit(1)
    
    print(f"{Colors.BLUE}🔍 Analyzing commits for version bump...{Colors.END}\n")
    
    # Determine crate path and tag
    if args.crate:
        crate_path = root / args.crate
        if not crate_path.exists():
            print(f"{Colors.RED}Error: Crate directory not found: {crate_path}{Colors.END}")
            sys.exit(1)
        
        cargo_toml = crate_path / "Cargo.toml"
        if not cargo_toml.exists():
            print(f"{Colors.RED}Error: Cargo.toml not found: {cargo_toml}{Colors.END}")
            sys.exit(1)
        
        latest_tag = get_latest_tag_for_crate(args.crate)
        commits = get_commits_since_tag(latest_tag, str(crate_path.relative_to(root)))
        
        print(f"{Colors.YELLOW}Crate:{Colors.END} {args.crate}")
    else:
        cargo_toml = root / "Cargo.toml"
        latest_tag = get_latest_tag_for_crate(None)
        commits = get_commits_since_tag(latest_tag)
        
        print(f"{Colors.YELLOW}Scope:{Colors.END} Workspace")
    
    print(f"{Colors.YELLOW}Latest tag:{Colors.END} {latest_tag or 'none'}")
    print(f"{Colors.YELLOW}Commits analyzed:{Colors.END} {len(commits)}\n")
    
    if not commits:
        print(f"{Colors.YELLOW}No commits found since last release.{Colors.END}")
        sys.exit(0)
    
    # Analyze commits
    suggested_bump, breakdown = analyze_commits(commits)
    
    # Override if user specified
    bump_type = args.bump if args.bump else suggested_bump
    
    print_breakdown(breakdown)
    
    # Get current version and calculate new version
    try:
        current_version = get_current_version(cargo_toml)
        new_version = bump_version(current_version, bump_type)
        
        print(f"\n{Colors.MAGENTA}Version Change{Colors.END}")
        print(f"  Current: {Colors.YELLOW}{current_version}{Colors.END}")
        print(f"  New:     {Colors.GREEN}{new_version}{Colors.END}")
        print(f"  Bump:    {Colors.BLUE}{bump_type.upper()}{Colors.END}")
        
        if bump_type == 'none':
            print(f"\n{Colors.YELLOW}No version bump needed (only chore/docs/style commits){Colors.END}")
            sys.exit(0)
        
        if args.apply:
            update_cargo_version(cargo_toml, new_version)
            print(f"\n{Colors.GREEN}✓ Version updated in {cargo_toml.relative_to(root)}{Colors.END}")
            
            print(f"\n{Colors.YELLOW}Next steps:{Colors.END}")
            print(f"  1. Review changes: git diff {cargo_toml.relative_to(root)}")
            print(f"  2. Update CHANGELOG.md if needed")
            print(f"  3. Commit: git commit -m \"release({args.crate or 'workspace'}): bump to v{new_version}\"")
            print(f"  4. Tag: git tag -a {args.crate + '/' if args.crate else ''}v{new_version} -m \"Release v{new_version}\"")
            print(f"  5. Push: git push origin main && git push origin {args.crate + '/' if args.crate else ''}v{new_version}")
        else:
            print(f"\n{Colors.YELLOW}Run with --apply to update Cargo.toml{Colors.END}")
        
    except Exception as e:
        print(f"\n{Colors.RED}Error: {e}{Colors.END}")
        sys.exit(1)


if __name__ == '__main__':
    main()
