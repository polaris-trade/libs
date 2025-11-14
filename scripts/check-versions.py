#!/usr/bin/env python3
"""
Dependency Version Consistency Checker

This script validates that all Cargo.toml files in the workspace use dependency
versions that match the global dependency-versions.toml file.

Usage:
    python scripts/check-versions.py [--fix]
    
Options:
    --fix    Automatically update Cargo.toml files to match global versions
"""

import tomllib
import tomli_w
import sys
from pathlib import Path
from typing import Dict, List, Tuple
import argparse


class Colors:
    RED = '\033[91m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    BLUE = '\033[94m'
    END = '\033[0m'


def load_global_versions(root_path: Path) -> Dict:
    """Load the global dependency-versions.toml file."""
    versions_file = root_path / "dependency-versions.toml"
    if not versions_file.exists():
        print(f"{Colors.RED}Error: dependency-versions.toml not found{Colors.END}")
        sys.exit(1)
    
    with open(versions_file, 'rb') as f:
        return tomllib.load(f)


def normalize_version_spec(spec) -> str:
    """Normalize a version specification to a comparable string."""
    if isinstance(spec, str):
        return spec
    elif isinstance(spec, dict) and 'version' in spec:
        return spec['version']
    return ""


def extract_dependencies(global_versions: Dict) -> Dict[str, str]:
    """Extract all dependency names and versions from global config."""
    deps = {}
    
    # Skip metadata section
    for section_name, section in global_versions.items():
        if section_name == "metadata":
            continue
            
        # Handle nested sections (e.g., blockchain.evm)
        if isinstance(section, dict):
            for dep_name, dep_spec in section.items():
                if isinstance(dep_spec, dict) or isinstance(dep_spec, str):
                    version = normalize_version_spec(dep_spec)
                    if version:
                        deps[dep_name] = version
    
    return deps


def check_cargo_toml(cargo_path: Path, global_deps: Dict[str, str], fix: bool = False) -> List[Tuple[str, str, str]]:
    """
    Check a Cargo.toml file for version mismatches.
    
    Returns:
        List of (dependency_name, expected_version, actual_version) tuples
    """
    with open(cargo_path, 'rb') as f:
        cargo_toml = tomllib.load(f)
    
    mismatches = []
    modified = False
    
    # Check all dependency sections
    dep_sections = ['dependencies', 'dev-dependencies', 'build-dependencies']
    
    for section in dep_sections:
        if section not in cargo_toml:
            continue
            
        deps = cargo_toml[section]
        for dep_name, dep_spec in deps.items():
            if dep_name not in global_deps:
                continue  # Not a globally managed dependency
                
            actual_version = normalize_version_spec(dep_spec)
            expected_version = global_deps[dep_name]
            
            if actual_version != expected_version:
                mismatches.append((dep_name, expected_version, actual_version))
                
                if fix:
                    # Update the version
                    if isinstance(dep_spec, str):
                        cargo_toml[section][dep_name] = expected_version
                    elif isinstance(dep_spec, dict):
                        cargo_toml[section][dep_name]['version'] = expected_version
                    modified = True
    
    # Write back if we made changes
    if modified and fix:
        with open(cargo_path, 'w') as f:
            # Read original file to preserve formatting where possible
            with open(cargo_path, 'r') as rf:
                content = rf.read()
            
            # For now, we'll use tomli_w which will reformat
            # In production, consider using toml-edit crate via Rust
            tomli_w.dump(cargo_toml, f)
            print(f"{Colors.GREEN}✓ Fixed {cargo_path}{Colors.END}")
    
    return mismatches


def find_cargo_tomls(root_path: Path) -> List[Path]:
    """Find all Cargo.toml files in the workspace."""
    cargo_files = []
    
    # Find workspace members
    workspace_toml = root_path / "Cargo.toml"
    if workspace_toml.exists():
        with open(workspace_toml, 'rb') as f:
            workspace = tomllib.load(f)
            
        if 'workspace' in workspace and 'members' in workspace['workspace']:
            for member in workspace['workspace']['members']:
                member_cargo = root_path / member / "Cargo.toml"
                if member_cargo.exists():
                    cargo_files.append(member_cargo)
    
    return cargo_files


def main():
    parser = argparse.ArgumentParser(description='Check dependency version consistency')
    parser.add_argument('--fix', action='store_true', help='Automatically fix version mismatches')
    parser.add_argument('--root', type=str, default='.', help='Root directory of the workspace')
    args = parser.parse_args()
    
    root_path = Path(args.root).resolve()
    
    print(f"{Colors.BLUE}🔍 Checking dependency versions...{Colors.END}\n")
    
    # Load global versions
    global_versions = load_global_versions(root_path)
    global_deps = extract_dependencies(global_versions)
    
    print(f"Loaded {len(global_deps)} global dependency versions\n")
    
    # Find all Cargo.toml files
    cargo_files = find_cargo_tomls(root_path)
    
    if not cargo_files:
        print(f"{Colors.YELLOW}No Cargo.toml files found in workspace members{Colors.END}")
        return
    
    print(f"Checking {len(cargo_files)} workspace members...\n")
    
    # Check each file
    all_mismatches = []
    for cargo_file in cargo_files:
        mismatches = check_cargo_toml(cargo_file, global_deps, args.fix)
        if mismatches:
            all_mismatches.append((cargo_file, mismatches))
    
    # Report results
    if not all_mismatches:
        print(f"{Colors.GREEN}✓ All dependencies match global versions!{Colors.END}")
        sys.exit(0)
    else:
        print(f"{Colors.RED}✗ Found version mismatches:{Colors.END}\n")
        
        for cargo_file, mismatches in all_mismatches:
            print(f"{Colors.YELLOW}{cargo_file.relative_to(root_path)}{Colors.END}")
            for dep_name, expected, actual in mismatches:
                print(f"  {dep_name}: expected {expected}, got {actual}")
            print()
        
        if args.fix:
            print(f"{Colors.GREEN}Versions have been automatically fixed.{Colors.END}")
            print(f"{Colors.YELLOW}Note: Manual review recommended for complex specifications.{Colors.END}")
        else:
            print(f"{Colors.YELLOW}Run with --fix to automatically update versions.{Colors.END}")
        
        sys.exit(1 if not args.fix else 0)


if __name__ == '__main__':
    main()
