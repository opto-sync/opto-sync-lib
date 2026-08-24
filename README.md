# opto-sync-lib

Runtime-light local-first synchronization semantics above Opto Sync's pinned
engine boundary.

## Status

Bootstrap repository. Publication is disabled until the interfaces repository
is authoritative and a real resolver produces immutable dependency locks.

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

## First implementation gates

- Import reviewed state-machine specifications with exact provenance and
  preserve their temporal properties.
- Define the optimism-strategy decision table and observable ordering contract.
- Model retries, crashes, reconnects, duplicate delivery, and cancellation with
  bounded formal configurations.
- Run the same generated traces against TypeScript/RxJS, Dart/RxDart, Rust, and
  mobile lifecycle adapters.
- Reject dependency graphs that contain both a Zed-resolved engine and a pinned
  gitlink for the same logical engine revision.
- Keep all publication targets disabled until clean-room consumers prove
  deterministic packaging and runtime behavior.
