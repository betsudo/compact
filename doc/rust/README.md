# Compact Rust backend and runtime

[Milestone 0.3.0 closure report](0.3.0-closure.md) · [Acceptance crosswalk](evidence/0.3.0/closure/crosswalk.md)

[Historical proposed 0.4.0 Ledger 8 functional scope](0.4.0-ledger8-functional-scope.md) preserves the `ab132650` Aiken-inspired generated Rust direction and proposed capability seams for later Ledger9 work. Its illustrative APIs are not delivered interfaces, an accepted milestone scope or a Ledger9 support claim.

The Rust target uses the Compact frontend, a typed private IR, a Rust AST emitter (`syn`, `quote`, `proc-macro2`, `prettyplease`) and a native runtime built on Midnight ledger/zk primitives.

## Ledger 9 migration

Use the [Ledger 9 adoption plan](plans/ledger9-adoption.md) for the local
migration status and the [versions and targets guide](guides/versions-and-targets.md)
for the matching compiler, Rust and TypeScript runtime, and ZKIR tools.

## Retained Ledger 8 delivery

The [native Ledger8 candidate profile](guides/native-ledger8-profile.md) records exact versions, usable source SDK, stage-specific evidence, type/runtime boundaries, exclusions and remaining integration gates. It is a local handoff contract, not release or whole-goal completion.

- [Delivered features, architecture, examples, qualification and completion checklist](ledger8-delivery-and-completion.md): dated 18 October 2026; separates bounded local implementation from supported-profile release readiness and full language parity.
- [Keccak capability and proof-profile audit record](audit/2026-10-18-keccak-profile-boundary.md): native/ordinary checked support, the ZKIRv2 exclusion, observed target selection and the qualification needed for an extension.
- [Stateful product Vector widening](plans/stateful-product-vector-widening.md): reviewed bounded local compiler177 source/API and selected genuine proof/default-strict application qualification; full coverage and integration remain open.

- [Stateful Vector products with Vector components](plans/stateful-vector-product-elements.md): reviewed bounded compiler180 API/static/ingress/error qualification and two selected fresh proofs with separate funded default-strict applications. Seventeen required exports, including four intact k9 models, remain unproved; full integration is open.

- [Products below Vector components](plans/vector-component-products.md): reviewed bounded compiler183 source/API/static/ingress/producer qualification, 24 complete SDKs and 35 cases. Three existing complete sources now have reviewed fresh proofs and separately proved funded default-strict applications. Twenty-one required exports remain unproved, including all eighteen over-limit models. The complete cohort is wired into the required Linux compiler workflow and locally requalified in the arm64 Nix shell; hosted CI and full integration remain open.

- [Current-candidate isolated witnessed SDK handoff](plans/current-profile-sdk-handoff.md): reviewed repaired compiler 0.31.183 local qualification of the existing documented consumer, actual proof-feature library Clippy, typed refusals, fresh proof and separate funded strict transaction. Accepted compiler143 integration remains separate; full CI and profile handoff stay open.

- [Exact-package default bundled SDK](plans/current-package-bundled-sdk.md): current compiler183 package qualification passes with explicit installed-prefix admission, refusal of runtime overrides (including empty), runtime-byte controls and fresh selected proof/strict application. Prior SDK epochs and full integration remain separate.

- [Current-candidate Passport consumer handoff](plans/current-passport-handoff.md): reviewed local compiler183 qualification passes for the intact pure application, raw SDK and documented consumer, with independent provenance controls. Historical application/fixture epochs remain separate; full integration is open.

- [Current-candidate DID lifecycle handoff](plans/current-did-handoff.md): reviewed local compiler183 API and fresh selected deactivation proof/default-strict application pass with all twelve Programs preserved. Its original selected scope remains distinct from the later zero timestamp extension; full integration stays open.

- [Zero timestamp DID witness](plans/did-zero-timestamp.md): reviewed local compiler183 coverage of both original success cases across four routes, case-bound fresh proofs and separate funded strict applications. The zero proof now has explicit local workflow wiring; hosted CI and full integration remain open.

- [Finite native supported-profile handoff](plans/current-supported-profile-handoff.md): reconcile current stage-specific capabilities, exceptions, examples and remaining integration gates without reopening accepted work.

These records retain their exact source/evidence revisions. They do not claim PR merge, completed hosted validation or a released Ledger9 target.

