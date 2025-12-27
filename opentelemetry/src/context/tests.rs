use super::*;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, PartialEq)]
struct ValueA(u64);
#[derive(Debug, PartialEq)]
struct ValueB(u64);

#[test]
fn context_immutable() {
    // start with Current, which should be an empty context
    let cx = Context::current();
    assert_eq!(cx.get::<ValueA>(), None);
    assert_eq!(cx.get::<ValueB>(), None);

    // with_value should return a new context,
    // leaving the original context unchanged
    let cx_new = cx.with_value(ValueA(1));

    // cx should be unchanged
    assert_eq!(cx.get::<ValueA>(), None);
    assert_eq!(cx.get::<ValueB>(), None);

    // cx_new should contain the new value
    assert_eq!(cx_new.get::<ValueA>(), Some(&ValueA(1)));

    // cx_new should be unchanged
    let cx_newer = cx_new.with_value(ValueB(1));

    // Cx and cx_new are unchanged
    assert_eq!(cx.get::<ValueA>(), None);
    assert_eq!(cx.get::<ValueB>(), None);
    assert_eq!(cx_new.get::<ValueA>(), Some(&ValueA(1)));
    assert_eq!(cx_new.get::<ValueB>(), None);

    // cx_newer should contain both values
    assert_eq!(cx_newer.get::<ValueA>(), Some(&ValueA(1)));
    assert_eq!(cx_newer.get::<ValueB>(), Some(&ValueB(1)));
}

#[test]
fn nested_contexts() {
    let _outer_guard = Context::new().with_value(ValueA(1)).attach();

    // Only value `a` is set
    let current = Context::current();
    assert_eq!(current.get(), Some(&ValueA(1)));
    assert_eq!(current.get::<ValueB>(), None);

    {
        let _inner_guard = Context::current_with_value(ValueB(42)).attach();
        // Both values are set in inner context
        let current = Context::current();
        assert_eq!(current.get(), Some(&ValueA(1)));
        assert_eq!(current.get(), Some(&ValueB(42)));

        assert!(Context::map_current(|cx| {
            assert_eq!(cx.get(), Some(&ValueA(1)));
            assert_eq!(cx.get(), Some(&ValueB(42)));
            true
        }));
    }

    // Resets to only value `a` when inner guard is dropped
    let current = Context::current();
    assert_eq!(current.get(), Some(&ValueA(1)));
    assert_eq!(current.get::<ValueB>(), None);

    assert!(Context::map_current(|cx| {
        assert_eq!(cx.get(), Some(&ValueA(1)));
        assert_eq!(cx.get::<ValueB>(), None);
        true
    }));
}

#[test]
fn overlapping_contexts() {
    let outer_guard = Context::new().with_value(ValueA(1)).attach();

    // Only value `a` is set
    let current = Context::current();
    assert_eq!(current.get(), Some(&ValueA(1)));
    assert_eq!(current.get::<ValueB>(), None);

    let inner_guard = Context::current_with_value(ValueB(42)).attach();
    // Both values are set in inner context
    let current = Context::current();
    assert_eq!(current.get(), Some(&ValueA(1)));
    assert_eq!(current.get(), Some(&ValueB(42)));

    assert!(Context::map_current(|cx| {
        assert_eq!(cx.get(), Some(&ValueA(1)));
        assert_eq!(cx.get(), Some(&ValueB(42)));
        true
    }));

    drop(outer_guard);

    // `inner_guard` is still alive so both `ValueA` and `ValueB` should still be accessible
    let current = Context::current();
    assert_eq!(current.get(), Some(&ValueA(1)));
    assert_eq!(current.get(), Some(&ValueB(42)));

    drop(inner_guard);

    // Both guards are dropped and neither value should be accessible.
    let current = Context::current();
    assert_eq!(current.get::<ValueA>(), None);
    assert_eq!(current.get::<ValueB>(), None);
}

