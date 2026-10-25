# Constant vector projection (#674)

The source owner is `examples/rust_backend/constant_vector_projection.compact`.
`check_constant_vector_projection.py` generates Rust and TypeScript using the
selected compiler/frontend, then builds an independent consumer of the unedited
generated crate. Its metadata and lock checks retain repository dependency
identities. Outputs and reports must be absent before a run; failures are kept.

```sh
COMPACTC_SCHEME=/path/to/compactc-scheme \
python3 tools/compact-rust-backend/check_constant_vector_projection.py \
  --compiler /path/to/compactc \
  --output /fresh/vector-projection \
  --report /fresh/vector-projection.json
```

The seven consumer tests exercise independent first/last/singleton values,
non-Copy text and temporary results; ordinary native/recorded and functional
native/recorded stateful execution; owned vector reuse; one witness evaluation;
first witness failure; inactive branches; ordered private outputs; state/effects,
query-summed gas agreement and actual public replay. Three external compile-fail
binaries require E0308 for wrong length/type and E0382 for double execution.
The two `.compact` negatives must fail at frontend bounds before publication.

Backend unit tests independently cover typed projection, original operand and
index binding, bounds/type/scope forgery and preserving tuple-only operations.
The private occurrence registry binds this operation family; it does not claim
whole-plan equivalence or compiler-source authentication.

This gate generates TS but does not execute a pinned JS runtime oracle. It does
not generate keys, prove, verify, prepare/apply a funded transaction, establish
performance, or replace the complete required CI/proof gates. Those acceptance
items remain separately tracked in the bounded #674 plan. The full compiler
workflow retains its existing gates and adds this finite generated API check;
no hosted run has been dispatched by the local implementation.
