# Transient provenance: list and Merkle state sinks (#612)

Date: 2026-11-10. Parent: GPG-signed local `db2153a47` (symbolic caller
dependencies). Branch/worktree: `codex/rust-ast-transient-provenance-state-collections`
in a separate checkout. No remote publication.

## Scope

The explicitly invoked typed-IR pass now visits the remaining indexed Merkle
hash and default-position action inputs. For `insertHashIndex`, the hash and
position are separate sink inputs; for `insertIndexDefault`, the position is
the only dynamic sink input. Existing list, Merkle value, and Merkle hash
handlers now have genuine source-backed positives. Plain list/value and
default-position writes provide ordinary-input controls.
Both ordinary and historic tree forms are covered. A transient-controlled
branch into an ordinary list or Merkle write is reported as control evidence,
separate from sink-data evidence.

## Source and evidence

`examples/rust_backend/transient_provenance_state_collections.compact` was
compiled by the pinned pre-existing `compactc`/`compactc-scheme` pair with
`--target rust --skip-zk --emit-rust-ir`. The new retained schema 20 fixture is
`tools/compact-rust-backend/tests/transient-provenance-state-collections-source-ir.json`
(SHA-256 `ac82441910741c59ec721626505ceaf3bdea915231fb698216c692ce475d95a8`).
No shared fixture was regenerated. Four new tests check nine exact positive
data sink inputs, separate branch control at list/Merkle writes, no positive
finding for ordinary Field values and ordinary default positions, conversion
steps at hash inputs, source locations, and deterministic reports. A focused
synthetic schema-20 IR analyzer test also drives transient-derived `position`
inputs in both indexed hash and indexed default actions, for ordinary and
historic trees, and asserts distinct position findings.

All 16 provenance integration-test targets passed: 49 tests, zero failures,
with `CARGO_BUILD_JOBS=2` and a separate target under
`/Users/ysh/.codex/local-evidence/transient-provenance-state-collections-20261110`.
The full focused run is retained as `provenance-tests.log` there. Exact-parent
and current backends compiled the same source with the same Scheme frontend;
`cmp` found generated `contract/lib.rs` and `contract/rust-capabilities.json`
byte-identical. SHA-256: Rust
`85cb1086f82115e21e6d975edf0c5d271b2e74b0fd836577ce3e911bae50f3c3`;
capabilities
`8fa21e4ce6c3ff54d1c50ed439cbc2ba3a069fdd84520610371497a3a090f8c0`.
The parent/current outputs are retained in the evidence directory.
Independent read-only review identified the missing positive test for
position inputs; the four synthetic analyzer regressions close that gap.
Final re-review found no remaining bounded-slice blocker or evidence overclaim.

## Limits

The current Rust source compiler rejected the attempted Field-to-`Uint<64>`
downcast, as retained in `field-to-uint-probe.log` in the evidence directory.
This corpus does not supply a genuine source case with a transient-derived
Merkle position. It exercises ordinary positions and the distinct indexed
action variants; the known-position positive is a synthetic schema-20 IR
analyzer regression and makes no source-reachability claim. Other write forms,
constructor effects and
cross-transaction state origins remain outside this analysis or unavailable.
No native/recorded result or FAB-byte equivalence, proof, broad suite, or
Forge claim is made.
