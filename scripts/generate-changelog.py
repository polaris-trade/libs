#!/usr/bin/env python3
"""
Changelog Generator

Generates CHANGELOG.md from conventional commits.
Can be run before release to preview and edit changelog.

Usage:
    python scripts/generate-changelog.py --crate data_types [--preview] [--update]
    python scripts/generate-changelog.py --workspace [--preview] [--update]
"""

import subprocess
import sys
import re
from pathlib import Path
from typing import Optional, List, Dict
import argparse
from datetime import datetime


class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    MAGENTA = '\033[95m'
    END = '\033[0m'


def get_latest_tag(crate_name: Optional[str] = None) -> Optional[str]:
    """Get the latest tag for a specific crate or workspace."""
    try:
        result = subprocess.run(
            ['git', 'tag', '--sort=-version:refname'],
            capture_output=True,
            text=True,
            check=True
        )
        tags = [tag.strip() for tag in result.stdout.split('\n') if tag.strip()]
        
        if crate_name:
            pattern = f"{crate_name}/v"
            for tag in tags:
                if tag.startswith(pattern):
                    return tag
        
        for tag in tags:
            if tag.startswith('v') and '/' not in tag:
                return tag
        
        return None
    except subprocess.CalledProcessError:
        return None


def get_commits_since_tag(tag: Optional[str], crate_path: Optional[str] = None) -> List[Dict[str, str]]:
    """Get commits with full information since the given tag."""
    if tag:
        ref_range = f"{tag}..HEAD"
    else:
        ref_range = "HEAD"
    
    # Format: hash|author|date|subject|body
    format_str = "%H|%an|%ai|%s|%b"
    cmd = ['git', 'log', f'--pretty=format:{format_str}', ref_range]
    
    if crate_path:
        cmd.extend(['--', crate_path])
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        commits = []
        
        for line in result.stdout.split('\n'):
            if not line.strip():
                continue
            
            parts = line.split('|', 4)
            if len(parts) >= 4:
                commits.append({
                    'hash': parts[0][:7],  # Short hash
                    'author': parts[1],
                    'date': parts[2][:10],  # Date only
                    'subject': parts[3],
                    'body': parts[4] if len(parts) > 4 else ''
                })
        
        return commits
    except subprocess.CalledProcessError:
        return []


