# Ledger 9 ten-step Bytes native Linux gate plan

Scope: the untouched pinned `primitives/bytes/manual_update/slow/large_reconstruction`
contract and its original compile/runtime fixtures at
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. This plan stages a native
`x86_64-linux` qualification after the local Darwin slice has a signed exact
head and audited k16 proof. It does not claim a Linux result.

## Source and artifact selection

Use an explicitly selected signed 40-character commit. Fetch the pinned
mainstream commit for byte comparison, verify the DCO/Assisted trailers and
signature, and require a clean checkout. Build `compactc` from that checkout
with the existing offline `compiler-producer.yml` workflow; record its
original-recipe receipt. Use released Compact runtime `0.18.0`, Ledger
`9.1.0.1`, `midnight-zkir` `3.2.0`, and Rust `1.99.0`. Do not import a
compiler or generated contract from another commit.

Run `check_upstream_ledger9_bytes_large_reconstruction_gap.py` for the exact
original paired TypeScript/Rust semantic receipt, using a fresh dedicated
Cargo target. Require result, ledger state, gas and all 315 ordered public
operations to match; require raw paired ZKIR equality and k16/40,652 rows.
The host test currently sets a documented 4 MiB thread stack. A default 2 MiB
thread overflows at the generated ordinary circuit entry on Darwin; Linux
must be measured separately and this setting must not be silently removed.

## Proof lane

Provide a **new target per exact head** and a separately retained proof
output. Verify official `bls_midnight_2p16` (12,583,300 bytes, SHA-256
`09c877216d6589b370263e18af40a030a901b41a7a7c37ef58c9901db41f05c6`)
and the k6 material for the distinct wrong verifier. Before keygen, freeze
tracked source, generated TS/Rust/ZKIR, Cargo depfile prerequisites, tool
bytes, checked programs and proof material in physical hash inventories.
Run the dedicated k16 proof gate, then its independent audit and separate
verifier. Require the wrong verifier, tampered proof, altered embedded IR and
altered raw ZKIR to refuse. Require one positive-fee Dust funded call under
default strictness, matching state, and unchanged replay refusal.

Reserve a runner with at least 40 GiB of free scratch and 96 GiB RAM for the
first native k16 measurement; the verified parameter itself is only 12 MiB,
while peak keygen/prover memory is not yet measured. Monitor scratch and
memory, and retain the proof-bound target until the final physical audit.
Upload receipts, independent verifier logs, source/depfile inventories,
proof/key hashes and peak resource measurements. Exclude disposable caches
only after their non-participation in proof binding is audited.

Use a workflow dispatch at an explicit SHA, then a narrow path/branch gate
for later publication. Do not add this expensive lane to every PR or
duplicate it across stacked PRs.
