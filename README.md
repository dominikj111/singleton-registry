# singleton-registry

A **thread-safe singleton registry** for Rust.  
Create isolated registries for storing and retrieving any type.  
Each type can have only **one instance** per registry.

> **⚠️ Breaking Change in v2.0**: This version uses a macro-based API. If you're upgrading from v1.x, you'll need to use `define_registry!` to create registries instead of using global functions. See the Quick Start below for the new API.

## Features

- **Synchronous API**: No async/await complexity - simple function calls
- **Thread-safe**: All operations safe across multiple threads
- **True singleton**: Only one instance per type per registry
- **Isolated registries**: Create multiple independent registries with `define_registry!`
- **Override-friendly**: Later registrations replace previous ones
- **Write-once, read-many**: Optimized for configuration and shared resources
- **Tracing support**: Optional callback system for monitoring

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
- `name::get::<T>()` - Retrieve a value as `Arc<T>`
- `name::get_cloned::<T>()` - Retrieve a cloned value (requires `Clone`)
- `name::contains::<T>()` - Check if a type is registered
- `name::set_trace_callback(callback)` - Set up tracing
- `name::clear_trace_callback()` - Clear tracing

## Use Cases

- **Application singletons** (Config, Logger, DatabasePool, etc.)
- **Isolated contexts** (per-module registries, test isolation)
- **Function helpers** and utility closures
- **Shared resources** and components
- **Service locator pattern** with type safety

## Design Philosophy

- **Explicit**: Must create registries with `define_registry!` - no hidden globals
- **Isolated**: Each registry is independent - no cross-contamination
- **Thread-safe**: All operations safe across threads

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
singleton-registry = "2.0"
```

## License

BSD-3-Clause
