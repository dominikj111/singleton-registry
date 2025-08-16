#![allow(dead_code)]

//! A thread-safe dependency injection registry for storing and retrieving global instances.
//! Currently designed for write-once, read-many pattern.
//!
//! This module provides a type-safe way to register and retrieve instances of any type
//! that implements `Send + Sync + 'static`.
//!
//! # Examples
//!
//! ```
//! use singleton_registry::{register, get};
//! use std::sync::Arc;
//!
//! // Register a value
//! register("Hello, World!".to_string());
//!
//! // Retrieve the value
//! let message: Arc<String> = get().unwrap();
//! assert_eq!(&*message, "Hello, World!");
//! ```

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    sync::{Arc, LazyLock, Mutex},
};

/// Global thread-safe registry storing type instances.
///
/// This is a `LazyLock` ensuring thread-safe lazy initialization of the underlying `Mutex<HashMap>`.
/// The registry maps `TypeId` to `Arc<dyn Any + Send + Sync>` for type-erased storage.
static GLOBAL_REGISTRY: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// -------------------------------------------------------------------------------------------------
// Tracing callback support
// -------------------------------------------------------------------------------------------------

/// Events emitted by the dependency-injection registry.
#[derive(Debug)]
pub enum RegistryEvent {
    /// A value was registered.
    Register {
        type_name: &'static str,
    },
    /// A value was requested with `di_get`.
    Get {
        type_name: &'static str,
        found: bool,
    },
    /// A `di_contains` check was performed.
    Contains {
        type_name: &'static str,
        found: bool,
    },
    Clear {},
}

impl fmt::Display for RegistryEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryEvent::Register { type_name } => {
                write!(f, "register {{ type_name: {type_name} }}")
            }
            RegistryEvent::Get { type_name, found } => {
                write!(f, "get {{ type_name: {type_name}, found: {found} }}")
            }
            RegistryEvent::Contains { type_name, found } => {
                write!(f, "contains {{ type_name: {type_name}, found: {found} }}")
            }
            RegistryEvent::Clear {} => write!(f, "Clearing the Registry"),
        }
    }
}

/// Type alias for the user-supplied tracing callback.
///
/// The callback receives a reference to a `RegistryEvent` every time the registry is
/// interacted with. It must be thread-safe because the registry itself is globally shared.
pub type TraceCallback = dyn Fn(&RegistryEvent) + Send + Sync + 'static;

/// Holds an optional user-defined tracing callback.
static TRACE_CALLBACK: LazyLock<Mutex<Option<Arc<TraceCallback>>>> =
    LazyLock::new(|| Mutex::new(None));

/// Sets a tracing callback that will be invoked on every registry interaction.
///
/// Pass `None` (or call `clear_trace_callback`) to disable tracing.
///
/// # Example
/// ```rust
/// use singleton_registry::{set_trace_callback, RegistryEvent};
///
/// set_trace_callback(|event| println!("[registry-trace] {:?}", event));
/// ```
pub fn set_trace_callback(callback: impl Fn(&RegistryEvent) + Send + Sync + 'static) {
    let mut guard = TRACE_CALLBACK.lock().unwrap_or_else(|p| p.into_inner());
    *guard = Some(Arc::new(callback));
}

/// Clears the tracing callback (disables registry tracing).
pub fn clear_trace_callback() {
    let mut guard = TRACE_CALLBACK.lock().unwrap_or_else(|p| p.into_inner());
    *guard = None;
}

/// Convenience wrapper to emit a registry event using the current callback.
fn emit_event(event: &RegistryEvent) {
    // lock poisoning unlikely; if poisoned, keep emitting with recovered lock
    let guard = TRACE_CALLBACK.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(callback) = guard.as_ref() {
        callback(event);
    }
}

// -------------------------------------------------------------------------------------------------
// Registry
// -------------------------------------------------------------------------------------------------

/// Registers an `Arc<T>` in the global registry.
///
/// This is more efficient than `di_register` when you already have an `Arc`,
/// as it avoids creating an additional reference count.
///
/// # Safety
///
/// If the registry's lock is poisoned (which can happen if a thread panicked while
/// holding the lock), this function will recover the lock and continue execution.
/// This is safe because the registry is used in a read-only manner after the
/// initial registration phase in `main.rs`.
///
/// # Arguments
///
/// * `value` - The `Arc`-wrapped value to register. The inner type must implement
///   `Send + Sync + 'static`.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use singleton_registry::{register_arc, get};
///
/// let value = Arc::new("shared".to_string());
/// register_arc(value.clone());
///
/// let retrieved: Arc<String> = get().expect("Failed to get value");
/// assert_eq!(&*retrieved, "shared");
/// ```
pub fn register_arc<T: Send + Sync + 'static>(value: Arc<T>) {
    emit_event(&RegistryEvent::Register {
        type_name: std::any::type_name::<T>(),
    });

    GLOBAL_REGISTRY
        .lock()
        // The registry is used as read only, so we do not expect a poisoned lock.
        // Poisoning only occurs if a thread panics while holding the lock.
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(TypeId::of::<T>(), value);
}

