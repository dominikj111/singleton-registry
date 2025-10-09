//! Integration tests demonstrating how to use the singleton registry WITHOUT the macro.
//!
//! This shows the manual implementation approach, which gives you full control
//! over the registry setup. This is useful when you need custom behavior or
//! want to understand how the macro works under the hood.
//!
//! NOTE: All tests use #[serial] because they share the same static registry (MY_REGISTRY).
//! Running them in parallel would cause interference and non-deterministic failures.

use serial_test::serial;
use singleton_registry::{RegistryApi, RegistryEvent};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

/// Type alias for the trace callback (same as in registry_trait.rs)
type TraceCallback = LazyLock<Mutex<Option<Arc<dyn Fn(&RegistryEvent) + Send + Sync>>>>;

// ============================================================================
// Manual Registry Implementation (Without Macro)
// ============================================================================

/// Define the static storage for our registry
static MY_STORAGE: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Define the static trace callback storage
static MY_TRACE: TraceCallback = LazyLock::new(|| Mutex::new(None));

/// Our custom registry API implementation
struct MyRegistry;

impl RegistryApi for MyRegistry {
    fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> {
        &MY_STORAGE
    }

    fn trace() -> &'static TraceCallback {
        &MY_TRACE
    }
}

/// Constant instance of our registry
const MY_REGISTRY: MyRegistry = MyRegistry;

// ============================================================================
// Tests Using Manual Implementation
// ============================================================================

#[test]
#[serial]
fn test_basic_register_and_get() {
    // Register a value using the manual registry
    MY_REGISTRY.register(42i32);

    // Retrieve it
    let value: Arc<i32> = MY_REGISTRY.get().unwrap();
    assert_eq!(*value, 42);
}

#[test]
#[serial]
fn test_register_multiple_types() {
    // Register different types
    MY_REGISTRY.register(100u32);
    MY_REGISTRY.register("Hello".to_string());
    MY_REGISTRY.register(3.14f64);

    // Retrieve them
    let num: Arc<u32> = MY_REGISTRY.get().unwrap();
    let text: Arc<String> = MY_REGISTRY.get().unwrap();
    let pi: Arc<f64> = MY_REGISTRY.get().unwrap();

    assert_eq!(*num, 100);
    assert_eq!(&**text, "Hello");
    assert_eq!(*pi, 3.14);
}

#[test]
#[serial]
fn test_contains_check() {
    // Register a value
    MY_REGISTRY.register(999i64);

    // Check if type exists
    assert!(MY_REGISTRY.contains::<i64>().unwrap());

    // Check for non-existent type
    assert!(!MY_REGISTRY.contains::<i8>().unwrap());
}

#[test]
#[serial]
fn test_get_cloned() {
    // Register a String
    MY_REGISTRY.register("cloned".to_string());

    // Get a cloned copy (owned value, not Arc)
    let value: String = MY_REGISTRY.get_cloned().unwrap();
    assert_eq!(value, "cloned");
}

#[test]
#[serial]
fn test_overwrite_same_type() {
    // Register initial value
    MY_REGISTRY.register(10u16);

    // Overwrite with new value
    MY_REGISTRY.register(20u16);

    // Should retrieve the latest value
    let value: Arc<u16> = MY_REGISTRY.get().unwrap();
    assert_eq!(*value, 20);
}

#[test]
#[serial]
fn test_with_tracing() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Counter for trace events
    let event_count = Arc::new(AtomicUsize::new(0));
    let event_count_clone = Arc::clone(&event_count);

    // Set up trace callback
    MY_REGISTRY.set_trace_callback(move |_event| {
        event_count_clone.fetch_add(1, Ordering::SeqCst);
    });

    // Perform operations that trigger events
    MY_REGISTRY.register(777i32); // +1 event
    let _: Arc<i32> = MY_REGISTRY.get().unwrap(); // +1 event
    MY_REGISTRY.contains::<i32>().unwrap(); // +1 event

    // Verify events were traced
    assert_eq!(event_count.load(Ordering::SeqCst), 3);

    // Clean up trace callback
    MY_REGISTRY.clear_trace_callback();
}

