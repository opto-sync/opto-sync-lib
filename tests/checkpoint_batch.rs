use opto_sync_lib::{CheckpointDecision, CheckpointLedger, PolicyError};

#[test]
fn batch_admits_monotonic_operations_in_input_order() {
    let initial = CheckpointLedger::new("0", 3).unwrap();
    let (advanced, decisions) = initial
        .observe_batch([("op-a", "1"), ("op-a", "1"), ("op-b", "2")])
        .unwrap();
    assert_eq!(
        decisions,
        vec![
            CheckpointDecision::Advanced,
            CheckpointDecision::Duplicate,
            CheckpointDecision::Advanced,
        ]
    );
    assert_eq!(advanced.checkpoint(), "2");
    assert_eq!(initial.checkpoint(), "0");

    let (replayed, decisions) = advanced
        .observe_batch([("op-a", "1"), ("op-b", "2")])
        .unwrap();
    assert_eq!(replayed, advanced);
    assert_eq!(decisions, vec![CheckpointDecision::Duplicate; 2]);
}

#[test]
fn invalid_late_item_fails_the_whole_batch_without_publishing_a_partial_ledger() {
    let initial = CheckpointLedger::new("0", 3).unwrap();
    assert_eq!(
        initial.observe_batch([("op-a", "1"), ("op-b", "0")]),
        Err(PolicyError::CheckpointRegressed)
    );
    assert_eq!(
        initial.observe_batch([("op-a", "1"), ("invalid id", "2")]),
        Err(PolicyError::InvalidOperationId)
    );
    assert_eq!(
        initial.observe_batch([("op-a", "1"), ("op-b", "00")]),
        Err(PolicyError::InvalidCheckpoint)
    );
    assert_eq!(initial.checkpoint(), "0");

    let (advanced, decisions) = initial.observe_batch([("op-a", "1")]).unwrap();
    assert_eq!(advanced.checkpoint(), "1");
    assert_eq!(decisions, vec![CheckpointDecision::Advanced]);
}

#[test]
fn window_exhaustion_applies_to_unique_ids_across_batch() {
    let initial = CheckpointLedger::new("0", 2).unwrap();
    assert_eq!(
        initial.observe_batch([("op-a", "1"), ("op-b", "2"), ("op-c", "3")]),
        Err(PolicyError::DeduplicationWindowFull)
    );
    let (advanced, decisions) = initial
        .observe_batch([("op-a", "1"), ("op-b", "2"), ("op-a", "2")])
        .unwrap();
    assert_eq!(advanced.checkpoint(), "2");
    assert_eq!(decisions[2], CheckpointDecision::Duplicate);
}

#[test]
fn empty_batch_is_a_noop_snapshot() {
    let original = CheckpointLedger::new("7", 1).unwrap();
    let (next, decisions) = original
        .observe_batch(std::iter::empty::<(&str, &str)>())
        .unwrap();
    assert_eq!(next, original);
    assert!(decisions.is_empty());
}
