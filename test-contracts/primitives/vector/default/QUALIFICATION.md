# Pinned Vector default qualification

The three files in `semantics/` are byte-identical to Compact main at
`672bd0f226f47a7d036c5c074ce71fbe1af1925e`. The original runtime test
requires the returned `Vector<4, Field>` to contain four zeros. The contract
also writes the default vector to public ledger state and asserts its first and
last elements are zero. There are no original compile refusals in this family.

The local preflight is
`tools/compact-rust-backend/check_upstream_ledger9_vector_default.py`. It must
run at a clean, signed exact head with the compiler recipe, compiler binaries,
Nix runtime distribution and its source recipe as explicit inputs. Its receipt
binds the pinned source, original assertion, compiler, runtime, generated
TypeScript, Rust, raw ZKIR, metadata, ordinary result, recorded result and
public-state replay. A passing preflight is required before proof qualification.

## Fresh proof gate to implement after preflight

1. Read and hash the exact-head preflight receipt and all of its named source,
   compiler, runtime and generated artifacts. Reject a changed tree, absent
   artifact, mismatched SHA, stale recipe, wrong pinned original or wrong
   `pure: false` / `proof: true` capability. Do not compile a different source
   while proving.
2. Reinvoke the generated TypeScript circuit from a fresh initial state and
   serialize its actual `proofData` with the SDK's
   `proofDataIntoSerializedPreimage`. Bind that preimage, the raw
   `vector_default_semantics.zkir`, circuit name, public inputs and outputs,
   and recorded state transition to the proof runner input receipt.
3. Generate a fresh proof and verify it independently with Ledger 9.1.0.1 SDK
   verifier data. Record SHA-256 and sizes of proving/verifying keys, proof,
   preimage, raw ZKIR, verifier input and verifier output. Reject a corrupted
   proof, wrong verifier key, wrong circuit or altered public transcript.
4. Check the internal zero assertions, four-zero result and declared
   `default_result` state against the independently verified transcript. Since
   the initial and final vectors may both be zero, final-state equality alone
   does not establish that the circuit executed its write. Require the recorded
   verify operation and its replayed transition as well.

## Funded application gate to implement after proof

Build a transaction from that same proved call, with real funded dust inputs,
a positive fee and the default strictness policy. Verify and apply the
transaction to a fresh Ledger 9.1.0.1 state. Bind the transaction, proof and
funding input identities in the receipt. Check the resulting application state,
contract address, recorded call and communication commitment against the
qualified preflight and proof. Attempt to apply the identical transaction
again; require `IntentAlreadyExists` and byte-identical ledger state before and
after the refusal. A strict verification check of an already applied
transaction may succeed; the replay refusal must be measured at application.

These proof and funded gates have **not** run for this family. Local Darwin
preflight will not substitute for the required native Linux integration gate.
