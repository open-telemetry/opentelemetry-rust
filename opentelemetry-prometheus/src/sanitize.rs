/// sanitize returns a string that is truncated to 100 characters if it's too
/// long, and replaces non-alphanumeric characters to underscores.
pub(crate) fn sanitize<T: Into<String>>(s: T) -> String {
    let s = s.into();
    if s.is_empty() {
        return s;
    }
    let res = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    if res.chars().next().map(|c| c.is_ascii_digit()) == Some(true) {
        return format!("key_{}", res);
    }

    if res.chars().next().map(|c| c == '_') == Some(true) {
        return format!("key{}", res);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_data() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("replace character", "test/key-1", "test_key_1"),
            (
                "add prefix if starting with digit",
                "0123456789",
                "key_0123456789",
            ),
            (
                "add prefix if starting with _",
                "_0123456789",
                "key_0123456789",
            ),
            (
                "starts with _ after sanitization",
                "/0123456789",
                "key_0123456789",
            ),
            (
                "valid input",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789",
            ),
        ]
    }

    #[test]
    fn sanitize_key_names() {
        for (name, raw, sanitized) in key_data() {
            assert_eq!(sanitize(raw), sanitized, "{} doesn't match", name)
        }
    }
}
