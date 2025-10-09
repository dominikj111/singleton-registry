use std::fmt;

#[derive(Debug, PartialEq)]
pub enum RegistryError {
    RegistryLock,
    TypeMismatch,
    TypeNotFound,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistryError::RegistryLock => write!(f, "Failed to acquire registry lock"),
            RegistryError::TypeMismatch => write!(f, "Type mismatch in registry"),
            RegistryError::TypeNotFound => write!(f, "Type not found in registry"),
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
        let err = RegistryError::TypeMismatch;
        assert_eq!(err.to_string(), "Type mismatch in registry");
    }

    #[test]
    fn test_type_not_found_display() {
        let err = RegistryError::TypeNotFound;
        assert_eq!(err.to_string(), "Type not found in registry");
    }

    #[test]
    fn test_debug_format() {
        let err = RegistryError::TypeNotFound;
        assert_eq!(format!("{:?}", err), "TypeNotFound");
    }

    #[test]
    fn test_equality() {
        assert_eq!(RegistryError::RegistryLock, RegistryError::RegistryLock);
        assert_ne!(RegistryError::RegistryLock, RegistryError::TypeNotFound);
    }

    #[test]
    fn test_error_trait() {
        let err: &dyn std::error::Error = &RegistryError::TypeNotFound;
        assert_eq!(err.to_string(), "Type not found in registry");
    }
}
