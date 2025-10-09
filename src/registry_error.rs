use std::fmt;

/// Error type for registry operations.
///
/// All fallible registry operations return this error type to indicate
/// what went wrong during the operation.
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    /// Failed to acquire the registry lock (lock poisoning).
    ///
    /// This is automatically recovered in most operations, but exposed
    /// here for completeness.
    RegistryLock,

    /// Type mismatch during downcast (should never happen in practice).
    ///
    /// Includes the type name that was requested.
    TypeMismatch {
        /// The type name that was requested
        type_name: &'static str,
    },

    /// The requested type was not found in the registry.
    ///
    /// Includes the type name that was requested.
    TypeNotFound {
        /// The type name that was requested
        type_name: &'static str,
    },
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::RegistryLock => write!(f, "Failed to acquire registry lock"),
            RegistryError::TypeMismatch { type_name } => {
                write!(f, "Type mismatch in registry for type: {}", type_name)
            }
            RegistryError::TypeNotFound { type_name } => {
                write!(f, "Type not found in registry: {}", type_name)
            }
        }
    }
}

impl std::error::Error for RegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_lock_display() {
        let err = RegistryError::RegistryLock;
        assert_eq!(err.to_string(), "Failed to acquire registry lock");
    }

    #[test]
    fn test_type_mismatch_display() {
        let err = RegistryError::TypeMismatch { type_name: "i32" };
        assert_eq!(err.to_string(), "Type mismatch in registry for type: i32");
    }

    #[test]
    fn test_type_not_found_display() {
        let err = RegistryError::TypeNotFound {
            type_name: "String",
        };
        assert_eq!(err.to_string(), "Type not found in registry: String");
    }

    #[test]
    fn test_debug_format() {
        let err = RegistryError::TypeNotFound {
            type_name: "String",
        };
        assert!(format!("{:?}", err).contains("TypeNotFound"));
    }

    #[test]
    fn test_equality() {
        assert_eq!(RegistryError::RegistryLock, RegistryError::RegistryLock);
        assert_ne!(
            RegistryError::RegistryLock,
            RegistryError::TypeNotFound {
                type_name: "String"
            }
        );
    }

    #[test]
    fn test_error_trait() {
        let err: &dyn std::error::Error = &RegistryError::TypeNotFound {
            type_name: "String",
        };
        assert_eq!(err.to_string(), "Type not found in registry: String");
    }
}
