# Contributing to vfv

Thank you for your interest in contributing to vfv!

> **日本語版**: [CONTRIBUTING_ja.md](CONTRIBUTING_ja.md)

## Development Setup

### Prerequisites

- Rust 1.85 or later
- Git

### Building from Source

```bash
git clone https://github.com/noumi0k/vfv.git
cd vfv
cargo build
```

### Running Tests

```bash
cargo test
```

### Running with Debug Output

```bash
cargo run -- ~/some/directory
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Run tests with `cargo test`
6. Commit your changes with a descriptive message
7. Push to your branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all clippy warnings (`cargo clippy`)
- Keep functions small and focused
- Add comments for non-obvious logic

## Commit Messages

- Use the imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Reference issues when applicable

## Reporting Issues

When reporting issues, please include:

- vfv version (`vfv --version`)
- OS and version
- Steps to reproduce
- Expected vs actual behavior

## For Maintainers

### Releasing

We use [cargo-release](https://github.com/crate-ci/cargo-release) to automate version bumping, tagging, and changelog updates.

```bash
# Install cargo-release (once)
cargo install cargo-release

# Dry run (preview changes without executing)
cargo release patch

# Execute release
cargo release patch --execute   # 0.1.1 → 0.1.2
cargo release minor --execute   # 0.1.1 → 0.2.0
cargo release major --execute   # 0.1.1 → 1.0.0
```

This will:
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` (`[Unreleased]` → `[version] - date`)
3. Create commit: "Release {version}"
4. Create tag: `v{version}`
5. Push to remote

GitHub Actions will then automatically:
- Build binaries for Linux/macOS/Windows
- Create GitHub Release with binaries
- Publish to crates.io

### Manual Release (without cargo-release)

1. Update `CHANGELOG.md` with the new version
2. Update version in `Cargo.toml`
3. Commit: `git commit -am "Release vX.Y.Z"`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push && git push --tags`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
