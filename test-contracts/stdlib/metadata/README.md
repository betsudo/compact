# Circuit signature metadata

The Compact programs in this directory retain the circuit-signature cases from
`compiler/test.ss` at pinned mainstream commit
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. `check-signatures.mjs`
ports their JavaScript assertions for exported names, purity, provability,
primitive and curve tags, exact unsigned bounds, struct and enum shape,
nominal aliases, sequence normalization, and opaque types. The P-256 point case uses its original fixture under
`secp256r1/point/upstream-tags`.
Additional local guards distinguish the standard-library `JubjubPoint` from a
user opaque with the same string, preserve an own `__proto__` circuit key,
exclude inherited JavaScript properties from signature lookups, and report an
exported enum name collision before emitting duplicate JavaScript bindings.

Run `node check.mjs <compactc> <built-compact-runtime> <new-output-dir>` from
this directory. The runner compiles each program with `--skip-zk`, binds the
generated modules to the selected runtime, and executes all assertions.

The pinned `declaredInterfaces` export and contract-typed argument signature
are separate open compiler IR and secure-call gaps. This frontend refuses the
original exported contract argument, even when its declaration matches the
compiled contract. The runner checks that refusal.
The original upstream `Adder.compact` supplies only `addTo`; the current
compiler rejects a declaration that also names `peek`. The runner retains and
checks that refusal, then uses `Adder-compatible.compact` with `peek` present
to isolate the exported contract-argument refusal.
