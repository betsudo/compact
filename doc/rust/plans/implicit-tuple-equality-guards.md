<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Preserve implicit tuple widening in constructor equality guards

## Demonstrated gap and baseline

This continuation of #720 starts at signed documentation head `91179e374`,
preserving the accepted execution head `7f690b7a6`, the canonical development
line, all unpublished slices and their source, failure and proof evidence.
The clean baseline passes 66 constructor unit tests, 14 constructor renderer
tests, 49 coercion-owner tests and strict library Clippy on macOS arm64.

Seven complete genuine sources were characterized with original compiler
0.31.209: flat equality and inequality, nested tuple equality, a mixed literal
and variable tuple, reversed equality operands, asymmetric tuple element widths,
and a captured Field Vector reused across iterations and two loops. Every source
generates raw IR and TypeScript; every Rust generation is refused at tuple
typing. No SDK execution or cryptographic result is claimed for those probes.
The original inputs stay Uint<8>; comparison context requires Uint<16> in one
or more tuple positions. Literal inference already retains its expected bound.
The historical source and raw IR must remain byte exact compatibility fixtures.

## Owners and intended API

- `compiler/rust-ir-passes.ss`, `typed-expression-ir-value`: investigate and
  retain the original analyzed component source type passed through tuple casts
  to variable references, only under the scoped constructor-condition parameter
  introduced in 209. The constructor has no callable variable-type table;
  ignoring this existing source annotation loses the needed widening.
- Existing `pure_semantics` coercion planning independently validates every
  tuple element and unsigned bound; `pure_native` and `value_lowering` render
  its checked facts through structured Rust AST. Do not relax that relation.
- Constructor condition admission and `constructor_vm` retain infallibility,
  both-arm validation, original-owner diagnostics and left-before-right order.
- Existing source/API, TypeScript, model and proof owners retain exact compiler,
  source, raw IR, generated SDK, parameter and immutable verifier identities.

Generated `initial_state(ConstructorContext, ...)` keeps the exact Compact
parameter types, including borrowed caller-owned Vectors and typed tuples.
No public API, runtime ABI, IR schema, dependency or recording operation changes.
Local branch inventory identifies 0.31.210 as unused among the inspected local
heads; this is a source identifier, not a release or global availability claim.

## Bounded implementation and acceptance

1. Preserve complete 209 source/raw IR refusals and add a compatibility test
   that still rejects their mismatched tuple types. Current compiler output is
   tested independently; never patch generated IR or accept either outcome.
2. Add the minimal constructor-condition tuple context transport through the
   existing Coerce node. Nested tuple/Vector elements reuse the recursive checked
   coercion owner. Preserve scalar, literal, ordinary circuit and initializer
   behavior, fallible operand refusals and narrowing rejection.
3. Require all seven complete sources to generate current raw IR, TypeScript,
   ordinary and functional Rust SDKs, with original source-origin ownership.
   Extend the maintained scope gate to the new exact tuple shape; keep the old
   209 refusal explicitly historical, not as a current success criterion.
4. Compare actual generated initial states against an independent live TS
   oracle. Cover both operators, both operand orders, zero/255, wider right
   values 256/65535, differences in each nested/component position, literal
   inference, asymmetric widths and owned Vector capture/reuse across loops.
   Include marked wrong-width/shape ordinary compile failures and valid twins.
5. Exercise malformed private IR: component narrowing, tuple shape mismatch,
   invalid inactive arms and the first original failing coercion owner. Run
   focused Rust/Python checks while iterating; retain full integration gates.
6. Before keys, observe every export of every intact contract under automatic
   k<=8. Then run applicable fresh selected later-entrypoint proofs with actual
   verification, exact binding negatives, and separately proved DEFAULT-strict
   transaction application/replay. Preserve all failures; no trimming or forced
   k. These proofs do not establish constructor/deployment cryptography.
7. Obtain fresh Claude CLI source/caller and actual-result reviews, retain their
   limits, and record meaningful progress in the existing midnight knowledge
   base and CAS checkpoint. Commit locally with GPG, DCO and Assisted-by trailers.