#[test]
fn too_many_contexts() {
    let mut guards: Vec<ContextGuard> = Vec::with_capacity(ContextStack::MAX_POS as usize);
    let stack_max_pos = ContextStack::MAX_POS as u64;
    // Fill the stack up until the last position
    for i in 1..stack_max_pos {
        let cx_guard = Context::current().with_value(ValueB(i)).attach();
        assert_eq!(Context::current().get(), Some(&ValueB(i)));
        assert_eq!(cx_guard.cx_pos, i as u16);
        guards.push(cx_guard);
    }
    // Let's overflow the stack a couple of times
    for _ in 0..16 {
        let cx_guard = Context::current().with_value(ValueA(1)).attach();
        assert_eq!(cx_guard.cx_pos, ContextStack::MAX_POS);
        assert_eq!(Context::current().get::<ValueA>(), None);
        assert_eq!(Context::current().get(), Some(&ValueB(stack_max_pos - 1)));
        guards.push(cx_guard);
    }
    // Drop the overflow contexts
    for _ in 0..16 {
        guards.pop();
        assert_eq!(Context::current().get::<ValueA>(), None);
        assert_eq!(Context::current().get(), Some(&ValueB(stack_max_pos - 1)));
    }
    // Drop one more so we can add a new one
    guards.pop();
    assert_eq!(Context::current().get::<ValueA>(), None);
    assert_eq!(Context::current().get(), Some(&ValueB(stack_max_pos - 2)));
    // Push a new context and see that it works
    let cx_guard = Context::current().with_value(ValueA(2)).attach();
    assert_eq!(cx_guard.cx_pos, ContextStack::MAX_POS - 1);
    assert_eq!(Context::current().get(), Some(&ValueA(2)));
    assert_eq!(Context::current().get(), Some(&ValueB(stack_max_pos - 2)));
    guards.push(cx_guard);
    // Let's overflow the stack a couple of times again
    for _ in 0..16 {
        let cx_guard = Context::current().with_value(ValueA(1)).attach();
        assert_eq!(cx_guard.cx_pos, ContextStack::MAX_POS);
        assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(2)));
        assert_eq!(Context::current().get(), Some(&ValueB(stack_max_pos - 2)));
        guards.push(cx_guard);
    }
}

/// Tests that a new ContextStack is created with the correct initial capacity.
#[test]
fn test_initial_capacity() {
    let stack = ContextStack::default();
    assert_eq!(stack.stack.capacity(), ContextStack::INITIAL_CAPACITY);
}

/// Tests that map_current_cx correctly accesses the current context.
#[test]
fn test_map_current_cx() {
    let mut stack = ContextStack::default();
    let test_value = ValueA(42);
    stack.current_cx = Context::new().with_value(test_value);

    let result = stack.map_current_cx(|cx| {
        assert_eq!(cx.get::<ValueA>(), Some(&ValueA(42)));
        true
    });
    assert!(result);
}

/// Tests popping contexts in non-sequential order.
#[test]
fn test_pop_id_out_of_order() {
    let mut stack = ContextStack::default();

    // Push three contexts
    let cx1 = Context::new().with_value(ValueA(1));
    let cx2 = Context::new().with_value(ValueA(2));
    let cx3 = Context::new().with_value(ValueA(3));

    let id1 = stack.push(cx1);
    let id2 = stack.push(cx2);
    let id3 = stack.push(cx3);

    // Pop middle context first - should not affect current context
    stack.pop_id(id2);
    assert_eq!(stack.current_cx.get::<ValueA>(), Some(&ValueA(3)));
    assert_eq!(stack.stack.len(), 3); // Length unchanged for middle pops

    // Pop last context - should restore previous valid context
    stack.pop_id(id3);
    assert_eq!(stack.current_cx.get::<ValueA>(), Some(&ValueA(1)));
    assert_eq!(stack.stack.len(), 1);

    // Pop first context - should restore to empty state
    stack.pop_id(id1);
    assert_eq!(stack.current_cx.get::<ValueA>(), None);
    assert_eq!(stack.stack.len(), 0);
}

/// Tests edge cases in context stack operations. IRL these should log
/// warnings, and definitely not panic.
#[test]
fn test_pop_id_edge_cases() {
    let mut stack = ContextStack::default();

    // Test popping BASE_POS - should be no-op
    stack.pop_id(ContextStack::BASE_POS);
    assert_eq!(stack.stack.len(), 0);

    // Test popping MAX_POS - should be no-op
    stack.pop_id(ContextStack::MAX_POS);
    assert_eq!(stack.stack.len(), 0);

    // Test popping invalid position - should be no-op
    stack.pop_id(1000);
    assert_eq!(stack.stack.len(), 0);

    // Test popping from empty stack - should be safe
    stack.pop_id(1);
    assert_eq!(stack.stack.len(), 0);
}

