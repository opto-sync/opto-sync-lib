# opto-sync-lib

Runtime-light local-first synchronization semantics above Opto Sync's pinned
engine boundary.

## Status

Implemented policy library; publication remains disabled pending differential
trace replay in downstream runtimes. The Rust crate consumes
`opto-sync-interfaces` at immutable commit
`b92b3a2eb43eeb183144521a188ae465a013951e` and deliberately has no dependency
on `syncer.c` or `syncer.rs`.

## Implemented surface

- `decide` is the deterministic state-machine authority for remote-confirmed,
  local-acknowledged, and background-durable writes.
- `RetryPolicy` supplies bounded full-jitter delays from caller-provided
  entropy and makes exhaustion explicit.
- `CheckpointLedger` enforces monotonic checkpoints and bounded operation-ID
  de-duplication.
- `MergeCapability` lets a final application supply exactly one reviewed
  `syncer.c` or `syncer.rs` engine; `replay_pending` never installs another.
- `formal/optimism-strategies.json` is the machine-readable decision table and
  `formal/traces.v1.json` covers success, retry, duplicate outcome,
  cancellation, offline queueing, and reconnect.

## Ownership boundary

This repository will own reusable, engine-agnostic orchestration semantics:

- optimism strategies such as remote-confirmed, local-acknowledged, and
  background-durable writes;
- deterministic stream de-duplication and local/remote observation ordering;
- retry, cancellation, deadline, checkpoint, tombstone, and conflict-result
  policies;
- transport-neutral state machines that can be refined by RxJS, RxDart, Rust,
  Kotlin, Swift, Java, and other client runtimes;
- conformance fixtures that prove equivalent decisions across implementations.

It does **not** own a second merge engine. `syncer.c` and `syncer.rs` remain the
canonical language implementations of the merge semantics. Native, WebAssembly,
and FFI consumers must resolve exactly one reviewed engine revision in a final
composition.

It also does not own IndexedDB, SQLite, PostgreSQL, Supabase, HTTP, WebSocket,
TCP, service-worker, or mobile-background implementations. Those adapters live
in `opto-sync-clients` and consume the policies defined here.

## Required dependency graph

`opto-sync-lib` depends on `opto-sync-interfaces` only. Engine integration is an
explicit host-supplied capability, never an implicit second installation.

## Validation

```sh
python3 scripts/verify_policy.py
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
```

The Rust tests exhaustively close terminal states, replay every committed
trace, prove checkpoint monotonicity and duplicate idempotence, bound retry
delays, and exercise host-supplied merge replay. Publication remains disabled
until TypeScript/RxJS, Dart/RxDart, and mobile lifecycle adapters replay the
same traces in their owning repositories.
