use opto_sync_lib::{
    decide, replay_pending, CheckpointDecision, CheckpointLedger, MergeCapability,
    OptimismStrategy, PolicyError, RetryPolicy, WriteEvent, WriteState,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct TraceDocument {
    traces: Vec<Trace>,
}

#[derive(Deserialize)]
struct Trace {
    name: String,
    strategy: OptimismStrategy,
    events: Vec<WriteEvent>,
    states: Vec<WriteState>,
}

#[test]
fn generated_traces_replay_against_the_rust_policy() {
    let document: TraceDocument = serde_json::from_str(include_str!("../formal/traces.v1.json"))
        .expect("parse canonical policy traces");
    for trace in document.traces {
        assert_eq!(trace.states.len(), trace.events.len() + 1, "{}", trace.name);
        let mut state = trace.states[0];
        for (index, event) in trace.events.into_iter().enumerate() {
            state = decide(trace.strategy, state, event)
                .unwrap_or_else(|error| panic!("{} step {index}: {error}", trace.name))
                .next;
            assert_eq!(
                state,
                trace.states[index + 1],
                "{} step {index}",
                trace.name
            );
        }
    }
}

#[test]
fn terminal_states_are_closed_except_for_idempotent_remote_outcomes() {
    let strategies = [
        OptimismStrategy::RemoteConfirmed,
        OptimismStrategy::LocalAcknowledged,
        OptimismStrategy::BackgroundDurable,
    ];
    let terminal = [
        WriteState::Confirmed,
        WriteState::Rejected,
        WriteState::Cancelled,
    ];
    let events = [
        WriteEvent::LocalAccepted,
        WriteEvent::DispatchStarted,
        WriteEvent::ConnectivityLost,
        WriteEvent::ConnectivityRestored,
        WriteEvent::RetryableFailure,
        WriteEvent::RetryElapsed,
        WriteEvent::RetryExhausted,
        WriteEvent::RemoteApplied,
        WriteEvent::RemoteRejected,
        WriteEvent::CancelRequested,
    ];
    for strategy in strategies {
        for state in terminal {
            for event in events {
                let result = decide(strategy, state, event);
                match (state, event) {
                    (WriteState::Confirmed, WriteEvent::RemoteApplied)
                    | (WriteState::Rejected, WriteEvent::RemoteRejected)
                    | (WriteState::Rejected, WriteEvent::RetryExhausted) => {
                        assert_eq!(result.expect("idempotent terminal outcome").next, state)
                    }
                    _ => assert_eq!(result, Err(PolicyError::InvalidTransition)),
                }
            }
        }
    }
}

#[test]
fn retry_delay_is_bounded_and_exhaustion_is_explicit() {
    let policy = RetryPolicy {
        maximum_attempts: 4,
        base_delay_ms: 100,
        maximum_delay_ms: 500,
    };
    assert!(policy.full_jitter_delay_ms(0, u64::MAX).unwrap() <= 100);
    assert!(policy.full_jitter_delay_ms(3, u64::MAX).unwrap() <= 500);
    assert_eq!(
        policy.full_jitter_delay_ms(4, 0),
        Err(PolicyError::RetryExhausted)
    );
}

#[test]
fn checkpoints_are_monotonic_and_operation_ids_are_idempotent() {
    let mut ledger = CheckpointLedger::new("0", 2).unwrap();
    assert_eq!(
        ledger.observe("operation-a", "1").unwrap(),
        CheckpointDecision::Advanced
    );
    assert_eq!(
        ledger.observe("operation-a", "1").unwrap(),
        CheckpointDecision::Duplicate
    );
    assert_eq!(
        ledger.observe("operation-b", "0"),
        Err(PolicyError::CheckpointRegressed)
    );
    assert_eq!(ledger.checkpoint(), "1");
}

struct LastIntentWins;

impl MergeCapability for LastIntentWins {
    type Error = ();

    fn merge(&self, _base: &str, incoming: &str) -> Result<String, Self::Error> {
        Ok(incoming.to_owned())
    }
}

#[test]
fn pending_replay_uses_one_host_supplied_engine_capability() {
    let merged = replay_pending(
        &LastIntentWins,
        r#"{"value":0}"#,
        [r#"{"value":1}"#, r#"{"value":2}"#],
    )
    .unwrap();
    assert_eq!(merged, r#"{"value":2}"#);
}
