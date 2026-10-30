//! Structure-aware generators for the W3C Trace Context headers.
//!
//! Each generator builds a header shaped like the grammar in `src/grammars/`, reading every decision
//! from the fuzzer input. The `*Mutated` variants corrupt a few chars of a generated header to reach
//! error paths with inputs that are still close to the grammar.

use crate::oracle::{MAX_KEY_VALUE_LEN, MAX_LIST_MEMBERS};
use arbitrary::{Arbitrary, Unstructured};
use std::fmt;

/// Chars inserted by [`mutate`], besides fully arbitrary ones. Mostly delimiters and chars that are
/// easy to mishandle in header values.
const INTERESTING_CHARS: &[char] = &[
    ' ', '\t', '\r', '\n', '\0', ',', '=', '-', '+', '_', '*', '/', '@', '0', 'A', 'F', 'Z',
    '\u{7f}', '\u{a0}', '\u{3000}', '\u{feff}', 'é',
];

const LCALPHA_DIGIT: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
const KEY_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789_-*/@";

/// Picks a repetition count up to `max`. Usually short, so long repetitions don't use up the input,
/// but a quarter of the time close to `max`, which is set just above a limit worth crossing.
fn repeat_count(u: &mut Unstructured<'_>, max: usize) -> arbitrary::Result<usize> {
    if u.ratio(1, 4)? {
        u.int_in_range(max.saturating_sub(16)..=max)
    } else {
        u.int_in_range(0..=max.min(7))
    }
}

/// Applies one to four random edits (replace, delete, duplicate or insert a char) to `s`.
fn mutate(u: &mut Unstructured<'_>, s: String) -> arbitrary::Result<String> {
    let mut chars: Vec<char> = s.chars().collect();
    for _ in 0..u.int_in_range(1..=4u8)? {
        let pos = u.int_in_range(0..=chars.len())?;
        match u.choose_index(4)? {
            0 if pos < chars.len() => chars[pos] = mutation_char(u)?,
            1 if pos < chars.len() => {
                chars.remove(pos);
            }
            2 if pos < chars.len() => chars.insert(pos, chars[pos]),
            _ => chars.insert(pos, mutation_char(u)?),
        }
    }
    Ok(chars.into_iter().collect())
}

fn mutation_char(u: &mut Unstructured<'_>) -> arbitrary::Result<char> {
    if u.arbitrary()? {
        u.choose(INTERESTING_CHARS).copied()
    } else {
        u.arbitrary()
    }
}

/// A printable ASCII char other than " ", "," and "=".
#[derive(Debug)]
pub struct NblkChr(pub char);

impl<'a> Arbitrary<'a> for NblkChr {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Replace the excluded chars rather than skipping them, biasing slightly towards "_".
        Ok(NblkChr(match u.int_in_range(0x21u8..=0x7E)? {
            b',' | b'=' => '_',
            c => char::from(c),
        }))
    }
}

/// A char allowed after the first char of a `tracestate` key.
#[derive(Debug)]
pub struct KeyChar(pub char);

impl<'a> Arbitrary<'a> for KeyChar {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.choose(KEY_CHARS).map(|&b| KeyChar(char::from(b)))
    }
}

/// A `tracestate` key of up to `MAX_LEN` chars:
/// `(lcalpha / DIGIT) *(lcalpha / DIGIT / "_" / "-" / "*" / "/" / "@")`.
#[derive(Debug)]
pub struct Key<const MAX_LEN: usize>(pub String);

impl<'a, const MAX_LEN: usize> Arbitrary<'a> for Key<MAX_LEN> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut key = String::from(char::from(*u.choose(LCALPHA_DIGIT)?));
        for _ in 0..repeat_count(u, MAX_LEN - 1)? {
            key.push(KeyChar::arbitrary(u)?.0);
        }
        Ok(Key(key))
    }
}

/// A `tracestate` value of up to `MAX_LEN` chars: printable ASCII without "," and "=", not ending in
/// a space.
#[derive(Debug)]
pub struct Value<const MAX_LEN: usize>(pub String);

