## Serialization is an API

An encoding determines what a verifier and another implementation will receive. Boolean encoding must admit its defined values. Enum encoding must use the declaration's ordinal bounds. Products must preserve element order and padding. Wide integers must retain their full width.

The migration implemented Boolean, fixed Bytes, native Uint and Field, product encoders/decoders, enum ordinals, wide Uint through 248 bits, foreign base/scalar fields and empty vectors.

## Empty does not mean invalid

A zero-length vector occupies no wire bytes even when its element type lacks a codec. That is a real source-language edge case, and both mainstream references compile the empty point-vector fixture.

The tested JubjubScalar wire probes instead fail internally in the pinned mainstream source and released binary. The local branch gives a source error. This is recorded as a known boundary rather than inventing codec compatibility.

## A leading character can be protocol data

The opaque-string regression concerns a leading U+FEFF. A decoder that strips it can change a receipt, archived value, equality result or Map key.

The intact fixture exercises those operations in sequence. Six proof preimages, independent proofs and separately funded applications preserve the original value. A consistently stripped value may form a different valid statement. The test must bind the honest proof to the original statement to detect that substitution.

## Byte equality is content equality

Two separately allocated Uint8Arrays with identical bytes should compare according to Compact's value semantics. The TypeScript path previously exposed a reference-equality mismatch.

The port checks length and contents, including values nested in products and vectors. All 23 original cases passed generated execution, SDK preflight, independent proofs and funded strict replay checks. The unchanged Rust and raw ZKIR outputs made the location of the fix particularly clear.

## Before and after · Compare bytes from two sources

### Before · Two objects with the same contents compare unequal

~~~typescript
// This JavaScript example reproduces the reference-equality pitfall.
const stored = new Uint8Array([4, 8]);
const received = new Uint8Array([4, 8]);
stored === received; // false
~~~

### After · Compare the byte sequence

~~~typescript
// The behavior the generated comparison must preserve.
stored.length === received.length &&
  stored.every((byte, index) => byte === received[index]); // true
~~~

Changing the second byte to 9 produces false. The same rule applies when the arrays appear inside a product or Vector. The Compact source compares the values directly, and generated code supplies the value semantics.

## Events are ordered effects

The event migration covered ShieldedSpend, ShieldedBurn, ShieldedMint, ShieldedReceive, their unshielded counterparts, Paused, Unpaused and Misc. Each event has its own tag and payload.

Optional and Either fields preserve inactive bytes as required by the source representation. Disclosure analysis must also inspect those fields. A branch that is inactive at runtime cannot become a way to hide undeclared witness disclosure.

SDK controls substitute a tag, byte field, domain separator, recipient or event order. The selected funded transaction tests inspect the emitted ledger events and replay behavior.

## Nested defaults are observable

A historic Merkle tree default inside a Map begins with empty history. A top-level reset has its own blank-root behavior. Reusing the wrong constructor can produce a plausible tree with the wrong historic semantics.

Both intact plain and historic upstream fixtures now generate typed Rust nested slots. Their eight original exports have selected fresh proof and funded application coverage on arm64.

## Later qualification receipts

The later BOM receipt at 360af593b records six proofs and six funded export calls. The nested Merkle receipt at 3a178e916 records eight proofs and eight funded calls. Opaque-byte equality at 7a4fab772 records 23 of each. These source-specific results supersede the earlier open-proof descriptions preserved in the plan snapshot.

The [public delivery register](../sources/ledger9-delivery-register.json) retains the receipt identities and selected funded-result summaries without witness or key material.

### Source trail

- [Compiler compatibility issue #728](https://github.com/betsudo/compact/issues/728)
- [ABI and generated-interface issue #731](https://github.com/betsudo/compact/issues/731)
- [Original opaque-byte equality change](https://github.com/LFDT-Minokawa/compact/commit/eb17cc004b5f025ef964dd7991f96197f010e732)
- [Published compatibility source snapshot](../sources/ledger9-upstream-compatibility.md)