/// Registers a value of type `T` in the global registry.
///
/// This is a convenience wrapper around `di_register_arc` that takes ownership
/// of the value and wraps it in an `Arc` automatically.
///
/// # Safety
///
/// If the registry's lock is poisoned (which can happen if a thread panicked while
/// holding the lock), this function will recover the lock and continue execution.
/// This is safe because the registry is used in a read-only manner after the
/// initial registration phase in `main.rs`.
///
/// # Arguments
///
/// * `value` - The value to register. Must implement `Send + Sync + 'static`.
///
/// # Examples
///
/// ```
/// use singleton_registry::{register, get};
/// use std::sync::Arc;
///
/// // Register a primitive
/// register(42i32);
///
/// // Register a string
/// register("Hello".to_string());
///
/// // Retrieve values
/// let num: Arc<i32> = get().expect("Failed to get i32");
/// let s: Arc<String> = get().expect("Failed to get String");
///
/// assert_eq!(*num, 42);
/// assert_eq!(&*s, "Hello");
/// ```
pub fn register<T: Send + Sync + 'static>(value: T) {
    register_arc::<T>(Arc::new(value));
}

/// Checks if a value of type `T` is registered in the global registry.
///
/// # Returns
///
/// - `Ok(true)` if the type is registered
/// - `Ok(false)` if the type is not found
/// - `Err(String)` if failed to acquire the registry lock
///
/// # Examples
///
/// ```
/// use singleton_registry::{register, contains};
///
/// // Check for unregistered type
/// assert!(!contains::<i32>().expect("Failed to check registry"));
///
/// // Register and check
/// register(42i32);
/// assert!(contains::<i32>().expect("Failed to check registry"));
/// ```
pub fn contains<T: Send + Sync + 'static>() -> Result<bool, String> {
    let found = GLOBAL_REGISTRY
        .lock()
        .map(|m| m.contains_key(&TypeId::of::<T>()))
        .map_err(|_| "Failed to acquire registry lock".to_string())?;

    emit_event(&RegistryEvent::Contains {
        type_name: std::any::type_name::<T>(),
        found,
    });

    Ok(found)
}

/// Retrieves a value of type `T` from the global registry.
///
/// # Returns
///
/// - `Ok(Arc<T>)` if the type is found and the downcast is successful
/// - `Err(String)` in the following cases:
///   - Failed to acquire the registry lock
///   - Type `T` is not found in the registry
///   - Type mismatch (found a different type with the same TypeId)
///
/// # Examples
///
/// ```
/// use singleton_registry::{register, get};
/// use std::sync::Arc;
///
/// // Register and retrieve a value
/// register(42i32);
/// let num: Arc<i32> = get().expect("Failed to get i32");
/// assert_eq!(*num, 42);
///
/// // Handle missing value
/// let result: Result<Arc<String>, _> = get();
/// assert!(result.is_err());
/// ```
pub fn get<T: Send + Sync + 'static>() -> Result<Arc<T>, String> {
    let map = GLOBAL_REGISTRY
        .lock()
        .map_err(|_| "Failed to acquire registry lock")?;

    let any_arc_opt = map.get(&TypeId::of::<T>()).cloned();

    // Determine result and emit tracing event in a single place.
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

    emit_event(&RegistryEvent::Get {
        type_name: std::any::type_name::<T>(),
        found: result.is_ok(),
    });

    result
}

/// Retrieves a clone of the value stored in the registry for the given type.
///
/// This function returns an owned value by cloning the value stored in the registry.
/// The type `T` must implement `Clone`. This is useful if you need to own the value
/// rather than share it via `Arc<T>`.
///
/// # Errors
/// Returns an error if the value for the given type is not found in the registry.
///
/// # Examples
/// ```
/// use singleton_registry::{register, get_cloned};
///
/// register("hello".to_string());
/// let value: String = get_cloned::<String>().expect("Value should be present");
/// assert_eq!(value, "hello");
/// ```
pub fn get_cloned<T: Send + Sync + Clone + 'static>() -> Result<T, String> {
    let arc = get::<T>()?;
    Ok((*arc).clone())
}

/// Returns a `'static` reference to a value stored in the registry.
///
/// This function is here only for educational purpose and future reference. Better to avoid it.
///
/// # Safety
/// This function intentionally leaks the `Arc<T>` to extend its lifetime to `'static`.
/// Only use this for values that are truly immutable and meant to live for the entire
/// lifetime of the application (true singletons). Never use for values that may be
/// mutated or replaced, or if you plan to clear the registry at runtime.
///
/// If you need shared access, prefer using `Arc<T>` via `get`.
#[doc(hidden)]
pub fn get_ref<T: Send + Sync + Clone + 'static>() -> Result<&'static T, String> {
    let arc = get::<T>()?;
    let ptr = Arc::into_raw(arc);
    Ok(unsafe { &*ptr })
}

