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

The owner's observed cache inventory contained about 10.05 GB of target archives
within 11.55 GB total, exceeding the stated 10 GB cache budget. Automatically saving
another large target for every PR head competes with reusable dependency and
future circuit-artifact snapshots. Trusted branch publication reduces future
write amplification. PRs based directly on `codex/rust-backend-ast` can restore
its compatible caches, subject to GitHub's cache scope and key matching rules.

## Cold-build trade-off

GitHub permits restore from the current ref, the default branch, and a PR's
immediate base branch; it does not give a stacked PR access to every ancestor's
caches. The repository's default branch is `main`, not the Rust-AST branch.
A stacked PR based on a non-publishing feature branch can therefore build cold
on each update once its old PR-scoped entries are unavailable. Changing locks
or workflow/configuration hashes can also prevent a compatible target restore.
See [GitHub's cache access rules](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#restrictions-for-accessing-a-cache).

This local candidate trades PR-local persistence for less archive publication.
Cold stacked-PR build time against the 45-minute lane timeout remains unmeasured;
observe it before broad adoption. A save establishes only the current matrix
lane's success, not success of independent jobs or the entire workflow. Cached
compilation outputs never substitute for the ordinary tests or required gate.

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
