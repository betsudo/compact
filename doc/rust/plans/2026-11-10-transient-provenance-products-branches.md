# Transient provenance: products and branches (#612)

Date: 2026-11-10. Parent: signed local `ab9fe143c4456d1fae4bb637dfab91f0f8441641`.
Branch/worktree: `codex/rust-ast-transient-provenance-aggregates`, isolated
from earlier slices and other sessions. No remote publication.

## Scope and policy

Extend the explicit schema 20 typed-IR pass through tuple, vector and struct
literals and their static projections. Each component retains its own known
producer and unknown contribution. Hashing a whole product joins all
components; projecting an unrelated component does not inherit another
component's producer. Branch values join both arms component by component
when their shapes agree. Mismatched or unknown projection shape is unavailable.

Comparison inputs retain known provenance. Conditional selection is treated
conservatively as a control dependency except when both arms are identical
primitive literals. More general equivalence is not proved. A sink inside a branch records the selected lexical arm in
`Finding.conditions`; this is a possible-path diagnostic, not a reachability
proof. The pass tracks the value given to a sink; whether a sink itself runs
under a transient condition is outside this policy. Known transient evidence
survives joins with ordinary unknown inputs.

The expression-node/depth, route, step, finding and unavailable budgets remain
in force. Aggregate traversal stops at the node budget. This slice does not
resolve stateful writes, effectful returns, dynamic collection indexing,
loop/fold bodies or protocol acknowledgements. Absence of a finding does not
certify upgrade stability.

## Evidence

`examples/rust_backend/transient_provenance_aggregates.compact` was compiled
with the previously recorded `compactc`/`compactc-scheme` pair using
`--target rust --skip-zk --emit-rust-ir`; the new schema 20 fixture is
`tests/transient-provenance-aggregates-source-ir.json`. Positive and negative
source cases cover struct, tuple and vector projections, whole-product
hashing, branch data/control and a sink in one branch. No shared fixture was
regenerated. Separate Cargo target and outputs are under
`/Users/ysh/.codex/local-evidence/transient-provenance-aggregates-20261110`;
all Cargo commands use `CARGO_BUILD_JOBS=2`.

Results:

- `cargo check -p compact-rust-backend --lib` passed. The three focused
  provenance test targets passed: nine tests, zero failures. This includes
  source-backed projection hit/miss controls, a wide-product node-budget
  control and the reviewed identical-literal regression.
- Exact-parent and current backends compiled the same source with the same
  Scheme frontend. `cmp` found generated `contract/lib.rs` and
  `rust-capabilities.json` byte-identical. SHA-256 is
  `83f9158c9e6e44cd47ea79f73b74355c72ef1fe7d360534c9de7bd73bbe817b6`
  for Rust and
  `0291f136572c7e0dbc082e14d665527d694891b3f49f1fd3cc358d25f531a524`
  for capabilities. Outputs are `base-output-v2` and `branch-output-v2`
  under the evidence directory.
- Independent read-only review found that identical literal arms initially
  produced a false control-dependency finding. A bounded primitive-literal
  equality check and regression fix it. Re-review found no remaining
  actionable issue in this slice.

The Scheme frontend executable predates the parent; the byte comparison uses
separately built exact-parent and current Rust backends. No runtime/proof
execution or full #612 completion is claimed.
