# Changelog

All notable changes to this project will be documented in this file.
## [Unreleased]

### Other

- style: apply rustfmt
## [0.2.0] - 2026-02-06

### Added

- feat: add vfv init command with shell completions

### Changed

- refactor: add validation and improve error handling

### Documentation

- docs: reorder README sections and update setup instructions

### Fixed

- fix: correct release.toml format for cargo-release
- fix: security improvements and bug fixes

### Other

- chore: release 0.2.0
- test: add search tests and CLI integration tests
- ci: add git-cliff for automatic changelog generation
- Update CONTRIBUTING docs and remove outdated docs/
- Add cargo-release config and maintainer docs
## [0.1.1] - 2026-02-06

### Other

- Release v0.1.1: Publish to crates.io as vfv
- README_ja.mdにデモ動画を追加
- Add movie to README
- OSS公開準備の仕上げ
- 品質改善: バイナリサイズ最適化、テスト追加、ドキュメント修正
- OSS化のための準備
- TUI検索の基準ディレクトリを現在開いているディレクトリに変更し、-b オプションを追加
- Add exact match, path filtering, and TUI search options
- Support path matching when query contains /
- Update ratatui to 0.30 to fix lru vulnerability
- Add CLI search command for AI/script integration
- Add UX improvements: jump navigation, help screen, and more
- Initial commit: vfv - Terminal file viewer

