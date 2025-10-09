<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2025-10-09

### ⚠️ Breaking Changes

- **Complete API redesign**: v2.0 introduces a macro-based API that is incompatible with v1.x
- Registries must now be explicitly defined using `define_registry!(name)` macro
- No more global singleton functions - each registry is isolated
- Error handling now uses `Result<T, RegistryError>` for fallible operations

### Added

- **Macro-based registry creation**: `define_registry!` macro for creating isolated registries
- **Trait-based architecture**: `RegistryApi` trait with default implementations
- **Comprehensive error handling**: New `RegistryError` enum with detailed error types
  - `TypeNotFound` - requested type not in registry
  - `TypeMismatch` - internal type mismatch (rare)
  - `RegistryLock` - lock poisoning (auto-recovered)
- **Event tracing system**: `RegistryEvent` enum for monitoring operations
  - `Register`, `Get`, `Contains`, `Clear` events
  - Optional callback system via `set_trace_callback()`
- **New API methods**:
  - `register_arc()` - register pre-wrapped Arc values
  - `get_cloned()` - retrieve owned clones
  - `contains()` - check type existence
  - `set_trace_callback()` / `clear_trace_callback()` - event monitoring
  - `clear()` - test-only method to clear registry
- **Lock poisoning recovery**: Automatic recovery from poisoned locks
- **Comprehensive test suite**: 84+ tests covering all functionality
  - Unit tests for core operations
  - Integration tests for advanced patterns
  - Thread safety tests
  - Tracing and event tests
  - Cross-registry callback tests
- **MSRV specification**: Rust 1.80.0 (requires `LazyLock`)
- **Enhanced documentation**:
  - Detailed API documentation with examples
  - Safety restrictions for trace callbacks
  - Lock poisoning recovery documentation
  - Advanced usage patterns and examples

### Changed

- Registry storage now uses `LazyLock` instead of `lazy_static`
- All operations return `Result` for better error handling (except `register`)
- Thread safety now explicit with `Send + Sync` bounds
- Documentation significantly expanded with real-world examples

### Fixed

- Typo in documentation: "rarelly" → "rarely"
- Clippy warnings: `clone_on_copy`, `approx_constant`, `bool_assert_comparison`, `vec_init_then_push`

### Migration Guide (v1.x → v2.0)

**v1.x code:**

```rust
use singleton_registry::{register, get};

register(42i32);
let value = get::<i32>();
```

**v2.0 code:**

```rust
use singleton_registry::define_registry;

define_registry!(app);

app::register(42i32);
let value: Arc<i32> = app::get().unwrap();
```

**Key differences:**

1. Must define registries with `define_registry!` macro
2. `get()` returns `Result<Arc<T>, RegistryError>` instead of `Option<Arc<T>>`
3. Multiple isolated registries supported
4. Explicit error handling required

## [1.0.1] - 2025-08-16

### Changed

- Updated Cargo.toml metadata
- Specified synchronous API (no async overhead) in documentation
- Limited keywords to 5 (crates.io requirement)

### Fixed

- Updated .gitignore

### Documentation

- Clarified process-wide sharing behavior
- Added context resolution notes in README

## [1.0.0] - 2025-08-16

### Added

- Initial release
- Thread-safe global singleton registry
- Support for primitives, structs, and function pointers
- Basic `register()` and `get()` API
- Complete test coverage
- MIT license

[Unreleased]: https://github.com/dominikj111/singleton-registry/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/dominikj111/singleton-registry/compare/v1.0.1...v2.0.0
[1.0.1]: https://github.com/dominikj111/singleton-registry/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/dominikj111/singleton-registry/releases/tag/v1.0.0