/// Tests stack behavior when reaching maximum capacity.
/// Once we push beyond this point, we should end up with a context
/// that points _somewhere_, but mutating it should not affect the current
/// active context.
#[test]
fn test_push_overflow() {
    let mut stack = ContextStack::default();
    let max_pos = ContextStack::MAX_POS as usize;

    // Fill stack up to max position
    for i in 0..max_pos {
        let cx = Context::new().with_value(ValueA(i as u64));
        let id = stack.push(cx);
        assert_eq!(id, (i + 1) as u16);
    }

    // Try to push beyond capacity
    let cx = Context::new().with_value(ValueA(max_pos as u64));
    let id = stack.push(cx);
    assert_eq!(id, ContextStack::MAX_POS);

    // Verify current context remains unchanged after overflow
    assert_eq!(
        stack.current_cx.get::<ValueA>(),
        Some(&ValueA((max_pos - 2) as u64))
    );
}

/// Tests that:
/// 1. Parent context values are properly propagated to async operations
/// 2. Values added during async operations do not affect parent context
#[tokio::test]
async fn test_async_context_propagation() {
    // A nested async operation we'll use to test propagation
    async fn nested_operation() {
        // Verify we can see the parent context's value
        assert_eq!(
            Context::current().get::<ValueA>(),
            Some(&ValueA(42)),
            "Parent context value should be available in async operation"
        );

        // Create new context
        let cx_with_both = Context::current()
            .with_value(ValueA(43)) // override ValueA
            .with_value(ValueB(24)); // Add new ValueB

        // Run nested async operation with both values
        FutureContextExt::with_context(
        async {
            // Verify both values are available
            assert_eq!(
                Context::current().get::<ValueA>(),
                Some(&ValueA(43)),
                "Parent value should still be available after adding new value"
            );
            assert_eq!(
                Context::current().get::<ValueB>(),
                Some(&ValueB(24)),
                "New value should be available in async operation"
            );

            // Do some async work to simulate real-world scenario
            sleep(Duration::from_millis(10)).await;

            // Values should still be available after async work
            assert_eq!(
                Context::current().get::<ValueA>(),
                Some(&ValueA(43)),
                "Parent value should persist across await points"
            );
            assert_eq!(
                Context::current().get::<ValueB>(),
                Some(&ValueB(24)),
                "New value should persist across await points"
            );
        }
        , cx_with_both)
        .await;
    }

    // Set up initial context with ValueA
    let parent_cx = Context::new().with_value(ValueA(42));

    // Create and run async operation with the parent context explicitly propagated
    FutureContextExt::with_context(nested_operation(), parent_cx.clone()).await;

    // After async operation completes:
    // 1. Parent context should be unchanged
    assert_eq!(
        parent_cx.get::<ValueA>(),
        Some(&ValueA(42)),
        "Parent context should be unchanged"
    );
    assert_eq!(
        parent_cx.get::<ValueB>(),
        None,
        "Parent context should not see values added in async operation"
    );

    // 2. Current context should be back to default
    assert_eq!(
        Context::current().get::<ValueA>(),
        None,
        "Current context should be back to default"
    );
    assert_eq!(
        Context::current().get::<ValueB>(),
        None,
        "Current context should not have async operation's values"
    );
}

///
/// Tests that unnatural parent->child relationships in nested async
/// operations behave properly.
///
#[tokio::test]
async fn test_out_of_order_context_detachment_futures() {
    // This function returns a future, but doesn't await it
    // It will complete before the future that it creates.
    async fn create_a_future() -> impl std::future::Future<Output = ()> {
        // Create a future that will do some work, referencing our current
        // context, but don't await it.
        FutureContextExt::with_context(async {
            assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(42)));

            // Longer work
            sleep(Duration::from_millis(50)).await;
        },
        Context::current())
    }

    // Create our base context
    let parent_cx = Context::new().with_value(ValueA(42));

    // await our nested function, which will create and detach a context
    let future = FutureContextExt::with_context(create_a_future(), parent_cx).await;

    // Execute the future. The future that created it is long gone, but this shouldn't
    // cause issues.
    let _a = future.await;

    // Nothing terrible (e.g., panics!) should happen, and we should definitely not have any
    // values attached to our current context that were set in the nested operations.
    assert_eq!(Context::current().get::<ValueA>(), None);
    assert_eq!(Context::current().get::<ValueB>(), None);
}

#[test]
fn test_is_telemetry_suppressed() {
    // Default context has suppression disabled
    let cx = Context::new();
    assert!(!cx.is_telemetry_suppressed());

    // With suppression enabled
    let suppressed = cx.with_telemetry_suppressed();
    assert!(suppressed.is_telemetry_suppressed());
}

