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

## Aggregate adoption decision — 2026-10-19

Use **10,000,000,000 bytes** as the owner's total Actions cache budget. A complete
paginated, read-only API inventory at **2026-10-19 14:03:47 UTC** contained 45
entries totaling **11,075,071,944 bytes**, exceeding that budget by
**1,075,071,944 bytes**. Entries are counted separately by their actual cache
identity, including ref and version; equal key text does not imply one archive.

| Observed cache family | Entries | Stored bytes |
| --- | ---: | ---: |
| Bounded Rust compilation targets | 28 | 9,691,925,656 |
| Bounded Rust dependency downloads | 8 | 920,075,559 |
| Forge-lane downloads and public parameters | 8 | 461,886,783 |
| Existing stopped Forge snapshot | 1 | 1,183,946 |
| **Total** | **45** | **11,075,071,944** |

This is a dated observation, not a reservation or an enforced quota. The current
pressure is mainly compilation targets. It does not establish that a cloud Forge
service is necessary.

### What the local adoption candidate permits

- Keep new reusable Linux Forge-key publication disabled. Job-private key reuse,
  verified restores and genuine proof qualification can be measured separately.
- Preserve the **268,435,456-byte (256 MiB)** stopped-snapshot limit. It is a
  single-snapshot payload limit, not a repository-wide archive-retention limit.
- Preserve the full measured 80-bundle corpus result: **710,372,357 bytes** does
  not fit that limit. Do not truncate the corpus, silently raise the cap or claim
  that compressing it makes it fit. The separate experimental cutoff16 policy
  explicitly selects a different cache-eligible corpus; its complete native
  receipts and actual snapshot must be accepted before any Linux admission.
- Do not publish an additional cache archive when the complete retained inventory
  already exceeds the budget. A full budget means **skip publication**, with a
  recorded reason; compilation and every proof/test/assertion still execute.
  Ordinary cache misses retain their existing key-generation behavior. Integrity
  failures retain their existing refusal behavior.
- Preserve existing entries. This local decision authorizes no deletion, quota
  change, workflow activation or change to remote cache writers.

The current published workflows do **not** enforce this aggregate decision.
Trusted-branch-only Rust saves suppress some future writes, but even trusted
pushes can retain multiple large source-keyed targets. Existing Forge Apple
download/parameter cache actions can also save implicitly after successful jobs.

### Writers that must participate before claiming enforcement

| Workflow | Save-capable material |
| --- | --- |
| `forge-local-ci.yml` | Cargo downloads, public parameters, stopped Forge snapshots |
| `rust-backend-checks.yml` | Dependency downloads and compiled targets in each matrix lane |
| `compact-test.yml` | Downloads, targets, historical installer and self-update fixtures |
| `ci-specification.yml` | Cabal packages, store, binaries and build outputs |

Their current workflow/ref concurrency groups do not form one publication lock.
Older branch workflows, optional workflows if enabled, and manual API clients
can also bypass any new publisher. No global guarantee is valid until those
writers are coordinated or covered by an explicit operational freeze.

For a future coordinated publisher, serialize **complete inventory, admission,
actual upload and final accounting** across all participating writers. Require
`retained_bytes + candidate_archive_upper_bound <= 10,000,000,000` before upload.
The candidate bound must cover the actual official cache archive, including its
overhead; neither payload bytes nor directory size is sufficient. Unknown size,
incomplete pagination, service errors or uncoordinated writers prevent admission.
An already-existing immutable exact cache identity is a no-publication result;
a compatible prefix restore is not an exact-key hit. Keep event/ref authorization,
source/tool/parameter/verifier identity and all original successful test gates.

This is the explicit retention/admission design, **not an implemented global
publisher**. Do not add a competing archive/transport format merely to implement
it. Reuse the supported Forge snapshot lifecycle and official cache uploader if
the coordinated-writer prerequisite is approved and implemented.

### Compiler material is a separate handoff

The [build-once compiler candidate](compiler-build-once-ci.md) uses same-run
Actions artifacts with one-day retention, not reusable Actions cache entries.
Its complete runtime closure, source identity and consumers still require their
own qualification and transfer measurements. Moving compiler material through
artifacts neither reclaims the cache excess above nor establishes a total storage
cap. No circuit keys, proofs or passing test verdicts are included in that handoff.
