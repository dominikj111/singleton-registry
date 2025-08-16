//! # Singleton Registry
//!
//! A thread-safe dependency injection registry for storing and retrieving global instances.
//! Currently designed for write-once, read-many pattern.
//!
//! This crate provides a type-safe way to register and retrieve instances of any type
//! that implements `Send + Sync + 'static`.
//!
//! ## Quick Start
//!
//! ```rust
//! use singleton_registry::{register, get};
//! use std::sync::Arc;
//!
//! // Register a value
//! register("Hello, World!".to_string());
//!
//! // Retrieve the value
//! let message: Arc<String> = get().unwrap();
//! assert_eq!(&*message, "Hello, World!");
//! ```
//!
//! ## Features
//!
//! - **Thread-safe**: All operations are safe to use across multiple threads
//! - **Type-safe**: Values are stored and retrieved with full type information
//! - **Zero-cost abstractions**: Minimal runtime overhead
//! - **Tracing support**: Optional callback system for monitoring registry operations
//!
//! ## Main Functions
//!
//! - [`register`] - Register a value in the global registry
//! - [`register_arc`] - Register an Arc-wrapped value (more efficient if you already have an Arc)
//! - [`get`] - Retrieve a value as Arc<T>
//! - [`get_cloned`] - Retrieve a cloned value (requires Clone)
//! - [`contains`] - Check if a type is registered
//! - [`set_trace_callback`] - Set up tracing for registry operations

mod registry;

// Re-export the main public API
pub use registry::{
    clear_trace_callback, contains, get, get_cloned, register, register_arc, set_trace_callback,
    RegistryEvent, TraceCallback,
};
