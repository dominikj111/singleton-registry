# singleton-registry

A **thread-safe singleton registry** for Rust.  
Create isolated registries for storing and retrieving any type.  
Each type can have only **one instance** per registry.

> **⚠️ Breaking Change in v2.0**: This version uses a macro-based API. If you're upgrading from v1.x, you'll need to use `define_registry!` to create registries instead of using global functions. See the Quick Start below for the new API.

## Features & Design

- **Synchronous API**: No async/await complexity - simple function calls
- **Thread-safe**: All operations safe across multiple threads using `Arc` and `Mutex`
- **Isolated registries**: Create multiple independent registries with `define_registry!` - no hidden globals
- **True singleton**: Only one instance per type per registry
- **Write-once pattern**: Designed for initialization-time registration with optional runtime overrides
- **No removal**: Values can be overridden but not removed - provide default values for fail-safe operation
- **Override-friendly**: Later registrations replace previous ones
- **Tracing support**: Optional callback system for monitoring operations

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

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
singleton-registry = "2.0"
```

## License

BSD-3-Clause
