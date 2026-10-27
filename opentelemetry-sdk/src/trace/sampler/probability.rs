use crate::trace::{SamplingDecision, SamplingResult, ShouldSample};
use opentelemetry::{
    otel_warn,
    trace::{Link, SpanKind, TraceContextExt, TraceId, TraceState},
    Context, KeyValue,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 2^56, the maximum adjusted count and total number of 56-bit randomness values.
const MAX_ADJUSTED_COUNT: u64 = 1 << 56;

/// Mask for the least significant 56 bits (7 bytes) of a TraceId.
const RANDOMNESS_MASK: u64 = MAX_ADJUSTED_COUNT - 1;

/// Default precision for threshold encoding (4 hex digits).
const DEFAULT_SAMPLING_PRECISION: i32 = 4;

/// Smallest representable probability: 1 / 2^56.
const PROBABILITY_ZERO_THRESHOLD: f64 = 1.0 / (MAX_ADJUSTED_COUNT as f64);

/// Standalone consistent probability sampler according to the OpenTelemetry specification.
///
/// `ProbabilitySampler` implements 56-bit consistent probability sampling:
///
/// 1. A 56-bit rejection threshold $T$ is calculated from the configured sampling probability $p$
///    using adaptive precision encoding: $T = 2^{56} \times (1 - p)$.
/// 2. For each span, a 56-bit randomness value $R$ is determined:
///    - If a valid explicit randomness value `rv` is present in the input OpenTelemetry
///      [`TraceState`] (`ot` key), it is parsed and used.
///    - Otherwise, randomness is extracted from the least significant 56 bits (7 bytes) of the [`TraceId`].
/// 3. The decision rule evaluates $R \ge T$:
///    - If $R \ge T$, the span is sampled ([`SamplingDecision::RecordAndSample`]), and the outgoing
///      [`TraceState`] is updated to include the sampling threshold `th` under the `ot` key.
///      If that would exceed the value length limit, any old threshold is erased.
///    - If $R < T$, the span is dropped ([`SamplingDecision::Drop`]), leaving the input [`TraceState`] unmodified.
///
/// ### Standalone vs. Parent-Based Sampling
///
/// A standalone `ProbabilitySampler` does not respect the parent span's sampling decision.
/// To preserve parent sampling decisions while using probability sampling for root spans,
/// use [`ParentBasedSampler`](crate::trace::ParentBasedSampler) with `ProbabilitySampler`
/// as its root delegate.
/// The legacy [`Sampler::ParentBased`](crate::trace::Sampler::ParentBased) variant
/// forwards only the delegate's decision, discarding its attributes and modified TraceState.
///
/// ### Difference from `TraceIdRatioBased`
///
/// The legacy [`Sampler::TraceIdRatioBased`](crate::trace::Sampler::TraceIdRatioBased) uses a 63-bit ratio
/// threshold and does not propagate or interpret consistent sampling metadata (`th`, `rv`) in [`TraceState`].
/// In contrast, `ProbabilitySampler` adheres to the OpenTelemetry consistent probability sampling specification,
/// supporting interoperable threshold propagation, explicit randomness, and downstream sampling.
///
/// ### Status
///
/// This sampler implements the OpenTelemetry Consistent Probability Sampling specification,
/// currently in [Development](https://opentelemetry.io/docs/specs/otel/trace/tracestate-probability-sampling/).
/// Enable the `experimental_trace_probability_sampler` feature to use it.
///
/// # Examples
///
/// Standalone configuration:
///
/// ```
/// use opentelemetry_sdk::trace::{ProbabilitySampler, SdkTracerProvider};
///
/// let provider = SdkTracerProvider::builder()
///     .with_sampler(ProbabilitySampler::new(0.5))
///     .build();
/// ```
///
/// Configuring with [`ParentBasedSampler`](crate::trace::ParentBasedSampler):
///
/// ```
/// use opentelemetry_sdk::trace::{ParentBasedSampler, ProbabilitySampler, SdkTracerProvider};
///
/// let root_sampler = ProbabilitySampler::new(0.05);
/// let provider = SdkTracerProvider::builder()
///     .with_sampler(ParentBasedSampler::new(root_sampler))
///     .build();
/// ```
///
/// [`TraceState`]: opentelemetry::trace::TraceState
#[derive(Clone, Debug)]
pub struct ProbabilitySampler {
    threshold: u64,
    th_kv: Box<str>,
    warn_tracestate: Arc<AtomicBool>,
    warn_non_random: Arc<AtomicBool>,
}

impl ProbabilitySampler {
    /// Creates a new [`ProbabilitySampler`] with the given sampling probability.
    ///
    /// - Probabilities $\ge 1.0$ (including positive infinity) always sample with threshold 0 (`th:0`).
    /// - Probabilities $\le 0.0$, negative numbers, values below $2^{-56}$, and [`f64::NAN`] never sample.
    /// - Probabilities between $2^{-56}$ and $1.0$ are converted to 56-bit integer rejection thresholds
    ///   using adaptive hexadecimal precision encoding.
    pub fn new(probability: f64) -> Self {
        let (threshold, th_kv) = calculate_threshold(probability);
        Self {
            threshold,
            th_kv: th_kv.into_boxed_str(),
            warn_tracestate: Arc::new(AtomicBool::new(false)),
            warn_non_random: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl ShouldSample for ProbabilitySampler {
    fn should_sample(
        &self,
        parent_context: Option<&Context>,
        trace_id: TraceId,
        _name: &str,
        _span_kind: &SpanKind,
        _attributes: &[KeyValue],
        _links: &[Link],
    ) -> SamplingResult {
        let parent_span = parent_context.map(|cx| cx.span());
        // Root contexts can carry user-provided TraceState even without valid span IDs.
        let parent_sc = parent_span
            .as_ref()
            .map(|s| s.span_context())
            .filter(|sc| !sc.is_valid() || sc.trace_id() == trace_id);
        let empty_trace_state = TraceState::default();
        let parent_trace_state = parent_sc
            .map(|sc| sc.trace_state())
            .unwrap_or(&empty_trace_state);

        if self.threshold == MAX_ADJUSTED_COUNT {
            return SamplingResult {
                decision: SamplingDecision::Drop,
                attributes: Vec::new(),
                trace_state: parent_trace_state.clone(),
            };
        }

        let existing_otts = parent_trace_state.get("ot");

        if self.threshold > 0 {
            let explicit_rv = existing_otts.and_then(tracestate_randomness);
            let randomness = match explicit_rv {
                Some(rv) => rv,
                None => {
                    if let Some(sc) = parent_sc {
                        if sc.is_valid()
                            && !sc.trace_flags().is_random()
                            && !self.warn_non_random.swap(true, Ordering::Relaxed)
                        {
                            otel_warn!(
                                name: "ProbabilitySampler.PresumingRandomTraceId",
                                message = "The ProbabilitySampler sampler is presuming TraceIDs are random and expects the Trace random flag to be set in confirmation. Please upgrade your caller(s) to use W3C Trace Context Level 2."
                            );
                        }
                    }
                    let bytes = trace_id.to_bytes();
                    let least_half = u64::from_be_bytes([
                        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                        bytes[15],
                    ]);
                    least_half & RANDOMNESS_MASK
                }
            };

            if randomness < self.threshold {
                return SamplingResult {
                    decision: SamplingDecision::Drop,
                    attributes: Vec::new(),
                    trace_state: parent_trace_state.clone(),
                };
            }
        }

        let new_otts =
            insert_or_update_trace_state_th_key_value(existing_otts.unwrap_or(""), &self.th_kv);

        let trace_state = if existing_otts == Some(new_otts.as_str()) {
            parent_trace_state.clone()
        } else {
            match parent_trace_state.insert("ot", new_otts) {
                Ok(combined) => combined,
                Err(err) => {
                    if !self.warn_tracestate.swap(true, Ordering::Relaxed) {
                        otel_warn!(
                            name: "ProbabilitySampler.TracestateCombineFailed",
                            message = "The sampling threshold could not be propagated because the updated 'ot' TraceState value exceeds the value length limit. Any previous threshold has been removed. Shorten the 'ot' value to allow the sampling threshold to be propagated.",
                            error = format!("{err}")
                        );
                    }
                    // The parent's threshold no longer describes this decision. Keep its
                    // randomness and other concerns, but erase stale probability metadata.
                    let without_th =
                        insert_or_update_trace_state_th_key_value(existing_otts.unwrap_or(""), "");
                    if existing_otts == Some(without_th.as_str()) {
                        parent_trace_state.clone()
                    } else if without_th.is_empty() {
                        parent_trace_state.delete("ot")
                    } else {
                        parent_trace_state
                            .insert("ot", without_th)
                            .unwrap_or_else(|_| parent_trace_state.delete("ot"))
                    }
                }
            }
        };

        SamplingResult {
            decision: SamplingDecision::RecordAndSample,
            attributes: Vec::new(),
            trace_state,
        }
    }
}

/// Matches the exponent returned by Go's math.Frexp for positive normal values.
/// The probability bounds ensure both p and 1 - p are normal before calling this.
fn frexp_exp(f: f64) -> i32 {
    let bits = f.to_bits();
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    exp_bits - 1022
}

/// Calculates the rejection threshold and encoded representation for a given probability.
fn calculate_threshold(probability: f64) -> (u64, String) {
    const MAXP: i32 = 14;
    const HBITS: i32 = 4;

    if probability >= 1.0 {
        return (0, "th:0".to_string());
    }
    if probability.is_nan() || probability < PROBABILITY_ZERO_THRESHOLD {
        return (MAX_ADJUSTED_COUNT, String::new());
    }

    let exp_f = frexp_exp(probability);
    let exp_r = frexp_exp(1.0 - probability);
    let precision = MAXP.min(
        (DEFAULT_SAMPLING_PRECISION + exp_f / -HBITS)
            .max(DEFAULT_SAMPLING_PRECISION + exp_r / -HBITS),
    );

    let scaled = (probability * MAX_ADJUSTED_COUNT as f64).round() as u64;
    let mut threshold = MAX_ADJUSTED_COUNT - scaled;

    let shift = HBITS * (MAXP - precision);
    if shift != 0 {
        let half = 1u64 << (shift - 1);
        threshold += half;
        threshold >>= shift;
        threshold <<= shift;
    }

    let hex_full = format!("{threshold:014x}");
    let tvalue = hex_full.trim_end_matches('0');
    let tvalue = if tvalue.is_empty() { "0" } else { tvalue };

    (threshold, format!("th:{tvalue}"))
}

/// Checks whether the string slice is a valid 14-character lowercase hex randomness value.
#[inline]
fn is_valid_rv(value: &str) -> bool {
    value.len() == 14
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Inserts or updates the threshold (`th`) key-value pair in the OpenTelemetry `ot` TraceState value.
///
/// Existing unrelated sub-keys are preserved. An empty `th_kv` erases thresholds.
/// Malformed `rv` sub-keys and duplicates are discarded.
fn insert_or_update_trace_state_th_key_value(existing_otts: &str, th_kv: &str) -> String {
    let mut updated = String::with_capacity(th_kv.len() + 1 + existing_otts.len());
    updated.push_str(th_kv);
    let mut seen_rv = false;
    for field in existing_otts.split(';') {
        if field.starts_with("th:") {
            continue;
        }
        if let Some(rv_val) = field.strip_prefix("rv:") {
            // Use the first rv, as the Go SDK does, and keep downstream readers
            // from choosing a different value when the input contains duplicates.
            // Discard malformed rv sub-keys so invalid randomness is not propagated.
            if seen_rv {
                continue;
            }
            seen_rv = true;
            if !is_valid_rv(rv_val) {
                continue;
            }
        }
        if !field.is_empty() {
            if !updated.is_empty() {
                updated.push(';');
            }
            updated.push_str(field);
        }
    }
    updated
}

/// Extracts the explicit randomness value (`rv`) from the OpenTelemetry `ot` TraceState value if present.
///
/// A valid `rv` must be exactly 14 lowercase hexadecimal characters representing a 56-bit unsigned integer.
fn tracestate_randomness(otts: &str) -> Option<u64> {
    let value = otts
        .split(';')
        .find_map(|field| field.strip_prefix("rv:"))?;
    if !is_valid_rv(value) {
        return None;
    }
    u64::from_str_radix(value, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceFlags, TraceState};

    #[test]
    fn test_threshold_calculation_reference_vectors() {
        let cases = [
            (0.5, 0x80000000000000u64, "th:8"),
            (1.0 / 3.0, 0xaaab0000000000u64, "th:aaab"),
            (2.0 / 3.0, 0x55550000000000u64, "th:5555"),
            (1.0 / 10.0, 0xe6660000000000u64, "th:e666"),
            (1.0 / 256.0, 0xff000000000000u64, "th:ff"),
            (1.0 / 65536.0, 0xffff0000000000u64, "th:ffff"),
            (1.0 / 1048576.0, 0xfffff000000000u64, "th:fffff"),
            (0.25, 0xc0000000000000u64, "th:c"),
            (1.0, 0, "th:0"),
            (1.5, 0, "th:0"),
            (
                PROBABILITY_ZERO_THRESHOLD,
                RANDOMNESS_MASK,
                "th:ffffffffffffff",
            ),
            (
                f64::from_bits(PROBABILITY_ZERO_THRESHOLD.to_bits() - 1),
                MAX_ADJUSTED_COUNT,
                "",
            ),
            (
                f64::from_bits(PROBABILITY_ZERO_THRESHOLD.to_bits() + 1),
                RANDOMNESS_MASK,
                "th:ffffffffffffff",
            ),
            (
                f64::from_bits(1.0_f64.to_bits() - 1),
                8,
                "th:00000000000008",
            ),
            (
                f64::from_bits(1.0_f64.to_bits() - 2),
                16,
                "th:0000000000001",
            ),
            (0.0001, 0xfff97240000000, "th:fff9724"),
            (0.5 + 1.0 / 131072.0, 0x80000000000000, "th:8"),
            (0.5 - 1.0 / 131072.0, 0x80010000000000, "th:8001"),
        ];

        for (prob, want_th, want_th_kv) in cases {
            let sampler = ProbabilitySampler::new(prob);
            assert_eq!(sampler.threshold, want_th, "threshold for prob {prob}");
            assert_eq!(sampler.th_kv.as_ref(), want_th_kv, "th_kv for prob {prob}");
        }
    }

    #[test]
    fn threshold_is_monotone_across_precision_boundaries() {
        let mut probabilities = vec![0.0, 1.0];
        for exponent in 1..=56 {
            let p = 2.0_f64.powi(-exponent);
            for bits in [p.to_bits() - 1, p.to_bits(), p.to_bits() + 1] {
                let p = f64::from_bits(bits);
                probabilities.extend([p, 1.0 - p]);
            }
        }
        probabilities.sort_by(f64::total_cmp);
        let mut previous = MAX_ADJUSTED_COUNT;
        for p in probabilities {
            let sampler = ProbabilitySampler::new(p);
            assert!(sampler.threshold <= previous, "threshold increased at {p}");
            previous = sampler.threshold;
            if sampler.threshold < MAX_ADJUSTED_COUNT {
                let encoded = sampler.th_kv.strip_prefix("th:").unwrap();
                let decoded =
                    u64::from_str_radix(encoded, 16).unwrap() << (4 * (14 - encoded.len()));
                assert_eq!(decoded, sampler.threshold);
                for (r, expected) in [
                    (sampler.threshold.saturating_sub(1), sampler.threshold == 0),
                    (sampler.threshold, true),
                    (RANDOMNESS_MASK, true),
                ] {
                    let result = sampler.should_sample(
                        None,
                        TraceId::from(u128::from(r)),
                        "boundary",
                        &SpanKind::Internal,
                        &[],
                        &[],
                    );
                    assert_eq!(
                        result.decision == SamplingDecision::RecordAndSample,
                        expected,
                        "p={p}, R={r}"
                    );
                }
            }
        }
    }

    #[test]
    fn compatibility_warning_only_marks_non_root_implicit_non_random_decisions() {
        for remote in [true, false] {
            for (valid_parent, flags, ot, warn) in [
                (false, TraceFlags::default(), "foo:bar", false),
                (true, TraceFlags::RANDOM, "foo:bar", false),
                (true, TraceFlags::default(), "rv:80000000000000", false),
                (true, TraceFlags::default(), "rv:bad", true),
                (true, TraceFlags::default(), "foo:bar", true),
            ] {
                let sampler = ProbabilitySampler::new(0.5);
                let cloned = sampler.clone();
                assert!(Arc::ptr_eq(
                    &sampler.warn_non_random,
                    &cloned.warn_non_random
                ));
                let _ = sampler.should_sample(
                    None,
                    TraceId::from(1),
                    "root",
                    &SpanKind::Internal,
                    &[],
                    &[],
                );
                assert!(!cloned.warn_non_random.load(Ordering::Relaxed));
                let cx = Context::new().with_remote_span_context(SpanContext::new(
                    TraceId::from(if valid_parent { 1 } else { 0 }),
                    SpanId::from(1),
                    flags,
                    remote,
                    TraceState::from_key_value([("ot", ot)]).unwrap(),
                ));
                let _ = sampler.should_sample(
                    Some(&cx),
                    TraceId::from(1),
                    "child",
                    &SpanKind::Internal,
                    &[],
                    &[],
                );
                assert_eq!(cloned.warn_non_random.load(Ordering::Relaxed), warn);
                let _ = cloned.should_sample(
                    Some(&cx),
                    TraceId::from(1),
                    "child",
                    &SpanKind::Internal,
                    &[],
                    &[],
                );
                assert_eq!(sampler.warn_non_random.load(Ordering::Relaxed), warn);
            }
        }
    }

    #[test]
    fn test_tracestate_randomness_parsing() {
        const VALID_RV: &str = "0123456789abcd";
        const VALID_RV_VAL: u64 = 0x0123456789abcd;

        let cases = [
            (
                "rv at beginning",
                format!("rv:{VALID_RV}"),
                Some(VALID_RV_VAL),
            ),
            (
                "rv at beginning with more keys",
                format!("rv:{VALID_RV};th:0;other:value"),
                Some(VALID_RV_VAL),
            ),
            (
                "rv in middle",
                format!("th:0;rv:{VALID_RV};other:value"),
                Some(VALID_RV_VAL),
            ),
            (
                "rv at end",
                format!("th:0;other:value;rv:{VALID_RV}"),
                Some(VALID_RV_VAL),
            ),
            (
                "rv with leading zero",
                "rv:0fffffffffffff".to_string(),
                Some(0x0fffffffffffff),
            ),
            ("no rv key", "th:0;other:value".to_string(), None),
            (
                "maximum randomness",
                "rv:ffffffffffffff".to_string(),
                Some(RANDOMNESS_MASK),
            ),
            ("zero randomness", "rv:00000000000000".to_string(), Some(0)),
            ("unicode", "rv:é123456789abcd".to_string(), None),
            (
                "wrong separator",
                "foo:bar|rv:0123456789abcd".to_string(),
                None,
            ),
            ("extra colon", "rv:0123456789abcd:x".to_string(), None),
            (
                "first duplicate wins",
                "rv:0123456789abcd;rv:ffffffffffffff".to_string(),
                Some(VALID_RV_VAL),
            ),
            (
                "invalid first duplicate",
                "rv:bad;rv:0123456789abcd".to_string(),
                None,
            ),
            ("empty string", String::new(), None),
            ("rv value too short", "rv:0123456789abc".to_string(), None),
            ("rv value too long", "rv:0123456789abcdef".to_string(), None),
            ("rv with invalid hex", "rv:0123456789abcg".to_string(), None),
            (
                "rv with uppercase hex",
                "rv:FFFFFFFFFFFFFF".to_string(),
                None,
            ),
        ];

        for (name, otts, want) in cases {
            assert_eq!(tracestate_randomness(&otts), want, "case: {name}");
        }
    }

    #[test]
    fn test_insert_or_update_th() {
        let cases = [
            (
                "erases all thresholds",
                "th:0;rv:0123456789abcd;th:f;foo:bar",
                "",
                "rv:0123456789abcd;foo:bar",
            ),
            (
                "normalizes empty fields",
                ";th:0;;foo:bar;",
                "th:8",
                "th:8;foo:bar",
            ),
            (
                "preserves unknown subkeys",
                "newkey:ABC_-.;th:bad",
                "th:8",
                "th:8;newkey:ABC_-.",
            ),
            (
                "empty existing adds th at front",
                "",
                "th:123456789abcd",
                "th:123456789abcd",
            ),
            (
                "no th in existing adds th at front",
                "rv:0123456789abcd;other:value",
                "th:fedcba98765432",
                "th:fedcba98765432;rv:0123456789abcd;other:value",
            ),
            (
                "existing th is replaced",
                "rv:0123456789abcd;th:0ad;other:value",
                "th:0e1",
                "th:0e1;rv:0123456789abcd;other:value",
            ),
            (
                "th at front is replaced",
                "th:0ad;rv:0123456789abcd",
                "th:0e1",
                "th:0e1;rv:0123456789abcd",
            ),
            (
                "only th in existing is replaced",
                "th:0ad",
                "th:0e1",
                "th:0e1",
            ),
            (
                "th substring in another key (path:0) is left intact; th prepended",
                "path:0;rv:0123456789abcd",
                "th:fedcba98765432",
                "th:fedcba98765432;path:0;rv:0123456789abcd",
            ),
            (
                "discards malformed rv too short",
                "rv:0123456789abc;foo:bar",
                "th:8",
                "th:8;foo:bar",
            ),
            (
                "discards malformed rv too long",
                "rv:0123456789abcdef;foo:bar",
                "th:8",
                "th:8;foo:bar",
            ),
            (
                "discards malformed rv invalid char",
                "rv:0123456789abcg;foo:bar",
                "th:8",
                "th:8;foo:bar",
            ),
            (
                "discards uppercase rv",
                "rv:0123456789ABCD;foo:bar",
                "th:8",
                "th:8;foo:bar",
            ),
            ("discards empty rv", "rv:;foo:bar", "th:8", "th:8;foo:bar"),
            (
                "discards malformed first rv and ignores second valid rv",
                "rv:bad;rv:0123456789abcd;foo:bar",
                "th:8",
                "th:8;foo:bar",
            ),
            (
                "preserves valid rv and discards duplicate",
                "rv:0123456789abcd;rv:bad;foo:bar",
                "th:8",
                "th:8;rv:0123456789abcd;foo:bar",
            ),
        ];

        for (name, existing, th_kv, want) in cases {
            let got = insert_or_update_trace_state_th_key_value(existing, th_kv);
            assert_eq!(got, want, "case: {name}");
        }
    }

    #[test]
    fn test_boundary_conditions() {
        let sampler = ProbabilitySampler::new(0.5);
        let t = 0x80000000000000u64;
        assert_eq!(sampler.threshold, t);

        // Trace ID randomness: R = T - 1 -> Drop
        let trace_id_drop = TraceId::from(0x0000000000000000_007fffffffffffffu128);
        let res_drop =
            sampler.should_sample(None, trace_id_drop, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res_drop.decision, SamplingDecision::Drop);

        // Trace ID randomness: R = T -> RecordAndSample
        let trace_id_sample = TraceId::from(0x0000000000000000_0080000000000000u128);
        let res_sample =
            sampler.should_sample(None, trace_id_sample, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res_sample.decision, SamplingDecision::RecordAndSample);
        assert_eq!(res_sample.trace_state.get("ot"), Some("th:8"));

        // Trace ID randomness: R = T + 1 -> RecordAndSample
        let trace_id_sample_plus = TraceId::from(0x0000000000000000_0080000000000001u128);
        let res_sample_plus = sampler.should_sample(
            None,
            trace_id_sample_plus,
            "test",
            &SpanKind::Internal,
            &[],
            &[],
        );
        assert_eq!(res_sample_plus.decision, SamplingDecision::RecordAndSample);

        // Trace ID randomness: R = 0 -> Drop
        let trace_id_zero = TraceId::from(0u128);
        let res_zero =
            sampler.should_sample(None, trace_id_zero, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res_zero.decision, SamplingDecision::Drop);

        // Trace ID randomness: R = 2^56 - 1 -> RecordAndSample
        let trace_id_max = TraceId::from(0x0000000000000000_00ffffffffffffffu128);
        let res_max =
            sampler.should_sample(None, trace_id_max, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res_max.decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_high_bits_ignored_for_trace_id_randomness() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id_1 = TraceId::from(0x1234567890abcdef_1280000000000000u128);
        let trace_id_2 = TraceId::from(0xfedcba0987654321_ff80000000000000u128);

        let res1 = sampler.should_sample(None, trace_id_1, "test", &SpanKind::Internal, &[], &[]);
        let res2 = sampler.should_sample(None, trace_id_2, "test", &SpanKind::Internal, &[], &[]);

        assert_eq!(res1.decision, SamplingDecision::RecordAndSample);
        assert_eq!(res2.decision, SamplingDecision::RecordAndSample);
        assert_eq!(res1.trace_state, res2.trace_state);
    }

    #[test]
    fn test_explicit_rv_overrides_trace_id() {
        let sampler = ProbabilitySampler::new(0.5);

        // Trace ID would Drop (R = 0), but explicit rv is above threshold
        let trace_id = TraceId::from(1u128 << 64);
        let ts =
            TraceState::from_key_value([("ot", "rv:80000000000000"), ("vendor", "val")]).unwrap();
        let sc = SpanContext::new(trace_id, SpanId::from(1), TraceFlags::default(), false, ts);
        let cx = Context::new().with_remote_span_context(sc);

        let res = sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res.decision, SamplingDecision::RecordAndSample);
        assert_eq!(res.trace_state.get("ot"), Some("th:8;rv:80000000000000"));
        assert_eq!(res.trace_state.get("vendor"), Some("val"));

        // Trace ID would Sample (R = max), but explicit rv is below threshold
        let trace_id_sample = TraceId::from(0x0000000000000000_00ffffffffffffffu128);
        let ts_low =
            TraceState::from_key_value([("ot", "rv:00000000000000"), ("vendor", "val")]).unwrap();
        let sc_low = SpanContext::new(
            trace_id_sample,
            SpanId::from(1),
            TraceFlags::default(),
            false,
            ts_low,
        );
        let cx_low = Context::new().with_remote_span_context(sc_low);

        let res_drop = sampler.should_sample(
            Some(&cx_low),
            trace_id_sample,
            "test",
            &SpanKind::Internal,
            &[],
            &[],
        );
        assert_eq!(res_drop.decision, SamplingDecision::Drop);
        assert_eq!(res_drop.trace_state.get("ot"), Some("rv:00000000000000"));
        assert_eq!(res_drop.trace_state.get("vendor"), Some("val"));
    }

    #[test]
    fn test_malformed_rv_falls_back_to_trace_id() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(0x0000000000000000_0080000000000000u128);

        let cases = [
            "rv:0123456789abc",         // too short
            "rv:0123456789abcdef",      // too long
            "rv:0123456789abcg",        // invalid char
            "rv:80000000000000X",       // malformed
            "rv:8000000000000A",        // uppercase
            "rv:",                      // empty
            "rv:bad;rv:0123456789abcd", // malformed first duplicate
        ];

        for malformed in cases {
            let ts =
                TraceState::from_key_value([("ot", format!("{malformed};vendor:keep").as_str())])
                    .unwrap();
            let sc = SpanContext::new(trace_id, SpanId::from(1), TraceFlags::default(), false, ts);
            let cx = Context::new().with_remote_span_context(sc);

            let res =
                sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
            assert_eq!(
                res.decision,
                SamplingDecision::RecordAndSample,
                "malformed rv '{malformed}' should fall back to valid trace_id"
            );
            assert_eq!(
                res.trace_state.get("ot"),
                Some("th:8;vendor:keep"),
                "outgoing ot must discard malformed rv subkey while preserving unrelated keys"
            );
        }
    }

    #[test]
    fn test_tracestate_preserves_unrelated_vendors_and_subkeys() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(0x0000000000000000_0080000000000000u128);

        let ts = TraceState::from_key_value([
            ("ot", "foo:bar;rv:8123456789abcd;baz:123"),
            ("concurring_vendor", "important_data"),
            ("another_vendor", "v1"),
        ])
        .unwrap();

        let sc = SpanContext::new(trace_id, SpanId::from(1), TraceFlags::default(), false, ts);
        let cx = Context::new().with_remote_span_context(sc);

        let res = sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res.decision, SamplingDecision::RecordAndSample);
        let ot = res.trace_state.get("ot").unwrap();
        assert!(ot.starts_with("th:8;"));
        assert!(ot.contains("foo:bar"));
        assert!(ot.contains("rv:8123456789abcd"));
        assert!(ot.contains("baz:123"));
        assert_eq!(
            res.trace_state.get("concurring_vendor"),
            Some("important_data")
        );
        assert_eq!(res.trace_state.get("another_vendor"), Some("v1"));
    }

    #[test]
    fn test_tracestate_updates_existing_th() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(0x0000000000000000_0080000000000000u128);

        let ts = TraceState::from_key_value([
            ("ot", "rv:8123456789abcd;th:0ad;foo:bar"),
            ("vendor", "value"),
        ])
        .unwrap();

        let sc = SpanContext::new(trace_id, SpanId::from(1), TraceFlags::default(), false, ts);
        let cx = Context::new().with_remote_span_context(sc);

        let res = sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res.decision, SamplingDecision::RecordAndSample);
        assert_eq!(
            res.trace_state.get("ot"),
            Some("th:8;rv:8123456789abcd;foo:bar")
        );
        assert_eq!(res.trace_state.get("vendor"), Some("value"));
    }

    #[test]
    fn test_decision_ignores_parent_sampled_flag() {
        let sampler = ProbabilitySampler::new(0.5);

        // When R < T, drops even if parent was SAMPLED
        let trace_id_drop = TraceId::from(0x0000000000000000_007fffffffffffffu128);
        let sc_sampled = SpanContext::new(
            trace_id_drop,
            SpanId::from(1),
            TraceFlags::SAMPLED,
            false,
            TraceState::default(),
        );
        let cx_sampled = Context::new().with_remote_span_context(sc_sampled);
        let res_drop = sampler.should_sample(
            Some(&cx_sampled),
            trace_id_drop,
            "test",
            &SpanKind::Internal,
            &[],
            &[],
        );
        assert_eq!(res_drop.decision, SamplingDecision::Drop);

        // When R >= T, samples even if parent was NOT SAMPLED
        let trace_id_sample = TraceId::from(0x0000000000000000_0080000000000000u128);
        let sc_unsampled = SpanContext::new(
            trace_id_sample,
            SpanId::from(1),
            TraceFlags::default(),
            false,
            TraceState::default(),
        );
        let cx_unsampled = Context::new().with_remote_span_context(sc_unsampled);
        let res_sample = sampler.should_sample(
            Some(&cx_unsampled),
            trace_id_sample,
            "test",
            &SpanKind::Internal,
            &[],
            &[],
        );
        assert_eq!(res_sample.decision, SamplingDecision::RecordAndSample);
    }

    #[test]
    fn test_probability_one_always_samples() {
        for prob in [1.0, 1.5, f64::INFINITY] {
            let sampler = ProbabilitySampler::new(prob);
            assert_eq!(sampler.threshold, 0);

            // Even for randomness 0
            let res = sampler.should_sample(
                None,
                TraceId::from(0u128),
                "test",
                &SpanKind::Internal,
                &[],
                &[],
            );
            assert_eq!(res.decision, SamplingDecision::RecordAndSample);
            assert_eq!(res.trace_state.get("ot"), Some("th:0"));

            // Non-random parent must not trigger compatibility warning for unconditional decision
            let trace_id = TraceId::from(1u128);
            let ts = TraceState::from_key_value([("ot", "foo:bar"), ("vendor", "val")]).unwrap();
            let sc = SpanContext::new(trace_id, SpanId::from(1), TraceFlags::default(), false, ts);
            let cx = Context::new().with_remote_span_context(sc);
            let res_parent =
                sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
            assert_eq!(res_parent.decision, SamplingDecision::RecordAndSample);
            assert_eq!(res_parent.trace_state.get("ot"), Some("th:0;foo:bar"));
            assert_eq!(res_parent.trace_state.get("vendor"), Some("val"));
            assert!(!sampler.warn_non_random.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn test_probability_zero_never_samples() {
        for prob in [0.0, -1.0, f64::NAN, f64::NEG_INFINITY, 1e-18] {
            let sampler = ProbabilitySampler::new(prob);
            assert_eq!(sampler.threshold, MAX_ADJUSTED_COUNT);

            // Even for max randomness
            let res = sampler.should_sample(
                None,
                TraceId::from(RANDOMNESS_MASK as u128),
                "test",
                &SpanKind::Internal,
                &[],
                &[],
            );
            assert_eq!(res.decision, SamplingDecision::Drop);

            // Non-random parent must not trigger compatibility warning for unconditional decision
            let trace_id = TraceId::from(1u128);
            let ts = TraceState::from_key_value([("ot", "rv:ffffffffffffff"), ("vendor", "val")])
                .unwrap();
            let sc = SpanContext::new(
                trace_id,
                SpanId::from(1),
                TraceFlags::default(),
                false,
                ts.clone(),
            );
            let cx = Context::new().with_remote_span_context(sc);
            let res_parent =
                sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
            assert_eq!(res_parent.decision, SamplingDecision::Drop);
            assert_eq!(res_parent.trace_state, ts);
            assert!(!sampler.warn_non_random.load(Ordering::Relaxed));
        }
    }

    #[test]
    fn root_context_preserves_tracestate_and_explicit_randomness() {
        for (parent_trace_id, parent_span_id) in [
            (TraceId::INVALID, SpanId::from(1)),
            (TraceId::from(1), SpanId::INVALID),
        ] {
            for (probability, trace_id, rv, decision, expected_ot) in [
                (
                    0.5,
                    1u128 << 64,
                    "80000000000000",
                    SamplingDecision::RecordAndSample,
                    "th:8;rv:80000000000000;foo:bar",
                ),
                (
                    0.5,
                    0x80000000000000,
                    "00000000000000",
                    SamplingDecision::Drop,
                    "th:0;rv:00000000000000;foo:bar",
                ),
                (
                    0.0,
                    1u128 << 64,
                    "80000000000000",
                    SamplingDecision::Drop,
                    "th:0;rv:80000000000000;foo:bar",
                ),
                (
                    1.0,
                    1u128 << 64,
                    "00000000000000",
                    SamplingDecision::RecordAndSample,
                    "th:0;rv:00000000000000;foo:bar",
                ),
            ] {
                let sampler = ProbabilitySampler::new(probability);
                let ot = format!("th:0;rv:{rv};foo:bar");
                let state =
                    TraceState::from_key_value([("ot", ot.as_str()), ("vendor", "value")]).unwrap();
                let cx = Context::new().with_remote_span_context(SpanContext::new(
                    parent_trace_id,
                    parent_span_id,
                    TraceFlags::default(),
                    false,
                    state,
                ));
                let result = sampler.should_sample(
                    Some(&cx),
                    TraceId::from(trace_id),
                    "root",
                    &SpanKind::Internal,
                    &[],
                    &[],
                );
                assert_eq!(result.decision, decision);
                assert_eq!(result.trace_state.get("ot"), Some(expected_ot));
                assert_eq!(result.trace_state.get("vendor"), Some("value"));
                assert!(!sampler.warn_non_random.load(Ordering::Relaxed));
            }
        }
    }

    #[test]
    fn parent_from_another_trace_does_not_affect_sampling_or_tracestate() {
        let sampler = ProbabilitySampler::new(0.5);
        let new_trace_id_drop = TraceId::from(1u128 << 64);

        // Valid parent with different TraceId
        let other_trace_id = TraceId::from(2u128 << 64);
        let valid_other_sc = SpanContext::new(
            other_trace_id,
            SpanId::from(1),
            TraceFlags::default(),
            false,
            TraceState::from_key_value([("ot", "rv:ffffffffffffff"), ("vendor", "other")]).unwrap(),
        );
        let valid_other_cx = Context::new().with_remote_span_context(valid_other_sc);
        let res_diff_trace = sampler.should_sample(
            Some(&valid_other_cx),
            new_trace_id_drop,
            "test",
            &SpanKind::Internal,
            &[],
            &[],
        );
        // Does not inherit rv from a different trace
        assert_eq!(res_diff_trace.decision, SamplingDecision::Drop);
        assert_eq!(res_diff_trace.trace_state, TraceState::default());
    }

    #[test]
    fn test_tracestate_at_capacity_32_members() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(0x0000000000000000_0080000000000000u128);

        let mut kvs = Vec::new();
        for i in 0..32 {
            kvs.push((format!("k{i}"), format!("v{i}")));
        }
        let ts = TraceState::from_key_value(kvs).unwrap();
        let sc = SpanContext::new(trace_id, SpanId::from(1), TraceFlags::default(), false, ts);
        let cx = Context::new().with_remote_span_context(sc);

        let res = sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res.decision, SamplingDecision::RecordAndSample);
        assert_eq!(res.trace_state.get("ot"), Some("th:8"));
        // 32 members preserved (last member k31 evicted)
        assert_eq!(res.trace_state.into_iter().count(), 32);
    }

    #[test]
    fn test_tracestate_value_overflow_fallback() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(0x0000000000000000_0080000000000000u128);

        // Max allowed W3C value length is 256 chars. Construct a 255-character `ot` value
        // so that prepending "th:8;" (5 chars) exceeds the 256 limit (260 chars).
        let long_subvalue = "a".repeat(248);
        let long_ot = format!("custom:{long_subvalue}");
        assert_eq!(long_ot.len(), 255);

        let ts = TraceState::from_key_value([("ot", long_ot.as_str()), ("vendor", "val")]).unwrap();
        let sc = SpanContext::new(
            trace_id,
            SpanId::from(1),
            TraceFlags::default(),
            false,
            ts.clone(),
        );
        let cx = Context::new().with_remote_span_context(sc);

        assert!(!sampler.warn_tracestate.load(Ordering::Relaxed));
        let res = sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(res.decision, SamplingDecision::RecordAndSample);
        // Falls back safely to incoming trace state without panic
        assert_eq!(res.trace_state, ts);
        // Warning flag was set
        assert!(sampler.warn_tracestate.load(Ordering::Relaxed));
    }

    #[test]
    fn threshold_insertion_accepts_exactly_256_characters() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(0x80000000000000_u128);
        let ot = format!("custom:{}", "a".repeat(244));
        let cx = Context::new().with_remote_span_context(SpanContext::new(
            trace_id,
            SpanId::from(1),
            TraceFlags::RANDOM,
            true,
            TraceState::from_key_value([("vendor", "val"), ("ot", ot.as_str())]).unwrap(),
        ));
        let result =
            sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        let expected = format!("th:8;{ot}");
        assert_eq!(expected.len(), 256);
        assert_eq!(result.decision, SamplingDecision::RecordAndSample);
        assert_eq!(result.trace_state.get("ot"), Some(expected.as_str()));
        assert_eq!(result.trace_state.get("vendor"), Some("val"));
        assert!(!sampler.warn_tracestate.load(Ordering::Relaxed));
    }

    #[test]
    fn overflowing_threshold_replacement_erases_stale_threshold() {
        let sampler = ProbabilitySampler::new(1.0 / 3.0);
        let trace_id = TraceId::from(u128::from(RANDOMNESS_MASK));
        for old_threshold in ["0", "f"] {
            let prefix = format!("th:{old_threshold};rv:ffffffffffffff;custom:");
            let ot = format!("{prefix}{}", "a".repeat(256 - prefix.len()));
            let ts = TraceState::from_key_value([("ot", ot.as_str()), ("vendor", "val")]).unwrap();
            let cx = Context::new().with_remote_span_context(SpanContext::new(
                trace_id,
                SpanId::from(1),
                TraceFlags::RANDOM,
                true,
                ts,
            ));
            let result =
                sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
            assert_eq!(result.decision, SamplingDecision::RecordAndSample);
            assert_eq!(result.trace_state.get("ot"), Some(&ot[5..]));
            assert_eq!(result.trace_state.get("vendor"), Some("val"));
        }
    }

    #[test]
    fn duplicate_sampling_keys_are_not_propagated_on_sampled_spans() {
        let sampler = ProbabilitySampler::new(0.5);
        let trace_id = TraceId::from(1);
        let ts = TraceState::from_key_value([(
            "ot",
            "th:0;rv:80000000000000;th:f;rv:00000000000000;foo:bar",
        )])
        .unwrap();
        let cx = Context::new().with_remote_span_context(SpanContext::new(
            trace_id,
            SpanId::from(1),
            TraceFlags::RANDOM,
            true,
            ts,
        ));
        let result =
            sampler.should_sample(Some(&cx), trace_id, "test", &SpanKind::Internal, &[], &[]);
        assert_eq!(result.decision, SamplingDecision::RecordAndSample);
        assert_eq!(
            result.trace_state.get("ot"),
            Some("th:8;rv:80000000000000;foo:bar")
        );
    }
}
