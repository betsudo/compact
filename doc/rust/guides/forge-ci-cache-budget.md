# Keep Rust CI cache writes within a deliberate scope

The bounded Rust workflow restores dependency downloads and compilation targets
for every eligible run. Only a successful **push to
`refs/heads/codex/rust-backend-ast`** may save either cache, and only when its
restore did not find an exact primary-key hit. Pull requests, manual runs and
other refs remain restore-only. Save steps follow all lane checks, unchanged-lock
accounting and receipt upload; failures or skipped required checks prevent saves.

This preserves the existing OS, architecture, Rust toolchain, lane, features,
lockfile, Cargo/workflow configuration and source-SHA partitions. Saves use the
restore action's resolved primary key. A compatible prefix restore remains
available for compilation targets. Tests, assertions, required-job aggregation
and permissions are unchanged. The split actions use the same reviewed
`actions/cache` v5.0.5 commit as the previous combined action.

## Why this precedes more Forge caching

The owner's observed cache inventory contained about10.05GB of target archives
within11.55GB total, exceeding the stated10GB cache budget. Automatically saving
another large target for every PR head competes with reusable dependency and
future circuit-artifact snapshots. Trusted branch publication reduces future
write amplification while PRs can still restore compatible base-branch caches.

This is **future write suppression**, not a global retention cap or immediate
space reclamation. Existing entries remain under GitHub's retention/eviction
rules; this change deletes nothing, and trusted pushes can still create multiple
large source-keyed targets. Review actual inventory and eviction separately
before assigning a bounded Forge snapshot budget. No hosted time or storage
saving has yet been measured for this change.

Forge artifact snapshots remain separate from Rust target archives. Enabling
Forge in a Linux proof lane still requires exact Linux tool/profile qualification,
verified snapshot restore, real proofs/assertions on warm hits and a separate
uncached key-generation path. This cache-write change does not supply that
qualification or enable Forge by itself.
