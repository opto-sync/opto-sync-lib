#![forbid(unsafe_code)]
//! Runtime-neutral Opto Sync policy.
//!
//! This crate owns lifecycle decisions, not persistence or transport. Merge
//! behavior is supplied by the host through [`MergeCapability`], ensuring a
//! final composition resolves exactly one `syncer.c` or `syncer.rs` engine.

use std::collections::BTreeSet;

pub use opto_sync_interfaces as interfaces;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptimismStrategy {
    RemoteConfirmed,
    LocalAcknowledged,
    BackgroundDurable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteState {
    Proposed,
    LocallyApplied,
    DurablyQueued,
    InFlight,
    Backoff,
    Confirmed,
    Rejected,
    Cancelled,
}

impl WriteState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Confirmed | Self::Rejected | Self::Cancelled)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteEvent {
    LocalAccepted,
    DispatchStarted,
    ConnectivityLost,
    ConnectivityRestored,
    RetryableFailure,
    RetryElapsed,
    RetryExhausted,
    RemoteApplied,
    RemoteRejected,
    CancelRequested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Observation {
    OptimisticValue,
    DurableAcceptance,
    ConfirmedValue,
    Rejection,
    Cancellation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    PersistAndDispatch,
    PersistForBackgroundDispatch,
    Dispatch,
    WaitForConnectivity,
    ScheduleRetry,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Transition {
    pub next: WriteState,
    pub observation: Option<Observation>,
    pub effect: Option<Effect>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyError {
    InvalidTransition,
    InvalidRetryPolicy,
    RetryExhausted,
    InvalidOperationId,
    InvalidCheckpoint,
    CheckpointRegressed,
    DeduplicationWindowFull,
    MergeFailed,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidTransition => "invalid_transition",
            Self::InvalidRetryPolicy => "invalid_retry_policy",
            Self::RetryExhausted => "retry_exhausted",
            Self::InvalidOperationId => "invalid_operation_id",
            Self::InvalidCheckpoint => "invalid_checkpoint",
            Self::CheckpointRegressed => "checkpoint_regressed",
            Self::DeduplicationWindowFull => "deduplication_window_full",
            Self::MergeFailed => "merge_failed",
        })
    }
}

impl std::error::Error for PolicyError {}

