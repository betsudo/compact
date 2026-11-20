# Current-generated receive consumer

Builds the complete retained source IR with the current backend, preserving the
original Compact source, TypeScript oracle and existing runtime tests.

The receive cutover adds an internal recorded `receiveShielded` helper. Public
recorded bodies and export capabilities remain unchanged; this refresh qualifies
the newly generated crate and retains the original runtime oracle coverage. It
is not another selector retirement or a new proof qualification.

```sh
cargo test -p compact-rust-transient-receive-send-oracle-fixture --all-features
```

The Ledger 9 proof gate binds the complete compiler IR, generated Rust, raw and
binary v3 IR, and immutable verifier. The separately verified standalone proof
preserves the authoritative offer allocation and original typed results. A
funded deployment and distinct wallet-funded receive/send call apply under
default strictness. Exact replay leaves the ledger unchanged. The same local
transaction prover refuses the malformed singleton index 1 at the Merkle-root
constraint.
