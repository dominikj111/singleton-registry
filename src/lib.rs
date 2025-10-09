//! # Singleton Registry
//!
//! A thread-safe singleton registry for Rust.
//! Store and retrieve **any type** with isolated registries.
//! Each type can have only **one instance** per registry.
//!
//! ## Features
//!
//! - **Thread-safe**: All operations safe across multiple threads
//! - **Isolated registries**: Create multiple independent registries with `define_registry!`
//! - **True singleton**: Only one instance per type per registry
//! - **Override-friendly**: Later registrations replace previous ones
//! - **Write-once, read-many**: Optimized for configuration and shared resources
//! - **Tracing support**: Optional callback system for monitoring
//!
//! ## Usage
//!
//! ```rust
//! use singleton_registry::define_registry;
//! use std::sync::Arc;
//!
//! define_registry!(app);
//!
//! app::set_trace_callback(|event| {
//!     println!("Registry event: {}", event);
//! });
//!
//! app::register(12i32);
//! app::register("config".to_string());
//!
//! let multiply_by_two: fn(i32) -> i32 = |x| x * 2;
//! app::register(multiply_by_two);
//!
//! assert!(app::contains::<i32>().unwrap());
//!
//! let number: Arc<i32> = app::get().unwrap();
//! let config: Arc<String> = app::get().unwrap();
//! let doubler: Arc<fn(i32) -> i32> = app::get().unwrap();
//!
//! let result = doubler(21);
//!
//! assert_eq!(result, 42);
//! assert_eq!(*number, 12);
//! assert_eq!(&*config, "config");
//! ```
//!
//! ## Core API
//!
//! - [`define_registry!`] - Macro to create a registry module with free functions
//! - [`RegistryApi`] - Trait defining registry operations (for advanced usage)
//! - [`RegistryEvent`] - Events emitted during operations (for tracing)

mod macros;
mod registry_event;
mod registry_trait;

// Re-export the public API
pub use registry_event::RegistryEvent;
pub use registry_trait::RegistryApi;

// Macros are exported via #[macro_export] in macros.rs
// They are automatically available at crate root
