//! Engine-neutral scoped checkpoint decisions compatible with proposed AttoLang.
//! Trusted host must prove all scope/commit booleans: no authentication here.
pub const ATTO_SIGNED_MAX: u64 = i64::MAX as u64;

/// Fail closed unless this exact scoped checkpoint is committed.
pub const fn can_resume(
    active_generation: u64,
    cursor_generation: u64,
    local_checkpoint: u64,
    observed_checkpoint: u64,
    scope_verified: bool,
    snapshot_committed: bool,
) -> bool {
    active_generation > 0
        && active_generation <= ATTO_SIGNED_MAX
        && cursor_generation == active_generation
        && local_checkpoint <= ATTO_SIGNED_MAX
        && observed_checkpoint == local_checkpoint
        && scope_verified
        && snapshot_committed
}

/// Filtered projections may skip unrelated global revisions, not predecessors.
pub const fn can_apply_change(
    active_generation: u64,
    observed_generation: u64,
    local_checkpoint: u64,
    previous_checkpoint: u64,
    next_checkpoint: u64,
    scope_verified: bool,
    authoritative: bool,
) -> bool {
    active_generation > 0
        && active_generation <= ATTO_SIGNED_MAX
        && observed_generation == active_generation
        && local_checkpoint <= ATTO_SIGNED_MAX
        && previous_checkpoint == local_checkpoint
        && next_checkpoint > previous_checkpoint
        && next_checkpoint <= ATTO_SIGNED_MAX
        && scope_verified
        && authoritative
}

/// Old generation cursors must be discarded after a fresh committed snapshot.
pub const fn can_rotate(
    old_generation: u64,
    new_generation: u64,
    reset_checkpoint: u64,
    scope_verified: bool,
    new_snapshot_committed: bool,
) -> bool {
    old_generation > 0
        && old_generation < ATTO_SIGNED_MAX
        && new_generation > old_generation
        && new_generation <= ATTO_SIGNED_MAX
        && reset_checkpoint == 0
        && scope_verified
        && new_snapshot_committed
}