#[doc(hidden)]
pub fn clear() {
    emit_event(&RegistryEvent::Clear {});

    if let Ok(mut registry) = GLOBAL_REGISTRY.lock() {
        registry.clear();
    }
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::Arc;

    #[test]
    #[serial]
    fn test_register_and_get_primitive() -> Result<(), String> {
        // Clear any previous state
        clear();

        // Register a primitive type
        register(42i32);

        // Retrieve it 1
        let num: Arc<i32> = get()?;
        assert_eq!(*num, 42);

        // Retrieve it 2
        let num_2 = get::<i32>()?;
        assert_eq!(*num_2, 42);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_register_and_get_string() {
        // Clear the registry before the test
        clear();

        // Create and register a string
        let s = "test".to_string();
        register(s.clone());

        // Retrieve it and verify
        let retrieved: Arc<String> = get().expect("Failed to retrieve string");
        assert_eq!(&*retrieved, &s);

        // Clear the registry after the test
        clear();
    }

    #[test]
    #[serial]
    fn test_get_nonexistent() {
        clear();

        let result: Result<Arc<String>, _> = get();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Type not found in registry: alloc::string::String"
        );
    }

    #[test]
    #[serial]
    fn test_thread_safety() {
        clear();

        use std::sync::{mpsc, Arc, Barrier};
        use std::thread;

        let barrier = Arc::new(Barrier::new(2));
        let (main_tx, thread_rx) = mpsc::channel();
        let (thread_tx, main_rx) = mpsc::channel();

        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            register(100u32);
            thread_tx.send(100u32).unwrap();

            // Wait for the main thread to register its value
            let main_value: String = thread_rx.recv().unwrap();

            // Synchronize: ensure both threads have registered before retrieval
            barrier_clone.wait();

            let s: Arc<String> = get().expect("Failed to get string in thread");
            assert_eq!(&*s, &main_value);
        });

        let thread_value = main_rx.recv().unwrap();
        let num: Arc<u32> = get().expect("Failed to get u32 in main thread");
        assert_eq!(*num, thread_value);

        // Register a string in main thread
        let main_string = "main_thread_value".to_string();
        register(main_string.clone());
        main_tx.send(main_string.clone()).unwrap();

        // Synchronize: ensure both threads have registered before retrieval
        barrier.wait();

        handle.join().unwrap();
        clear();
    }

    #[test]
    #[serial]
    fn test_multiple_types() {
        clear();

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
        register(num_val.clone());
        register(text_val.clone());
        register(nums_val.clone());

        // Then retrieve and verify each one
        let num: Arc<Num> = get().expect("Num not found in registry");
        assert_eq!(num.0, num_val.0);

        let text: Arc<Text> = get().expect("Text not found in registry");
        assert_eq!(text.0, text_val.0);

        let nums: Arc<Numbers> = get().expect("Numbers not found in registry");
        assert_eq!(&nums.0, &nums_val.0);

        // Clear the registry after the test
        clear();
    }

    #[test]
    #[serial]
    fn test_custom_type() {
        clear();

        #[derive(Debug, PartialEq, Eq, Clone)]
        struct MyStruct {
            field: String,
        }

        let my_value = MyStruct {
            field: "test".into(),
        };
        register(my_value.clone());

        let retrieved: Arc<MyStruct> = get().unwrap();
        assert_eq!(&*retrieved, &my_value);
    }

    #[test]
    #[serial]
    fn test_tuple_type() -> Result<(), String> {
        clear();

        let tuple = (1, "test");
        register(tuple.clone());

        let retrieved = get::<(i32, &str)>()?;
        assert_eq!(&*retrieved, &tuple);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_overwrite_same_type() {
        clear();

        register(10i32);
        register(20i32); // should replace

        let num: Arc<i32> = get().unwrap();
        assert_eq!(*num, 20);
    }

    #[test]
    #[serial]
    fn test_di_get_cloned() {
        clear();
        register("hello".to_string());
        let value: String = get_cloned::<String>().expect("Value should be present");
        assert_eq!(value, "hello");
    }

    #[test]
    #[serial]
    fn test_di_get_ref() {
        clear();
        register("world".to_string());
        let value: &'static String = get_ref::<String>().expect("Value should be present");
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
        clear();
        assert!(!contains::<u32>().unwrap());
        register(1u32);
        assert!(contains::<u32>().unwrap());
    }

    #[test]
    #[serial]
    fn test_function_pointer_registration() {
        clear();

        // Test the function pointer example from README
        let multiply_by_two: fn(i32) -> i32 = |x| x * 2;
        register(multiply_by_two);

        let doubler: Arc<fn(i32) -> i32> = get().unwrap();
        let result = doubler(21);
        assert_eq!(result, 42);
    }

    #[test]
    #[serial]
    fn test_trace_callback_invoked() {
        clear();
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        set_trace_callback(|_e| {
            COUNT.fetch_add(1, Ordering::SeqCst);
        });
        register(5u8);
        assert_eq!(COUNT.load(Ordering::SeqCst), 1); // adjust after re-enabling emit
        clear_trace_callback();
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
