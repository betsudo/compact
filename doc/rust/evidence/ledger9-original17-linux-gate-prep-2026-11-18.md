# Original-token 17-circuit Linux gate preparation

This local source-only slice extends the manual gate tracked by
[issue #773](https://github.com/betsudo/compact/issues/773) from the signed
merged #772 source to the signed descendant that qualifies seventeen of the
twenty unchanged original token circuits. It retains the pinned original
Compact and TypeScript fixtures, released Compact 0.36.0 and Runtime 0.21.0
tarball hashes, Ledger 9.1.0.1 semantics, source-built compiler equality,
physical ZKIR 3.2.1 keys, and the `zkir`/`zkir-v3` launcher aliases.

The gate now selects ten unshielded and seven shielded unique circuits. It
requires twenty-three fresh standalone/transaction proof pairs, nineteen
byte-identical replay pairs, nine exact shielded refusal artifacts and the
separate shielded recipient self-color proof. All seven proof-smoke dispatch
flags and their expected output markers were checked against the pinned
signed local Darwin arm64 user-send qualification; that receipt supplies
local evidence for the names and counts, not a Linux pass.

The retained `SOURCE_HEAD=24f7df3` is the signed merged-#772 compiler source
pin. `require_compiler_source_tree()` still compares the compiler, Rust
runtime/backend, Cargo and Nix inputs exactly with the checkout. The added
shielded proof-smoke gates leave those inputs unchanged. A source-only
`--preflight-only` result means **prepared**, with `native_linux=false`;
the full path still refuses Darwin before creating proof output. The native
Linux x86_64 full run, live node and three immediate shielded sends remain
open. No generated keys, proof outputs, existing stores or archived evidence
are changed by this preparation.