impl<'a, const MAX_LEN: usize> Arbitrary<'a> for Value<MAX_LEN> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut value = String::new();
        for _ in 0..repeat_count(u, MAX_LEN - 1)? {
            value.push(if u.ratio(1, 8)? {
                ' '
            } else {
                NblkChr::arbitrary(u)?.0
            });
        }
        value.push(NblkChr::arbitrary(u)?.0);
        Ok(Value(value))
    }
}

/// Optional whitespace: up to three spaces or tabs.
#[derive(Debug)]
pub struct Ows(pub String);

impl<'a> Arbitrary<'a> for Ows {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        (0..u.int_in_range(0..=3u8)?)
            .map(|_| Ok(if u.arbitrary()? { ' ' } else { '\t' }))
            .collect::<arbitrary::Result<String>>()
            .map(Ows)
    }
}

/// A `tracestate` list member: `key "=" value`.
#[derive(Debug, Arbitrary)]
pub struct ListMember<const MAX_LEN: usize> {
    pub key: Key<MAX_LEN>,
    pub value: Value<MAX_LEN>,
}

/// A possibly empty `tracestate` list member, surrounded by optional whitespace.
#[derive(Debug, Arbitrary)]
pub struct MemberSlot<const MAX_LEN: usize> {
    pub before: Ows,
    pub member: Option<ListMember<MAX_LEN>>,
    pub after: Ows,
}

/// A `tracestate` list of up to `MAX_SLOTS` member slots.
#[derive(Debug)]
pub struct List<const MAX_SLOTS: usize, const MAX_LEN: usize>(pub Vec<MemberSlot<MAX_LEN>>);

impl<'a, const MAX_SLOTS: usize, const MAX_LEN: usize> Arbitrary<'a> for List<MAX_SLOTS, MAX_LEN> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // A derived `Vec` length would rarely reach the list member limit.
        (0..=repeat_count(u, MAX_SLOTS - 1)?)
            .map(|_| MemberSlot::arbitrary(u))
            .collect::<arbitrary::Result<_>>()
            .map(List)
    }
}

/// A `traceparent` version: "00" or a future version other than the forbidden "ff".
#[derive(Debug)]
pub struct Version(pub u8);

impl<'a> Arbitrary<'a> for Version {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.ratio(1, 4)? {
            u.int_in_range(0x01..=0xFE).map(Version)
        } else {
            Ok(Version(0))
        }
    }
}

/// A `traceparent` id of `N` bytes, all zeros (the invalid id) now and then.
#[derive(Debug)]
pub struct Id<const N: usize>(pub [u8; N]);

impl<'a, const N: usize> Arbitrary<'a> for Id<N> {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.ratio(1, 16)? {
            Ok(Id([0; N]))
        } else {
            u.arbitrary().map(Id)
        }
    }
}

/// The fields of a `traceparent` header. `extra` is only rendered for future versions.
#[derive(Debug, Arbitrary)]
pub struct TraceparentFields {
    pub version: Version,
    pub trace_id: Id<16>,
    pub parent_id: Id<8>,
    pub flags: u8,
    pub extra: Option<String>,
}

impl<const MAX_LEN: usize> fmt::Display for ListMember<MAX_LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.key.0, self.value.0)
    }
}

impl<const MAX_LEN: usize> fmt::Display for MemberSlot<MAX_LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.before.0)?;
        if let Some(member) = &self.member {
            write!(f, "{member}")?;
        }
        f.write_str(&self.after.0)
    }
}

impl<const MAX_SLOTS: usize, const MAX_LEN: usize> fmt::Display for List<MAX_SLOTS, MAX_LEN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, slot) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{slot}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02x}", self.0)
    }
}

impl<const N: usize> fmt::Display for Id<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

impl fmt::Display for TraceparentFields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}-{:02x}",
            self.version, self.trace_id, self.parent_id, self.flags
        )?;
        // Version 00 headers must not carry extra data.
        match &self.extra {
            Some(extra) if self.version.0 != 0 => write!(f, "-{extra}"),
            _ => Ok(()),
        }
    }
}

