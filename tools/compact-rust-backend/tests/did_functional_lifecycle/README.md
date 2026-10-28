# DID lifecycle capture and ordinary API refusals

The capture keeps the existing unit-composition witness/signature/case logic,
selects an explicit matching runtime and retains complete fresh observations.
It fingerprints the runtime production dependency closure, Node, external
shared libraries and wrappers. Host OS library paths remain metadata only.
The raw output includes host identities for private qualification storage;
only complete DID case rows and allowed source/profile metadata belong in
public checked-in fixture data. These are synthetic test keys/nonces.

The three Rust inputs are intentional compile-fail cases in a separate
consumer: wrong ordinary expectedVersion and functional signature types must
refuse with E0308, and consuming the same Program twice must refuse with
E0382. They must not be included in ordinary workspace all-feature builds.
