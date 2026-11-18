# Native Linux gate for the original token suite

The manual exact-head runner retains the unchanged pinned original Compact
contract and 19-case TypeScript test. It builds the signed source compiler
and ZKIR v3 through Nix, checks all twenty original circuit models, emits
one full Rust/TypeScript/ZKIR/key bundle, and executes the 19 TypeScript
assertions against the released Runtime 0.21.0 and Onchain 4.0.1 tarballs.

This extension prepares the proof gate for all ten unique original
unshielded circuits and the first shielded circuit,
`mintShieldedToSelfTest`. It requires each expected Effects/proof/strict
marker, 15 fresh standalone and transaction-bound proof pairs, replay
identity for all thirteen independently applied transaction paths, and the
exact shielded no-output-offer refusal. The follow-up gate pins signed
GPG/DCO source commit `24f7df322876c839cbceadefebb41cbd4dce7103`, whose
complete tree equals the #772 squash merge
`785eddfc5ec07036c98fdc988c6138e1c8cc834e`. It verifies the current
clean signed checkout with the repository public key. For a GitHub squash
merge checkout, pass `--signed-checkout-head` with a GPG/DCO-signed commit
whose complete tree equals the checkout. The signed compiler source pin must
also match the live compiler, Rust runtime/backend, Cargo and Nix inputs
byte-for-byte before the compiler build. Every generated and proof file is
hashed into the Linux receipt. If the signed #772 PR commit is absent from a
fresh clone after squash merge, fetch `refs/pull/772/head` before running the
gate and verify it resolves to the pinned SHA above.

Run `python3 tools/compact-rust-backend/check_ledger9_unshielded_linux_source_regression.py`
in the full signed-source checkout to verify the #772 signed/squash tree pair
and the stale compiler-pin refusal. This manual regression needs the #772 PR
commit object; hosted shallow Python discovery does not fetch that object.

The source-only `--preflight-only` mode checks pinned physical inputs,
signed checkout identity and compiler-source equality on any host. It does
not require an `Assisted-by: AI` trailer beyond the repository's GPG/DCO
policy. Full qualification requires native Linux
x86_64, a fresh output directory and at least 50 GiB free. A Darwin preflight
is preparation only; no native Linux proof, hosted CI or live-node result is
claimed until the gate is executed there. The runner does not start or alter
the shared Midnight stack.