## Exclusions and current state

No top-level constructor if, general arithmetic/fallible guard admission, new
coercion framework, accepted-capability reopening, scalar-triple model workaround,
CI optimization, publication, hosted trial or Forge work. Linux/full required CI
remains an integration gate. Do not infer proof or API acceptance from generation.

The initial210 candidate built correctly, but all seven Rust generations still
refused: preserving an outer safe-cast wrapper did not address the actual lost
variable source-type annotation. That ineffective change is removed in a
forward repair, retaining the failed epoch and complete outputs. The repair uses
the existing per-component source annotation under the scoped condition flag.
Its current original compiler, generated API, model, proof and application
acceptance remain open. The initial focused test run also caught a borrow/move
error in the new narrowing test; its expected diagnostic now clones the type
already borrowed by the parameter fixture. Stop new slices at
2026-11-08 09:05 UTC and preserve an exact continuation handoff.

## Current characterization and review follow-up

Original210 at `458cc6231` generates raw IR, TypeScript and Rust for all seven
complete sources (21 commands). This is generation evidence only. All 68
constructor unit tests pass. The historical renderer test initially overlooked
the original source-location wrapper; it is strengthened to require the exact
location and complete expected/actual tuple types. A separate Python control
authenticates all seven full source/raw hashes and intact three-export rosters.

Fresh Claude source review found no traced unsafe acceptance, but identified
two P2 coverage gaps: the existing analyzed-source fallback also repairs direct
aggregate variables and other parents, and the seven original probes lack a
maintained exact-condition/component-origin generation gate. Both findings
remain acceptance work. The external 50-case API draft is not an executed gate.
Do not restrict legitimate affected shapes merely to avoid their validation.

Eleven supplemental complete sources now have actual original209 and
candidate458cc210 comparisons, preserved under `supplemental/`. Direct tuple
and Vector variables, both tuple/Vector conversion directions, Uint-to-Field
tuple components, widened Vector capture and aliased Vector variables gain Rust
generation. A tuple hash-argument source already generates with209 and remains
a regression control. Numeric-alias tuple components still stop at checked tuple
typing; aliased tuple variables stop earlier at the Scheme cast owner. These
are remaining valid-Compact transport limitations, not invalid-language rules.

The struct-literal guard source exposes a real existing renderer panic in BOTH
209 and458cc210, after successful frontend/raw generation. Repair this concrete
baseline defect before dependent feature qualification: construct the existing
constructor `if` from a fixed AST skeleton and insert the checked condition AST,
following the existing Fold owner. This lets the AST printer retain required
grouping around struct-literal comparisons without parsing ambiguous condition
tokens. Preserve condition/arm validation and counter/error order. Unit controls
must round-trip both operators and with/without else; generation and native
compilation of the complete original source must confirm the fix.

The next maintained gate must include exact current conditions and component
source origins for all affected supported shapes, actual ordinary/functional
API behavior against live TS, static misuses, ownership and existing refusals.
Qualify every intact export before selected proofs; no source/model trimming.
The two compiler epochs and all failed attempts remain historical, not current
native/proof or integration receipts. Linux/full CI and the broader goal remain
open; Forge remains deferred.

At signed repair head `64ec7c326`, the focused checks pass 69 constructor,
15 renderer and 51 coercion tests plus strict library Clippy. Its rebuilt
original210 compiler independently runs all 18 complete repository sources
(54 generation commands): 16 Rust successes and the same two explicit alias
transport refusals, with TS success for all 18. The original struct source now
generates without the previously observed panic. This still does not prove
generated Rust compilation, actual constructor behavior or cryptography.

The maintained tuple scope gate now has explicit expected parameter/loop,
condition and original source-origin records for all 18 sources, and separate
wrapper/value ranges for each conversion. Aggregate cast wrappers can cover
the whole original comparison while their values keep the variable use-site;
do not force those original ranges to be equal. Source/HOST preparation checks
pass on the completed64ec outputs: 17 raw-success validators and eight mutated
copies refused without compiler/runtime execution. The new current-head CLI,
original recipe, expanded API/native cohort and fresh review are separate gates.