#[test]
fn test_with_telemetry_suppressed() {
    // Start with a normal context
    let cx = Context::new();
    assert!(!cx.is_telemetry_suppressed());

    // Create a suppressed context
    let suppressed = cx.with_telemetry_suppressed();

    // Original should remain unchanged
    assert!(!cx.is_telemetry_suppressed());

    // New context should be suppressed
    assert!(suppressed.is_telemetry_suppressed());

    // Test with values to ensure they're preserved
    let cx_with_value = cx.with_value(ValueA(42));
    let suppressed_with_value = cx_with_value.with_telemetry_suppressed();

    assert!(!cx_with_value.is_telemetry_suppressed());
    assert!(suppressed_with_value.is_telemetry_suppressed());
    assert_eq!(suppressed_with_value.get::<ValueA>(), Some(&ValueA(42)));
}

#[test]
fn test_enter_telemetry_suppressed_scope() {
    // Ensure we start with a clean context
    let _reset_guard = Context::new().attach();

    // Default context should not be suppressed
    assert!(!Context::is_current_telemetry_suppressed());

    // Add an entry to the current context
    let cx_with_value = Context::current().with_value(ValueA(42));
    let _guard_with_value = cx_with_value.attach();

    // Verify the entry is present and context is not suppressed
    assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(42)));
    assert!(!Context::is_current_telemetry_suppressed());

    // Enter a suppressed scope
    {
        let _guard = Context::enter_telemetry_suppressed_scope();

        // Verify suppression is active and the entry is still present
        assert!(Context::is_current_telemetry_suppressed());
        assert!(Context::current().is_telemetry_suppressed());
        assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(42)));
    }

    // After guard is dropped, should be back to unsuppressed and entry should still be present
    assert!(!Context::is_current_telemetry_suppressed());
    assert!(!Context::current().is_telemetry_suppressed());
    assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(42)));
}

#[test]
fn test_nested_suppression_scopes() {
    // Ensure we start with a clean context
    let _reset_guard = Context::new().attach();

    // Default context should not be suppressed
    assert!(!Context::is_current_telemetry_suppressed());

    // First level suppression
    {
        let _outer = Context::enter_telemetry_suppressed_scope();
        assert!(Context::is_current_telemetry_suppressed());

        // Second level. This component is unaware of Suppression,
        // and just attaches a new context. Since it is from current,
        // it'll already have suppression enabled.
        {
            let _inner = Context::current().with_value(ValueA(1)).attach();
            assert!(Context::is_current_telemetry_suppressed());
            assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(1)));
        }

        // Another scenario. This component is unaware of Suppression,
        // and just attaches a new context, not from Current. Since it is
        // not from current it will not have suppression enabled.
        {
            let _inner = Context::new().with_value(ValueA(1)).attach();
            assert!(!Context::is_current_telemetry_suppressed());
            assert_eq!(Context::current().get::<ValueA>(), Some(&ValueA(1)));
        }

        // Still suppressed after inner scope
        assert!(Context::is_current_telemetry_suppressed());
    }

    // Back to unsuppressed
    assert!(!Context::is_current_telemetry_suppressed());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_async_suppression() {
    async fn nested_operation() {
        assert!(Context::is_current_telemetry_suppressed());

        let cx_with_additional_value = Context::current().with_value(ValueB(24));

        FutureContextExt::with_context(async {
            assert_eq!(
                Context::current().get::<ValueB>(),
                Some(&ValueB(24)),
                "Parent value should still be available after adding new value"
            );
            assert!(Context::is_current_telemetry_suppressed());

            // Do some async work to simulate real-world scenario
            sleep(Duration::from_millis(10)).await;

            // Values should still be available after async work
            assert_eq!(
                Context::current().get::<ValueB>(),
                Some(&ValueB(24)),
                "Parent value should still be available after adding new value"
            );
            assert!(Context::is_current_telemetry_suppressed());
        },
        cx_with_additional_value)
        .await;
    }

    // Set up suppressed context, but don't attach it to current
    let suppressed_parent = Context::new().with_telemetry_suppressed();
    // Current should not be suppressed as we haven't attached it
    assert!(!Context::is_current_telemetry_suppressed());

    // Create and run async operation with the suppressed context explicitly propagated
    FutureContextExt::with_context(nested_operation(),
        suppressed_parent.clone())
        .await;

    // After async operation completes:
    // Suppression should be active
    assert!(suppressed_parent.is_telemetry_suppressed());

    // Current should still be not suppressed
    assert!(!Context::is_current_telemetry_suppressed());
}
