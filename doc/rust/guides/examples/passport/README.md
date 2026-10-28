# Passport two-test consumer (historical P1)

Additional review fixes are locally implemented and require fresh qualification:
Cargo compiler/lint override guards and config identities, bounded post-kill pipe
drain, exact full runtime-bundle membership, and tighter guide/source isolation.
Thirty focused controls pass; the prior v3 results below keep their own epoch.

The complete `consumer/src/lib.rs`, manifest and qualified lock are copied byte-for-byte from the original P1 archive. `passport-source/` contains the complete pinned Compact import closure and source manifest. See the [walkthrough](../../passport.md) and [historical receipt](../../../evidence/0.3.0/passport-p1/receipt.json).

Run generation from this directory, using the matching compiler/runtime distribution, to create `generated/contract/`; the [guide commands](../../passport.md#generate-and-consume) use this exact relative layout. Generated sources and the runtime bundle are intentionally not included in this documentation package. The handwritten manifest still points to `../generated/contract` from `consumer/`.

The archived two-test result belongs to its original compiler, source, lock and host. Relocation preserves the handwritten bytes and paths, but no test was rerun during ADR0350 staging. Before treating a new generation as accepted, qualify its compiler/runtime/output and dependency identities against that receipt. The newer lock-seeding recipe is guidance, not a claim that it was executed in P1.

The separate [current-profile gate](../../../plans/passport-current-profile-consumer.md)
passes at clean signed `7b55379b8`: it builds this unchanged manifest and Rust
source as an actual two-test crate against the untouched raw generated SDK and
matching bundled runtime. It applies the current qualified source-lock seeding
recipe and retains the original historical P1 lock separately. Earlier `2bbf4daaf`
ran only the unchanged Rust test source in a gate-written manifest; `834a41025`
qualified formatted SDK bytes. Those epochs retain their narrower scopes.
Repeat review and complete current-head CI/merge remain required. This does not
rename or renew historical P1.
