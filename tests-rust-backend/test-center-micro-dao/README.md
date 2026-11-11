# Original microDAO, current generated Rust

`build.rs` generates the complete original `test-center/test-contracts/micro-dao.compact`
from `micro-dao-source-schema20-ir.json` with the current backend. All seven public
recording surfaces must remain available. The older `token-query-schema20-ir.json`
is an unchanged dependency projection (2 pure and 1 stateful circuit versus the
complete source's 22 pure and 17 stateful circuits); it cannot substitute for the
complete source in the token proof adapter.

The `token_query` test compares retained TypeScript observations, native execution,
current recording, official token derivation, public transcript and independent
replay gas across the retained addresses. Other existing tests keep their original
scope. To run the token qualification:

```sh
cargo test -p compact-rust-test-center-micro-dao-fixture --all-features --test token_query
```

Use `check_compactc_target.py --micro-dao-token --proof` to compile the complete
original source and generate ZKIR v3 keys for `dao_voting_token`. The maintained
`--micro-dao-advance --proof` and `--micro-dao-reveal --proof` selectors use the
same recipe for their respective exports. The adapters bind the full retained
IR, exact generated Rust, all seven raw circuits, and the selected binary IR and
verifier before proving. The source control rejects a changed declaration and the
historical projection.

Each state call produces a fresh standalone proof, verifies it in a separate
process, and exercises a distinct default-strict Ledger 9 transaction with
Night-backed Dust and exact unchanged replay refusal. The token uses constructor
state. The three advance phases and two reveal ballots use explicit prior-state
fixtures. The reveal also checks the applied-state double-vote refusal.
