//! # Singleton Registry
//!
//! A thread-safe singleton registry for Rust.
//! Store and retrieve **any type** globally - structs, primitives, functions, or closures.
//! Each type can have only **one instance** registered at a time (true singleton pattern).
//! Designed for write-once, read-many pattern with minimal overhead.
//!
//! ## Quick Start
//!
//! ```rust
//! use singleton_registry::define_registry;
//! use std::sync::Arc;
//!
//! // Create registries - each is isolated
//! define_registry!(global);
//! define_registry!(cache);
//!
//! // Register values
//! global::register("Hello, World!".to_string());
//! cache::register(42i32);
//!
//! // Retrieve values
//! let message: Arc<String> = global::get().unwrap();
//! let number: Arc<i32> = cache::get().unwrap();
//!
//! assert_eq!(&*message, "Hello, World!");
//! assert_eq!(*number, 42);
//! ```
//!
//! ## Features
//!
//! - **Simple API**: Direct function calls - no async/await required
//! - **Thread-safe**: All operations are safe to use across multiple threads
//! - **Type-safe**: Values are stored and retrieved with full type information
//! - **True singleton**: Only one instance per type - later registrations override previous ones
//! - **Minimal overhead**: Efficient Arc-based storage with fast lookups
//! - **Tracing support**: Optional callback system for monitoring registry operations
//! - **No external dependencies**: Pure Rust implementation
//!
//! ## Main API
//!
//! - [`define_registry!`] - Macro to create a registry with ergonomic free functions
//! - [`RegistryApi`] - Trait providing the underlying registry operations
//! - [`RegistryEvent`] - Events emitted during registry operations (for tracing)

mod macros;
mod registry_event;
mod registry_trait;

// Re-export the public API
pub use registry_event::RegistryEvent;
pub use registry_trait::RegistryApi;

// Macros are exported via #[macro_export] in macros.rs
// They are automatically available at crate root
