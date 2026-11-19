use arbitrary::Arbitrary;
use opentelemetry::{
    trace::{SpanContext, TraceContextExt, TraceFlags, TraceState},
    Context, ContextGuard, SpanId, TraceId,
};

const CONTEXT_SLOTS: usize = 8;
const GUARD_SLOTS: usize = 16;
const MAX_OPERATIONS: usize = 256;
const ROOT_MARKER: u16 = 0xcafe;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MarkerA(u16);

#[derive(Clone, Debug, PartialEq, Eq)]
struct MarkerB(u16);

#[derive(Clone, Debug)]
struct ExpectedContext {
    marker_a: Option<u16>,
    marker_b: Option<u16>,
    telemetry_suppressed: bool,
    span_context: SpanContext,
}

impl Default for ExpectedContext {
    fn default() -> Self {
        Self {
            marker_a: None,
            marker_b: None,
            telemetry_suppressed: false,
            span_context: SpanContext::NONE,
        }
    }
}

#[derive(Clone)]
struct SavedContext {
    context: Context,
    expected: ExpectedContext,
}

struct SavedGuard {
    guard: ContextGuard,
    token: u64,
}

#[derive(Arbitrary, Debug)]
pub struct Program {
    operations: Vec<Operation>,
}

#[derive(Arbitrary, Debug)]
enum Operation {
    New {
        destination: u8,
    },
    SnapshotCurrent {
        destination: u8,
    },
    Clone {
        source: u8,
        destination: u8,
    },
    Clear {
        slot: u8,
    },
    WithMarkerA {
        source: u8,
        destination: u8,
        value: u16,
    },
    WithMarkerB {
        source: u8,
        destination: u8,
        value: u16,
    },
    WithTelemetrySuppressed {
        source: u8,
        destination: u8,
    },
    WithRemoteSpanContext {
        source: u8,
        destination: u8,
        trace_id: u128,
        span_id: u64,
        trace_flags: u8,
        is_remote: bool,
        trace_state_value: Option<u16>,
    },
    Attach {
        context: u8,
        guard: u8,
    },
    EnterTelemetrySuppressedScope {
        guard: u8,
    },
    DropGuard {
        guard: u8,
    },
}

pub fn run(program: Program) {
    assert_default_context("before fuzz iteration");
    let root_expected = ExpectedContext {
        marker_a: Some(ROOT_MARKER),
        ..ExpectedContext::default()
    };
    let root_guard = Context::new().with_value(MarkerA(ROOT_MARKER)).attach();

    let mut contexts: Vec<Option<SavedContext>> = (0..CONTEXT_SLOTS)
        .map(|_| {
            Some(SavedContext {
                context: Context::new(),
                expected: ExpectedContext::default(),
            })
        })
        .collect();
    let mut guards: Vec<Option<SavedGuard>> =
        std::iter::repeat_with(|| None).take(GUARD_SLOTS).collect();
    let mut active: Vec<(u64, ExpectedContext)> = Vec::new();
    let mut next_token = 0_u64;

    assert_current(&root_expected, "initial root context");

    for (step, operation) in program.operations.iter().take(MAX_OPERATIONS).enumerate() {
        match operation {
            Operation::New { destination } => {
                contexts[context_slot(*destination)] = Some(SavedContext {
                    context: Context::new(),
                    expected: ExpectedContext::default(),
                });
            }
            Operation::SnapshotCurrent { destination } => {
                contexts[context_slot(*destination)] = Some(SavedContext {
                    context: Context::current(),
                    expected: current_expected(&active, &root_expected).clone(),
                });
            }
            Operation::Clone {
                source,
                destination,
            } => {
                let source = contexts[context_slot(*source)].clone();
                contexts[context_slot(*destination)] = source;
            }
            Operation::Clear { slot } => {
                contexts[context_slot(*slot)] = None;
            }
            Operation::WithMarkerA {
                source,
                destination,
                value,
            } => {
                if let Some(mut saved) = contexts[context_slot(*source)].clone() {
                    saved.context = saved.context.with_value(MarkerA(*value));
                    saved.expected.marker_a = Some(*value);
                    contexts[context_slot(*destination)] = Some(saved);
                }
            }
            Operation::WithMarkerB {
                source,
                destination,
                value,
            } => {
                if let Some(mut saved) = contexts[context_slot(*source)].clone() {
                    saved.context = saved.context.with_value(MarkerB(*value));
                    saved.expected.marker_b = Some(*value);
                    contexts[context_slot(*destination)] = Some(saved);
                }
            }
            Operation::WithTelemetrySuppressed {
                source,
                destination,
            } => {
                if let Some(mut saved) = contexts[context_slot(*source)].clone() {
                    saved.context = saved.context.with_telemetry_suppressed();
                    saved.expected.telemetry_suppressed = true;
                    contexts[context_slot(*destination)] = Some(saved);
                }
            }
            Operation::WithRemoteSpanContext {
                source,
                destination,
                trace_id,
                span_id,
                trace_flags,
                is_remote,
                trace_state_value,
            } => {
                if let Some(mut saved) = contexts[context_slot(*source)].clone() {
                    let trace_state = trace_state_value.map_or(TraceState::NONE, |value| {
                        TraceState::from_key_value(vec![("fuzz".to_owned(), value.to_string())])
                            .expect("generated trace state is valid")
                    });
                    let span_context = SpanContext::new(
                        TraceId::from(trace_id | 1),
                        SpanId::from(span_id | 1),
                        TraceFlags::new(*trace_flags),
                        *is_remote,
                        trace_state,
                    );
                    saved.context = saved.context.with_remote_span_context(span_context.clone());
                    saved.expected.span_context = span_context;
                    contexts[context_slot(*destination)] = Some(saved);
                }
            }
            Operation::Attach { context, guard } => {
                if let Some(saved) = contexts[context_slot(*context)].clone() {
                    drop_guard(guard_slot(*guard), &mut guards, &mut active, &root_expected);
                    let token = next_token;
                    next_token += 1;
                    let attached = saved.context.attach();
                    active.push((token, saved.expected));
                    guards[guard_slot(*guard)] = Some(SavedGuard {
                        guard: attached,
                        token,
                    });
                }
            }
            Operation::EnterTelemetrySuppressedScope { guard } => {
                drop_guard(guard_slot(*guard), &mut guards, &mut active, &root_expected);
                let mut expected = current_expected(&active, &root_expected).clone();
                expected.telemetry_suppressed = true;
                let token = next_token;
                next_token += 1;
                let attached = Context::enter_telemetry_suppressed_scope();
                active.push((token, expected));
                guards[guard_slot(*guard)] = Some(SavedGuard {
                    guard: attached,
                    token,
                });
            }
            Operation::DropGuard { guard } => {
                drop_guard(guard_slot(*guard), &mut guards, &mut active, &root_expected);
            }
        }

        let expected = current_expected(&active, &root_expected);
        assert_current(expected, &format!("step {step}: {operation:?}"));
    }

    for slot in 0..guards.len() {
        drop_guard(slot, &mut guards, &mut active, &root_expected);
        assert_current(
            current_expected(&active, &root_expected),
            &format!("cleanup guard slot {slot}"),
        );
    }

    drop(contexts);
    assert_current(&root_expected, "after cleanup");
    drop(root_guard);
    assert_default_context("after fuzz iteration");
}

