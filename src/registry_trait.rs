//! Core trait defining registry behavior.
//!
//! This module provides the `RegistryApi` trait with default implementations for
//! type-safe registration, retrieval, and tracing of singleton instances.
//!
//! The registry is type-based: each type (`TypeId`) can have exactly one instance stored.
//! Registering a value of the same type will replace the previous instance.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::{RegistryError, RegistryEvent};

/// Type alias for the trace callback storage.
///
/// Note: This type is also defined in the `define_registry!` macro.
/// Keep both definitions in sync.
type TraceCallback = LazyLock<Mutex<Option<Arc<dyn Fn(&RegistryEvent) + Send + Sync>>>>;

/// Core trait defining registry behavior.
///
/// Provides default implementations for all registry operations, requiring only
/// two accessor methods (`storage` and `trace`) to be implemented by the implementor.
///
/// The registry stores singleton instances indexed by their type (`TypeId`).
/// Each type can have at most one instance stored at any given time.
pub trait RegistryApi {
    // -------------------------------------------------------------------------------------------------
    // Tracing
    // -------------------------------------------------------------------------------------------------

    /// Access the trace callback static.
    ///
    /// This method must be implemented to provide access to the registry's trace callback.
    fn trace() -> &'static TraceCallback;

    /// Set a tracing callback for registry operations.
    ///
    /// The callback will be invoked for every registry operation (register, get, contains).
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the trace lock is poisoned (due to a panic while holding the lock),
    /// this method automatically recovers by extracting the inner value.
    /// This is safe because trace operations are non-critical and idempotent.
    ///
    /// # Safety Restrictions
    ///
    /// The callback must NOT call any registry methods on the same registry,
    /// as this will cause a deadlock. The callback is invoked while holding
    /// the trace lock.
    fn set_trace_callback(&self, callback: impl Fn(&RegistryEvent) + Send + Sync + 'static) {
        let mut guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        *guard = Some(Arc::new(callback));
    }

    /// Clear the tracing callback.
    ///
    /// After calling this, no tracing events will be emitted.
    /// Note: This does not affect registered values, only the tracing callback.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the trace lock is poisoned, this method automatically recovers.
    fn clear_trace_callback(&self) {
        let mut guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        *guard = None;
    }

    /// Convenience wrapper to emit a registry event using the current callback.
    ///
    /// If a trace callback is set, this method will invoke it with the provided event.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// Lock poisoning is automatically recovered by extracting the inner value.
    ///
    /// # Panics
    ///
    /// If the callback itself panics, the panic will propagate to the caller.
    /// The registry lock is not held during callback execution, so this won't
    /// poison the registry storage.
    fn emit_event(&self, event: &RegistryEvent) {
        let guard = Self::trace().lock().unwrap_or_else(|p| p.into_inner());
        if let Some(callback) = guard.as_ref() {
            callback(event);
        }
    }

    // -------------------------------------------------------------------------------------------------
    // Registry
    // -------------------------------------------------------------------------------------------------

    /// Access the storage static.
    ///
    /// This method must be implemented to provide access to the registry's storage.
    fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>;

