/// Events emitted by the registry during operations.
///
/// These events are passed to the tracing callback set via `set_trace_callback`.
/// The `Clone` derive allows callbacks to store or forward events if needed.
///
/// # Examples
///
/// ```rust
/// use singleton_registry::RegistryEvent;
///
/// let event = RegistryEvent::Register { type_name: "i32" };
/// assert_eq!(event.to_string(), "register { type_name: i32 }");
/// ```
#[derive(Debug, Clone)]
pub enum RegistryEvent {
    /// A value was registered in the registry.
    Register {
        /// The type name of the registered value (e.g., "i32", "alloc::string::String")
        type_name: &'static str,
    },

    /// A value was requested from the registry.
    Get {
        /// The type name that was requested
        type_name: &'static str,
        /// Whether the value was found in the registry
        found: bool,
    },

    /// A type existence check was performed.
    Contains {
        /// The type name that was checked
        type_name: &'static str,
        /// Whether the type exists in the registry
        found: bool,
    },

    /// The registry was cleared.
    Clear {},
}

impl std::fmt::Display for RegistryEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryEvent::Register { type_name } => {
                write!(f, "register {{ type_name: {} }}", type_name)
            }
            RegistryEvent::Get { type_name, found } => {
                write!(f, "get {{ type_name: {}, found: {} }}", type_name, found)
            }
            RegistryEvent::Contains { type_name, found } => {
                write!(
                    f,
                    "contains {{ type_name: {}, found: {} }}",
                    type_name, found
                )
            }
            RegistryEvent::Clear {} => write!(f, "Clearing the Registry"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
