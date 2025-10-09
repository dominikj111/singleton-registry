# Contributing to singleton-registry

Thank you for your interest in contributing! 🦀

## Quick Start

1. **Fork and clone** the repository
2. **Create a branch**: `git checkout -b feature/your-feature`
3. **Make changes** following the guidelines below
4. **Run tests**: `cargo fmt && cargo test --all && cargo clippy -- -D warnings`
5. **Commit**: Use format `type: description` (types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`)
6. **Push and create PR** with a clear description

## Guidelines

### Code Quality

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- All tests must pass, no clippy warnings
- Document public APIs with examples
- Add tests for new functionality
- Use `#[serial]` for tests sharing state

### Project Philosophy

- **Simplicity over flexibility** - One instance per type by design
- **Safety over performance** - No unsafe code without strong justification
- **Explicit over implicit** - Clear error handling
- **Minimal dependencies** - Zero runtime dependencies
- **Stability** - SemVer compliance, breaking changes = major bump

### Before Submitting

- [ ] Tests pass: `cargo test --all`
- [ ] No warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] Formatted: `cargo fmt`
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (for significant changes)

## Reporting Issues

Include:

- Clear description and steps to reproduce
- Expected vs. actual behavior
- Minimal code example
- Rust version and OS

## Questions?

Open an issue or check existing discussions. This is a volunteer-maintained project.

## License

By contributing, you agree your contributions will be licensed under BSD-3-Clause.

---

See [README.md](README.md) roadmap for planned features.
