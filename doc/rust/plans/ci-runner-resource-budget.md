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

## Compiler-shell handoff; bounded implementation

The Linux producer now builds the **unchanged** `devShells.x86_64-linux.compiler`
output after the original compiler export. Nixpkgs documents that building a
[`mkShell` output](https://nixos.org/manual/nixpkgs/stable/#sec-pkgs-mkShell)
retains references to its build inputs. A separate same-run artifact carries that
complete requisite closure, with a decimal 10GB NAR/export limit. This avoids
repeating thousands of individual cache requests and small prerequisite builds
on each consumer. It still requires transfer and import on every ephemeral runner;
no reduction in time or total bytes is assumed before Linux measurement.

Only the ten existing job owners that invoke `nix develop .#compiler` import it
(including all three native scenario shards). Compiler-only consumers and the
optional macOS lane keep their original smaller compiler artifact. Every existing
consumer command, source/recipe check, Rust version check, proof, assertion,
negative, package and E2E gate remains unchanged. No flake, dependency or compiler
recipe is changed, and no proof or verification verdict is cached. Forge remains
deferred.

Consumers authenticate the immutable artifact ID, service ZIP digest, source SHA,
system, same run and actual producer attempt. Compiler and shell materials have
distinct names and identities. Before import, local Nix evaluation independently
establishes the exact shell derivation/output. Afterwards, the imported roster,
complete requisite closure, NAR identities and declared deriver must match even
on a warm store. The normal `nix develop` calls still set up and verify the actual
environment; importing prerequisites does not admit any test result.

The local arm64 shell probe found 330 paths and 2,390,370,584 NAR bytes, using
already present dependencies and building only the shell reference output.
At source `e6b48101e68df248531c0ac68702e0edef83875a`, the real helper exported
2,390,492,208 bytes in 36.08 seconds. A new isolated local store imported all 330
paths in 14.20 seconds; complete registry/requisite/NAR rosters matched and all
contents verified in 5.17 seconds. Separately, the original shell passed an offline
host-store probe with Rust/Cargo 1.99.0 in 3.50 seconds. This did not execute the
shell from the isolated store, authenticate a GitHub service artifact, or run any
proof. The first local harness stopped on an unsupported registry-query flag;
that failure and the corrected harness are retained alongside the same export.
These are local transfer observations, **not** a Linux cold-run benchmark or
measured CI savings. The real Linux bundle size and combined producer export/upload,
consumer download/import and subsequent environment setup times must be reported
from CI. Retain the full required gate and compare total assigned-runner minutes,
not only the renamed setup step.

The repeated Nix preparation steps are the first concrete target. Measure full
runner-minutes and elapsed time after a bounded implementation before claiming
savings. Further sharding can reduce elapsed time while increasing total runner
minutes through repeated toolchain/dependency builds; do not treat wall time as
resource cost. Run focused checks locally before publishing a coherent slice.
The owner's explicit Nix setup request is a separate bounded measurement slice
above the existing two PRs; it does not restart either healthy qualification.
Rerun only failed jobs on an unchanged reviewed head when that can resolve an
identified transient problem. Do not repeatedly restart a healthy full run.

Retained API metadata, raw failure logs, job and step timings, PR reopen and
owned failed-run cancellation receipts are under
`~/.codex/local-evidence/rust-ast-actions-resources-20261109/`.
