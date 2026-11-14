//! Pure, all-or-nothing checkpoint admission for bounded batches.
use crate::{CheckpointDecision, CheckpointLedger, PolicyError};

impl CheckpointLedger {
    /// Admit an ordered batch without mutating the existing snapshot.
    ///
    /// Decisions correspond one-for-one to the supplied operation IDs. If any
    /// observation is invalid, regresses the checkpoint, or fills the
    /// deduplication window, the entire batch fails and no partial ledger is
    /// returned. Duplicate IDs retain the semantics of `observe`.
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
                let (next, decision) = ledger.observe(operation_id, checkpoint)?;
                decisions.push(decision);
                Ok((next, decisions))
            },
        )
    }
}
