//! Integration tests for registering and retrieving closures and function pointers.
//!
//! This test demonstrates advanced usage where you can store callable types
//! in the registry, including function pointers, closures, and boxed functions.
//!
//! NOTE: All tests use #[serial] because they share the same registry (advanced).
//! Running them in parallel could cause interference.

use serial_test::serial;
use singleton_registry::define_registry;
use std::sync::Arc;

// Create a registry for these tests
define_registry!(functions);

#[test]
#[serial]
fn test_register_function_pointer() {
    // Define a function pointer type
    let multiply_by_two: fn(i32) -> i32 = |x| x * 2;

    // Register the function pointer
    functions::register(multiply_by_two);

    // Retrieve and use it
    let func: Arc<fn(i32) -> i32> = functions::get().unwrap();
    let result = func(21);
    assert_eq!(result, 42);
}

#[test]
#[serial]
fn test_register_boxed_closure() {
    // Create a boxed closure
    let add_ten: Box<dyn Fn(i32) -> i32 + Send + Sync> = Box::new(|x| x + 10);

    // Register it
    functions::register(add_ten);

    // Retrieve and use it
    let func: Arc<Box<dyn Fn(i32) -> i32 + Send + Sync>> = functions::get().unwrap();
    let result = func(32);
    assert_eq!(result, 42);
}

#[test]
#[serial]
fn test_register_function_with_state() {
    // Create a closure that captures state
    let multiplier = 3;
    let multiply: Box<dyn Fn(i32) -> i32 + Send + Sync> = Box::new(move |x| x * multiplier);

    // Register it
    functions::register(multiply);

    // Retrieve and use it
    let func: Arc<Box<dyn Fn(i32) -> i32 + Send + Sync>> = functions::get().unwrap();
    let result = func(14);
    assert_eq!(result, 42);
}

#[test]
#[serial]
fn test_register_multiple_function_types() {
    // You can register different function signatures as different types
    let int_func: fn(i32) -> i32 = |x| x + 1;
    let str_func: fn(&str) -> String = |s| format!("Hello, {}!", s);

    functions::register(int_func);
    functions::register(str_func);

    // Retrieve both
    let f1: Arc<fn(i32) -> i32> = functions::get().unwrap();
    let f2: Arc<fn(&str) -> String> = functions::get().unwrap();

    assert_eq!(f1(41), 42);
    assert_eq!(f2("World"), "Hello, World!");
}

#[test]
#[serial]
fn test_register_callback_pattern() {
    // Common pattern: register a callback function
    type Callback = Box<dyn Fn(String) + Send + Sync>;

    let messages = Arc::new(std::sync::Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    let callback: Callback = Box::new(move |msg| {
        messages_clone.lock().unwrap().push(msg);
    });

    functions::register(callback);

    // Retrieve and use the callback
    let cb: Arc<Callback> = functions::get().unwrap();
    cb("Test message".to_string());

    // Verify it worked
    let msgs = messages.lock().unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0], "Test message");
}
