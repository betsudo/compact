## Why reuse looked attractive

Compiler and proof work repeatedly constructs ZKIR, prover keys and verifier keys. Authenticated reuse of deterministic artifacts could shorten local iterations and CI, provided the cache key includes the identities that make the artifacts valid.

Those identities include source, compiler/backend, tool recipe, serialization, parameters and verifier material. A proof verdict belongs to a particular statement and verification operation. Caching reusable artifacts does not supply a fresh verdict for a new statement.

## What the local measurements showed

A frozen Mac comparison measured ordinary, cold-with-snapshot, warm and restored paths. The three-repetition medians were approximately 6.372 seconds ordinary, 9.708 cold-with-snapshot, 3.426 warm and 4.242 restored.

Those selected paths gave approximately 46.23% warm and 33.42% restored reductions. Both missed the stated greater-than-50% target. They were bounded local measurements and did not establish whole-CI savings.

## Correcting the key-generation interpretation

The earlier “1% key generation” interpretation was incomplete. The 46-second observation measured Forge-managed generation. Direct bypass generation was not timed separately. The total ZKIR, prover-key and verifier-key share remained unknown.

This correction changes the conclusion that can be drawn from the experiment. A small measured Forge-managed component does not quantify all generation in the pipeline.

## Operational work changed the priority

The Linux builder and store investigations accumulated important evidence, but the experiment was taking attention from the primary backend objective. Store metadata continuity also failed to establish fresh content or trust verification.

The owner explicitly deferred Forge CI adoption and returned the primary work to Rust AST ergonomics, type safety and semantic coverage. The continuation point became experiment/zkir-forge, with separate Forge branches and failure evidence retained.

## The decision remains useful

Selected contract-level reuse benefits were valid. Whole-CI value remained unproven. Deferral preserved the experiment and removed it as a dependency of backend development.

This distinction belongs in the book because it is a design decision about priorities and evidence. The experiment's former acceptance criteria were never completed merely by parking it.

### Source trail

- [Forge adoption issue #679](https://github.com/betsudo/compact/issues/679)
- [Reviewed baseline with Forge isolated, PR #664](https://github.com/betsudo/compact/pull/664)
- [CI optimisation through exact environment reuse, PR #723](https://github.com/betsudo/compact/pull/723)

The numerical account comes from the retained owner-approved Forge decision and benchmark notes. It is reproduced here as a historical local measurement.
