# Build the compiler once per CI platform

## Status and scope

This is a **local workflow candidate**, not hosted acceptance. Source controls and
mock boundary tests cover orchestration and refusals. Actual closure transfer,
standalone fresh-store import, GitHub partial-rerun output retention and hosted
execution remain unqualified. No build-time saving is claimed: exporting,
uploading, downloading and importing the runtime closure may cost more than the
compiler realization it avoids.

Local feasibility checks on 2026-10-19 used the existing immutable Darwin compiler
package selected by this candidate's Nix evaluation. Its eight-path runtime
closure exported as 87,377,808 bytes in 0.567 seconds. Official import into an
initially empty private local-store root took 0.637 seconds; requisite membership,
all NAR hashes and the declared deriver matched. The current helper's version,
help and fixture smoke also passed against the existing host package. These are
separate checks: executing from the host store does not prove standalone execution
from that private store, and no GitHub upload/download was measured.

## Candidate graph

`Compiler Build` retains its existing PR, manual and scheduled triggers. It calls
one reusable producer for Linux and another for Apple Silicon at the exact
checkout SHA. Linux consumers depend on the Linux producer; Apple consumers and
the optional Forge PR consumer share the Darwin producer. Apple does not run the
full Linux native suite.

The optional Forge PR path predicate retains its previous source paths and adds
the producer, caller and handoff-script dependencies. Documentation-only PRs do
not select it. Forge no longer starts a second independent PR workflow. Its
trusted Rust-AST push and manual Apple/Linux qualification paths retain conditional
same-platform producers. No push trigger is added to `Compiler Build`.

Existing compiler source tests, `compiler/go`, documentation and source-map checks,
default packaging, generated consumers, runtime/workspace tests, Clippy, package
rehearsal, archive consumers, E2E tests and reporting remain. Forge keeps its real
proofs, verification, assertions, corruption refusals, uncached qualification,
pinned tools, parameters and cache rules. A producer result is compiler material,
not a reusable proof or test verdict. Repeated `nix develop` calls in one job
normally reuse an already realized compiler; they are not counted as repeated
compiler builds.

## Material identity and provenance

The producer realizes **`.#compactc`**, not the distinct portable release package.
It runs version/help and a same-source fixture compilation, then uses official
Nix export for the complete runtime closure. The handoff retains sibling Scheme
frontend, native cache helper, packaged Rust runtime/macros and referenced store
paths rather than copying a lone executable.

The manifest binds exact source SHA, platform, `flake.nix` and `flake.lock` hashes,
run ID, actual producer attempt, evaluated derivation/output paths, runtime-closure
membership, canonical NAR hashes and export-stream SHA-256. A self-reported
manifest hash is **not producer provenance**.

Consumers receive the upload service's immutable artifact ID. They fetch that
exact artifact, independently read GitHub artifact-service metadata, require the
same run and expected source/platform/producer-attempt artifact name, and verify
the service ZIP digest before unpacking the two fixed material files. They check
manifest identities against their own checkout and Nix evaluation before import.
After official import, exact requisite membership and NAR identities are checked;
a warm store cannot hide an omitted dependency. Runtime-closure export does not
claim to transfer the derivation's build-time dependency closure.

Named consumers use an explicit initial success/nonempty-output gate even after
a failed or skipped producer. Missing, corrupt or mismatched material fails;
there is no latest-artifact search, implicit rebuild or stale fallback. Consumers
have only read access to repository contents and Actions artifacts; no release
credentials or inherited secrets are added.

## Partial reruns

The producer captures its attempt number in the actual producer step and returns
it alongside the immutable artifact ID. A consumer rerun may reuse an earlier
successful producer from the **same run, source and platform**, provided its
positive attempt is no greater than the current consumer attempt. The artifact
name and manifest remain bound to that original producer attempt; they are not
relabeled with the new consumer attempt. Tests and proofs execute again.

Whether GitHub retains the required job/reusable outputs on every partial-rerun
path has not been qualified. If those outputs are unavailable, the explicit gates
fail closed. They do not guess an artifact or retry a producer.

## Retention and independent workflows

Compiler material artifacts have **one-day retention**. Actions artifact storage
is separate from the repository's stated 10 GB Actions cache budget; this does not
establish a global retention cap or immediately reclaim old caches. Closure size
and transfer costs need measurement before adoption.

Release workflows retain their separate portable profiles and publication rules.
Paused optional debugging/fuzzing workflows remain independent and are not enabled
or migrated by this candidate.

## Explicit Linux native-owner experiment

A separate manual input, `native-forge-experiment`, defaults to false. The normal
`Check and test native Rust backend` step retains its direct
`bash tools/compact-rust-backend/check_native_ci.sh` command. Selecting the input
runs that same owner once through the existing `forge_ci.py` plan/run lifecycle;
it does not run the two-full-lap qualification driver or duplicate the direct
step. Apple gains no full native suite.

The experimental wrapper provisions the existing public d64 Forge source and
independently pinned Rust derivation, preserves a private stable proof runner,
authenticates official ledger fixtures and prepares/verifies all 13 declared SRS
sizes plus built-in material. It runs the original five small finite qualification
laps against the current actual compiler/tools/source/parameters before deriving
any native policy. The extracted admission checks are shared with the unchanged
two-full-lap qualification owner rather than weakening that qualification.

