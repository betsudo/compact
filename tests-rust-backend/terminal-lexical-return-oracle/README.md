# Terminal lexical return oracle

`build.rs` generates the fixture from the exact retained
`terminal-lexical-return-schema20-ir.json` using the current backend. The source
is `examples/rust_backend/terminal_lexical_return_oracle.compact`. This avoids
qualifying only a stale checked-in generated implementation after admission
changes.

The existing runtime test retains all 13 TypeScript oracle rows, including
lexical returns, nested calls, witness ordering, assertion failure and zero gas.
The proof smoke adapter `--terminal-lexical-return` binds the retained IR,
generated Rust, complete raw ZKIR and selected binary IR and verifier keys.
It separately verifies fresh Ledger 9 v3 proofs for `two`, `three`, `nested`
and `observed`. Each case funds its deployment and a distinct call proof,
applies under default strict validation, and checks exact unchanged replay
refusal. The initial contract data comes from explicit native prior state.
