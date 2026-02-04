//! Singleton replacement example for singleton-registry.
//!
//! Demonstrates:
//! - Runtime replacement of registered singletons
//! - Arc reference safety: old references remain valid after replacement
//! - Thread-safe concurrent access during replacement
//!
//! Run with: `cargo run --example singleton_replacement`

use singleton_registry::define_registry;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Create an isolated registry for this example
define_registry!(config);

/// Application configuration that might be hot-reloaded.
#[derive(Debug, Clone)]
struct AppSettings {
    api_endpoint: String,
    timeout_ms: u64,
    version: u32,
}

impl AppSettings {
    fn describe(&self) -> String {
        format!(
            "v{} -> {} (timeout: {}ms)",
            self.version, self.api_endpoint, self.timeout_ms
        )
    }
}

fn main() {
    println!("=== singleton-registry: Singleton Replacement ===\n");

    // -------------------------------------------------------------------------
    // 1. Register initial configuration
    // -------------------------------------------------------------------------
    println!("1. Registering initial configuration...");

    let initial_settings = AppSettings {
        api_endpoint: "https://api.v1.example.com".to_string(),
        timeout_ms: 5000,
        version: 1,
    };

    config::register(initial_settings);

    let settings_v1: Arc<AppSettings> = config::get().unwrap();
    println!("   Initial: {}", settings_v1.describe());
    println!("   Arc strong count: {}", Arc::strong_count(&settings_v1));

    // -------------------------------------------------------------------------
    // 2. Grab a reference before replacement
    // -------------------------------------------------------------------------
    println!("\n2. Holding reference to current configuration...");

    let held_reference: Arc<AppSettings> = config::get().unwrap();
    println!("   Held reference version: {}", held_reference.version);
    println!(
        "   Arc strong count (before replacement): {}",
        Arc::strong_count(&held_reference)
    );

    // -------------------------------------------------------------------------
    // 3. Replace the singleton
    // -------------------------------------------------------------------------
    println!("\n3. Replacing singleton with new configuration...");

    let updated_settings = AppSettings {
        api_endpoint: "https://api.v2.example.com".to_string(),
        timeout_ms: 10000,
        version: 2,
    };

    config::register(updated_settings);

    println!("   Replacement complete!");

    // -------------------------------------------------------------------------
    // 4. Prove old reference is still valid
    // -------------------------------------------------------------------------
    println!("\n4. Verifying old reference still works...");

    println!("   Old reference endpoint: {}", held_reference.api_endpoint);
    println!("   Old reference version: {}", held_reference.version);
    println!(
        "   Old reference Arc strong count: {}",
        Arc::strong_count(&held_reference)
    );

    // -------------------------------------------------------------------------
    // 5. New lookups get the replacement
    // -------------------------------------------------------------------------
    println!("\n5. New lookups return the replacement...");

    let settings_v2: Arc<AppSettings> = config::get().unwrap();
    println!("   New lookup endpoint: {}", settings_v2.api_endpoint);
    println!("   New lookup version: {}", settings_v2.version);

    // -------------------------------------------------------------------------
    // 6. Compare references
    // -------------------------------------------------------------------------
    println!("\n6. Comparing references...");

    println!(
        "   Old ptr: {:p}, New ptr: {:p}",
        Arc::as_ptr(&held_reference),
        Arc::as_ptr(&settings_v2)
    );
    println!(
        "   Same Arc? {}",
        Arc::ptr_eq(&held_reference, &settings_v2)
    );

    // -------------------------------------------------------------------------
    // 7. Demonstrate thread-safe concurrent access
    // -------------------------------------------------------------------------
    println!("\n7. Demonstrating thread-safe concurrent access...\n");

    // Spawn a thread that holds a reference and reads from it
    let reader_handle = {
        let ref_for_thread: Arc<AppSettings> = config::get().unwrap();
        thread::spawn(move || {
            println!(
                "   [Reader Thread] Got version {} before replacement starts",
                ref_for_thread.version
            );

            // Simulate work with the old reference
            thread::sleep(Duration::from_millis(100));

            // Reference is still valid even if main thread replaces the singleton
            println!(
                "   [Reader Thread] Still using version {} (unchanged)",
                ref_for_thread.version
            );

            ref_for_thread.version
        })
    };

    // Small delay to ensure reader thread starts
    thread::sleep(Duration::from_millis(10));

    // Replace while reader thread is running
    println!("   [Main Thread] Replacing to version 3...");
    config::register(AppSettings {
        api_endpoint: "https://api.v3.example.com".to_string(),
        timeout_ms: 15000,
        version: 3,
    });
    println!("   [Main Thread] Replacement complete");

    // Get the new version
    let settings_v3: Arc<AppSettings> = config::get().unwrap();
    println!("   [Main Thread] Current version: {}", settings_v3.version);

    // Wait for reader and verify it saw the old version
    let reader_saw_version = reader_handle.join().unwrap();
    println!(
        "\n   Reader thread safely used version {} throughout",
        reader_saw_version
    );

    // -------------------------------------------------------------------------
    // Summary
    // -------------------------------------------------------------------------
    println!("\n=== Example Complete ===");
    println!("Key takeaways:");
    println!("  - Replacement updates the registry atomically");
    println!("  - Existing Arc<T> references remain valid (reference counting)");
    println!("  - In-flight operations complete with their original reference");
    println!("  - New lookups get the latest registered singleton");
    println!("  - Thread-safe: no data races, no undefined behavior");
}
