use regex::Regex;

const JAVA_IDENTIFIER_RE: &str = r"[a-zA-Z\$_][a-zA-Z0-9_\$]*";

/// Returns true if a package name is valid
pub fn is_valid_package(package: &str) -> bool {
    let re = Regex::new(&format!(
        r"^{}(?:\.{})*$",
        JAVA_IDENTIFIER_RE, JAVA_IDENTIFIER_RE
    ))
    .unwrap();
    re.is_match(package)
}

/// Returns true if a class name is valid
pub fn is_valid_class(class_name: &str) -> bool {
    // This happens to use the same regex as a package name
    is_valid_package(class_name)
}

/// Returns true if a method name is valid
pub fn is_valid_method(method_name: &str) -> bool {
    let re = Regex::new(&format!(r"^{}$", JAVA_IDENTIFIER_RE)).unwrap();
    re.is_match(method_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_packages() {
        assert!(is_valid_package("com.example"));
        assert!(is_valid_package("my_package"));
    }

    #[test]
    fn test_invalid_packages() {
        assert!(!is_valid_package(""));
        assert!(!is_valid_package("123abc"));
    }

    #[test]
    fn test_valid_methods() {
        assert!(is_valid_method("myMethod"));
        assert!(is_valid_method("my_method"));
        assert!(is_valid_method("myMethod123"));
        assert!(is_valid_method("myMethod_123"));
        assert!(is_valid_method("$myMethod_123"));
        assert!(is_valid_method("_myMethod_123"));
    }

    #[test]
    fn test_invalid_methods() {
        assert!(!is_valid_method(""));
        assert!(!is_valid_method("123abc"));
        assert!(!is_valid_method("my method"));
        assert!(!is_valid_method("my.method"));
    }
}
