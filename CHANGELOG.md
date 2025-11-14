# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Multi-repo setup with centralized dependency management
- Conventional commits enforcement via commitlint
- Automated CI/CD with GitHub Actions
- Cross-platform build support (Linux, macOS, Windows)
- MSRV testing (3 stable versions + nightly)
- Static linking configuration
- Version consistency checker script
- Automated release notes generation
- GitHub Packages integration
- Comprehensive documentation (SETUP_GUIDE.md, ARCHITECTURE.md)
- Makefile with common development tasks
- Configuration sync script for multi-repo management

### Changed

- Updated dependency management to use workspace inheritance
- Restructured CI/CD workflows for better efficiency

### Security

- Added automated security audits via cargo-audit

## Template for Future Releases

## [X.Y.Z] - YYYY-MM-DD

### Added

- New features

### Changed

- Changes in existing functionality

### Deprecated

- Soon-to-be removed features

### Removed

- Removed features

### Fixed

- Bug fixes

### Security

- Security fixes

---

**Note:** Release notes are automatically generated from commit messages using conventional commits.
Manual entries in this file should supplement the automated changelog.