macro_rules! header_type {
    ($(#[$meta:meta])* $name:ident, |$u:ident| $generate:expr) => {
        $(#[$meta])*
        pub struct $name(String);

        impl $name {
            /// The generated header.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes `self`, returning the generated header.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        // Only print the header, so crash reports stay readable.
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }

        impl<'a> Arbitrary<'a> for $name {
            fn arbitrary($u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
                $generate.map($name)
            }
        }
    };
}

// About half of the generated list members are empty, so twice as many slots are needed to cross
// the list member limit. Long keys and values are left to `TsHeaderLongMembers`, as the fuzzer
// input would rarely be long enough to hold both.
const MANY_SLOTS: usize = 2 * MAX_LIST_MEMBERS + 8;
const SHORT_KEY_VALUE_LEN: usize = 8;
const FEW_SLOTS: usize = 8;
const LONG_KEY_VALUE_LEN: usize = MAX_KEY_VALUE_LEN + 4;

type ManyMembers = List<MANY_SLOTS, SHORT_KEY_VALUE_LEN>;
type LongMembers = List<FEW_SLOTS, LONG_KEY_VALUE_LEN>;

header_type!(
    /// A `tracestate` header matched by the grammar, crossing the list member limit.
    TsHeader,
    |u| ManyMembers::arbitrary(u).map(|list| list.to_string())
);

header_type!(
    /// A `tracestate` header matched by the grammar, with keys and values crossing the length limit.
    TsHeaderLongMembers,
    |u| LongMembers::arbitrary(u).map(|list| list.to_string())
);

header_type!(
    /// A `tracestate` header close to, but not necessarily matched by, the grammar.
    TsHeaderMutated,
    |u| ManyMembers::arbitrary(u).and_then(|list| mutate(u, list.to_string()))
);

header_type!(
    /// A `traceparent` header matched by the grammar.
    Traceparent,
    |u| TraceparentFields::arbitrary(u).map(|fields| fields.to_string())
);

header_type!(
    /// A `traceparent` header close to, but not necessarily matched by, the grammar.
    TraceparentMutated,
    |u| TraceparentFields::arbitrary(u).and_then(|fields| mutate(u, fields.to_string()))
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::{TraceStateParser, TraceStateRule, TraceparentParser, TraceparentRule};
    use crate::test_inputs;
    use pest::Parser;

    fn generate<T: for<'a> Arbitrary<'a>>(into_string: fn(T) -> String) -> Vec<String> {
        test_inputs(1024)
            .map(|data| into_string(T::arbitrary(&mut Unstructured::new(&data)).unwrap()))
            .collect()
    }

    fn members(header: &str) -> Vec<(&str, &str)> {
        header
            .split(',')
            .filter_map(|member| member.trim_matches([' ', '\t']).split_once('='))
            .collect()
    }

    fn assert_tracestate_grammar(headers: &[String]) {
        for header in headers {
            assert!(
                TraceStateParser::parse(TraceStateRule::tracestate, header).is_ok(),
                "{header:?}"
            );
        }
    }

    #[test]
    fn generated_tracestate_headers_match_the_grammar() {
        let headers = generate(TsHeader::into_string);
        assert_tracestate_grammar(&headers);
        assert!(headers
            .iter()
            .any(|header| members(header).len() > MAX_LIST_MEMBERS));
    }

    #[test]
    fn generated_long_tracestate_members_match_the_grammar() {
        let headers = generate(TsHeaderLongMembers::into_string);
        assert_tracestate_grammar(&headers);
        let members: Vec<(&str, &str)> = headers.iter().flat_map(|h| members(h)).collect();
        assert!(members.iter().any(|(key, _)| key.len() > MAX_KEY_VALUE_LEN));
        assert!(members
            .iter()
            .any(|(_, value)| value.len() > MAX_KEY_VALUE_LEN));
    }

    #[test]
    fn generated_traceparent_headers_match_the_grammar() {
        for header in generate(Traceparent::into_string) {
            assert!(
                TraceparentParser::parse(TraceparentRule::traceparent, &header).is_ok(),
                "{header:?}"
            );
        }
    }

    #[test]
    fn mutated_headers_differ_from_the_grammar_sometimes() {
        let headers = generate(TsHeaderMutated::into_string);
        assert!(headers.iter().any(|header| TraceStateParser::parse(
            TraceStateRule::tracestate,
            header
        )
        .is_err()));
        let headers = generate(TraceparentMutated::into_string);
        assert!(headers.iter().any(|header| {
            TraceparentParser::parse(TraceparentRule::traceparent, header).is_err()
        }));
    }
}
