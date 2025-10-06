//! Registry trait and event types for the singleton registry.
//!
//! This module defines the core trait that all macro-generated registries implement,
//! as well as the event types used for tracing registry operations.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::RegistryEvent;

/// Core trait defining registry behavior.
///
/// All macro-generated registries implement this trait via a zero-sized `Api` struct.
/// The trait provides default implementations for all operations, requiring only
/// two accessor methods to be implemented.
///
/// ```rust
/// use singleton_registry::define_registry;
///
/// define_registry!(MY_REGISTRY);
///
/// // Trait-based usage
/// MY_REGISTRY::API.register(42i32);
/// let value = MY_REGISTRY::API.get::<i32>().unwrap();
/// ```
pub trait RegistryApi {
    /// Access the storage static.
    ///
    /// This method must be implemented to provide access to the registry's storage.
    fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>;

    /// Access the trace callback static.
    ///
    /// This method must be implemented to provide access to the registry's trace callback.
    fn trace() -> &'static LazyLock<Mutex<Option<Arc<dyn Fn(&RegistryEvent) + Send + Sync>>>>;

    /// Register a value in the registry.
    ///
    /// This takes ownership of the value and wraps it in an `Arc` automatically.
    /// If a value of the same type is already registered, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # define_registry!(EXAMPLE);
    /// EXAMPLE::API.register(42i32);
    /// EXAMPLE::API.register("Hello".to_string());
    /// ```
    fn register<T: Send + Sync + 'static>(&self, value: T) {
        self.register_arc(Arc::new(value));
    }

    /// Register an Arc-wrapped value in the registry.
    ///
    /// This is more efficient than `register` when you already have an `Arc`,
    /// as it avoids creating an additional reference count.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # use std::sync::Arc;
    /// # define_registry!(EXAMPLE);
    /// let value = Arc::new(42i32);
    /// EXAMPLE::API.register_arc(value);
    /// ```
    fn register_arc<T: Send + Sync + 'static>(&self, value: Arc<T>) {
        // Emit trace event
        let guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(callback) = guard.as_ref() {
            callback(&RegistryEvent::Register {
                type_name: std::any::type_name::<T>(),
            });
        }
        drop(guard);

        // Register the value
        Self::storage()
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(TypeId::of::<T>(), value);
    }

    /// Retrieve a value from the registry.
    ///
    /// Returns `Ok(Arc<T>)` if the type is found, or an error message if not found
    /// or if there's a type mismatch.
    ///
    /// # Errors
    ///
    /// - Returns `Err` if the type `T` is not found in the registry
    /// - Returns `Err` if there's a type mismatch (extremely rare)
    /// - Returns `Err` if the registry lock is poisoned
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # use std::sync::Arc;
    /// # define_registry!(EXAMPLE);
    /// EXAMPLE::API.register(42i32);
    ///
    /// let value: Arc<i32> = EXAMPLE::API.get().unwrap();
    /// assert_eq!(*value, 42);
    /// ```
    fn get<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, String> {
        let map = Self::storage()
            .lock()
            .map_err(|_| "Failed to acquire registry lock".to_string())?;

        let any_arc_opt = map.get(&TypeId::of::<T>()).cloned();
        drop(map);

        let result: Result<Arc<T>, String> = match any_arc_opt {
            Some(any_arc) => any_arc.downcast::<T>().map_err(|_| {
                format!(
                    "Type mismatch in registry for type: {}",
                    std::any::type_name::<T>()
                )
            }),
            None => Err(format!(
                "Type not found in registry: {}",
                std::any::type_name::<T>()
            )),
        };

        // Emit trace event
        let guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(callback) = guard.as_ref() {
            callback(&RegistryEvent::Get {
                type_name: std::any::type_name::<T>(),
                found: result.is_ok(),
            });
        }

        result
    }

    /// Retrieve a cloned value from the registry.
    ///
    /// This returns an owned value by cloning the value stored in the registry.
    /// The type `T` must implement `Clone`.
    ///
    /// # Errors
    ///
    /// - Returns `Err` if the type `T` is not found in the registry
    /// - Returns `Err` if there's a type mismatch
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # define_registry!(EXAMPLE);
    /// EXAMPLE::API.register("hello".to_string());
    ///
    /// let value: String = EXAMPLE::API.get_cloned().unwrap();
    /// assert_eq!(value, "hello");
    /// ```
    fn get_cloned<T: Send + Sync + Clone + 'static>(&self) -> Result<T, String> {
        let arc = self.get::<T>()?;
        Ok((*arc).clone())
    }

    /// Check if a type is registered in the registry.
    ///
    /// Returns `Ok(true)` if the type is registered, `Ok(false)` if not found,
    /// or an error if the registry lock is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # define_registry!(EXAMPLE);
    ///
    /// assert!(!EXAMPLE::API.contains::<i32>().unwrap());
    /// EXAMPLE::API.register(42i32);
    /// assert!(EXAMPLE::API.contains::<i32>().unwrap());
    /// ```
    fn contains<T: Send + Sync + 'static>(&self) -> Result<bool, String> {
        let found = Self::storage()
            .lock()
            .map(|m| m.contains_key(&TypeId::of::<T>()))
            .map_err(|_| "Failed to acquire registry lock".to_string())?;

        // Emit trace event
        let guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(callback) = guard.as_ref() {
            callback(&RegistryEvent::Contains {
                type_name: std::any::type_name::<T>(),
                found,
            });
        }

        Ok(found)
    }

    /// Set a tracing callback for registry operations.
    ///
    /// The callback will be invoked for every registry operation (register, get, contains).
    /// This is useful for debugging, logging, or monitoring registry usage.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # define_registry!(EXAMPLE);
    ///
    /// EXAMPLE::API.set_trace_callback(|event| {
    ///     println!("Registry event: {:?}", event);
    /// });
    ///
    /// EXAMPLE::API.register(42i32); // Will trigger the callback
    /// ```
    fn set_trace_callback(&self, callback: impl Fn(&RegistryEvent) + Send + Sync + 'static) {
        let mut guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(Arc::new(callback));
    }

    /// Clear the tracing callback.
    ///
    /// After calling this, no tracing events will be emitted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use singleton_registry::{define_registry, RegistryApi};
    /// # define_registry!(EXAMPLE);
    ///
    /// EXAMPLE::API.set_trace_callback(|event| println!("{:?}", event));
    /// EXAMPLE::API.clear_trace_callback(); // Stop tracing
    /// ```
    fn clear_trace_callback(&self) {
        let mut guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        *guard = None;
    }

    #[doc(hidden)]
    fn get_ref<T: Send + Sync + Clone + 'static>(&self) -> Result<&'static T, String> {
        let arc = self.get::<T>()?;
        let ptr = Arc::into_raw(arc);
        Ok(unsafe { &*ptr })
    }

    #[doc(hidden)]
    fn clear(&self) {
        // Emit trace event
        let guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(callback) = guard.as_ref() {
            callback(&RegistryEvent::Clear {});
        }
        drop(guard);

        if let Ok(mut registry) = Self::storage().lock() {
            registry.clear();
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::RegistryEvent;

    use super::RegistryApi;

    use serial_test::serial;
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::{Arc, LazyLock, Mutex};

    static STORAGE: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    static TRACE: LazyLock<Mutex<Option<Arc<dyn Fn(&RegistryEvent) + Send + Sync>>>> =
        LazyLock::new(|| Mutex::new(None));

    struct Api;

    impl RegistryApi for Api {
        fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> {
            &STORAGE
        }

        fn trace() -> &'static LazyLock<Mutex<Option<Arc<dyn Fn(&RegistryEvent) + Send + Sync>>>> {
            &TRACE
        }
    }

    const API: Api = Api;

    #[test]
    #[serial]
    fn test_register_and_get_primitive() -> Result<(), String> {
        // Clear any previous state
        API.clear();

        // Register a primitive type
        API.register(42i32);

        // Retrieve it 1
        let num: Arc<i32> = API.get()?;
        assert_eq!(*num, 42);

        // Retrieve it 2
        let num_2 = API.get::<i32>()?;
        assert_eq!(*num_2, 42);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_register_and_get_string() {
        // Clear the registry before the test
        API.clear();

        // Create and register a string
        let s = "test".to_string();
        API.register(s.clone());

        // Retrieve it and verify
        let retrieved: Arc<String> = API.get().expect("Failed to retrieve string");
        assert_eq!(&*retrieved, &s);

        // Clear the registry after the test
        API.clear();
    }

    #[test]
    #[serial]
    fn test_get_nonexistent() {
        API.clear();

        let result: Result<Arc<String>, _> = API.get();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Type not found in registry: alloc::string::String"
        );
    }

    #[test]
    #[serial]
    fn test_thread_safety() {
        API.clear();

        use std::sync::{mpsc, Arc, Barrier};
        use std::thread;

        let barrier = Arc::new(Barrier::new(2));
        let (main_tx, thread_rx) = mpsc::channel();
        let (thread_tx, main_rx) = mpsc::channel();

        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            API.register(100u32);
            thread_tx.send(100u32).unwrap();

            // Wait for the main thread to register its value
            let main_value: String = thread_rx.recv().unwrap();

            // Synchronize: ensure both threads have registered before retrieval
            barrier_clone.wait();

            let s: Arc<String> = API.get().expect("Failed to get string in thread");
            assert_eq!(&*s, &main_value);
        });

        let thread_value = main_rx.recv().unwrap();
        let num: Arc<u32> = API.get().expect("Failed to get u32 in main thread");
        assert_eq!(*num, thread_value);

        // Register a string in main thread
        let main_string = "main_thread_value".to_string();
        API.register(main_string.clone());
        main_tx.send(main_string.clone()).unwrap();

        // Synchronize: ensure both threads have registered before retrieval
        barrier.wait();

        handle.join().unwrap();
        API.clear();
    }

    #[test]
    #[serial]
    fn test_multiple_types() {
        API.clear();

        // Define wrapper types to ensure unique TypeIds
        #[derive(Debug, PartialEq, Eq, Clone)]
        struct Num(i32);
        #[derive(Debug, PartialEq, Eq, Clone)]
        struct Text(String);
        #[derive(Debug, PartialEq, Eq, Clone)]
        struct Numbers(Vec<i32>);

        // Create the values
        let num_val = Num(42);
        let text_val = Text("hello".to_string());
        let nums_val = Numbers(vec![1, 2, 3]);

        // Register all types first
        API.register(num_val.clone());
        API.register(text_val.clone());
        API.register(nums_val.clone());

        // Then retrieve and verify each one
        let num: Arc<Num> = API.get().expect("Num not found in registry");
        assert_eq!(num.0, num_val.0);

        let text: Arc<Text> = API.get().expect("Text not found in registry");
        assert_eq!(text.0, text_val.0);

        let nums: Arc<Numbers> = API.get().expect("Numbers not found in registry");
        assert_eq!(&nums.0, &nums_val.0);

        // Clear the registry after the test
        API.clear();
    }

    #[test]
    #[serial]
    fn test_custom_type() {
        API.clear();

        #[derive(Debug, PartialEq, Eq, Clone)]
        struct MyStruct {
            field: String,
        }

        let my_value = MyStruct {
            field: "test".into(),
        };
        API.register(my_value.clone());

        let retrieved: Arc<MyStruct> = API.get().unwrap();
        assert_eq!(&*retrieved, &my_value);
    }

    #[test]
    #[serial]
    fn test_tuple_type() -> Result<(), String> {
        API.clear();

        let tuple = (1, "test");
        API.register(tuple.clone());

        let retrieved = API.get::<(i32, &str)>()?;
        assert_eq!(&*retrieved, &tuple);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_overwrite_same_type() {
        API.clear();

        API.register(10i32);
        API.register(20i32); // should replace

        let num: Arc<i32> = API.get().unwrap();
        assert_eq!(*num, 20);
    }

    #[test]
    #[serial]
    fn test_di_get_cloned() {
        API.clear();
        API.register("hello".to_string());
        let value: String = API.get_cloned::<String>().expect("Value should be present");
        assert_eq!(value, "hello");
    }

    #[test]
    #[serial]
    fn test_di_get_ref() {
        API.clear();
        API.register("world".to_string());
        let value: &'static String = API.get_ref::<String>().expect("Value should be present");
        assert_eq!(value, "world");

        // WARNING: The following line causes undefined behavior (UB).
        // After calling `di_clear`, the original `String` has been dropped and its memory deallocated,
        // but `value` is still a reference to the old memory location. Accessing or printing `value`
        // after this point is use-after-free, which is always UB in Rust. This may cause a crash,
        // memory corruption, or appear to "work" by accident, depending on the allocator and OS.
        // This code is for demonstration purposes only—never use a leaked reference after the value is dropped!
        // di_clear(); // value is dropped
        // let _ = value.len();
        // eprintln!("{}", value);
    }

    #[test]
    #[serial]
    fn test_di_contains() {
        API.clear();
        assert!(!API.contains::<u32>().unwrap());
        API.register(1u32);
        assert!(API.contains::<u32>().unwrap());
    }

    #[test]
    #[serial]
    fn test_function_pointer_registration() {
        API.clear();

        // Test the function pointer example from README
        let multiply_by_two: fn(i32) -> i32 = |x| x * 2;
        API.register(multiply_by_two);

        let doubler: Arc<fn(i32) -> i32> = API.get().unwrap();
        let result = doubler(21);
        assert_eq!(result, 42);
    }

    #[test]
    #[serial]
    fn test_trace_callback_invoked() {
        API.clear();
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        API.set_trace_callback(|_e| {
            COUNT.fetch_add(1, Ordering::SeqCst);
        });
        API.register(5u8);
        assert_eq!(COUNT.load(Ordering::SeqCst), 1); // adjust after re-enabling emit
        API.clear_trace_callback();
    }

    // -------------------------------------------------------------
    // Display implementation tests
    // -------------------------------------------------------------

    #[test]
    fn test_display_register() {
        let ev = RegistryEvent::Register { type_name: "i32" };
        assert_eq!(ev.to_string(), "register { type_name: i32 }");
    }

    #[test]
    fn test_display_get() {
        let ev = RegistryEvent::Get {
            type_name: "String",
            found: true,
        };
        assert_eq!(ev.to_string(), "get { type_name: String, found: true }");
    }

    #[test]
    fn test_display_contains() {
        let ev = RegistryEvent::Contains {
            type_name: "u8",
            found: false,
        };
        assert_eq!(ev.to_string(), "contains { type_name: u8, found: false }");
    }

    #[test]
    fn test_display_clear() {
        let ev = RegistryEvent::Clear {};
        assert_eq!(ev.to_string(), "Clearing the Registry");
    }
}
