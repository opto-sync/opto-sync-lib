# Opto Sync policy-library instructions

- `opto-sync-interfaces` owns wire declarations; pin it immutably and do not
  fork public request, result, connectivity, or telemetry shapes here.
- This repository owns pure lifecycle policy only. It must not open databases,
  sockets, browser stores, mobile services, or background workers.
- Never add `syncer.c` or `syncer.rs` as a direct dependency. Applications
  supply exactly one reviewed merge engine through `MergeCapability`.
- Preserve terminal-state closure, operation-ID idempotence, monotonic
  checkpoints, retry bounds, cancellation, and the committed trace corpus.
- Keep publication disabled until downstream TypeScript, Dart, Rust, and mobile
  adapters replay the same traces from immutable revisions.