    /// Register a value in the registry.
    ///
    /// Takes ownership of the value and wraps it in an `Arc` automatically.
    /// If a value of the same type is already registered, it will be replaced.
    ///
    /// # Design Note
    ///
    /// This method does not return a `Result` because registration is designed
    /// for the "write-once" pattern during application startup (and rarely at runtime for rewrite). Lock poisoning
    /// is automatically recovered. If registration must succeed, ensure your
    /// application initialization doesn't panic while holding registry locks.
    fn register<T: Send + Sync + 'static>(&self, value: T) {
        self.register_arc(Arc::new(value));
    }

    /// Register an Arc-wrapped value in the registry.
    ///
    /// More efficient than `register` when you already have an `Arc`,
    /// as it avoids creating an additional reference count.
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the storage lock is poisoned, this method automatically recovers.
    /// This is safe because the insert operation is idempotent.
    fn register_arc<T: Send + Sync + 'static>(&self, value: Arc<T>) {
        self.emit_event(&RegistryEvent::Register {
            type_name: std::any::type_name::<T>(),
        });

        // Register the value
        Self::storage()
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(TypeId::of::<T>(), value);
    }

    /// Retrieve a value from the registry.
    ///
    /// Returns `Ok(Arc<T>)` if the type is found.
    ///
    /// # Errors
    ///
    /// - Type `T` is not found in the registry
    /// - Type mismatch (extremely rare)
    /// - Registry lock is poisoned
    fn get<T: Send + Sync + 'static>(&self) -> Result<Arc<T>, RegistryError> {
        let map = Self::storage()
            .lock()
            .map_err(|_| RegistryError::RegistryLock)?;

        let any_arc_opt = map.get(&TypeId::of::<T>()).cloned();

        drop(map);

        let result: Result<Arc<T>, RegistryError> = match any_arc_opt {
            Some(any_arc) => any_arc
                .downcast::<T>()
                .map_err(|_| RegistryError::TypeMismatch {
                    type_name: std::any::type_name::<T>(),
                }),
            None => Err(RegistryError::TypeNotFound {
                type_name: std::any::type_name::<T>(),
            }),
        };

        self.emit_event(&RegistryEvent::Get {
            type_name: std::any::type_name::<T>(),
            found: result.is_ok(),
        });

        result
    }

    /// Retrieve a cloned value from the registry.
    ///
    /// Returns an owned value by cloning the value stored in the registry.
    /// The type `T` must implement `Clone`.
    ///
    /// # Errors
    ///
    /// - Type `T` is not found in the registry
    /// - Type mismatch
    fn get_cloned<T: Send + Sync + Clone + 'static>(&self) -> Result<T, RegistryError> {
        let arc = self.get::<T>()?;
        Ok((*arc).clone())
    }

    /// Check if a type is registered in the registry.
    ///
    /// Returns `Ok(true)` if the type is registered, `Ok(false)` if not found.
    ///
    /// # Errors
    ///
    /// - Registry lock is poisoned
    fn contains<T: Send + Sync + 'static>(&self) -> Result<bool, RegistryError> {
        let found = Self::storage()
            .lock()
            .map(|m| m.contains_key(&TypeId::of::<T>()))
            .map_err(|_| RegistryError::RegistryLock)?;

        self.emit_event(&RegistryEvent::Contains {
            type_name: std::any::type_name::<T>(),
            found,
        });

        Ok(found)
    }

    // EDUCATIONAL: Memory leak demonstration (commented out)
    //
    // This method demonstrates a common pitfall when working with Arc::into_raw().
    // It leaks memory because the Arc reference count is never decremented.
    // Every call to this method leaks one Arc reference permanently.
    //
    // #[doc(hidden)]
    // fn get_ref<T: Send + Sync + Clone + 'static>(&self) -> Result<&'static T, RegistryError> {
    //     let arc = self.get::<T>()?;
    //     let ptr = Arc::into_raw(arc);  // ⚠️ MEMORY LEAK: Arc is never freed
    //     Ok(unsafe { &*ptr })
    // }

    /// Clear all registered values from the registry.
    ///
    /// This method is primarily intended for testing. It removes all registered
    /// values but does NOT affect:
    /// - Already-retrieved `Arc<T>` references (they remain valid)
    /// - The tracing callback (use `clear_trace_callback()` to clear that)
    ///
    /// # Lock Poisoning Recovery
    ///
    /// If the storage lock is poisoned, this method silently fails.
    /// This is acceptable for a test-only method.
    #[doc(hidden)]
    fn clear(&self) {
        self.emit_event(&RegistryEvent::Clear {});

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
    use crate::RegistryError;

    use super::{RegistryApi, TraceCallback};

    use serial_test::serial;
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::{Arc, LazyLock, Mutex};

    static STORAGE: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));

    static TRACE: TraceCallback = LazyLock::new(|| Mutex::new(None));

    struct Api;

    impl RegistryApi for Api {
        fn storage() -> &'static LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> {
            &STORAGE
        }

        fn trace() -> &'static TraceCallback {
            &TRACE
        }
    }

    const API: Api = Api;

    #[test]
    #[serial]
    fn test_register_and_get_primitive() -> Result<(), RegistryError> {
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
        let retrieved: Arc<String> = API.get().unwrap();
        assert_eq!(&*retrieved, &s);

        // Clear the registry after the test
        API.clear();
    }

    #[test]
    #[serial]
    fn test_get_nonexistent() {
        API.clear();

        let result: Result<Arc<String>, RegistryError> = API.get();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            RegistryError::TypeNotFound {
                type_name: "alloc::string::String"
            }
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

            let s: Arc<String> = API.get().unwrap();
            assert_eq!(&*s, &main_value);
        });

        let thread_value = main_rx.recv().unwrap();
        let num: Arc<u32> = API.get().unwrap();
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
        let num: Arc<Num> = API.get().unwrap();
        assert_eq!(num.0, num_val.0);

        let text: Arc<Text> = API.get().unwrap();
        assert_eq!(text.0, text_val.0);

        let nums: Arc<Numbers> = API.get().unwrap();
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
    fn test_tuple_type() -> Result<(), RegistryError> {
        API.clear();

        let tuple = (1, "test");
        API.register(tuple);

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
    fn test_get_cloned() {
        API.clear();
        API.register("hello".to_string());
        let value: String = API.get_cloned::<String>().unwrap();
        assert_eq!(value, "hello");
    }

    // EDUCATIONAL: Memory leak test (commented out)
    //
    // This test demonstrates the memory leak in the get_ref() method above.
    // Uncomment this along with get_ref() to see the leak in action.
    //
    // #[test]
    // #[serial]
    // fn test_get_ref() {
    //     API.clear();
    //     API.register("world".to_string());
    //     let value: &'static String = API.get_ref::<String>().unwrap();
    //     assert_eq!(value, "world");
    //
    //     // WARNING: The following line causes undefined behavior (UB).
    //     // After calling `clear`, the original `String` has been dropped and its memory deallocated,
    //     // but `value` is still a reference to the old memory location. Accessing or printing `value`
    //     // after this point is use-after-free, which is always UB in Rust. This may cause a crash,
    //     // memory corruption, or appear to "work" by accident, depending on the allocator and OS.
    //     // This code is for demonstration purposes only—never use a leaked reference after the value is dropped!
    //     // API.clear(); // value is dropped
    //     // let _ = value.len();
    //     // eprintln!("{}", value);
    // }

    #[test]
    #[serial]
    fn test_contains() {
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
    fn test_trace_callback_register_event() {
        API.clear();
        use std::sync::{Arc as StdArc, Mutex as StdMutex};
        let events = StdArc::new(StdMutex::new(Vec::new()));
        let events_clone = events.clone();

        API.set_trace_callback(move |e| {
            events_clone.lock().unwrap().push(format!("{}", e));
        });

        API.register(5u8);

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0], "register { type_name: u8 }");

        API.clear_trace_callback();
    }

    #[test]
    #[serial]
    fn test_trace_callback_get_event() {
        API.clear();
        use std::sync::{Arc as StdArc, Mutex as StdMutex};
        let events = StdArc::new(StdMutex::new(Vec::new()));
        let events_clone = events.clone();

        API.set_trace_callback(move |e| {
            events_clone.lock().unwrap().push(format!("{}", e));
        });

        API.register(42i32);
        let _ = API.get::<i32>();

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0], "register { type_name: i32 }");
        assert_eq!(captured[1], "get { type_name: i32, found: true }");

        API.clear_trace_callback();
    }

    #[test]
    #[serial]
    fn test_trace_callback_contains_event() {
        API.clear();
        use std::sync::{Arc as StdArc, Mutex as StdMutex};
        let events = StdArc::new(StdMutex::new(Vec::new()));
        let events_clone = events.clone();

        API.set_trace_callback(move |e| {
            events_clone.lock().unwrap().push(format!("{}", e));
        });

        let _ = API.contains::<String>();
        API.register("test".to_string());
        let _ = API.contains::<String>();

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 3);
        assert_eq!(
            captured[0],
            "contains { type_name: alloc::string::String, found: false }"
        );
        assert_eq!(captured[1], "register { type_name: alloc::string::String }");
        assert_eq!(
            captured[2],
            "contains { type_name: alloc::string::String, found: true }"
        );

        API.clear_trace_callback();
    }

    #[test]
    #[serial]
    fn test_trace_callback_clear_event() {
        API.clear();
        use std::sync::{Arc as StdArc, Mutex as StdMutex};
        let events = StdArc::new(StdMutex::new(Vec::new()));
        let events_clone = events.clone();

        API.set_trace_callback(move |e| {
            events_clone.lock().unwrap().push(format!("{}", e));
        });

        API.clear();

        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0], "Clearing the Registry");

        API.clear_trace_callback();
    }

    #[test]
    #[serial]
    fn test_clear_trace_callback_stops_events() {
        API.clear();
        use std::sync::{Arc as StdArc, Mutex as StdMutex};
        let events = StdArc::new(StdMutex::new(Vec::new()));
        let events_clone = events.clone();

        // Set callback and register a value
        API.set_trace_callback(move |e| {
            events_clone.lock().unwrap().push(format!("{}", e));
        });

        API.register(10u16);

        // Verify event was captured
        {
            let captured = events.lock().unwrap();
            assert_eq!(captured.len(), 1);
            assert_eq!(captured[0], "register { type_name: u16 }");
        }

        // Clear the callback
        API.clear_trace_callback();

        // Perform more operations - these should NOT be traced
        API.register(20u16);
        let _ = API.get::<u16>();
        let _ = API.contains::<u16>();

        // Verify no new events were captured
        let captured = events.lock().unwrap();
        assert_eq!(captured.len(), 1); // Still only the first event
    }

    #[test]
    #[serial]
    fn test_register_arc_directly() {
        API.clear();
        let value = Arc::new(42i32);
        let clone = value.clone();
        API.register_arc(value);

        let retrieved: Arc<i32> = API.get().unwrap();
        assert_eq!(*retrieved, 42);
        assert_eq!(Arc::strong_count(&clone), 3); // clone + registry + retrieved
    }
}
