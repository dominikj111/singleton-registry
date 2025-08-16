# singleton-registry

A **thread-safe singleton registry** for Rust.  
Store and retrieve **any type** globally - structs, primitives, functions, or closures.  
Each type can have only **one instance** registered at a time (true singleton pattern).  
Designed for write-once, read-many pattern with zero-cost abstractions.

## Features

- **Thread-safe**: All operations are safe to use across multiple threads
- **Type-safe**: Values are stored and retrieved with full type information
- **True singleton**: Only one instance per type - later registrations override previous ones
- **Zero-cost abstractions**: Minimal runtime overhead
- **Tracing support**: Optional callback system for monitoring registry operations
- **No external dependencies**: Pure Rust implementation

## Quick Start

```rust
use singleton_registry::{register, get};
use std::sync::Arc;

// Register a value (only one String can be registered)
register("Hello, World!".to_string());

// Later registration of same type overrides the previous one
register("New message!".to_string());

// Retrieve the latest value
let message: Arc<String> = get().unwrap();
assert_eq!(&*message, "New message!");
```

## Advanced Usage

```rust
use singleton_registry::{register, get, contains, set_trace_callback};
use std::sync::Arc;

// Register different types
register(42i32);
register("config".to_string());

// Register a function pointer
let multiply_by_two: fn(i32) -> i32 = |x| x * 2;
register(multiply_by_two);

// Check if a type is registered
assert!(contains::<i32>().unwrap());

// Retrieve values
let number: Arc<i32> = get().unwrap();
let config: Arc<String> = get().unwrap();
let doubler: Arc<fn(i32) -> i32> = get().unwrap();

// Use the function
let result = doubler(21); // returns 42

// Set up tracing
set_trace_callback(|event| {
    println!("Registry event: {}", event);
});
```

## API Reference

- `register(value)` - Register a value in the global registry
- `register_arc(arc_value)` - Register an Arc-wrapped value (more efficient)
- `get::<T>()` - Retrieve a value as `Arc<T>`
- `get_cloned::<T>()` - Retrieve a cloned value (requires `Clone`)
- `contains::<T>()` - Check if a type is registered
- `set_trace_callback(callback)` - Set up tracing for registry operations
- `clear_trace_callback()` - Disable tracing

## Use Cases

- **Application singletons** (Config, Logger, DatabasePool, etc.)
- **Global variables** and constants shared across modules
- **Function helpers** and utility closures accessible anywhere
- **Service location** for shared components
- **Cross-cutting concerns** shared across modules

## Design Philosophy

- **Simple**: Clean API without complex macros or derive attributes
- **Safe**: Values stored in `Arc<T>` with full type checking
- **Global**: One central registry shared across the entire program
- **Singleton**: Each type can only have one registered instance - true singleton behavior
- **Override-friendly**: Later registrations replace previous ones for the same type
- **Efficient**: Write-once, read-many pattern optimized for performance

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
singleton-registry = "1.0"
```

## License

BSD-3-Clause