Its capacity experiment explicitly selects cutoff **16**, not compiler default
**8**. The stopped private snapshot is capped at 256 MiB; no Actions cache restore,
save or reusable snapshot publication is added. Full native gates, current
source/tool postchecks and independent uncached two-fixture proof/corruption
qualification remain mandatory. Failures are fatal without a direct fallback.

This integration is local source work only. Current-head Linux hosted
qualification, same-head compiler handoff and wrapped full-owner acceptance remain
pending. Earlier accepted Linux receipts retain their own source identity and
cannot qualify this candidate. Aggregate cache budget/rollout remains on hold;
the per-snapshot cap is not a repository-wide storage limit.

### Focused `linux-owner` manual qualification

The existing Forge qualification workflow adds a manual `linux-owner` selection.
It runs only the same-platform Linux compiler producer/import and Linux consumer;
Apple is not selected. Existing provisioning, authenticated fixtures, all 13 SRS
and current five small finite laps execute once, then the qualified-input mode of
`run_native_ci_forge.py` admits those exact current tools/receipts and runs the
original native owner once. It neither repeats provisioning/finite qualification
nor invokes the two-full-lap driver. `linux-native` retains that original two-lap
qualification independently. Missing qualification inputs or stale/mutated
premises refuse before native plan execution.

This focused route leaves the ordinary Compiler Build license gate unchanged.
Its eventual successful receipt would qualify this finite orchestration, not
remove inherited full-tree header debt or replace ordinary required checks.
Current-head hosted execution and value measurements remain pending. Push routing
now includes producer/caller/handoff scripts so a change to those dependencies
cannot silently miss trusted-branch Forge qualification.

### Linux owner material trial

The explicit `linux-owner` dispatch also consumes original Forge Nix material from
an immutable pinned Forge reusable producer. It does not install the portable
release payload or reinterpret it as the previously qualified Nix executable.
The artifact contains the full original runtime, independent Rust compiler and
source closure, not circuit keys, parameters, proofs or passing test outcomes.

The consumer authenticates the same caller repository/run and original producer
attempt, immutable artifact ID/name and GitHub service ZIP digest before reading
its fixed six-file roster. It checks file sizes/hashes and independently evaluates
public `d64f52db7239b0ca4891fb0316dca6adb54a5db5` package/Rust/source/derivation
identities before importing. Complete NAR/requisite/binary identities and the
existing source/derivation checks remain mandatory. A mismatch fails; the owner
route does not rebuild Forge as a fallback. Other Linux qualification modes keep
their original provisioning path.

Fresh finite consumer qualification still runs after material import. The trial
then measures the balanced complete larger-contract loop and runs the original
native owner once with its independent uncached proof controls. Diagnostic JSON
is retained; no reusable key snapshot is published. Material producer/import,
contract-key reuse, and actual proof execution are separate measured phases.
Source tests alone establish neither runtime handoff success nor useful speedup.

## Explicit one-seed pilot (source candidate)

The manual `native-seed-publish` input defaults to false. Publication is limited
to `linux-owner` on the fixed feature ref
`refs/heads/codex/issue-679-trusted-native-seed`, with an explicit owner-approved
single-writer-window label. PR/wrong-ref/wrong-mode publication and combining
publication with supplied restore identities refuse before expensive producers.
The label records a human operational assumption, not a cryptographic mutex or
a repository-wide quota guarantee. Actual publisher execution requires a concrete
window approval after review; source preparation alone is not that approval.

After the unchanged single native owner, independent uncached proofs/refusals and
live postguards succeed, `prepare_native_seed.py` reuses the existing official
`prepare-save` owner. It binds the exact official primary key, snapshot ID and
partition at `$RUNNER_TEMP/forge-linux-supplied-snapshot`, the same path used by the
pinned cache restore action. No-growth and already-existing identities must not
refresh a seed. Integrity errors remain failures; archive/budget admission denial
records no upload while retaining genuine proof receipts.

The publisher action is source-pinned to Forge
`8e4efdeb2ac905fa73dac7ebd35d82a2119fb24f`; this auxiliary checkout does not
change the qualified d64 Rust payload or the 2614 material producer. Its locked
Node24 dependencies are installed before native source planning. At actual archive
admission and immediately before upload, the action invokes the keyed, fixed-path
Compact `--guard-plan` callback and official Forge report. These read-only checks
revalidate the compiler, Scheme frontend, proof runner, helper, sources and live
parameters without rerunning proofs or recreating the snapshot. The job retains
`actions: read`; the proper Node Action uses the SDK's injected runtime token.

The coordinated Forge Node24 Action uses the pinned official cache SDK to create
and measure the **actual** upload archive before reservation. It audits complete
cache inventory and active/pending writers under the declared window, includes
the exact archive bytes in the 10,000,000,000-byte check and refuses unknown or
uncoordinated state. The existing snapshot payload cap stays 256MiB; payload size
is not substituted for upload archive size. No duplicate `actions/cache/save`,
custom tar transport, cache deletion, automatic generation refresh or new runtime
cryptographic profile is introduced. Only one explicit seed per pilot
ref/partition is permitted; a future refresh needs another owner decision.

This is feature-branch-only transport. A cache scoped to this pilot ref is not
automatically available to ordinary PRs targeting another base. Production PR
restore-only wiring, approved trusted-base publication, current changed-head
publisher/restore qualification and aggregate retention policy remain separate
open work. GitHub controls cache eviction/expiration; there is no invented
per-cache TTL. Compiler artifact one-day retention is unrelated.
