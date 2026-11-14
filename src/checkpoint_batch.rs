//! Pure, all-or-nothing checkpoint admission for bounded batches.
//! A hard observation cap prevents duplicate-only or infinite input iterators
//! from growing the returned decision vector without a resource limit.
use crate::{CheckpointDecision, CheckpointLedger, PolicyError};

/// Maximum total observations in one batch, including duplicates.
/// Matches the external checkpoint-batch CLI admission bound.
pub const MAX_BATCH_OBSERVATIONS: usize = 1_024;

impl CheckpointLedger {
    /// Admit an ordered batch without mutating the existing snapshot.
    ///
    /// Decisions correspond one-for-one to the supplied operation IDs. If any
    /// observation is invalid, regresses the checkpoint, fills the
    /// deduplication window, or exceeds the total observation cap, the entire
    /// batch fails and no partial ledger is returned. Duplicate IDs retain
    /// the semantics of `observe`. The total cap applies to *every* item,
    /// including repeats, so an infinite iterator terminates with an error.
    pub fn observe_batch<'a, I>(
        &self,
        observations: I,
    ) -> Result<(Self, Vec<CheckpointDecision>), PolicyError>
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        observations.into_iter().try_fold(
            (self.clone(), Vec::new()),
            |(ledger, mut decisions), (operation_id, checkpoint)| {
                if decisions.len() >= MAX_BATCH_OBSERVATIONS {
                    return Err(PolicyError::DeduplicationWindowFull);
                }
                let (next, decision) = ledger.observe(operation_id, checkpoint)?;
                decisions.push(decision);
                Ok((next, decisions))
            },
        )
    }
}
