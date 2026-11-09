<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# CI runner resource usage and bounded guard repair

## Measured state, 2026-11-09

PR #721 and draft #722 had no runs while repository Actions were disabled. Once the
owner enabled Actions, each PR was closed/reopened once at its unchanged head.
This created real `pull_request` checks without a new commit or manual-dispatch
run. Manual dispatch does not populate the PR checks section or satisfy required
PR checks; see [GitHub's status-check guidance](https://docs.github.com/en/enterprise-cloud%40latest/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks).

Two retained successful full compiler runs consumed approximately 346 and 347
assigned-runner minutes each while taking about 58 and 55 minutes elapsed:
[42242984695](https://github.com/betsudo/compact/actions/runs/42242984695) and
[42225886955](https://github.com/betsudo/compact/actions/runs/42225886955).
These are sums of job start/end intervals for assigned runners, not authoritative
billing measurements or estimates of savings. The sample is two runs, not the
whole account history. The longest jobs in these samples took about 37–38 minutes.
Nine `Verify consumer toolchain inside Nix` steps in the first sample together
spent about 76 runner-minutes preparing the environment before their main tests.

The repository is public and uses standard runners. GitHub states that standard
runner use is free for public repositories; resource use still matters, and the
exact reason for the repository-wide disablement is not available from the
repository API. A normal hosted job limit cancels a job; these measurements do
not establish why GitHub disabled Actions. Retain the owner's actual notice if
available. See [billing](https://docs.github.com/en/actions/concepts/billing-and-usage)
and [limits](https://docs.github.com/en/actions/reference/limits).

## Concrete repair in PR #722

The new PR run exposed two workflow-control errors: a descriptive step rename
broke name-based test extraction. The tests now locate the equality owner using
its unique stable step ID. The actual shell test includes the added complete
tuple scope command, exact argument/environment/target forwarding, and a scope
failure that must stop execution before subsequent producers. Other literal
shell control tests retain their original bodies.

Resource admission changes preserve all required Linux proof/native/package/E2E
owners and the existing 105-case tuple proof gate:

- Run source/version/header and workflow-handoff controls before the compiler
  producer and workspace Rust jobs allocate expensive runners.
- Admit each expensive consumer only when every declared prerequisite succeeded.
  Preserve its own source/material checks as well. A failed compiler no longer
  causes the entire dependent graph to allocate runners only to reject missing
  material. The always-run final gate still fails if any required owner failed,
  was cancelled or skipped. Nothing becomes optional or `continue-on-error`.
- Give cheap source/header jobs 10-minute caps and the final gate 5 minutes. Existing
  heavy-job caps remain unchanged until the new Linux results provide duration
  evidence; do not cut valid proof cohorts to meet a shorter timer.
- Retain existing per-PR cancellation of superseded runs. Do not create duplicate
  manual runs to repair missing PR checks. PR merge trees and manual head trees
  can differ; they keep distinct concurrency scopes, as do separate PRs with
  the same source branch and different bases. macOS remains a manual opt-in.

The 33 workflow controls exercise exact shell failure propagation, every
prerequisite's success/failure/skipped/cancelled/empty decisions, final required
membership and distinct PR/manual concurrency identities. Actionlint also checks
both changed workflows. These checks establish wiring behavior, not hosted
performance improvements or proof acceptance.

## Next measured improvement; not implemented here

Extend the existing authenticated build-once producer with the exact pinned CI
prerequisites needed by its consumers, including the existing Nix compiler-shell
closure. Keep same-source/system/run/attempt/tool/recipe identities and consumer
verification. Transfer immutable build inputs/libraries only: never cache proof
or verification verdicts, and keep fresh keys/proofs/assertions/negative controls
in every required lane. Forge adoption remains deferred.

The repeated Nix preparation steps are the first concrete target. Measure full
runner-minutes and elapsed time after a bounded implementation before claiming
savings. Further sharding can reduce elapsed time while increasing total runner
minutes through repeated toolchain/dependency builds; do not treat wall time as
resource cost. Keep no more than the existing two published stack slices active,
run focused checks locally before pushing a coherent slice, and rerun only failed
jobs on an unchanged reviewed head when rerunning can resolve an identified
transient problem. Do not repeatedly restart a healthy full qualification.

Retained API metadata, raw failure logs, job and step timings, PR reopen and
owned failed-run cancellation receipts are under
`~/.codex/local-evidence/rust-ast-actions-resources-20261109/`.
