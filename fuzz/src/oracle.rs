//! Oracles for the W3C Trace Context headers.
//!
//! The pest grammars in `src/grammars/` are turned into parsers by `pest_derive`. Combined with the
//! limits a grammar can't express, they compute what a spec compliant implementation must return.

use opentelemetry::propagation::Extractor;
use opentelemetry::trace::{SpanId, TraceFlags, TraceId};
use pest::Parser;

/// The maximum number of list-members a `tracestate` may hold.
pub const MAX_LIST_MEMBERS: usize = 32;

/// The maximum length of a `tracestate` list-member key or value.
pub const MAX_KEY_VALUE_LEN: usize = 256;

mod tracestate_parser {
    #[derive(pest_derive::Parser)]
    #[grammar = "grammars/tracestate.pest"]
    pub struct TraceStateParser;
}

mod traceparent_parser {
    #[derive(pest_derive::Parser)]
    #[grammar = "grammars/traceparent.pest"]
    pub struct TraceparentParser;
}

pub use traceparent_parser::{Rule as TraceparentRule, TraceparentParser};
pub use tracestate_parser::{Rule as TraceStateRule, TraceStateParser};

/// Returns whether `key` is a valid `tracestate` list-member key.
pub fn is_valid_key(key: &str) -> bool {
    key.len() <= MAX_KEY_VALUE_LEN
        && TraceStateParser::parse(TraceStateRule::single_key, key).is_ok()
}

/// Returns whether `value` is a valid `tracestate` list-member value.
pub fn is_valid_value(value: &str) -> bool {
    value.len() <= MAX_KEY_VALUE_LEN
        && TraceStateParser::parse(TraceStateRule::single_value, value).is_ok()
}

/// The oracle for `tracestate` parsing: the list-members a spec compliant parser keeps for `header`,
/// or `None` if it must reject the header.
///
/// Only the first 32 non-empty list-members are considered. Later ones are dropped without being
/// validated.
pub fn expected_tracestate(header: &str) -> Option<Vec<(String, String)>> {
    let mut members: Vec<(String, String)> = Vec::new();
    // A list-member can't contain a ",", so splitting on it yields the grammar's member slots.
    for slot in header.split(',') {
        if members.len() == MAX_LIST_MEMBERS {
            break;
        }
        let slot = TraceStateParser::parse(TraceStateRule::single_member_slot, slot)
            .ok()?
            .next()
            .expect("single_member_slot always produces a pair");
        let Some(member) = slot
            .into_inner()
            .find(|pair| pair.as_rule() == TraceStateRule::list_member)
        else {
            // Empty and whitespace only list members are ignored.
            continue;
        };

        let mut inner = member.into_inner();
        let key = inner.next().expect("list_member has a key").as_str();
        let value = inner.next().expect("list_member has a value").as_str();
        if key.len() > MAX_KEY_VALUE_LEN
            || value.len() > MAX_KEY_VALUE_LEN
            || members.iter().any(|(existing, _)| existing == key)
        {
            return None;
        }
        members.push((key.to_owned(), value.to_owned()));
    }
    Some(members)
}

/// The oracle for `traceparent` parsing: the ids and the sampled flag a spec compliant parser
/// extracts from `header`, or `None` if it must reject the header.
pub fn expected_traceparent(header: &str) -> Option<(TraceId, SpanId, TraceFlags)> {
    let pairs = TraceparentParser::parse(TraceparentRule::traceparent, header).ok()?;
    let field = |rule| {
        pairs
            .clone()
            .flatten()
            .find(|pair| pair.as_rule() == rule)
            .expect("traceparent has all fields")
            .as_str()
    };

    let trace_id = TraceId::from_hex(field(TraceparentRule::trace_id)).ok()?;
    let span_id = SpanId::from_hex(field(TraceparentRule::parent_id)).ok()?;
    let flags = u8::from_str_radix(field(TraceparentRule::trace_flags), 16).ok()?;
    if trace_id == TraceId::INVALID || span_id == SpanId::INVALID {
        return None;
    }
    // Only the sampled flag is defined, the others must be ignored.
    Some((
        trace_id,
        span_id,
        TraceFlags::new(flags) & TraceFlags::SAMPLED,
    ))
}

/// An [`Extractor`] that may hold several `tracestate` header fields.
#[derive(Debug, Default)]
pub struct MultiFieldExtractor {
    pub traceparent: Option<String>,
    pub tracestate: Vec<String>,
}

impl Extractor for MultiFieldExtractor {
    fn get(&self, key: &str) -> Option<&str> {
        match key {
            "traceparent" => self.traceparent.as_deref(),
            "tracestate" => self.tracestate.first().map(String::as_str),
            _ => None,
        }
    }

    fn keys(&self) -> Vec<&str> {
        let mut keys = Vec::new();
        if self.traceparent.is_some() {
            keys.push("traceparent");
        }
        if !self.tracestate.is_empty() {
            keys.push("tracestate");
        }
        keys
    }

    fn get_all(&self, key: &str) -> Option<Vec<&str>> {
        match key {
            "tracestate" if !self.tracestate.is_empty() => {
                Some(self.tracestate.iter().map(String::as_str).collect())
            }
            key => self.get(key).map(|value| vec![value]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_tracestate_follows_the_spec() {
        let pairs = |members: &[(&str, &str)]| {
            Some(
                members
                    .iter()
                    .map(|(key, value)| (key.to_string(), value.to_string()))
                    .collect::<Vec<_>>(),
            )
        };
        assert_eq!(expected_tracestate(""), pairs(&[]));
        assert_eq!(
            expected_tracestate(" foo=bar ,, \t,a@b= x"),
            pairs(&[("foo", "bar"), ("a@b", " x")])
        );
        assert_eq!(expected_tracestate("foo=bar,foo=baz"), None);
        assert_eq!(expected_tracestate("Foo=bar"), None);
        assert_eq!(expected_tracestate("foo=bar\tbaz"), None);
        assert_eq!(expected_tracestate("foo="), None);
        assert_eq!(expected_tracestate("foo"), None);
        assert_eq!(expected_tracestate(&format!("{}=v", "k".repeat(257))), None);

        let members: Vec<String> = (0..MAX_LIST_MEMBERS).map(|i| format!("k{i}=v")).collect();
        let header = format!("{},INVALID", members.join(","));
        assert_eq!(
            expected_tracestate(&header).unwrap().len(),
            MAX_LIST_MEMBERS
        );
    }

    #[test]
    fn expected_traceparent_follows_the_spec() {
        let ids = "4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7";
        let valid = |header: &str| expected_traceparent(header).is_some();
        assert!(valid(&format!("00-{ids}-01")));
        assert!(valid(&format!("00-{ids}-ff")));
        assert!(valid(&format!("cc-{ids}-01-extra")));
        assert!(!valid(&format!("00-{ids}-01-extra")));
        assert!(!valid(&format!("ff-{ids}-01")));
        assert!(!valid(&format!("00-{}-01", ids.to_uppercase())));
        assert!(!valid(&format!(" 00-{ids}-01")));
        assert!(!valid(
            "00-00000000000000000000000000000000-00f067aa0ba902b7-01"
        ));
        assert!(!valid(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01"
        ));
        assert_eq!(
            expected_traceparent(&format!("00-{ids}-03")).unwrap().2,
            TraceFlags::SAMPLED
        );
    }
}
