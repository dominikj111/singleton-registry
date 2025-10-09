//! Integration tests for tracing and event monitoring.
//!
//! This test demonstrates how to use the tracing callback system to monitor
//! registry operations, which is useful for debugging and logging.

use singleton_registry::define_registry;
use std::sync::Arc;

#[test]
fn test_basic_tracing() {
    define_registry!(traced1);

    // Set up event collection
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Register a trace callback
    traced1::set_trace_callback(move |event| {
        events_clone.lock().unwrap().push(format!("{}", event));
    });

    // Perform operations
    traced1::register(42i32);
    let _: Arc<i32> = traced1::get().unwrap();
    let _ = traced1::contains::<i32>();

    // Verify events were captured
    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 3);
    assert!(captured[0].contains("register"));
    assert!(captured[1].contains("get"));
    assert!(captured[2].contains("contains"));
}

#[test]
fn test_trace_register_event() {
    define_registry!(traced2);

    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    traced2::set_trace_callback(move |event| {
        events_clone.lock().unwrap().push(format!("{}", event));
    });

    traced2::register(999u32);

    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0], "register { type_name: u32 }");

    traced2::clear_trace_callback();
}

#[test]
fn test_trace_get_found_and_not_found() {
    define_registry!(traced3);

    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    traced3::set_trace_callback(move |event| {
        events_clone.lock().unwrap().push(format!("{}", event));
    });

    // Register and get (found)
    traced3::register(123i64);
    let _: Arc<i64> = traced3::get().unwrap();

    // Try to get non-existent type (not found)
    let _: Result<Arc<f32>, _> = traced3::get();

    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 3);
    assert!(captured[1].contains("found: true"));
    assert!(captured[2].contains("found: false"));

    traced3::clear_trace_callback();
}

#[test]
fn test_trace_contains_event() {
    define_registry!(traced4);

    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    traced4::set_trace_callback(move |event| {
        events_clone.lock().unwrap().push(format!("{}", event));
    });

    // Check before registering (not found)
    let _ = traced4::contains::<String>();

    // Register
    traced4::register("test".to_string());

    // Check after registering (found)
    let _ = traced4::contains::<String>();

    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 3);
    assert!(captured[0].contains("contains"));
    assert!(captured[0].contains("found: false"));
    assert!(captured[2].contains("contains"));
    assert!(captured[2].contains("found: true"));

    traced4::clear_trace_callback();
}

#[test]
fn test_clear_trace_callback() {
    define_registry!(traced5);

    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Set callback
    traced5::set_trace_callback(move |event| {
        events_clone.lock().unwrap().push(format!("{}", event));
    });

    // Perform operation (should be traced)
    traced5::register(1u8);

    // Clear callback
    traced5::clear_trace_callback();

    // Perform more operations (should NOT be traced)
    traced5::register(2u8);
    let _: Arc<u8> = traced5::get().unwrap();

    // Verify only the first operation was traced
    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 1);
}

#[test]
fn test_trace_callback_with_custom_logic() {
    define_registry!(traced6);

    // Example: Count operations by type
    let register_count = Arc::new(std::sync::Mutex::new(0));
    let get_count = Arc::new(std::sync::Mutex::new(0));
    let contains_count = Arc::new(std::sync::Mutex::new(0));

    let reg_clone = register_count.clone();
    let get_clone = get_count.clone();
    let con_clone = contains_count.clone();

    traced6::set_trace_callback(move |event| {
        let event_str = format!("{}", event);
        if event_str.contains("register") {
            *reg_clone.lock().unwrap() += 1;
        } else if event_str.contains("get") {
            *get_clone.lock().unwrap() += 1;
        } else if event_str.contains("contains") {
            *con_clone.lock().unwrap() += 1;
        }
    });

    // Perform various operations
    traced6::register(10i16);
    traced6::register(20i16);
    let _: Arc<i16> = traced6::get().unwrap();
    let _: Arc<i16> = traced6::get().unwrap();
    let _ = traced6::contains::<i16>();

    // Verify counts
    assert_eq!(*register_count.lock().unwrap(), 2);
    assert_eq!(*get_count.lock().unwrap(), 2);
    assert_eq!(*contains_count.lock().unwrap(), 1);

    traced6::clear_trace_callback();
}

#[test]
fn test_trace_callback_replacement() {
    define_registry!(traced7);

    let events1 = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events2 = Arc::new(std::sync::Mutex::new(Vec::new()));

    let e1_clone = events1.clone();
    let e2_clone = events2.clone();

    // Set first callback
    traced7::set_trace_callback(move |event| {
        e1_clone.lock().unwrap().push(format!("{}", event));
    });

    traced7::register(100usize);

    // Replace with second callback
    traced7::set_trace_callback(move |event| {
        e2_clone.lock().unwrap().push(format!("{}", event));
    });

    traced7::register(200usize);

    // First callback should have 1 event, second should have 1 event
    assert_eq!(events1.lock().unwrap().len(), 1);
    assert_eq!(events2.lock().unwrap().len(), 1);

    traced7::clear_trace_callback();
}

#[test]
fn test_callback_can_use_different_registry() {
    define_registry!(main_registry);
    define_registry!(log_registry);

    use std::sync::Mutex;

    // Use a Vec to collect all events since registry only stores one value per type
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Set up a trace callback that logs to a different registry
    main_registry::set_trace_callback(move |event| {
        // This is SAFE - we're using a different registry
        events_clone.lock().unwrap().push(format!("{}", event));
        log_registry::register(format!("Last event: {}", event));
    });

    // Register a value in main registry
    main_registry::register(42i32);

    // Verify the main value was registered
    let value: Arc<i32> = main_registry::get().unwrap();
    assert_eq!(*value, 42);

    // Verify the trace was logged
    let captured = events.lock().unwrap();
    assert!(captured[0].contains("register"));
    assert!(captured[0].contains("i32"));

    // Verify we can also use another registry in the callback
    let last_log: Arc<String> = log_registry::get().unwrap();
    assert!(last_log.contains("get"));

    main_registry::clear_trace_callback();
}
