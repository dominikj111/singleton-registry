# singleton-registry

[![crates.io](https://img.shields.io/crates/v/singleton-registry.svg)](https://crates.io/crates/singleton-registry)
[![docs.rs](https://docs.rs/singleton-registry/badge.svg)](https://docs.rs/singleton-registry)
[![license](https://img.shields.io/badge/license-BSD--3--Clause-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.80%2B-orange.svg)](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html)
[![dependencies](https://deps.rs/repo/github/dominikj111/singleton-registry/status.svg)](https://deps.rs/repo/github/dominikj111/singleton-registry)
[![status](https://img.shields.io/badge/status-stable-brightgreen.svg)](CHANGELOG.md)

A **thread-safe singleton registry** for Rust.  
Create isolated registries for storing and retrieving any type.  
Each type can have only **one instance** per registry.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
singleton-registry = "2.1.1"
```

## Features & Design

- **Synchronous API**: No async/await complexity - simple function calls
- **Thread-safe**: All operations safe across multiple threads using `Arc` and `Mutex`
- **Isolated registries**: Create multiple independent registries with `define_registry!` - no hidden globals
- **True singleton**: Only one instance per type per registry
- **Write-once pattern**: Designed for initialization-time registration with optional runtime overrides
- **No removal**: Values can be overridden but not removed - provide default values for fail-safe operation
- **Override-friendly**: Later registrations replace previous ones
- **Tracing support**: Optional callback system for monitoring operations

## Design Philosophy

This crate implements a **service locator** pattern in the sense Martin Fowler defined in his 2004 article [_Inversion of Control Containers and the Dependency Injection Pattern_](https://martinfowler.com/articles/injection.html#UsingAServiceLocator) — extended with explicit contracts as Rust traits.

Fowler's known trade-off applies: every caller has a dependency on the locator itself, and the dependencies of a component are not visible from its signature. This crate accepts that trade-off deliberately — the benefit is that a component can ask for a capability that may not exist yet and degrade gracefully (via `try_get`) rather than failing at construction time.

### Contracts as Traits

A **contract** is a trait (interface) that defines the API a singleton must fulfill. By registering trait objects (`Arc<dyn MyTrait>`), you decouple consumers from concrete implementations. Any part of your system can request the contract without knowing which implementation backs it.

### Singleton Replacement & Arc Safety

When you re-register a type, the registry atomically replaces the stored `Arc`. However, **existing references remain valid**:

```rust
let old_ref: Arc<MyService> = registry::get().unwrap();  // Holds Arc clone
registry::register(new_service);                          // Registry updated
// old_ref still works - it holds the previous Arc until dropped
let new_ref: Arc<MyService> = registry::get().unwrap();  // Gets new instance
```

This enables runtime replacement (e.g., hot-swapping configurations) without breaking in-flight operations.

### Unit Testing Without Mocking Libraries

Register mock implementations during test setup:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    define_registry!(test_registry);
    
    #[test]
    fn test_with_mock() {
        test_registry::register(Arc::new(MockService) as Arc<dyn ServiceContract>);
        // Test code uses test_registry::get() - no external mocking crate needed
    }
}
```

### Enforcing Good Architecture

The registry pattern encourages:

- **Interface segregation**: Define focused contracts (traits)
- **Dependency inversion**: Depend on abstractions, not concretions
- **Single responsibility**: Each singleton handles one concern

## Quick Start

```rust
use singleton_registry::define_registry;
use std::sync::Arc;

define_registry!(global);
define_registry!(cache);

global::register("Hello, World!".to_string());
cache::register(42i32);

let message: Arc<String> = global::get().unwrap();
let number: Arc<i32> = cache::get().unwrap();

assert_eq!(&*message, "Hello, World!");
assert_eq!(*number, 42);
```

## Advanced Usage

```rust
use singleton_registry::define_registry;
use std::sync::Arc;

define_registry!(app);

app::set_trace_callback(|event| {
    println!("Registry event: {}", event);
});

app::register(12i32);
app::register("config".to_string());

let multiply_by_two: fn(i32) -> i32 = |x| x * 2;
app::register(multiply_by_two);

assert!(app::contains::<i32>().unwrap());

let number: Arc<i32> = app::get().unwrap();
let config: Arc<String> = app::get().unwrap();
let doubler: Arc<fn(i32) -> i32> = app::get().unwrap();

let result = doubler(21);

assert_eq!(result, 42);
assert_eq!(*number, 12);
assert_eq!(&*config, "config");
```

## Multiple Isolated Registries

```rust
use singleton_registry::define_registry;

define_registry!(database);
define_registry!(cache);
define_registry!(config);

database::register("postgresql://localhost".to_string());
cache::register("redis://localhost".to_string());
config::register("app_config".to_string());

let db_conn = database::get::<String>().unwrap();
let cache_conn = cache::get::<String>().unwrap();
```

## API Reference

Each registry created with `define_registry!(name)` provides:

- `name::register(value)` - Register a value
- `name::register_arc(arc_value)` - Register an Arc-wrapped value
- `name::get::<T>()` - Retrieve a value as `Arc<T>` (returns `Result`)
- `name::try_get::<T>()` - Retrieve a value as `Option<Arc<T>>` (returns `None` instead of `Err`)
- `name::get_cloned::<T>()` - Retrieve a cloned value (requires `Clone`, returns `Result`)
- `name::contains::<T>()` - Check if a type is registered (returns `Result`)
- `name::set_trace_callback(callback)` - Set up tracing
- `name::clear_trace_callback()` - Clear tracing

## Error Handling

All fallible operations return `Result<T, RegistryError>`:

```rust
pub enum RegistryError {
    /// Type not found in the registry
    TypeNotFound { type_name: &'static str },

    /// Type mismatch during retrieval (should never happen)
    TypeMismatch { type_name: &'static str },

    /// Failed to acquire registry lock (automatically recovered)
    RegistryLock,
}
```

**Example:**

```rust
use singleton_registry::define_registry;

define_registry!(app);

// Handle errors explicitly
match app::get::<String>() {
    Ok(value) => println!("Found: {}", value),
    Err(e) => eprintln!("Error: {}", e),  // "Type not found in registry: alloc::string::String"
}

// Or use ? operator
fn get_config() -> Result<std::sync::Arc<String>, singleton_registry::RegistryError> {
    app::get::<String>()
}
```

**Note on Lock Poisoning:** The registry automatically recovers from poisoned locks by extracting the inner value. This is safe because registry operations are idempotent.

## Use Cases

- **Application singletons** (Config, Logger, DatabasePool, etc.)
- **Isolated contexts** (per-module registries, test isolation)
- **Function helpers** and utility closures
- **Shared resources** and components
- **Service locator pattern** with type safety

**Best Practice:** Register all required types during initialization-time to ensure `get()` never fails during runtime.

## Roadmap

### Future Considerations

- `get_or_default()` - Convenience method with fallback values
- `get_or_init()` - Lazy initialization support
- `register_if_absent()` - Conditional registration

### Non-Goals

- Async support (keeping it synchronous by design)
- Removal operations — override with a null object (a no-op implementation satisfying the same trait contract) to safely "disable" a registered value without risking a missing-type panic at call sites

See [CHANGELOG.md](CHANGELOG.md) for version history and [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## Running Examples

The `examples/` directory contains runnable demonstrations:

```bash
# Basic usage: primitives, structs, get(), get_cloned(), contains()
cargo run --example basic_usage

# Trait contracts: register and retrieve trait objects
cargo run --example trait_contracts

# Singleton replacement: Arc reference safety during runtime swaps
cargo run --example singleton_replacement
```

## JigsawFlow

This crate is the core building block of the [JigsawFlow Microkernel](https://github.com/dominikj111/JigsawFlow) pattern — a capability-driven architecture for offline-first, hot-swappable, language-agnostic applications. The registry is what makes the pattern possible: everything else in JigsawFlow is built on top of it. It is fully usable standalone.

## Porting to Other Languages

For implementing a similar registry in TypeScript, C++, or another language, porting guidance has been consolidated into the [JigsawFlow PLAN.md](https://github.com/dominikj111/JigsawFlow/blob/main/PLAN.md) (Sections 2 and 11) — covering language-agnostic API surface, token design, thread safety, and reference counting across all ports.

## License

BSD-3-Clause
