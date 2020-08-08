/// sanitize returns a string that is truncated to 100 characters if it's too
/// long, and replaces non-alphanumeric characters to underscores.
pub(crate) fn sanitize<T: AsRef<str>>(raw: T) -> String {
    let mut escaped = raw
        .as_ref()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .peekable();

    let prefix = if escaped.peek().map_or(false, |c| c.is_ascii_digit()) {
        "key_"
    } else if escaped.peek().map_or(false, |&c| c == '_') {
        "key"
    } else {
        ""
    };

    prefix.chars().chain(escaped).take(100).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn key_data() -> Vec<(&'static str, Cow<'static, str>, Cow<'static, str>)> {
        vec![
            (
                "replace character",
                "test/key-1".into(),
                "test_key_1".into(),
            ),
            (
                "add prefix if starting with digit",
                "0123456789".into(),
                "key_0123456789".into(),
            ),
            (
                "add prefix if starting with _",
                "_0123456789".into(),
                "key_0123456789".into(),
            ),
            (
                "starts with _ after sanitization",
                "/0123456789".into(),
                "key_0123456789".into(),
            ),
            (
                "limits to 100",
                "a".repeat(101).into(),
                "a".repeat(100).into(),
            ),
            (
                "valid input",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789".into(),
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789".into(),
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