- [Nested product request-field baseline](plans/nested-product-request-field.md): local compiler184 metadata repair passes strict complete-SDK lint, four typed routes and marked static controls. Nested widening, automatic consumer-driver wiring and full integration remain separate; the compiler183 application/proof epochs are not relabelled.

- [Outer Vector product semantics](plans/outer-vector-product-semantics.md): reviewed local compiler191 qualification of the complete91-command native/API/ingress protocol, all16 models and all15 eligible fresh direct proofs with separate funded DEFAULT-strict applications/replay refusals. The intact family branch remains k9 and unproved. Earlier application/proof epochs are not relabelled; full CI/Linux and integration remain open.

- [Outer Vector proof CI wiring](plans/outer-vector-product-proof-ci.md): merged through #704 at canonical `9dc8d22b6`, with all nine required ordinary checks passed. Retained tree-equivalent Linux execution includes all 91 native checks, 15 fresh verified proofs and 15 separate strict applications. Hosted macOS arm64 remains optional and unqualified by skipped jobs. The cron runs only on default `main`; #706 records the post-merge coverage/cost-policy follow-up.

- [Constructor Vector iterable operands](plans/constructor-vector-operands.md): reviewed local compiler194 qualification at `847758e3` covers 14 complete SDKs, nested lexical captures, constructor state/witness/owned-value behavior, all 42 intact models and 17 selected fresh later-entrypoint proof cases with separate funded DEFAULT-strict applications and replay controls. Earlier compiler192 results retain their own identities. The maintained combined CI command is being qualified separately; constructor cryptography/total cost, new-head CI/Linux and integration remain open.

- [Constructor equality conditions](plans/constructor-equality-conditions.md) (#707): local compiler195 qualification covers eight complete SDKs, all 24 intact k6 models and sixteen selected later-entrypoint proofs with separate funded DEFAULT-strict applications/replays. The current maintained command passes at `041b74e47`; independent actual-result review accepts its bounded local results. Required Linux wiring is implemented locally, with hosted qualification still required. Top-level constructor `if`, constructor cryptography and full integration remain separate.

## Checked recording checkpoint

The current recording-effect branch is documented in [Checked recording and source diagnostics](guides/checked-recording.md) and the [capability and retirement book](recording-effect-algebra/capability-and-retirement-book.md). The compiler/runtime ABI is **53**, private Rust IR is **20**, finalized capability reports use **3**, and private source-origin transport uses **3**. Package versions remain separate.

The previous milestone reports below retain their own revisions and evidence. Current closure requires the final local CLI and authenticated historical qualification, followed by independent review; their success is not asserted by this documentation draft.

## Guides

- [Start with a task: compile, implement witnesses, test and integrate](guides/index.md)
- [ContractLab testkit](../../testkit-rs/README.md)
- [Unit-valued mutation initializers and ordered comparisons](guides/unit-initializers.md)

- [Compiler backend and generated crate guide](../../tools/compact-rust-backend/README.md)
- [Runtime APIs, ledger ownership and ABI](../../runtime-rs/README.md)
- [Mechanical representation derives](../../runtime-rs-macros/README.md)
- [Rust architecture decision register](adr/README.md): 360 published records, topic index, original aliases and dated engineering history
- [ADR publication provenance](adr/publication-manifest.json) and [evidence conventions](adr/references.md)

## Accepted baseline and governance

Current 0.3.0 engineering acceptance uses implementation `f9a49666`, private IR20 and runtime ABI50; see the [closure report](0.3.0-closure.md) for its composite qualification, application scope and known limits. Package versions are unchanged.

Milestone 2 acceptance belongs to implementation revision [`03c03a39`](https://github.com/MediaNoxLabs/compact/commit/03c03a39a2d39c43c91143b9526ca1c2ffae706c): compiler 0.31.133, private IR schema 20 and runtime ABI 49. See the [public closeout](https://github.com/MediaNoxLabs/compact/issues/103#issuecomment-6017383781) for source/behavior/proof/live evidence scopes and distribution limits. Publishing these documents does not rerun implementation validation.

The Rust ADR series records branch implementation decisions. [CoIPs](../../coips/README.md) remain the separate upstream proposal and adoption process, with independent numbering and review. No new milestone is opened by this publication.
