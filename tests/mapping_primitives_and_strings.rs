//! Integration tests for registering and retrieving primitive types and strings.
//!
//! This test demonstrates the basic usage of the singleton registry with
//! common Rust types like integers, floats, booleans, and strings.
//!
//! NOTE: All tests use #[serial] because they share the same registry (advanced).
//! Running them in parallel could cause interference.

use serial_test::serial;
use singleton_registry::define_registry;
use std::sync::Arc;

// Create a registry for these tests
define_registry!(primitives);

#[test]
#[serial]
fn test_register_and_retrieve_integer() {
    // Register an integer value
    primitives::register(42i32);

    // Retrieve it back
    let value: Arc<i32> = primitives::get().unwrap();
    assert_eq!(*value, 42);
}

#[test]
#[serial]
fn test_register_and_retrieve_string() {
    // Register a String
    primitives::register("Hello, World!".to_string());

    // Retrieve it back
    let value: Arc<String> = primitives::get().unwrap();
    assert_eq!(&**value, "Hello, World!");
}

#[test]
#[serial]
fn test_register_and_retrieve_str_reference() {
    // Register a &'static str
    primitives::register("some static string 123");

    // Retrieve it back
    let value: Arc<&str> = primitives::get().unwrap();
    assert_eq!(*value, "some static string 123");
}

#[test]
#[serial]
fn test_get_cloned_string() {
    // Register a String
    primitives::register("some cloned value 123".to_string());

    // Get a cloned copy (owned value, not Arc)
    let value: String = primitives::get_cloned().unwrap();
    assert_eq!(value, "some cloned value 123");
}

#[test]
#[serial]
fn test_register_and_retrieve_float() {
    // Register a float value
    primitives::register(3.14f64);

    // Retrieve it back
    let value: Arc<f64> = primitives::get().unwrap();
    assert_eq!(*value, 3.14);
}

#[test]
#[serial]
fn test_register_and_retrieve_boolean() {
    // Register a boolean
    primitives::register(true);

    // Retrieve it back
    let value: Arc<bool> = primitives::get().unwrap();
    assert_eq!(*value, true);
}

#[test]
#[serial]
fn test_overwrite_same_type() {
    // Register an initial value
    primitives::register(100u32);

    // Overwrite with a new value of the same type
    primitives::register(200u32);

    // Should retrieve the latest value
    let value: Arc<u32> = primitives::get().unwrap();
    assert_eq!(*value, 200);
}

#[test]
#[serial]
fn test_contains_check() {
    // Register a value
    primitives::register(999i64);

    // Check if type exists
    assert!(primitives::contains::<i64>().unwrap());

    // Check for non-existent type
    assert!(!primitives::contains::<i8>().unwrap());
}

#[test]
#[serial]
fn test_register_arc_directly() {
    // Create an Arc manually
    let value = Arc::new(777u16);

    // Register the Arc directly (more efficient)
    primitives::register_arc(value.clone());

    // Retrieve it
    let retrieved: Arc<u16> = primitives::get().unwrap();
    assert_eq!(*retrieved, 777);
}
