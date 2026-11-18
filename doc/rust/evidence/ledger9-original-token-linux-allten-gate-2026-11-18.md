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
exact shielded no-output-offer refusal. The prepared gate verifies its
current signed head and the signed `cdb4cb64629af69c2b48ea1f3edfae638e7369a6`
proof-source ancestor using the repository public key. The compiler source
still must match that ancestor. Every generated and proof file is hashed
into the Linux receipt.

The source-only `--preflight-only` mode checks pinned physical inputs and
signature/ancestry on any host. Full qualification requires native Linux
x86_64, a fresh output directory and at least 50 GiB free. A Darwin preflight
is preparation only; no native Linux proof, hosted CI or live-node result is
claimed until the gate is executed there. The runner does not start or alter
the shared Midnight stack.
