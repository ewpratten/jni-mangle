//! Utilities for escaping strings to be used in mangled names.

/// Escapes any bad characters in a string
pub fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        // Escape each encountered character
        let escaped = match c {
            // Basic Escapes
            '_' => "_1".to_string(),
            ';' => "_2".to_string(),
            '[' => "_3".to_string(),
            '.' => "_".to_string(),

            // More complex cases
            _ => {
                if c.is_ascii() {
                    // Return the character as a string
                    c.to_string()
                } else {
                    // Return the character as a string of hex digits
                    let raw_char = c as u32;
                    format!("_0{}", format!("{:04x}", raw_char).to_lowercase())
                }
            }
        };

        // Push the escaped character(s) to the result string
        result.push_str(&escaped);
    }
    result
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_basic_escapes() {
        assert_eq!(escape_string("Hello_world"), "Hello_1world");
        assert_eq!(escape_string("Hello;world"), "Hello_2world");
        assert_eq!(escape_string("Hello[world"), "Hello_3world");
    }

    #[test]
    fn test_non_ascii_escapes() {
        assert_eq!(escape_string("Hello🙂world"), "Hello_01f642world");
    }
}