fn context_slot(slot: u8) -> usize {
    usize::from(slot) % CONTEXT_SLOTS
}

fn guard_slot(slot: u8) -> usize {
    usize::from(slot) % GUARD_SLOTS
}

fn current_expected<'a>(
    active: &'a [(u64, ExpectedContext)],
    root: &'a ExpectedContext,
) -> &'a ExpectedContext {
    active.last().map_or(root, |(_, expected)| expected)
}

fn drop_guard(
    slot: usize,
    guards: &mut [Option<SavedGuard>],
    active: &mut Vec<(u64, ExpectedContext)>,
    root: &ExpectedContext,
) {
    if let Some(saved) = guards[slot].take() {
        if let Some(position) = active.iter().position(|(token, _)| *token == saved.token) {
            active.remove(position);
        }
        drop(saved.guard);
        assert_current(current_expected(active, root), "guard drop");
    }
}

fn assert_default_context(description: &str) {
    Context::map_current(|current| {
        assert_eq!(
            current.get::<MarkerA>(),
            None,
            "MarkerA leaked {description}"
        );
        assert_eq!(
            current.get::<MarkerB>(),
            None,
            "MarkerB leaked {description}"
        );
        assert!(
            !current.is_telemetry_suppressed(),
            "telemetry suppression leaked {description}"
        );
        let span = current.span();
        assert_eq!(
            span.span_context(),
            &SpanContext::NONE,
            "span context leaked {description}"
        );
    });
}

fn assert_current(expected: &ExpectedContext, description: &str) {
    Context::map_current(|current| {
        assert_eq!(
            current.get::<MarkerA>().map(|marker| marker.0),
            expected.marker_a,
            "MarkerA mismatch after {description}"
        );
        assert_eq!(
            current.get::<MarkerB>().map(|marker| marker.0),
            expected.marker_b,
            "MarkerB mismatch after {description}"
        );
        assert_eq!(
            current.is_telemetry_suppressed(),
            expected.telemetry_suppressed,
            "telemetry suppression mismatch after {description}"
        );
        let span = current.span();
        assert_eq!(
            span.span_context(),
            &expected.span_context,
            "span context mismatch after {description}"
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_out_of_order_smoke_test() {
        run(Program {
            operations: vec![
                Operation::WithMarkerA {
                    source: 0,
                    destination: 0,
                    value: 1,
                },
                Operation::WithMarkerA {
                    source: 1,
                    destination: 1,
                    value: 2,
                },
                Operation::Attach {
                    context: 0,
                    guard: 0,
                },
                Operation::Attach {
                    context: 1,
                    guard: 1,
                },
                Operation::DropGuard { guard: 0 },
                Operation::DropGuard { guard: 1 },
            ],
        });
    }
}
