pub fn evaluate(query: &str) -> Option<String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (expr, is_explicit) = if let Some(stripped) = trimmed.strip_prefix('=') {
        (stripped.trim(), true)
    } else if let Some(stripped) = trimmed.strip_prefix("calc ") {
        (stripped.trim(), true)
    } else {
        (trimmed, false)
    };

    if expr.is_empty() {
        return None;
    }

    if !is_explicit {
        let has_math_char = expr
            .chars()
            .any(|character| character.is_ascii_digit() || "+-*/^%()=√".contains(character));
        if !has_math_char {
            return None;
        }
    }

    let mut context = fend_core::Context::new();
    let result = fend_core::evaluate(expr, &mut context).ok()?;
    let main_result = result.get_main_result().trim();

    if main_result.is_empty() || main_result.eq_ignore_ascii_case(expr) || main_result.len() > 100 {
        return None;
    }

    Some(main_result.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(evaluate("2 + 2"), Some("4".to_string()));
        assert_eq!(evaluate("10 * 5 + 2"), Some("52".to_string()));
        assert_eq!(evaluate("(10 + 5) * 2"), Some("30".to_string()));
    }

    #[test]
    fn test_percentages_and_functions() {
        assert_eq!(evaluate("20% of 100"), Some("20".to_string()));
        assert_eq!(evaluate("sqrt(144)"), Some("12".to_string()));
    }

    #[test]
    fn test_unit_conversion() {
        let miles = evaluate("10 km to m");
        assert_eq!(miles, Some("10000 m".to_string()));
    }

    #[test]
    fn test_no_false_positives() {
        assert_eq!(evaluate("firefox"), None);
        assert_eq!(evaluate("spotify"), None);
        assert_eq!(evaluate("code"), None);
        assert_eq!(evaluate("100"), None);
    }

    #[test]
    fn test_explicit_prefix() {
        assert_eq!(evaluate("= 5 * 5"), Some("25".to_string()));
        assert_eq!(evaluate("calc 3 + 3"), Some("6".to_string()));
    }
}