def categorize_commits(commits: List[Dict[str, str]]) -> Dict[str, List[Dict[str, str]]]:
    """Categorize commits by type."""
    categories = {
        'breaking': [],
        'features': [],
        'fixes': [],
        'performance': [],
        'refactor': [],
        'docs': [],
        'tests': [],
        'build': [],
        'ci': [],
        'chore': [],
        'other': []
    }
    
    for commit in commits:
        subject = commit['subject']
        
        # Check for breaking changes
        if re.search(r'^(\w+)(?:\([^)]+\))?!:|BREAKING CHANGE:', subject, re.IGNORECASE) or \
           'BREAKING CHANGE' in commit['body']:
            categories['breaking'].append(commit)
        elif re.match(r'^feat(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['features'].append(commit)
        elif re.match(r'^fix(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['fixes'].append(commit)
        elif re.match(r'^perf(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['performance'].append(commit)
        elif re.match(r'^refactor(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['refactor'].append(commit)
        elif re.match(r'^docs(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['docs'].append(commit)
        elif re.match(r'^test(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['tests'].append(commit)
        elif re.match(r'^build(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['build'].append(commit)
        elif re.match(r'^ci(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['ci'].append(commit)
        elif re.match(r'^chore(?:\([^)]+\))?:', subject, re.IGNORECASE):
            categories['chore'].append(commit)
        else:
            categories['other'].append(commit)
    
    return categories


def format_commit_line(commit: Dict[str, str]) -> str:
    """Format a single commit for the changelog."""
    subject = commit['subject']
    
    # Remove type prefix for cleaner changelog
    subject = re.sub(r'^(\w+)(?:\([^)]+\))?:\s*', '', subject)
    
    return f"- {subject} ([{commit['hash']}](../../commit/{commit['hash']}))"


def generate_changelog_section(version: str, categories: Dict[str, List[Dict[str, str]]]) -> str:
    """Generate a changelog section for a version."""
    today = datetime.now().strftime('%Y-%m-%d')
    changelog = f"## [{version}] - {today}\n\n"
    
    section_map = {
        'breaking': ('💥 Breaking Changes', True),
        'features': ('✨ Features', True),
        'fixes': ('🐛 Bug Fixes', True),
        'performance': ('🚀 Performance', True),
        'refactor': ('♻️ Refactoring', False),
        'docs': ('📚 Documentation', False),
        'tests': ('✅ Tests', False),
        'build': ('🔨 Build System', False),
        'ci': ('⚙️ CI/CD', False),
    }
    
    for category, (title, important) in section_map.items():
        if categories[category]:
            changelog += f"### {title}\n\n"
            for commit in categories[category]:
                changelog += format_commit_line(commit) + "\n"
            changelog += "\n"
    
    return changelog


def read_existing_changelog(changelog_path: Path) -> str:
    """Read existing changelog content."""
    if changelog_path.exists():
        with open(changelog_path, 'r') as f:
            return f.read()
    return ""


def update_changelog(changelog_path: Path, new_section: str):
    """Update CHANGELOG.md with new section."""
    existing = read_existing_changelog(changelog_path)
    
    # Find where to insert (after # Changelog header and [Unreleased] section)
    if existing:
        # Look for the first version section or end of file
        match = re.search(r'\n## \[', existing)
        if match:
            insert_pos = match.start() + 1
            new_content = existing[:insert_pos] + new_section + existing[insert_pos:]
        else:
            # Append after header
            lines = existing.split('\n')
            header_end = 0
            for i, line in enumerate(lines):
                if line.strip() and not line.startswith('#'):
                    header_end = i
                    break
            
            new_content = '\n'.join(lines[:header_end]) + '\n\n' + new_section + '\n'.join(lines[header_end:])
    else:
        # Create new changelog
        new_content = f"""# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

{new_section}
"""
    
    with open(changelog_path, 'w') as f:
        f.write(new_content)


def main():
    parser = argparse.ArgumentParser(description='Generate changelog from conventional commits')
    parser.add_argument('--crate', type=str, help='Crate name (e.g., data_types)')
    parser.add_argument('--workspace', action='store_true', help='Generate for entire workspace')
    parser.add_argument('--version', type=str, required=True, help='Version to generate changelog for')
    parser.add_argument('--preview', action='store_true', help='Preview changelog without writing')
    parser.add_argument('--update', action='store_true', help='Update CHANGELOG.md file')
    parser.add_argument('--root', type=str, default='.', help='Repository root directory')
    args = parser.parse_args()
    
    root = Path(args.root).resolve()
    
    if not args.crate and not args.workspace:
        print(f"{Colors.RED}Error: Specify either --crate or --workspace{Colors.END}")
        sys.exit(1)
    
    print(f"{Colors.BLUE}📝 Generating changelog...{Colors.END}\n")
    
    # Determine paths
    if args.crate:
        crate_path = root / args.crate
        if not crate_path.exists():
            print(f"{Colors.RED}Error: Crate directory not found: {crate_path}{Colors.END}")
            sys.exit(1)
        
        changelog_path = crate_path / "CHANGELOG.md"
        latest_tag = get_latest_tag(args.crate)
        commits = get_commits_since_tag(latest_tag, str(crate_path.relative_to(root)))
        
        print(f"{Colors.YELLOW}Crate:{Colors.END} {args.crate}")
    else:
        changelog_path = root / "CHANGELOG.md"
        latest_tag = get_latest_tag(None)
        commits = get_commits_since_tag(latest_tag)
        
        print(f"{Colors.YELLOW}Scope:{Colors.END} Workspace")
    
    print(f"{Colors.YELLOW}Latest tag:{Colors.END} {latest_tag or 'none'}")
    print(f"{Colors.YELLOW}Version:{Colors.END} {args.version}")
    print(f"{Colors.YELLOW}Commits:{Colors.END} {len(commits)}\n")
    
    if not commits:
        print(f"{Colors.YELLOW}No commits found since last release.{Colors.END}")
        sys.exit(0)
    
    # Categorize and generate
    categories = categorize_commits(commits)
    changelog_section = generate_changelog_section(args.version, categories)
    
    if args.preview or not args.update:
        print(f"{Colors.GREEN}Preview:{Colors.END}\n")
        print(changelog_section)
        
        if not args.update:
            print(f"\n{Colors.YELLOW}Run with --update to write to CHANGELOG.md{Colors.END}")
    
    if args.update:
        update_changelog(changelog_path, changelog_section)
        print(f"{Colors.GREEN}✓ CHANGELOG.md updated at {changelog_path.relative_to(root)}{Colors.END}")
        
        print(f"\n{Colors.YELLOW}Next steps:{Colors.END}")
        print(f"  1. Review and edit {changelog_path.relative_to(root)} if needed")
        print(f"  2. Commit: git add {changelog_path.relative_to(root)}")
        print(f"  3. Continue with release process")


if __name__ == '__main__':
    main()
