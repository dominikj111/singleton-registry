//! Integration tests for registry isolation and multiple registries.
//!
//! This test demonstrates that multiple registries are completely isolated
//! from each other, which is the key feature of v2.0.

use singleton_registry::define_registry;
use std::sync::Arc;

#[test]
fn test_multiple_isolated_registries() {
    // Create three separate registries
    define_registry!(database);
    define_registry!(cache);
    define_registry!(config);

    // Register different values in each
    database::register("postgresql://localhost".to_string());
    cache::register("redis://localhost".to_string());
    config::register("app_config".to_string());

    // Retrieve from each registry
    let db: Arc<String> = database::get().unwrap();
    let cache_val: Arc<String> = cache::get().unwrap();
    let cfg: Arc<String> = config::get().unwrap();

    // Verify each registry has its own value
    assert_eq!(&**db, "postgresql://localhost");
    assert_eq!(&**cache_val, "redis://localhost");
    assert_eq!(&**cfg, "app_config");
}

#[test]
fn test_same_type_different_registries() {
    // Create two registries
    define_registry!(reg_a);
    define_registry!(reg_b);

    // Register the same type with different values
    reg_a::register(100i32);
    reg_b::register(200i32);

    // Each registry maintains its own value
    let a: Arc<i32> = reg_a::get().unwrap();
    let b: Arc<i32> = reg_b::get().unwrap();

    assert_eq!(*a, 100);
    assert_eq!(*b, 200);
}

#[test]
fn test_registry_does_not_leak_between_instances() {
    define_registry!(isolated_a);
    define_registry!(isolated_b);

    // Register in one registry
    isolated_a::register("only in A".to_string());

    // Other registry should not have it
    assert!(isolated_a::contains::<String>().unwrap());
    assert!(!isolated_b::contains::<String>().unwrap());

    // Attempting to get from empty registry should fail
    let result: Result<Arc<String>, _> = isolated_b::get();
    assert!(result.is_err());
}

#[test]
fn test_multiple_types_in_multiple_registries() {
    define_registry!(multi_a);
    define_registry!(multi_b);

    // Register different types in each
    multi_a::register(42i32);
    multi_a::register("hello".to_string());

    multi_b::register(std::f64::consts::PI);
    multi_b::register(true);

    // Verify isolation
    assert!(multi_a::contains::<i32>().unwrap());
    assert!(multi_a::contains::<String>().unwrap());
    assert!(!multi_a::contains::<f64>().unwrap());
    assert!(!multi_a::contains::<bool>().unwrap());

    assert!(multi_b::contains::<f64>().unwrap());
    assert!(multi_b::contains::<bool>().unwrap());
    assert!(!multi_b::contains::<i32>().unwrap());
    assert!(!multi_b::contains::<String>().unwrap());
}

#[test]
fn test_registry_scoping() {
    // Demonstrate that registries can be scoped to different modules/contexts
    mod module_a {
        use singleton_registry::define_registry;
        define_registry!(scoped);

        pub fn setup() {
            scoped::register("module A".to_string());
        }

        pub fn get_value() -> String {
            use std::sync::Arc;
            let val: Arc<String> = scoped::get().unwrap();
            val.to_string()
        }
    }

    mod module_b {
        use singleton_registry::define_registry;
        define_registry!(scoped);

        pub fn setup() {
            scoped::register("module B".to_string());
        }

        pub fn get_value() -> String {
            use std::sync::Arc;
            let val: Arc<String> = scoped::get().unwrap();
            val.to_string()
        }
    }

    // Each module has its own registry
    module_a::setup();
    module_b::setup();

    assert_eq!(module_a::get_value(), "module A");
    assert_eq!(module_b::get_value(), "module B");
}

#[test]
fn test_registry_with_tracing_isolation() {
    define_registry!(traced_a);
    define_registry!(traced_b);

    // Set up tracing only for one registry
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let events_clone = events.clone();

    traced_a::set_trace_callback(move |event| {
        events_clone.lock().unwrap().push(format!("{}", event));
    });

    // Register in both
    traced_a::register(1i32);
    traced_b::register(2i32);

    // Only traced_a should have events
    let captured = events.lock().unwrap();
    assert_eq!(captured.len(), 1);
    assert!(captured[0].contains("register"));
}
