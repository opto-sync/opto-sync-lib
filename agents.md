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

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.