pub fn decide(
    strategy: OptimismStrategy,
    state: WriteState,
    event: WriteEvent,
) -> Result<Transition, PolicyError> {
    use Effect::{
        Dispatch, PersistAndDispatch, PersistForBackgroundDispatch, ScheduleRetry, Stop,
        WaitForConnectivity,
    };
    use Observation::{
        Cancellation, ConfirmedValue, DurableAcceptance, OptimisticValue, Rejection,
    };
    use WriteEvent::{
        CancelRequested, ConnectivityLost, ConnectivityRestored, DispatchStarted, LocalAccepted,
        RemoteApplied, RemoteRejected, RetryElapsed, RetryExhausted, RetryableFailure,
    };
    use WriteState::{
        Backoff, Cancelled, Confirmed, DurablyQueued, InFlight, LocallyApplied, Proposed, Rejected,
    };

    let transition = match (strategy, state, event) {
        (OptimismStrategy::RemoteConfirmed, Proposed, LocalAccepted) => Transition {
            next: InFlight,
            observation: None,
            effect: Some(Dispatch),
        },
        (OptimismStrategy::LocalAcknowledged, Proposed, LocalAccepted) => Transition {
            next: LocallyApplied,
            observation: Some(OptimisticValue),
            effect: Some(PersistAndDispatch),
        },
        (OptimismStrategy::BackgroundDurable, Proposed, LocalAccepted) => Transition {
            next: DurablyQueued,
            observation: Some(DurableAcceptance),
            effect: Some(PersistForBackgroundDispatch),
        },
        (_, LocallyApplied | DurablyQueued, DispatchStarted) => Transition {
            next: InFlight,
            observation: None,
            effect: Some(Dispatch),
        },
        (_, InFlight, ConnectivityLost) => Transition {
            next: DurablyQueued,
            observation: None,
            effect: Some(WaitForConnectivity),
        },
        (_, DurablyQueued, ConnectivityRestored) => Transition {
            next: DurablyQueued,
            observation: None,
            effect: Some(Dispatch),
        },
        (_, InFlight, RetryableFailure) => Transition {
            next: Backoff,
            observation: None,
            effect: Some(ScheduleRetry),
        },
        (_, Backoff, RetryElapsed) => Transition {
            next: InFlight,
            observation: None,
            effect: Some(Dispatch),
        },
        (_, InFlight | Backoff | DurablyQueued, RetryExhausted) => Transition {
            next: Rejected,
            observation: Some(Rejection),
            effect: Some(Stop),
        },
        (_, InFlight, RemoteApplied) => Transition {
            next: Confirmed,
            observation: Some(ConfirmedValue),
            effect: Some(Stop),
        },
        (_, InFlight, RemoteRejected) => Transition {
            next: Rejected,
            observation: Some(Rejection),
            effect: Some(Stop),
        },
        (_, Confirmed, RemoteApplied) => Transition {
            next: Confirmed,
            observation: None,
            effect: None,
        },
        (_, Rejected, RemoteRejected | RetryExhausted) => Transition {
            next: Rejected,
            observation: None,
            effect: None,
        },
        (_, current, CancelRequested) if !current.is_terminal() => Transition {
            next: Cancelled,
            observation: Some(Cancellation),
            effect: Some(Stop),
        },
        _ => return Err(PolicyError::InvalidTransition),
    };
    Ok(transition)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetryPolicy {
    pub maximum_attempts: u32,
    pub base_delay_ms: u64,
    pub maximum_delay_ms: u64,
}

impl RetryPolicy {
    pub const fn is_valid(self) -> bool {
        self.maximum_attempts > 0
            && self.base_delay_ms > 0
            && self.maximum_delay_ms >= self.base_delay_ms
    }

    pub fn full_jitter_delay_ms(self, attempt: u32, entropy: u64) -> Result<u64, PolicyError> {
        if !self.is_valid() {
            return Err(PolicyError::InvalidRetryPolicy);
        }
        if attempt >= self.maximum_attempts {
            return Err(PolicyError::RetryExhausted);
        }
        let exponent = attempt.min(62);
        let ceiling = self
            .base_delay_ms
            .saturating_mul(1_u64 << exponent)
            .min(self.maximum_delay_ms);
        Ok(entropy % ceiling.saturating_add(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointDecision {
    Advanced,
    Duplicate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckpointLedger {
    checkpoint: u64,
    seen_operation_ids: BTreeSet<String>,
    maximum_seen_operations: usize,
}

impl CheckpointLedger {
    pub fn new(checkpoint: &str, maximum_seen_operations: usize) -> Result<Self, PolicyError> {
        let checkpoint = parse_decimal(checkpoint)?;
        if maximum_seen_operations == 0 {
            return Err(PolicyError::DeduplicationWindowFull);
        }
        Ok(Self {
            checkpoint,
            seen_operation_ids: BTreeSet::new(),
            maximum_seen_operations,
        })
    }

    pub fn checkpoint(&self) -> String {
        self.checkpoint.to_string()
    }

    pub fn observe(
        &mut self,
        operation_id: &str,
        checkpoint: &str,
    ) -> Result<CheckpointDecision, PolicyError> {
        validate_operation_id(operation_id)?;
        let checkpoint = parse_decimal(checkpoint)?;
        if self.seen_operation_ids.contains(operation_id) {
            return Ok(CheckpointDecision::Duplicate);
        }
        if checkpoint < self.checkpoint {
            return Err(PolicyError::CheckpointRegressed);
        }
        if self.seen_operation_ids.len() >= self.maximum_seen_operations {
            return Err(PolicyError::DeduplicationWindowFull);
        }
        self.seen_operation_ids.insert(operation_id.to_owned());
        self.checkpoint = checkpoint;
        Ok(CheckpointDecision::Advanced)
    }
}

pub trait MergeCapability {
    type Error;

    fn merge(&self, base: &str, incoming: &str) -> Result<String, Self::Error>;
}

pub fn replay_pending<'a, C, I>(
    capability: &C,
    authoritative: &str,
    pending_oldest_first: I,
) -> Result<String, PolicyError>
where
    C: MergeCapability,
    I: IntoIterator<Item = &'a str>,
{
    let mut view = authoritative.to_owned();
    for pending in pending_oldest_first {
        view = capability
            .merge(&view, pending)
            .map_err(|_| PolicyError::MergeFailed)?;
    }
    Ok(view)
}

fn parse_decimal(value: &str) -> Result<u64, PolicyError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(PolicyError::InvalidCheckpoint);
    }
    value.parse().map_err(|_| PolicyError::InvalidCheckpoint)
}

fn validate_operation_id(value: &str) -> Result<(), PolicyError> {
    if value.is_empty()
        || value.len() > 128
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return Err(PolicyError::InvalidOperationId);
    }
    Ok(())
}
