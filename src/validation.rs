use crate::error::{PrmError, PrmResult};

/// Validates that a string is not blank (empty or whitespace-only).
/// Returns the trimmed string on success.
pub fn non_blank(value: &str, field: &str) -> PrmResult<String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        Err(PrmError::BlankField {
            field: field.to_string(),
        })
    } else {
        Ok(trimmed)
    }
}

/// Validates that an integer is positive (> 0).
pub fn positive(value: i32, field: &str) -> PrmResult<i32> {
    if value <= 0 {
        Err(PrmError::NonPositive {
            field: field.to_string(),
        })
    } else {
        Ok(value)
    }
}

/// Validates that a set/vec is non-empty.
pub fn non_empty_set<T>(value: &[T], field: &str) -> PrmResult<()> {
    if value.is_empty() {
        Err(PrmError::EmptySet {
            field: field.to_string(),
        })
    } else {
        Ok(())
    }
}

/// Validates an optional positive integer (None is valid, Some(n) must be positive).
pub fn optional_positive(value: Option<i32>, field: &str) -> PrmResult<Option<i32>> {
    match value {
        None => Ok(None),
        Some(n) => positive(n, field).map(Some),
    }
}

/// Trims an optional string, returning None if blank.
pub fn trim_optional(value: Option<&str>) -> Option<String> {
    value
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_blank_accepts_valid_string() {
        assert_eq!(non_blank("hello", "name").unwrap(), "hello");
    }

    #[test]
    fn non_blank_trims_whitespace() {
        assert_eq!(non_blank("  hello  ", "name").unwrap(), "hello");
    }

    #[test]
    fn non_blank_rejects_empty() {
        assert!(non_blank("", "name").is_err());
    }

    #[test]
    fn non_blank_rejects_whitespace_only() {
        assert!(non_blank("   ", "name").is_err());
    }

    #[test]
    fn positive_accepts_positive() {
        assert_eq!(positive(5, "days").unwrap(), 5);
    }

    #[test]
    fn positive_rejects_zero() {
        assert!(positive(0, "days").is_err());
    }

    #[test]
    fn positive_rejects_negative() {
        assert!(positive(-1, "days").is_err());
    }

    #[test]
    fn non_empty_set_accepts_non_empty() {
        assert!(non_empty_set(&["a"], "topics").is_ok());
    }

    #[test]
    fn non_empty_set_rejects_empty() {
        let empty: &[String] = &[];
        assert!(non_empty_set(empty, "topics").is_err());
    }

    #[test]
    fn optional_positive_accepts_none() {
        assert_eq!(optional_positive(None, "days").unwrap(), None);
    }

    #[test]
    fn optional_positive_accepts_positive() {
        assert_eq!(optional_positive(Some(7), "days").unwrap(), Some(7));
    }

    #[test]
    fn optional_positive_rejects_zero() {
        assert!(optional_positive(Some(0), "days").is_err());
    }

    #[test]
    fn trim_optional_trims() {
        assert_eq!(trim_optional(Some("  hi  ")), Some("hi".to_string()));
    }

    #[test]
    fn trim_optional_returns_none_for_blank() {
        assert_eq!(trim_optional(Some("   ")), None);
    }

    #[test]
    fn trim_optional_returns_none_for_none() {
        assert_eq!(trim_optional(None), None);
    }
}
