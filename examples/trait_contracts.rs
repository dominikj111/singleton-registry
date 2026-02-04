//! Trait contracts example for singleton-registry.
//!
//! Demonstrates the **contract-based dependency injection** pattern:
//! - Define traits (contracts) that specify behavior
//! - Register concrete implementations as `Arc<dyn Trait>`
//! - Retrieve and use via trait methods
//! - Swap implementations at runtime
//!
//! Run with: `cargo run --example trait_contracts`

use singleton_registry::define_registry;
use std::sync::Arc;

// Create an isolated registry for this example
define_registry!(services);

// =============================================================================
// Contract Definitions (Traits)
// =============================================================================

/// Contract for a logging service.
/// Any implementation must provide these methods.
trait Logger: Send + Sync {
    fn log(&self, message: &str);
    fn name(&self) -> &str;
}

/// Contract for a notification service.
trait Notifier: Send + Sync {
    fn notify(&self, recipient: &str, message: &str);
    fn service_type(&self) -> &str;
}

// =============================================================================
// Concrete Implementations
// =============================================================================

/// Console-based logger implementation.
struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("[CONSOLE] {}", message);
    }

    fn name(&self) -> &str {
        "ConsoleLogger"
    }
}

/// File-based logger implementation (simulated).
struct FileLogger {
    path: String,
}

impl Logger for FileLogger {
    fn log(&self, message: &str) {
        println!("[FILE:{}] {}", self.path, message);
    }

    fn name(&self) -> &str {
        "FileLogger"
    }
}

/// Email notification implementation.
struct EmailNotifier {
    smtp_server: String,
}

impl Notifier for EmailNotifier {
    fn notify(&self, recipient: &str, message: &str) {
        println!(
            "[EMAIL via {}] To: {} - {}",
            self.smtp_server, recipient, message
        );
    }

    fn service_type(&self) -> &str {
        "Email"
    }
}

/// SMS notification implementation.
struct SmsNotifier {
    api_key: String,
}

impl Notifier for SmsNotifier {
    fn notify(&self, recipient: &str, message: &str) {
        println!(
            "[SMS via API:{}] To: {} - {}",
            &self.api_key[..8],
            recipient,
            message
        );
    }

    fn service_type(&self) -> &str {
        "SMS"
    }
}

// =============================================================================
// Application Code (Uses Contracts, Not Implementations)
// =============================================================================

/// Business logic that depends on Logger and Notifier contracts.
/// It doesn't know or care which concrete implementation is used.
fn process_order(order_id: u32) {
    // Retrieve the logger contract
    // Note: We get Arc<Arc<dyn Logger>> due to how trait objects are stored
    // Use get_cloned() to get Arc<dyn Logger> directly
    let logger: Arc<dyn Logger> = services::get_cloned().unwrap();
    let notifier: Arc<dyn Notifier> = services::get_cloned().unwrap();

    logger.log(&format!("Processing order #{}", order_id));
    logger.log("Validating payment...");
    logger.log("Order confirmed!");

    notifier.notify("customer@example.com", &format!("Order #{} confirmed!", order_id));
}

fn main() {
    println!("=== singleton-registry: Trait Contracts ===\n");

    // -------------------------------------------------------------------------
    // 1. Register initial implementations
    // -------------------------------------------------------------------------
    println!("1. Registering initial implementations...");

    // Register ConsoleLogger as the Logger contract
    services::register(Arc::new(ConsoleLogger) as Arc<dyn Logger>);

    // Register EmailNotifier as the Notifier contract
    services::register(Arc::new(EmailNotifier {
        smtp_server: "smtp.example.com".to_string(),
    }) as Arc<dyn Notifier>);

    println!("   Logger: ConsoleLogger");
    println!("   Notifier: EmailNotifier");

    // -------------------------------------------------------------------------
    // 2. Use the contracts (business logic is decoupled)
    // -------------------------------------------------------------------------
    println!("\n2. Processing order with initial implementations...\n");

    process_order(1001);

    // -------------------------------------------------------------------------
    // 3. Swap implementations at runtime
    // -------------------------------------------------------------------------
    println!("\n3. Swapping to different implementations...");

    // Replace Logger with FileLogger
    services::register(Arc::new(FileLogger {
        path: "/var/log/app.log".to_string(),
    }) as Arc<dyn Logger>);

    // Replace Notifier with SmsNotifier
    services::register(Arc::new(SmsNotifier {
        api_key: "sk_live_abc123xyz789".to_string(),
    }) as Arc<dyn Notifier>);

    println!("   Logger: FileLogger");
    println!("   Notifier: SmsNotifier");

    // -------------------------------------------------------------------------
    // 4. Same business logic, different behavior
    // -------------------------------------------------------------------------
    println!("\n4. Processing another order with new implementations...\n");

    process_order(1002);

    // -------------------------------------------------------------------------
    // 5. Verify current implementations
    // -------------------------------------------------------------------------
    println!("\n5. Verifying current implementations...");

    let logger: Arc<dyn Logger> = services::get_cloned().unwrap();
    let notifier: Arc<dyn Notifier> = services::get_cloned().unwrap();

    println!("   Current Logger: {}", logger.name());
    println!("   Current Notifier: {}", notifier.service_type());

    // -------------------------------------------------------------------------
    // Summary
    // -------------------------------------------------------------------------
    println!("\n=== Example Complete ===");
    println!("The registry enables contract-based DI:");
    println!("  - Business logic depends on traits (Logger, Notifier)");
    println!("  - Implementations can be swapped without changing consumer code");
    println!("  - Perfect for testing: register mocks during test setup");
}