#[test]
#[serial]
fn test_register_arc_directly() {
    // Create an Arc manually
    let value = Arc::new(555u32);

    // Register it directly
    MY_REGISTRY.register_arc(value);

    // Retrieve it
    let retrieved: Arc<u32> = MY_REGISTRY.get().unwrap();
    assert_eq!(*retrieved, 555);
}

#[test]
#[serial]
fn test_custom_struct() {
    #[derive(Debug, Clone)]
    struct Config {
        host: String,
        port: u16,
    }

    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
    };

    // Register custom struct
    MY_REGISTRY.register(config);

    // Retrieve it
    let retrieved: Arc<Config> = MY_REGISTRY.get().unwrap();
    assert_eq!(retrieved.host, "localhost");
    assert_eq!(retrieved.port, 8080);
}

#[test]
#[serial]
fn test_trait_object() {
    trait Service: Send + Sync {
        fn name(&self) -> &str;
    }

    struct MyService;
    impl Service for MyService {
        fn name(&self) -> &str {
            "MyService"
        }
    }

    // Register as trait object
    let service: Arc<dyn Service> = Arc::new(MyService);
    MY_REGISTRY.register(service);

    // Retrieve it
    let retrieved: Arc<Arc<dyn Service>> = MY_REGISTRY.get().unwrap();
    assert_eq!(retrieved.name(), "MyService");
}

// ============================================================================
// Multiple Manual Registries Example
// ============================================================================

/// Second registry for isolation testing
static ANOTHER_STORAGE: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static ANOTHER_TRACE: TraceCallback = LazyLock::new(|| Mutex::new(None));

struct AnotherRegistry;

impl RegistryApi for AnotherRegistry {
    fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> {
        &ANOTHER_STORAGE
    }

    fn trace() -> &'static TraceCallback {
        &ANOTHER_TRACE
    }
}

const ANOTHER: AnotherRegistry = AnotherRegistry;

#[test]
#[serial]
fn test_multiple_manual_registries() {
    // Register different values in each registry
    MY_REGISTRY.register(100i32);
    ANOTHER.register(200i32);

    // Verify isolation
    let my_val: Arc<i32> = MY_REGISTRY.get().unwrap();
    let another_val: Arc<i32> = ANOTHER.get().unwrap();

    assert_eq!(*my_val, 100);
    assert_eq!(*another_val, 200);
}

// ============================================================================
// Comparison: Macro vs Manual
// ============================================================================

#[cfg(test)]
mod comparison {
    use super::*;
    use singleton_registry::define_registry;

    #[test]
    fn test_macro_approach() {
        // Using the macro (simpler)
        // NOTE: No #[serial] needed - this test creates its own 'easy' registry
        define_registry!(easy);

        easy::register(42i32);
        let value: Arc<i32> = easy::get().unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    #[serial]
    fn test_manual_approach() {
        // Using manual implementation (more control)
        MY_REGISTRY.register(42i32);
        let value: Arc<i32> = MY_REGISTRY.get().unwrap();
        assert_eq!(*value, 42);
    }
}

// ============================================================================
// Advanced: Custom Registry with Additional Features
// ============================================================================

#[cfg(test)]
mod advanced {
    use super::*;

    /// A registry wrapper with additional features
    struct EnhancedRegistry {
        inner: MyRegistry,
    }

    impl EnhancedRegistry {
        const fn new() -> Self {
            Self { inner: MyRegistry }
        }

        /// Register with logging
        fn register_with_log<T: Send + Sync + 'static>(&self, value: T) {
            println!("Registering type: {}", std::any::type_name::<T>());
            self.inner.register(value);
        }

        /// Get with logging
        fn get_with_log<T: Send + Sync + 'static>(
            &self,
        ) -> Result<Arc<T>, singleton_registry::RegistryError> {
            println!("Getting type: {}", std::any::type_name::<T>());
            self.inner.get()
        }
    }

    #[test]
    #[serial]
    fn test_enhanced_registry() {
        let registry = EnhancedRegistry::new();

        registry.register_with_log(42i32);
        let value: Arc<i32> = registry.get_with_log().unwrap();
        assert_eq!(*value, 42);
    }
}
