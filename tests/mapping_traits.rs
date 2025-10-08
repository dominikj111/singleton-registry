//! Integration tests for registering and retrieving trait objects.
//!
//! This test demonstrates how to store trait objects in the registry,
//! which is useful for dependency injection and plugin systems.
//!
//! NOTE: All tests use #[serial] because they share the same registry (advanced).
//! Running them in parallel could cause interference.

use serial_test::serial;
use singleton_registry::define_registry;
use std::sync::Arc;

// Create a registry for these tests
define_registry!(traits);

// Define some example traits
trait Logger: Send + Sync {
    fn get_name(&self) -> &str;
}

trait Calculator: Send + Sync {
    fn calculate(&self, a: i32, b: i32) -> i32;
}

trait Formatter: Send + Sync {
    fn format(&self, value: &str) -> String;
}

// Implementations
struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn get_name(&self) -> &str {
        "ConsoleLogger"
    }
}

struct AddCalculator;
impl Calculator for AddCalculator {
    fn calculate(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

struct UppercaseFormatter;
impl Formatter for UppercaseFormatter {
    fn format(&self, value: &str) -> String {
        value.to_uppercase()
    }
}

#[test]
#[serial]
fn test_register_multiple_trait_objects() {
    // Register different trait objects (need Arc for trait objects)
    traits::register(Arc::new(ConsoleLogger) as Arc<dyn Logger>);
    traits::register(Arc::new(AddCalculator) as Arc<dyn Calculator>);
    traits::register(Arc::new(UppercaseFormatter) as Arc<dyn Formatter>);

    // Retrieve and use them
    let calc: Arc<Arc<dyn Calculator>> = traits::get().unwrap();
    assert_eq!(calc.calculate(40, 2), 42);

    let fmt: Arc<Arc<dyn Formatter>> = traits::get().unwrap();
    assert_eq!(fmt.format("hello"), "HELLO");

    // Alternative: get_cloned() - clones the stored value (Arc in this case)
    // Registry stores: Arc<dyn Formatter>
    // get() returns: Arc<Arc<dyn Formatter>> (registry wraps in Arc)
    // get_cloned() returns: Arc<dyn Formatter> (Arc::clone is cheap - just increments ref count)
    let fmt: Arc<dyn Formatter> = traits::get_cloned().unwrap();
    assert_eq!(fmt.format("hello"), "HELLO");
}

#[test]
#[serial]
fn test_dependency_injection_pattern() {
    // Common DI pattern: register dependencies, retrieve in components
    traits::register(Arc::new(AddCalculator) as Arc<dyn Calculator>);

    // Component retrieves its dependency
    struct Component;
    impl Component {
        fn process(&self) -> i32 {
            let calc: Arc<Arc<dyn Calculator>> = traits::get().unwrap();
            calc.calculate(20, 22)
        }
    }

    assert_eq!(Component.process(), 42);
}

#[test]
#[serial]
fn test_polymorphism_with_concrete_and_trait_types() {
    // Demonstrate polymorphism: same struct registered as different types
    #[derive(Clone)]
    struct MultiLogger {
        prefix: String,
    }

    impl Logger for MultiLogger {
        fn get_name(&self) -> &str {
            &self.prefix
        }
    }

    impl MultiLogger {
        fn get_prefix(&self) -> &str {
            &self.prefix
        }
    }

    // Test 1: Concrete type - has access to all methods
    traits::register(MultiLogger {
        prefix: "[CONCRETE]".to_string(),
    });

    let concrete = traits::get::<MultiLogger>().unwrap();
    assert_eq!(concrete.get_prefix(), "[CONCRETE]"); // Concrete method
    assert_eq!(concrete.get_name(), "[CONCRETE]"); // Trait method

    // Test 2: Trait object (needs Arc + dyn) - only has trait methods
    traits::register(Arc::new(MultiLogger {
        prefix: "[TRAIT]".to_string(),
    }) as Arc<dyn Logger>);

    let trait_obj = traits::get::<Arc<dyn Logger>>().unwrap();
    // Note: trait_obj.get_prefix() would not compile - concrete methods not available
    assert_eq!(trait_obj.get_name(), "[TRAIT]"); // ✅ Trait method works

    // Test 3: Override - second registration replaces first
    traits::register(Arc::new(ConsoleLogger) as Arc<dyn Logger>);
    assert_eq!(
        traits::get::<Arc<dyn Logger>>().unwrap().get_name(),
        "ConsoleLogger"
    );

    traits::register(Arc::new(MultiLogger {
        prefix: "[OVERRIDE]".to_string(),
    }) as Arc<dyn Logger>);
    assert_eq!(
        traits::get::<Arc<dyn Logger>>().unwrap().get_name(),
        "[OVERRIDE]"
    );

    // Both types exist separately
    assert!(traits::contains::<MultiLogger>().unwrap());
    assert!(traits::contains::<Arc<dyn Logger>>().unwrap());
}
