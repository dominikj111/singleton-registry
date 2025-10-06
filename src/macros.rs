//! Macros for creating singleton registries.
//!
//! This module provides a simple macro-based approach to create type-safe,
//! thread-safe singleton registries with zero external dependencies.

/// Creates a complete singleton registry with a single macro invocation.
///
/// The macro generates a module containing:
/// - Storage static (hidden)
/// - Trace callback static (hidden)
/// - An `Api` struct that implements `RegistryApi`
///
/// # Examples
///
/// ```rust
/// use singleton_registry::define_registry;
/// use std::sync::Arc;
///
/// // Create a global registry
/// define_registry!(global);
///
/// // Register values (ergonomic free functions)
/// global::register(42i32);
/// global::register("Hello".to_string());
///
/// // Retrieve values
/// let num: Arc<i32> = global::get().unwrap();
/// let msg: Arc<String> = global::get().unwrap();
///
/// assert_eq!(*num, 42);
/// assert_eq!(&**msg, "Hello");
/// ```
///
/// # Multiple Registries
///
/// You can create multiple isolated registries:
///
/// ```rust
/// use singleton_registry::define_registry;
///
/// define_registry!(database);
/// define_registry!(cache);
/// define_registry!(config);
///
/// // Each registry is completely isolated
/// database::register("db_connection".to_string());
/// cache::register("redis_connection".to_string());
///
/// // No interference between registries
/// assert!(database::get::<String>().is_ok());
/// assert!(cache::get::<String>().is_ok());
/// ```
///
/// # Trait-Based Usage
///
/// If you need trait-based usage, the `API` constant is available:
///
/// ```rust
/// use singleton_registry::{define_registry, RegistryApi};
/// use std::sync::Arc;
///
/// define_registry!(app);
///
/// // Use API constant for trait-based access
/// app::API.register(100i32);
/// let value: Arc<i32> = app::API.get().unwrap();
/// ```
#[macro_export]
macro_rules! define_registry {
    ($name:ident) => {
        pub mod $name {
            use std::sync::{Arc, LazyLock, Mutex};
            use std::collections::HashMap;
            use std::any::{TypeId, Any};

            // Storage for registered values (module-private)
            static STORAGE: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
                LazyLock::new(|| Mutex::new(HashMap::new()));

            // Trace callback storage (module-private)
            static TRACE: LazyLock<Mutex<Option<Arc<dyn Fn(&$crate::RegistryEvent) + Send + Sync>>>> =
                LazyLock::new(|| Mutex::new(None));

            /// Zero-sized type that implements the registry API.
            ///
            /// All registry operations are provided by the `RegistryApi` trait's
            /// default implementations. This struct only provides access to the statics.
            struct Api;

            impl $crate::RegistryApi for Api {
                fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> {
                    &STORAGE
                }

                fn trace() -> &'static LazyLock<Mutex<Option<Arc<dyn Fn(&$crate::RegistryEvent) + Send + Sync>>>> {
                    &TRACE
                }

                // All other methods (register, get, contains, etc.) are provided by
                // the trait's default implementations!
            }

            /// Convenient constant for accessing the registry API.
            const API: Api = Api;

            // Free functions for ergonomic usage - they delegate to API

            /// Register a value in the registry.
            pub fn register<T: Send + Sync + 'static>(value: T) {
                use $crate::RegistryApi;
                API.register(value)
            }

            /// Register an Arc-wrapped value in the registry.
            pub fn register_arc<T: Send + Sync + 'static>(value: Arc<T>) {
                use $crate::RegistryApi;
                API.register_arc(value)
            }

            /// Retrieve a value from the registry.
            pub fn get<T: Send + Sync + 'static>() -> Result<Arc<T>, String> {
                use $crate::RegistryApi;
                API.get()
            }

            /// Retrieve a cloned value from the registry.
            pub fn get_cloned<T: Send + Sync + Clone + 'static>() -> Result<T, String> {
                use $crate::RegistryApi;
                API.get_cloned()
            }

            /// Check if a type is registered in the registry.
            pub fn contains<T: Send + Sync + 'static>() -> Result<bool, String> {
                use $crate::RegistryApi;
                API.contains::<T>()
            }

            /// Set a tracing callback for registry operations.
            pub fn set_trace_callback(callback: impl Fn(&$crate::RegistryEvent) + Send + Sync + 'static) {
                use $crate::RegistryApi;
                API.set_trace_callback(callback)
            }

            /// Clear the tracing callback.
            pub fn clear_trace_callback() {
                use $crate::RegistryApi;
                API.clear_trace_callback()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    // use crate::RegistryApi;
    use std::sync::Arc;

    #[test]
    fn test_define_registry_macro() {
        define_registry!(test_reg);

        // Test register and get (ergonomic free functions)
        test_reg::register(100i32);
        let value: Arc<i32> = test_reg::get().unwrap();
        assert_eq!(*value, 100);

        // Test contains
        assert!(test_reg::contains::<i32>().unwrap());
        assert!(!test_reg::contains::<f64>().unwrap());
    }

    #[test]
    fn test_multiple_registries() {
        define_registry!(reg_a);
        define_registry!(reg_b);

        // Register different values in each
        reg_a::register(1i32);
        reg_b::register(2i32);

        // Verify isolation
        let a_val: Arc<i32> = reg_a::get().unwrap();
        let b_val: Arc<i32> = reg_b::get().unwrap();

        assert_eq!(*a_val, 1);
        assert_eq!(*b_val, 2);
    }

    #[test]
    fn test_tracing() {
        define_registry!(trace_test);

        use std::sync::Mutex;
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();

        trace_test::set_trace_callback(move |event| {
            events_clone.lock().unwrap().push(format!("{}", event));
        });

        trace_test::register(42i32);
        let _: Arc<i32> = trace_test::get().unwrap();
        let _ = trace_test::contains::<i32>();

        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 3);
        assert!(recorded[0].contains("register"));
        assert!(recorded[1].contains("get"));
        assert!(recorded[2].contains("contains"));
    }
}
