# Passport two-test consumer (historical P1)

The complete `consumer/src/lib.rs`, manifest and qualified lock are copied byte-for-byte from the original P1 archive. `passport-source/` contains the complete pinned Compact import closure and source manifest. See the [walkthrough](../../passport.md) and [historical receipt](../../../evidence/0.3.0/passport-p1/receipt.json).

Run generation from this directory, using the matching compiler/runtime distribution, to create `generated/contract/`; the [guide commands](../../passport.md#generate-and-consume) use this exact relative layout. Generated sources and the runtime bundle are intentionally not included in this documentation package. The handwritten manifest still points to `../generated/contract` from `consumer/`.

The archived two-test result belongs to its original compiler, source, lock and host. Relocation preserves the handwritten bytes and paths, but no test was rerun during ADR0350 staging. Before treating a new generation as accepted, qualify its compiler/runtime/output and dependency identities against that receipt. The newer lock-seeding recipe is guidance, not a claim that it was executed in P1.

The separate [current-profile gate](../../../plans/passport-current-profile-consumer.md)
now executes an unchanged copy of this complete consumer against a freshly
generated SDK with its matching bundled runtime. Its fresh local compiler142 result
belongs to signed `7d4fb8bea`; full PR/CI integration is still required. This does
not rename or renew the historical P1 receipt.
