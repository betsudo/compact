# Keccak and the pinned Ledger8 proof profile

**Recorded:** 18 October 2026, Asia/Kuala_Lumpur
**Implementation inspected:** `3739976cacdfd0f439fc54af63463750cbf9c31d`
**Related:** [#581](https://github.com/betsudo/compact/issues/581), [#666](https://github.com/betsudo/compact/issues/666)

This is an engineering capability/qualification record. It is not a security audit, a new proof result or authorization to upgrade the target.

## Finding

Native Rust Keccak and the ordinary typed checked Keccak operation already exist. The experimental functional surface excludes Keccak under its selected **Ledger8.1.3 / ZKIR v2** profile. The distinction is between existing host/checked-operation support and qualified proving/application support for that exact profile.

| Stage | Recorded status |
| --- | --- |
| Runtime/native hashing | Existing implementation uses the established Compact FAB semantics. |
| Ordinary checked effect operation | Delivered locally under #581; typed inputs/result and exactly-once lowering. That delivery explicitly did not claim a Keccak proof. |
| Experimental functional projection | `Operation::Keccak256` is explicitly refused in the current supported subset. |
| Pinned v2 frontend/proving path | The frontend rejects Keccak and directs the caller to `--feature-zkir-v3`. |
| A complete Ledger8 Keccak proof/application flow | Not qualified by the current delivery. |

## Source evidence

- [V2 lowering](../../../compiler/zkir-passes.ss) registers a Keccak handler that reports its lack of v2 support and suggests the v3 flag. The same restriction appears in [upstream ledger-8](https://github.com/LFDT-Minokawa/compact/blob/ledger-8/compiler/zkir-passes.ss).
- [Functional admission](../../../tools/compact-rust-backend/src/recorded/effect_plan/functional.rs) returns a checked subset refusal for the operation. It is a deliberate current projection boundary, not a claim that Rust cannot compute the hash.
- [The regression](../../../tools/compact-rust-backend/src/recorded/effect_plan/functional_crypto_tests.rs) named `native_keccak_syntax_does_not_grant_pinned_zkir_v2_functional_admission` checks both ordinary syntax availability and the functional refusal.
- [Target inputs](../../../flake.nix) select `ledger-8.1.3` for v2 and revision `3a7ae0361d03e6eed4fc91e31f6600f334f605d5` for v3. The [ledger-version reporter](../../../compiler/ledger-version.ss) derives its reported selection from those inputs.

## Actual CLI observation

The current qualified development CLI, with its matching Scheme component, returned:

```text
compactc --ledger-version
ledger-8.1.3

compactc --feature-zkir-v3 --ledger-version
3a7ae0361d03e6eed4fc91e31f6600f334f605d5
```

These are target-selection observations. No Keccak circuit was proved or ledger-applied by these commands. Merely passing the flag does not qualify the selected dependency revision with the pinned Rust runtime, provider, parameters, key serialization and ledger-application graph. ZKIR version and ledger version are independent axes.

## Plan and completion boundary

The approved roadmap finishes the maintained Ledger8 baseline before newer-target/ZKIRv3 qualification. There is currently no accepted commitment to backport proved Keccak to the pinned Ledger8.1.3/v2 profile. #581 records typed-operation work; its open issue state must not be read as an outstanding promise of full Ledger8 proof support.

A Ledger8-compatible extension remains a feasibility question. This record does not establish that every possible Ledger8 configuration is incapable of supporting Keccak.

Before enabling a new functional capability:

1. Identify an exact compatible frontend, ZKIR, runtime, provider, parameters and verifier/ledger graph.
2. Use one intact source contract through compilation, native execution, recording, preparation, proving, verification and funded default-strict application.
3. Retain input/encoding boundaries, wrong-binding and corrupt-proof controls, and measure proving cost/resources.
4. If compatibility is demonstrated, introduce explicit target capability/lowering and its tests/ADR; preserve the existing v2 refusal. Otherwise keep the feature on a separately qualified newer target.

Enabling native-only functionality would be a separate API/target policy decision and must not imply proof support. Forge artifact compatibility likewise needs the exact new profile; a cache hit is not a compatibility test.

## Record ownership

The problem is a target mismatch: native hashing and checked syntax cannot establish compatibility with the selected proving and ledger-application graph. The decision is to retain the v2 refusal until a separately qualified target demonstrates that complete path. [#581](https://github.com/betsudo/compact/issues/581) and [#666](https://github.com/betsudo/compact/issues/666) provide public tracking; the [delivery guide](../ledger8-delivery-and-completion.md) explains current features and release criteria. This record is self-contained and does not claim historical ADRs or raw qualification archives are published here.
