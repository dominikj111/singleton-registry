//! Basic usage example for singleton-registry.
//!
//! Demonstrates:
//! - Registering primitives, strings, and custom structs
//! - Retrieving values with `get()` (returns `Arc<T>`)
//! - Retrieving cloned values with `get_cloned()` (returns `T`)
//! - Checking registration status with `contains()`
//!
//! Run with: `cargo run --example basic_usage`

use singleton_registry::define_registry;
use std::sync::Arc;

// Create an isolated registry for this example
define_registry!(app);

// Custom struct to demonstrate complex types
#[derive(Debug, Clone, PartialEq)]
struct AppConfig {
    name: String,
    version: u32,
    debug_mode: bool,
}

fn main() {
    println!("=== singleton-registry: Basic Usage ===\n");

    // -------------------------------------------------------------------------
    // 1. Register primitives
    // -------------------------------------------------------------------------
    println!("1. Registering primitives...");

    app::register(42i32);
    app::register(3.14f64);
    app::register(true);

    println!("   Registered: i32(42), f64(3.14), bool(true)");

    // -------------------------------------------------------------------------
    // 2. Register a String
    // -------------------------------------------------------------------------
    println!("\n2. Registering a String...");

    app::register("Hello, singleton-registry!".to_string());

    println!("   Registered: String");

    // -------------------------------------------------------------------------
    // 3. Register a custom struct
    // -------------------------------------------------------------------------
    println!("\n3. Registering a custom struct...");

    let config = AppConfig {
        name: "MyApp".to_string(),
        version: 1,
        debug_mode: true,
    };
    app::register(config);

    println!("   Registered: AppConfig");

    // -------------------------------------------------------------------------
    // 4. Check if types are registered with contains()
    // -------------------------------------------------------------------------
    println!("\n4. Checking registration status with contains()...");

    println!("   contains::<i32>()       = {}", app::contains::<i32>().unwrap());
    println!("   contains::<String>()    = {}", app::contains::<String>().unwrap());
    println!("   contains::<AppConfig>() = {}", app::contains::<AppConfig>().unwrap());
    println!("   contains::<Vec<u8>>()   = {}", app::contains::<Vec<u8>>().unwrap()); // Not registered

    // -------------------------------------------------------------------------
    // 5. Retrieve values with get() - returns Arc<T>
    // -------------------------------------------------------------------------
    println!("\n5. Retrieving values with get() -> Arc<T>...");

    let number: Arc<i32> = app::get().unwrap();
    let pi: Arc<f64> = app::get().unwrap();
    let flag: Arc<bool> = app::get().unwrap();
    let message: Arc<String> = app::get().unwrap();
    let cfg: Arc<AppConfig> = app::get().unwrap();

    println!("   i32:       {}", *number);
    println!("   f64:       {}", *pi);
    println!("   bool:      {}", *flag);
    println!("   String:    {}", *message);
    println!("   AppConfig: {:?}", *cfg);

    // -------------------------------------------------------------------------
    // 6. Retrieve cloned values with get_cloned() - returns T
    // -------------------------------------------------------------------------
    println!("\n6. Retrieving cloned values with get_cloned() -> T...");

    // get_cloned() requires the type to implement Clone
    let number_owned: i32 = app::get_cloned().unwrap();
    let message_owned: String = app::get_cloned().unwrap();
    let cfg_owned: AppConfig = app::get_cloned().unwrap();

    println!("   i32 (owned):       {}", number_owned);
    println!("   String (owned):    {}", message_owned);
    println!("   AppConfig (owned): {:?}", cfg_owned);

    // -------------------------------------------------------------------------
    // 7. Handle missing types gracefully
    // -------------------------------------------------------------------------
    println!("\n7. Handling missing types...");

    match app::get::<Vec<u8>>() {
        Ok(value) => println!("   Found Vec<u8>: {:?}", value),
        Err(e) => println!("   Error (expected): {}", e),
    }

    // -------------------------------------------------------------------------
    // Summary
    // -------------------------------------------------------------------------
    println!("\n=== Example Complete ===");
    println!("The registry now contains 5 singletons (i32, f64, bool, String, AppConfig).");
}
