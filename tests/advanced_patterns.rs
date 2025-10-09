//! Integration tests for advanced usage patterns.
//!
//! This test demonstrates real-world patterns and best practices for using
//! the singleton registry in production applications.
//!
//! NOTE: All tests use #[serial] because they share the same registry (advanced).
//! Running them in parallel could cause interference.

use serial_test::serial;
use singleton_registry::define_registry;
use std::sync::Arc;

// Create a registry for these tests
define_registry!(advanced);

#[test]
#[serial]
fn test_configuration_pattern() {
    // Common pattern: Store application configuration
    #[derive(Clone, Debug, PartialEq)]
    struct AppConfig {
        database_url: String,
        api_key: String,
        max_connections: u32,
    }

    let config = AppConfig {
        database_url: "postgresql://localhost/mydb".to_string(),
        api_key: "secret-key-123".to_string(),
        max_connections: 100,
    };

    advanced::register(config.clone());

    // Retrieve config anywhere in the app
    let retrieved: Arc<AppConfig> = advanced::get().unwrap();
    assert_eq!(*retrieved, config);
}

#[test]
#[serial]
fn test_service_locator_pattern() {
    // Pattern: Register services and locate them by type
    struct DatabaseService {
        connection_string: String,
    }

    struct CacheService {
        redis_url: String,
    }

    struct LoggingService {
        log_level: String,
    }

    // Register services
    advanced::register(DatabaseService {
        connection_string: "postgres://localhost".to_string(),
    });

    advanced::register(CacheService {
        redis_url: "redis://localhost".to_string(),
    });

    advanced::register(LoggingService {
        log_level: "INFO".to_string(),
    });

    // Locate services
    let db: Arc<DatabaseService> = advanced::get().unwrap();
    let cache: Arc<CacheService> = advanced::get().unwrap();
    let logger: Arc<LoggingService> = advanced::get().unwrap();

    assert_eq!(db.connection_string, "postgres://localhost");
    assert_eq!(cache.redis_url, "redis://localhost");
    assert_eq!(logger.log_level, "INFO");
}

#[test]
#[serial]
fn test_factory_pattern() {
    // Pattern: Register factory functions
    type UserFactory = Box<dyn Fn(String) -> User + Send + Sync>;

    #[derive(Debug, PartialEq)]
    struct User {
        name: String,
        id: u32,
    }

    let id_counter = Arc::new(std::sync::Mutex::new(0u32));
    let counter_clone = id_counter.clone();

    let factory: UserFactory = Box::new(move |name| {
        let mut id = counter_clone.lock().unwrap();
        *id += 1;
        User { name, id: *id }
    });

    advanced::register(factory);

    // Use the factory
    let factory: Arc<UserFactory> = advanced::get().unwrap();
    let user1 = factory("Alice".to_string());
    let user2 = factory("Bob".to_string());

    assert_eq!(user1.id, 1);
    assert_eq!(user2.id, 2);
}

#[test]
#[serial]
fn test_shared_state_pattern() {
    // Pattern: Share mutable state safely
    let counter = Arc::new(std::sync::Mutex::new(0));
    advanced::register(counter.clone());

    // Access from multiple "components"
    let retrieved: Arc<Arc<std::sync::Mutex<i32>>> = advanced::get().unwrap();
    *retrieved.lock().unwrap() += 10;

    let retrieved2: Arc<Arc<std::sync::Mutex<i32>>> = advanced::get().unwrap();
    *retrieved2.lock().unwrap() += 32;

    // Verify shared state
    assert_eq!(*counter.lock().unwrap(), 42);
}

#[test]
#[serial]
fn test_channel_communication_pattern() {
    // Pattern: Register channels for event communication
    use std::sync::mpsc;

    type EventSender = mpsc::Sender<String>;

    let (tx, rx) = mpsc::channel::<String>();
    advanced::register(tx);

    // Send events from anywhere
    let sender: Arc<EventSender> = advanced::get().unwrap();
    sender.send("Event 1".to_string()).unwrap();
    sender.send("Event 2".to_string()).unwrap();

    // Receive events
    let event1: String = rx.recv().unwrap();
    let event2: String = rx.recv().unwrap();

    assert_eq!(event1, "Event 1");
    assert_eq!(event2, "Event 2");
}

#[test]
#[serial]
fn test_lazy_initialization_pattern() {
    // Pattern: Lazy initialization with Once
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn get_expensive_resource() -> Arc<String> {
        INIT.call_once(|| {
            // Expensive initialization
            let resource = "Expensive Resource".to_string();
            advanced::register(resource);
        });

        advanced::get().unwrap()
    }

    // First call initializes
    let res1 = get_expensive_resource();
    // Subsequent calls reuse
    let res2 = get_expensive_resource();

    assert_eq!(&**res1, "Expensive Resource");
    assert_eq!(&**res2, "Expensive Resource");
}

#[test]
#[serial]
fn test_plugin_system_pattern() {
    // Pattern: Plugin system with trait objects
    trait Plugin: Send + Sync {
        fn name(&self) -> &str;
        #[allow(dead_code)]
        fn execute(&self) -> String;
    }

    struct LogPlugin;
    impl Plugin for LogPlugin {
        fn name(&self) -> &str {
            "Logger"
        }
        fn execute(&self) -> String {
            "Logging...".to_string()
        }
    }

    struct CachePlugin;
    impl Plugin for CachePlugin {
        fn name(&self) -> &str {
            "Cache"
        }
        fn execute(&self) -> String {
            "Caching...".to_string()
        }
    }

    // Register plugins by wrapping in a container
    type PluginRegistry = Vec<Arc<dyn Plugin>>;

    let plugins: PluginRegistry = vec![Arc::new(LogPlugin), Arc::new(CachePlugin)];

    advanced::register(plugins);

    // Use plugins
    let registry: Arc<PluginRegistry> = advanced::get().unwrap();
    assert_eq!(registry.len(), 2);
    assert_eq!(registry[0].name(), "Logger");
    assert_eq!(registry[1].name(), "Cache");
}

#[test]
#[serial]
fn test_type_safe_builder_pattern() {
    // Pattern: Builder pattern with registry
    #[derive(Clone)]
    struct DatabaseConfig {
        host: String,
        port: u16,
        database: String,
    }

    struct DatabaseConfigBuilder {
        host: Option<String>,
        port: Option<u16>,
        database: Option<String>,
    }

    impl DatabaseConfigBuilder {
        fn new() -> Self {
            Self {
                host: None,
                port: None,
                database: None,
            }
        }

        fn host(mut self, host: String) -> Self {
            self.host = Some(host);
            self
        }

        fn port(mut self, port: u16) -> Self {
            self.port = Some(port);
            self
        }

        fn database(mut self, database: String) -> Self {
            self.database = Some(database);
            self
        }

        fn build_and_register(self) {
            let config = DatabaseConfig {
                host: self.host.unwrap_or_else(|| "localhost".to_string()),
                port: self.port.unwrap_or(5432),
                database: self.database.unwrap_or_else(|| "default".to_string()),
            };
            advanced::register(config);
        }
    }

    // Use builder to configure and register
    DatabaseConfigBuilder::new()
        .host("db.example.com".to_string())
        .port(3306)
        .database("myapp".to_string())
        .build_and_register();

    // Retrieve the built config
    let config: Arc<DatabaseConfig> = advanced::get().unwrap();
    assert_eq!(config.host, "db.example.com");
    assert_eq!(config.port, 3306);
    assert_eq!(config.database, "myapp");
}
