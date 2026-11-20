# secp256k1 Bytes<32> casts

[The upstream composite test][upstream] is retained byte-for-byte in
[`upstream-constant-casts.ss.txt`](upstream-constant-casts.ss.txt), including
its Compact source templates and JavaScript assertions. The archive contains
lines 77668–77732 from compiler/test.ss at the pinned revision. Its SHA-256 is
`b4ff27a19610d93f173d42c9a072c31c0f3249dd46519a1d5a1b603661150326`.

[`constant-k1.compact`](constant-k1.compact) renders the original decimal
constant and retains the secp256k1 ledger declarations, `castK1`, and the
secp256k1 statements from `castZero`. The executable assertions preserve the
original false/true inputs and expected secp256k1 ledger values. They use
the local synchronous runtime's context shape. The archived P-256,
Curve25519 and native-field statements and assertions remain available for
their own coverage.

[`dynamic-k1.compact`](dynamic-k1.compact) exercises both cast directions
through the ledger with typed returns. The assertion code uses independent
little-endian byte vectors and modulus arithmetic at zero, one, the
192- and 248-bit boundaries, each modulus and its neighbors, and the
all-ones byte vector. It also checks an inactive and an active guard,
runtime argument widths and field ranges, and source refusals for four
unsupported width casts.

Run with the same compiler and runtime arguments as the [family runner](../README.md).
It requests both targets for each positive fixture, executes the generated
TypeScript assertions using Vitest, and checks the source refusals on all
three target modes. Generated artifacts and compiler logs stay beneath the
requested output directory. Proof qualification has a separate gate.

[upstream]: https://github.com/LFDT-Minokawa/compact/blob/672bd0f226f47a7d036c5c074ce71fbe1af1925e/compiler/test.ss#L77668-L77732
