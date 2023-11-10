use opentelemetry::metrics::Unit;
use std::borrow::Cow;

const NON_APPLICABLE_ON_PER_UNIT: [&str; 8] = ["1", "d", "h", "min", "s", "ms", "us", "ns"];

pub(crate) fn get_unit_suffixes(unit: &Unit) -> Option<Cow<'static, str>> {
    // no unit return early
    if unit.as_str().is_empty() {
        return None;
    }

    // direct match with known units
    if let Some(matched) = get_prom_units(unit.as_str()) {
        return Some(Cow::Borrowed(matched));
    }

    // converting foo/bar to foo_per_bar
    // split the string by the first '/'
    // if the first part is empty, we just return the second part if it's a match with known per unit
    // e.g
    // "test/y" => "per_year"
    // "km/s" => "kilometers_per_second"
    if let Some((first, second)) = unit.as_str().split_once('/') {
        return match (
            NON_APPLICABLE_ON_PER_UNIT.contains(&first),
            get_prom_units(first),
            get_prom_per_unit(second),
        ) {
            (true, _, Some(second_part)) | (false, None, Some(second_part)) => {
                Some(Cow::Owned(format!("per_{}", second_part)))
            }
            (false, Some(first_part), Some(second_part)) => {
                Some(Cow::Owned(format!("{}_per_{}", first_part, second_part)))
            }
            _ => None,
        };
    }

    // Unmatched units and annotations are ignored
    // e.g. "{request}"
    None
}

fn get_prom_units(unit: &str) -> Option<&'static str> {
    match unit {
        // Time
        "d" => Some("days"),
        "h" => Some("hours"),
        "min" => Some("minutes"),
        "s" => Some("seconds"),
        "ms" => Some("milliseconds"),
        "us" => Some("microseconds"),
        "ns" => Some("nanoseconds"),

        // Bytes
        "By" => Some("bytes"),
        "KiBy" => Some("kibibytes"),
        "MiBy" => Some("mebibytes"),
        "GiBy" => Some("gibibytes"),
        "TiBy" => Some("tibibytes"),
        "KBy" => Some("kilobytes"),
        "MBy" => Some("megabytes"),
        "GBy" => Some("gigabytes"),
        "TBy" => Some("terabytes"),
        "B" => Some("bytes"),
        "KB" => Some("kilobytes"),
        "MB" => Some("megabytes"),
        "GB" => Some("gigabytes"),
        "TB" => Some("terabytes"),

        // SI
        "m" => Some("meters"),
        "V" => Some("volts"),
        "A" => Some("amperes"),
        "J" => Some("joules"),
        "W" => Some("watts"),
        "g" => Some("grams"),

        // Misc
        "Cel" => Some("celsius"),
        "Hz" => Some("hertz"),
        "1" => Some("ratio"),
        "%" => Some("percent"),
        _ => None,
    }
}

fn get_prom_per_unit(unit: &str) -> Option<&'static str> {
    match unit {
        "s" => Some("second"),
        "m" => Some("minute"),
        "h" => Some("hour"),
        "d" => Some("day"),
        "w" => Some("week"),
        "mo" => Some("month"),
        "y" => Some("year"),
        _ => None,
    }
}

#[allow(clippy::ptr_arg)]
pub(crate) fn sanitize_name(s: &Cow<'static, str>) -> Cow<'static, str> {
    // prefix chars to add in case name starts with number
    let mut prefix = "";

    // Find first invalid char
    if let Some((replace_idx, _)) = s.char_indices().find(|(i, c)| {
        if *i == 0 && c.is_ascii_digit() {
            // first char is number, add prefix and replace reset of chars
            prefix = "_";
            true
        } else {
            // keep checking
            !c.is_alphanumeric() && *c != '_' && *c != ':'
        }
    }) {
        // up to `replace_idx` have been validated, convert the rest
        let (valid, rest) = s.split_at(replace_idx);
        Cow::Owned(
            prefix
                .chars()
                .chain(valid.chars())
                .chain(rest.chars().map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' || c == ':' {
                        c
                    } else {
                        '_'
                    }
                }))
                .collect(),
        )
    } else {
        s.clone() // no invalid chars found, return existing
    }
}

pub(crate) fn sanitize_prom_kv(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_sanitization() {
        let tests = vec![
            ("nam€_with_3_width_rune", "nam__with_3_width_rune"),
            ("`", "_"),
            (
                r##"! "#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWKYZ[]\^_abcdefghijklmnopqrstuvwkyz{|}~"##,
                "________________0123456789:______ABCDEFGHIJKLMNOPQRSTUVWKYZ_____abcdefghijklmnopqrstuvwkyz____",
            ),

            ("Avalid_23name", "Avalid_23name"),
            ("_Avalid_23name", "_Avalid_23name"),
            ("1valid_23name", "_1valid_23name"),
            ("avalid_23name", "avalid_23name"),
            ("Ava:lid_23name", "Ava:lid_23name"),
            ("a lid_23name", "a_lid_23name"),
            (":leading_colon", ":leading_colon"),
            ("colon:in:the:middle", "colon:in:the:middle"),
            ("", ""),
        ];

        for (input, want) in tests {
            assert_eq!(want, sanitize_name(&input.into()), "input: {input}")
        }
    }

    #[test]
    fn test_get_unit_suffixes() {
        let test_cases = vec![
            // Direct match
            ("g", Some(Cow::Borrowed("grams"))),
            // Per unit
            ("test/y", Some(Cow::Owned("per_year".to_owned()))),
            ("1/y", Some(Cow::Owned("per_year".to_owned()))),
            ("m/s", Some(Cow::Owned("meters_per_second".to_owned()))),
            // No match
            ("invalid", None),
            ("invalid/invalid", None),
            ("seconds", None),
            ("", None),
            // annotations
            ("{request}", None),
        ];
        for (unit_str, expected_suffix) in test_cases {
            let unit = Unit::new(unit_str);
            assert_eq!(get_unit_suffixes(&unit), expected_suffix);
        }
    }
}
