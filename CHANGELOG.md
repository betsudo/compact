# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Toolchain 0.31.212, language 0.23.105, runtime 0.18.0]

Unreleased Ledger 9 migration.

### Changed

- Compare `Opaque<"Uint8Array">` values by length and contents in generated
  TypeScript. Distinct arrays with identical bytes now compare equal.

- The Compact standard library's `jubjubSchnorrVerify` now rejects the
  identity verification key with a failed assertion. This is a breaking change
  for callers that pass the default `JubjubPoint`.

### Added

- Expose generated circuit signature metadata and runtime descriptor types for
  exported Compact circuits.
- Support the CompactStandardLibrary Boolean serialization built-ins in
  TypeScript and Rust, with a checked decoder that accepts only 0 and 1.
- Support fixed Bytes serialization and checked prefix decoding in the
  TypeScript and Rust targets.
- Support checked little-endian serialization of native Uint values in
  the TypeScript and Rust targets.
- Support canonical little-endian Field serialization in the TypeScript
  and Rust targets.
- Decode fixed-width structs, tuples and vectors with checked Boolean
  components in the TypeScript and Rust backends.
- Encode fixed-width structs, tuples and vectors with little-endian
  components and zero padding in the TypeScript and Rust backends.
- Encode and decode enum ordinals with checked bounds in both backends.
- Encode and decode Uint values through Uint<248> in the Rust backend.
- Encode and decode Secp256k1Base with the Ledger 9 wire format in both backends.
- Extend Ledger 9 wire encoding and modular decoding to the remaining
  secp256k1, P-256 and Curve25519 base and scalar fields.
- Allow zero-length vectors in wire codecs when their element type
  has no wire codec.
- Support default insertion into maps of plain and historic Merkle trees
  in ordinary and recorded Rust.
- Support standalone `ShieldedMint` emission in ZKIR v3, TypeScript and
  ordinary and recorded Rust.
- Support standalone `ShieldedReceive` emission and its corrected
  commitment, ciphertext and contract-address wire order.
- Support standalone `UnshieldedSpend` emission with structural sender
  encoding in ZKIR v3, TypeScript and Rust.
- Support standalone `UnshieldedReceive` emission with structural
  recipient encoding in ZKIR v3, TypeScript and Rust.
- Support standalone `UnshieldedMint` emission with its 80-byte payload
  in ZKIR v3, TypeScript and Rust.
- Support standalone `UnshieldedBurn` emission with its 113-byte payload
  in ZKIR v3, TypeScript and Rust.
- Support `Paused` and `Unpaused` events and their zero-byte codecs
  in ZKIR v3, TypeScript and Rust.
- Support the `Misc` event and its 288-byte name and payload codec
  in ZKIR v3, TypeScript and Rust.
- Support standalone `ShieldedBurn` emission in ZKIR v3, TypeScript and
  ordinary and recorded Rust.
- Support standalone `ShieldedSpend` emission in ZKIR v3, TypeScript and
  ordinary and recorded Rust. Successful circuit contexts expose the
  VM-decoded event and its emitting contract address.

### Changed

- Pair the synchronous TypeScript runtime with the published Ledger 9 Onchain
  Runtime 4.0.1 package. Generated TypeScript selects the new runtime minor.
- Emit one ZKIR v3 impact per transcript operation, as required by the Ledger 9
  transaction prover.
- Generated Rust requires Rust 1.99 and Compact runtime ABI 54. Use
  `--rust-functional-ledger9` for functional generation with the paired Ledger 9
  compiler and runtime.

### Fixed

- Preserve nested comparisons inside stateful Boolean equality and inequality
  when emitting Rust, including the original mixed foreign-field cast case.
- Avoid an internal compiler error for addition and multiplication of small
  nominal Uint values when the result has the same range as the operands.
  Retain the run-time overflow check when the result can exceed that range.
- Successful assertions produce the empty tuple in generated TypeScript.
  A conditional can combine an assertion and an empty tuple literal.
- Check witness disclosures through vector spreads, including conditions on
  nested Boolean values.
- Preserve leading U+FEFF characters when decoding stored opaque strings.
- ZKIR v3 encodes Merkle tree defaults with the Ledger 9 depth field.
  Circuit generation supports reset and default insertion through depth 32.

## [Toolchain 0.31.210, language 0.23.105, runtime 0.16.102]

Unreleased local Rust-backend slice; generated API, native and proof
qualification remain separate acceptance gates.

### Fixed

- Preserve implicit tuple component widening in equality guards inside
  supported constructor Vector loops, including nested tuples and captured
  Vectors. Reuse checked coercion; retain narrowing and fallible guard refusals.
  Ordinary circuit and value initializer transport remain unchanged.
- Assemble constructor guard conditions as structured AST so equality with a
  leading struct literal does not panic while parsing generated condition tokens.

## [Toolchain 0.31.209, language 0.23.105, runtime 0.16.102]

Unreleased local Rust-backend slice; native and proof qualification remain
separate acceptance gates.

### Fixed

- Preserve checked implicit unsigned widening in equality guards inside
  supported constructor Vector loops. Keep general casts, value initializer
  restrictions and ordinary circuit lowering with their existing owners.

### Added

- Add a separate complete mixed-width Uint, tuple, Vector and struct equality
  SDK cohort, with both operators, TypeScript comparisons, ordinary and
  functional Rust routes, public VM replay and marked type/shape misuse
  controls with valid twins. Proof qualification and full required CI remain
  integration requirements; this entry is not an execution receipt.

## [Toolchain 0.31.195, language 0.23.105, runtime 0.16.102]

Unreleased local Rust-backend slice; qualification remains in progress.

### Added

- Support typed equality and inequality guards inside supported constructor
  Vector loops. Reuse the checked pure lowering and structured Rust AST emitter;
  keep value initializer restrictions and fallible operand refusals unchanged.
- Add complete constructor equality qualification to the required Linux consumer:
  current source/API checks, all 24 intact models and sixteen selected fresh
  later-entrypoint proof cases with separate strict applications and replays.
  Retain every preceding gate and independent diagnostics.

## [Toolchain 0.31.194, language 0.23.105, runtime 0.16.102]

Unreleased constructor reconciliation checkpoint; required CI and integration
remain pending. Bounded local execution retains its exact source identities.

### Fixed

- Regenerate the welcome constructor fixture after owned Vector-element emission,
  and require full license headers before source code or module documentation.

- Preserve outer lexical Vector bindings through nested constructor loop,
  sequence and conditional lowering; retain original source/proof fixtures and
  add a separate complete nested-capture regression.

- Add a maintained constructor qualification command to the required Linux gate:
  complete source/API controls, all 42 intact models, and 17 selected fresh
  later-entrypoint proof cases with separate strict applications and replays.
  Reuse one bounded external Cargo target across its stages and preserve all
  other gates. These proofs do not prove constructor initialization.

- Preserve the checked constructor Vector operand extension alongside canonical
  wide-zero and outer-Vector behavior through a forward source reconciliation.
  Earlier compiler192 and compiler193 evidence retains its original identity.

## [Toolchain 0.31.193, language 0.23.105, runtime 0.16.102]

Unreleased reconciliation checkpoint; no release or completed qualification is implied.

### Fixed

- Make hosted macOS arm64 compiler qualification an explicit manual opt-in;
  preserve complete required Linux gates and fail closed on requested Mac runs.

- Run the complete outer-Vector API and fifteen-proof qualification in the
  required Linux consumer, retaining native checks and failure diagnostics.
- Reconcile the preserved outer-Vector product stack with the canonical wide-Uint
  multiplication-by-zero backend. Retain both typed lowering paths, generated
  consumers, proof suites and CI diagnostics without rewriting either history.

## [Toolchain 0.31.192, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Admit checked single-Vector constructor loop operands from local values,
  helpers, witnesses and conditional expressions. Prepare nonparameter operands
  once before iteration while retaining the existing borrowed parameter route,
  lexical/type/layout/resource validation and earlier invalid-kind refusals.
- Bind borrowed constructor Vector elements as typed values before their use in
  ledger operations, copying Copy elements and cloning owned elements without
  cloning or moving the input Vector.

## [Toolchain 0.31.191, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Retain the invocation directory for default source-map path fallbacks and
  serialize path strings as valid JSON, preserving source operands, mappings
  and explicit sourceRoot overrides through staged publication.
- Refresh an existing standalone Rust frontend manifest after map relocation
  and Rust library emission, using the main compiler's unchanged manifest hasher.
  Missing frontend manifests remain optional; no metadata is synthesized.

## [Toolchain 0.31.190, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Keep tuple-wrapped component Vector/product inputs usable under strict generated
  SDK Clippy. Scope existing reasoned metadata to the exact private request field
  and constructor signatures; preserve carriers, bodies and execution semantics.

## [Toolchain 0.31.189, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Compose existing checked product conversions directly below outer Vector
  product elements. Preserve nested tuple arity, fixed Vector dimensions,
  original bounds and ordered helper/witness evaluation through the existing
  checked coercion and structured AST owners. New identity carriers, narrowing,
  runtime, ABI and resource limits are unchanged.

## [Toolchain 0.31.188, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Preserve strict generated-SDK Clippy for nested tuple children in outer Vector
  execution results with reasoned method-local type-complexity metadata. Keep
  exact public carriers, consuming request ownership, pure-library output and
  conversion admission unchanged.
- Extend the existing complete request-carrier regression gate with typed
  execution, VM replay and compile-time misuse controls for this carrier shape.

## [Toolchain 0.31.187, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Preserve frontend-supplied widening through fixed Vector leaves nested below
  component products. Keep exact dimensions, tuple arities, source bounds and
  ordered evaluation through the existing checked conversion and AST owners.
  Narrowing, new identity carriers and top-level nested product admission remain
  unchanged; runtime, ABI and resource limits are unchanged.

## [Toolchain 0.31.186, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Preserve exact nested Vector/product request carriers under strict generated
  SDK Clippy when a component product contains a fixed Vector, directly or in a
  singleton tuple. Attach only the existing reasoned private-field metadata;
  public types, ownership, execution and widening admission are unchanged.

## [Toolchain 0.31.185, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Classify frontend-supplied scalar tuple subtree widenings below a component
  Vector in the existing ordered-value owner. Preserve every tuple arity,
  Vector dimension, original source bound and helper/witness evaluation order;
  checked coercion, validation, resource limits and runtime semantics are unchanged.

## [Toolchain 0.31.184, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Keep same-type nested product inputs usable under strict generated-crate
  Clippy by attaching reasoned type-complexity metadata to the immutable
  Program's private input field. The exact carrier, visibility, checked
  execution and conversion-admission rules are unchanged.

## [Toolchain 0.31.183, language 0.23.105, runtime 0.16.102]

Unreleased Rust-backend source checkpoint; no release or hosted-CI result is implied.

### Fixed

- Preserve checked fixed-Vector/product conversions through ordered stateful
  values, including a characterized product level below Vector components.
  Retain original bounds, product arity, once-only evaluation and witness order;
  unsupported deeper forms continue to receive explicit checked refusals.
- Preserve complete generated Rust request carriers with reasoned field-local
  type-complexity annotations where nested inputs require them.
- Require the maintained current-source API and selected proof/strict-transaction
  cohort in the existing native CI definition, using the shared compiler package,
  pinned proof tool/material and exact source/IR/Program/verifier bindings.
  Other required exports retain their explicit unproved/over-limit status.

## [Toolchain 0.31.173, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve supplied Vector conversions in stateful local initializers. Evaluate
  each original source value before conversion and retain local bindings with
  their surrounding actions in the existing ordered body. Native/recording
  validation, source bounds and runtime ABI are unchanged.

## [Toolchain 0.31.172, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve frontend-supplied monotone Vector value conversions in ordered stateful
  returns, including helper, parameter, query and witness values. Retain original
  source bounds, actions and nested/empty Vector structure before the checked
  conversion. Existing native/recording validation and runtime ABI are unchanged.

## [Toolchain 0.31.170, language 0.23.105, runtime 0.16.102]

### Fixed

- Lower effectful Vector children in the existing native stateful aggregate owner,
  preserving successful pure Vector AST output, child evaluation order and exact
  element types. Uniform ordered-return source carriers retain their original
  checked bound before widening. No validator, runtime or ABI change.

- Retain the declared Vector context for direct stateful literal returns. Use
  existing ordered return plans for the complete action sequence and apply
  frontend-supplied aggregate casts after reaching their original source value.
  Preserve empty literal construction, intermediate bounds and condition order;
  reuse existing checked coercion and Rust AST owners without an IR/runtime ABI
  change.

## [Toolchain 0.31.167, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve frontend-supplied source types for struct member projections in wider
  aggregate literals. Reuse existing checked coercion and struct-field ownership,
  retaining the complete base expression, field bounds and evaluation order.
  Standalone field and struct literal behavior remains unchanged; no runtime ABI
  or IR schema change is introduced.

## [Toolchain 0.31.166, language 0.23.105, runtime 0.16.102]

### Fixed

- Retain frontend-inferred literal source element types through pure aggregate
  cast lowering. Apply existing checked Coerce to projected numeric values and
  complete checked casts in their declared element context, preserving source
  bounds, evaluation order and accepted scalar/identity behavior. Keep existing
  literal construction for empty and nested carriers; no new conversion relation,
  runtime ABI, IR variant or executor is introduced.

## [Toolchain 0.31.165, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve checked widening when a pure Vector source is typed through an alias,
  including alias chains and explicit frontend-approved casts. Reuse existing
  Coerce validation and Rust AST emission; preserve identity-alias output and
  nominal frontend refusals without changing other alias families or stateful
  expected-type ingress.

## [Toolchain 0.31.164, language 0.23.105, runtime 0.16.102]

### Fixed

- Apply the existing checked Vector conversion after a complete named pure
  Fold when its enclosing declaration requires a wider numeric Vector result.
  Preserve the original accumulator/reducer bounds and legacy operand order.
- Resolve Vector alias carriers at expected-type and pure cast ingress,
  including reducer Vector literals; unrelated alias families keep their owners.
## [Toolchain 0.31.145, language 0.23.105, runtime 0.16.102]

### Added

- Checked Rust emission for wide-Uint multiplication with a zero-typed operand.
  Both operands still execute once in source order; other wide products remain
  explicitly unsupported. Existing generated carriers and runtime checks are
  retained, including the exact zero result type.

### Fixed

- Use the declared singleton-zero result bound for ordinary stateful zero
  multiplication returns, including conditional arms and lexical bindings,
  through the existing typed expression owner.

## [Toolchain 0.31.144, language 0.23.105, runtime 0.16.102]

### Added

- Rust emission for unambiguous constant-valued Vector indices using local
  aliases and unsigned arithmetic. The existing compiler evaluator retains
  bounds authority; checked projection preserves operand evaluation and the
  typed FixedVector API. Runtime-derived or ambiguous index facts still refuse.
- Complete source fixtures, generated Rust/TypeScript ownership controls and
  separate genuine proof/application gates for the new and retained literal
  Vector sources.

### Fixed

- Preserve exact inferred unsigned literal and declared stateful arithmetic
  maxima, including valid zero-valued constant bindings.
- Refresh complete current compiler IR inputs used by proof runners while
  retaining historical compatibility inputs separately. Check the complete
  mixed-width source binding before the full native proof sequence.

## [Toolchain 0.31.143, language 0.23.105, runtime 0.16.102]

### Added

- An executable isolated witnessed functional SDK example and maintained
  qualification gate. The raw bundled SDK shares one runtime with an explicit
  paired source-testkit package. Typed witness/ownership controls, ContractLab
  replay, fresh ProofLab proofs, immutable mutation checks and separately proved
  default-strict funded application/replay retain their distinct evidence.
- Exact package/lock/runtime/material membership and actual checked-profile
  proof executable binding, with local/CI diagnostic retention. This selected
  fixture does not prove constructor/deployment execution or a wallet lifecycle;
  compiler semantics, public APIs, runtime ABI and ledger profile are unchanged.

## [Toolchain 0.31.142, language 0.23.105, runtime 0.16.102]

### Added

- Current-profile standalone SDK qualification for the unchanged original
  Passport pure library: live TypeScript behavior, the documented Rust consumer
  and adoption tests, independent ordinary type refusals, strict Clippy and
  source/dependency identity controls. Matching runtime sources stay inside the
  generated consumer closure. Original source and historical oracle authorities
  are retained; no stateful export or proof applicability is introduced.

## [Toolchain 0.31.141, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve supported Unit mutation initializers in statement scopes and before
  trailing actions, using the existing ordered lexical binding walk. Typed
  ordinary methods and consuming functional Programs retain source order,
  previous-scope lookup, shadowing and first-failure behavior without helper
  circuits or new runtime/IR types.

### Added

- Mixed Field/Unit statement-scope source cases, independent type/result/reuse
  refusal binaries for both existing and statement APIs, and actual compilation
  checks for the formerly refused boundary sources. Selected statement-call
  proof/verification and strict-application controls extend the maintained gate;
  they do not prove constructors, seeded prestates or every export.

## [Toolchain 0.31.140, language 0.23.105, runtime 0.16.102]

### Fixed

- Emit functional Circuit steps through private block-local receivers instead
  of chaining expanded bind expressions. This keeps structured generated Rust
  practical to format while preserving typed values, source order, lazy
  operands and first-failure behavior. Checked programs, runtime ABI and public
  APIs are unchanged; source admission and resource limits are unchanged.

## [Toolchain 0.31.139, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve source order, lexical scope and once-only effects in supported
  Unit-valued Set mutation initializers. Checked Unit equality and inequality
  evaluate both operands before producing their Boolean result, retaining
  witness calls and first-error behavior through structured Rust AST emission.
  Ordinary typed methods and consuming functional Programs need no helper
  circuit; existing non-Unit initializer and resource boundaries remain explicit.

### Added

- Complete source-generated Unit initializer coverage against independent
  expectations and live TypeScript across native, recorded and functional
  routes, compile-fail/refusal controls and recording replay. Separate selected
  insertion/reinsertion proofs verify with mutation controls; independently
  proved funded transactions pass default-strict application and replay checks.
  These selected calls do not prove constructors, seeded prestates or all exports.

## [Toolchain 0.31.138, language 0.23.105, runtime 0.16.102]

### Fixed

- Emit checked fallible Unit-valued or empty-tuple pure calls as statements in functional Rust
  programs, preserving one evaluation and error propagation without Unit locals
  rejected by strict Clippy. Source admission and runtime ABI are unchanged;
  other result types keep their existing rendering.

### Added

- A complete unchanged original-DID functional fixture on Ledger8.1.3, selected
  constructor-derived deactivation behavior against live TypeScript, four-route
  authorization/failure controls, typed API refusals and a fresh selected standalone
  proof with immutable verification/mutation controls. An independently proved
  transaction passes funded default-strict application and replay rejection.
  Keep all twelve generated programs; do not imply twelve proved exports.

## [Toolchain 0.31.137, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve inferred scalar-pair return conversions around selected Rust ledger
  effects, including mixed Uint/Field results and existing inline Fold reducers.
  Reuse checked aggregate coercion and structured Rust emission to evaluate the
  condition, tuple and components once in source order. Narrowing and unsupported
  aggregate shapes retain their explicit boundaries; runtime ABI is unchanged.

## [Toolchain 0.31.136, language 0.23.105, runtime 0.16.102]

### Fixed

- Preserve selected ledger mutations, lexical scope and witness/checked-arithmetic
  order when inferred Uint-to-Field conversions or monotone Uint branch joins
  wrap a terminal Rust circuit return. Reuse the checked return plans and
  structured Rust emission; other effect-bearing initializer and narrowing
  combinations remain explicitly outside this bounded fix.

## [Toolchain 0.31.135, language 0.23.105, runtime 0.16.102]

### Added

- Lower Rust recording through typed effects, composition checking and
  independent plan validation. Supported operations compose without a
  whole-contract shape admission fallback; unsupported operations report
  checked refusals at the failed node.

### Changed

- Align the native Rust dependency graph with official ledger8.1.3 and pin
  the TypeScript runtime's onchain dependency to 3.1.2. Version the changed
  TypeScript runtime package as 0.16.102; the Compact language remains 0.23.105.
- Generated Rust requires runtime ABI52 and compatibility schema2. Use the
  matching compiler/runtime/macros source pair. Rust crate versions remain
  0.2.0 in this source; a coordinated crate-version and consumer-lock migration
  is required before registry publication of this changed API cohort.
- Set the Rust backend MSRV check to 1.89, matching its declared minimum and
  locked dependencies. The independent toolchain-manager CLI retains its
  separately inherited Rust1.88 minimum.

## [Toolchain 0.31.134, language 0.23.105, runtime 0.16.101]

### Fixed

- In ZKIR v2, select the Jubjub subgroup identity for inactive `ecAdd` and
  `ecMul` point operands. This prevents an untaken branch's zero-masked ledger
  or witness point from being decoded as an invalid curve point. Active
  operands, scalar inputs and public transcript constraints are preserved.
  The shared circuit emitter serves both TypeScript and Rust targets. Affected
  circuits have different ZKIR and require matching regenerated prover and
  verifier artifacts; old keys must not be paired with the new circuits.
  Unconditional point operations and the ZKIR v3 emitter are unchanged.

## [Toolchain 0.31.133, language 0.23.105, runtime 0.16.101]

### Added

- Preserve circuit, witness, constructor, and type-alias declaration locations
  in the private Rust IR so renderer diagnostics can name the Compact source.

## [Toolchain 0.31.132, language 0.23.105, runtime 0.16.101]

### Added

- Preserve ledger declaration locations in the private Rust IR and report them
  when Rust backend rendering rejects a declaration.

## [Toolchain 0.31.131, language 0.23.105, runtime 0.16.101]

### Added

- Derive checked Compact unit-enum codecs for generated Rust, preserving the
  existing representation and rejecting invalid ordinal tags.

## [Toolchain 0.31.130, language 0.23.105, runtime 0.16.101]

### Added

- Generate typed root `ListSlot<T>` descriptors for Compact List declarations;
  native and recorded List circuits use the same named slot and compiler VM
  programs. Record push, pop, reset, length, emptiness, and head reads with
  replayable observed values.
- Prove, verify, validate, and apply generated List calls against ledger-8
  artifacts, including constructor seeded head and mutation paths. A separate
  Cargo consumer rejects a wrong List element at Rust type checking.

### Fixed

- Use the declared element alignment's maximum encoded size for List head
  `Concat`, and omit suppressed root-path operations from List reset. Both
  changes align the recorded public transcript with Compact's emitted ZKIR.

## [Toolchain 0.31.129, language 0.23.105, runtime 0.16.101]

### Added

- Generate typed `MapSlot<K, V>` descriptors for scalar Map declarations and
  use them in native and recorded Map circuit bodies. Preserve key/value type
  checks and compiler-derived physical paths.
- Prove, verify, validate, and apply generated Map operations and constructor
  Map lookups through ledger-8 in the packaged offline gate.

### Changed

- Keep nested Map declarations on the existing constructor/runtime path until
  nested values have a typed `CellValue` representation; do not emit a scalar
  `MapSlot` for them.

## [Toolchain 0.31.128, language 0.23.105, runtime 0.16.101]

### Added

- Record complete root Set insert, remove, reset, membership, size, and
  emptiness VM traces through generated typed methods. Preserve observed Gather
  values in Verify programs and withhold recorded methods for unsupported Set
  expressions or control flow.
- Prove, verify, validate, and apply emitted ledger-8 Set calls, including a
  Set membership query followed by a Cell write.

### Fixed

- Make Set reset use Compact's keyed insertion opcode sequence. The previous
  final-state-equivalent sequence failed the emitted ZKIR public statement.

## [Toolchain 0.31.127, language 0.23.105, runtime 0.16.101]

### Added

- Generate typed `SetSlot<T>` descriptors from Compact Set declarations, preserving
  compiler-derived physical paths and element types. Native Set membership,
  insert, remove, reset, size, and emptiness calls now use these named slots.
- Exercise a generated Set from a separate Cargo consumer and reject a wrong
  Set element type at compile time.

## [Toolchain 0.31.126, language 0.23.105, runtime 0.16.101]

### Changed

- Generate native Cell and Counter reads/writes/updates through the same named,
  typed ledger descriptors used by recorded methods. Counter reset now has a
  typed slot operation. Physical paths remain compiler-derived.
- Check from an external consumer that a wrong Cell value type is rejected by
  Rust, while existing compiler-backed fixtures and ledger proof flows retain
  their behavior.

## [Toolchain 0.31.125, language 0.23.105, runtime 0.16.101]

### Added

- Record ordered root Field reads, witness calls, Field addition, and internal
  Field-returning circuit calls in the generated Rust trace. Keep this
  effect-aware recorder separate from native stateful emission.
- Prove, verify, and apply the `nested_witness_call_oracle` exports using
  Compact-emitted ledger-8 artifacts, including a Cell read used as a call
  argument and an internal witnessed return value.

## [Toolchain 0.31.124, language 0.23.105, runtime 0.16.101]

### Added

- Record supported nested Unit-returning stateful calls in the generated Rust
  trace, preserving argument evaluation and witness output order. Unsupported
  nested shapes continue to have no recorded entry point.
- Prove, verify, and apply generated nested Counter calls with and without an
  argument, and a nested two-witness Field Cell call, in the offline ledger-8
  gate.

## [Toolchain 0.31.123, language 0.23.105, runtime 0.16.101]

### Added

- Emit complete recorded traces for witnessed root Field Cell writes and Field
  reads. A borrowed `Contract<W>::recording()` handle uses the contract's
  witnesses while preserving the existing `contract.recording` API for calls
  without witnesses.
- Prove, verify, and apply a generated two-witness Field Cell call in the
  offline ledger-8 gate, checking private output order and final state.

## [Toolchain 0.31.122, language 0.23.105, runtime 0.16.101]

### Changed

- Generated recorded Cell and Counter methods now use the contract's named
  typed ledger descriptors instead of embedding numeric paths in their bodies.

## [Toolchain 0.31.121, language 0.23.105, runtime 0.16.101]

### Added

- Expose typed Cell and Counter descriptors under `ledger_slots` in generated
  Rust crates, with native execution and recording methods backed by the
  existing ledger VM runtime.

## [Toolchain 0.31.120, language 0.23.105, runtime 0.16.101]

### Added

- Generated Rust crates reexport their matching runtime and forward the
  `ledger-transaction` Cargo feature, allowing a consumer to use the optional
  proof call adapter through one generated crate dependency.

## [Toolchain 0.31.119, language 0.23.105, runtime 0.16.101]

### Added

- Record exported root Boolean Cell writes and reads, including parameterized
  writes, through the typed Rust recording handle.
- Align root Cell write and read VM operations with Compact proof programs.
  Prove, validate, and apply generated Cell write and read calls in the offline
  ledger-8 gate.
- Add an opt-in runtime adapter that replays and partitions a recorded result
  into a ledger-8 call prototype, keeping transaction construction out of the
  generated circuit body.

## [Toolchain 0.31.118, language 0.23.105, runtime 0.16.101]

### Added

- Expose circuits with complete recorded ledger traces as typed methods on
  `contract.recording` in generated Rust crates. Keep the module functions for
  callers that use them directly.

## [Toolchain 0.31.117, language 0.23.105, runtime 0.16.101]

### Added

- Allow generated Rust contracts to share one local runtime source package with
  `compactc --target rust --rust-runtime-root <path>`. A combined Cargo consumer
  test checks two independently generated contracts in one dependency graph.

## [Toolchain 0.31.116, language 0.23.105, runtime 0.16.101]

### Added

- Record exported Counter reads as verifying ledger operations and return the
  typed `Uint<64>` observation alongside the replayable public trace.

## [Toolchain 0.31.115, language 0.23.105, runtime 0.16.101]

### Added

- Emit `ledger_contract::recorded` functions for exported unit circuits made
  solely of Counter increments and decrements. These functions execute the
  circuit and retain the ordered ledger VM program for replay and transaction
  building.
- Include the matching Rust runtime and derive-macro crates in Rust compiler
  output, so a generated contract builds without depending on a stale Git
  revision of the runtime.

## [Toolchain 0.31.114, language 0.23.105, runtime 0.16.101]

### Added

- Expose exported Rust circuits through a typed `Contract<W>` facade while
  retaining the existing module functions and witness signatures.

### Fixed

- Keep compiler-generated temporary bindings and Compact type names from
  warning in Rust consumer builds; lint allowances stay within generated
  modules.

## [Toolchain 0.31.113, language 0.23.105, runtime 0.16.101]

### Fixed

- Remove generated output symlinks without following them or changing their
  targets during compiler cleanup.

## [Toolchain 0.31.112, language 0.23.105, runtime 0.16.101]

### Fixed

- Keep Compact circuit and type names in generated Rust without emitting Rust
  naming warnings from the generated modules.

## [Toolchain 0.31.111, language 0.23.105, runtime 0.16.101]

### Fixed

- Lower Set and Map operation arguments at their declared types in the Rust
  backend, so direct Vector keys compile and match TypeScript ledger state.

## [Toolchain 0.31.110, language 0.23.105, runtime 0.16.101]

### Added

- Add `compactc --target rust` with a standalone Cargo contract crate and
  repeatable target selection for Rust and TypeScript.
- Include generated Rust files in the contract manifest and check the public
  compiler target from a separate Cargo consumer in CI.

### Fixed

- Use the default Darwin Chez toolchain to avoid an LLVM/libc++ version
  mismatch.

## [Toolchain 0.31.109, language 0.23.105, runtime 0.16.101]

### Fixed

- Code generation for certain casts.

## [Toolchain 0.31.108, language 0.23.105, runtime 0.16.101]

### Added

- Add `ecNeg` to the standard library for JubJub point negation.

## [Toolchain 0.31.107, language 0.23.104, runtime 0.16.101]

### Fixed

- Fix [issue #456](https://github.com/LFDT-Minokawa/compact/issues/456), a ZKIR
  v2 bug in Schnorr signature verification.  This change also fixes a bug in
  Schnorr signature verification for the experimental ZKIR v3 backend.

### Internal notes

- The Schnorr signature verification feature is unreleased (added in toolchain
  0.31.104).

## [Toolchain 0.31.106, language 0.23.104, runtime 0.16.101]

### Added

- The compiler now writes a manifest to the file `contract-manifest.json` in the
  `compiler` subdirectory of the target directory.  The manifest contains sizes
  and sha256 sums for each of the generated files except `contract-manifest.json`
  itself.

### Changed

- The compiler now removes and recreates the `contract` subdirectory of target
  directory.  While previous versions removed and recreated the `compiler`, `zkir`,
  and `keys` directory they left the `contract` subdirectory _and its contents_
  in place and instead replaced only the target files `index.dts`, `index.js`,
  and `index.js.map`.

- The compiler now always creates the `keys` subdirectory of the target
  directory.  The `keys` directory will be empty, however, if the --skip-zk
  flag is used, the zkir binary isn't found, or none of the contracts circuits
  require proofs.

## [Toolchain 0.31.105, language 0.23.104, runtime 0.16.101]

- The ZKIR v3 format, behind the feature flag `--feature-zkir-v3`, has changed
  so that:
    - it contains an `outputs` field that is a list containing the type of each
      output of the circuit
    - it contains a single `output` instruction that specifies a list of output
      operands and is the last instruction

## [Toolchain 0.31.104, language 0.23.104, runtime 0.16.101]

### Added

- Schnorr signature verification over the JubJub embedded curve, via the new
  `JubjubSchnorrSignature` struct and `jubjubSchnorrVerify` circuit in the
  standard library.

## [Toolchain 0.31.103, language 0.23.103, runtime 0.16.100]

### Added

- Adds `keccak256` to the standard library, with the same signature as
  `persistentHash`.  Adds `keccak256` to the Compact runtime with the same
  signature as `persistentHash`.  `keccak256` requires the experimental feature
  flag `--feature-zkir-v3` to work in a circuit that directly or indirectly uses
  the public ledger state.  It is a compiler error to use it in a such a circuit
  using the ZKIR v2 backend.

## [Toolchain 0.31.102, language 0.23.102, runtime 0.16.0]

### Added

- `eval` and `arguments` which are reserved words in strict mode of JavaScript are now
  added as future reserved words in Compact. Previously Compact accepted a contract using
  these as an identifier, resulting in producing an invalid JavaScript output.

### Fixed

- Lexer matches the convention used by
  ECMAScript (https://tc39.es/ecma262/#sec-names-and-keywords) and
  UAX #31 (https://www.unicode.org/reports/tr31/#Table_Lexical_Classes_for_Identifiers):
  the lexer accepts Unicode `ID_Start` (`Lu Ll Lt Lm Lo Nl`) plus `_` and `$`.
  Previously it accepted all alphabetic charactes which includes some non-`ID-Start`
  characters which are invalid in JavaScript.
  `identifier-subsequent?` now follows Unicode `ID_Continue` (`Lu Ll Lt Lm Lo Nl Mn Mc Nd Pc`).
  Previously it included som non-`ID-Continue` characters.

## [Toolchain 0.31.101, language 0.23.101, runtime 0.16.0]

### Added

Adds `event` and `log` as future keywords that are reserved.

## [Toolchain 0.31.0, language 0.23.0, runtime 0.16.0]

This release includes all changes for compiler versions in the range between
0.30.100 and 0.31.0; language versions in the range between 0.22.100 and 0.23.0;
and Compact runtime versions in the range between 0.15.100 and 0.16.0.

## [Toolchain 0.30.107, language 0.22.101, runtime 0.15.101]

### Fixed

Various zkir operators that can result in assertion failures and thus should
be executed conditionally do not have guards are thus actually executed
unconditionally.  This can result in proof failures for correct transactions.
For example, casting an unsigned integer value to a smaller unsigned type will
always cause the proof to fail when the value is too big for that type, even if
the cast occurs in a branch that is not taken in the Compact code.

The intent is to add guards to these operators in the next version of zkir.
In the meantime, the compiler implements workarounds that arrange to invoke
these operators with inputs that cannot cause assertion failures when the
guard would be false.

The downside of these workarounds is that they can increase the size of the
generated circuit.
The size increase arises from conditional use (i.e., use in the `then` or
`else` part of an `if` statement or expression) of:

- downcasts from Uint types to smaller Uint types,
- downcasts of Field to Uint types,
- conversions of byte vectors to and from fields or unsigned integers,
- conversions of vectors to byte vectors, and
- uses of relational comparison expressions (<, <=, >=, and >) with inputs
  that might be unknown.

If the increase in circuit size is problematic for a particular contract, developers
should consider moving downcasts, conversions, and relational comparisons outside
of `if` expressions where possible until zkir supports the required guards and the
compiler workarounds have been removed.

## [Toolchain 0.30.106, language 0.22.101, runtime 0.15.101]

### Added

- Adds a `ledger` key to `contract-info.json` listing the contract's
  ledger fields. Each entry contains the field name, path index,
  export status, storage kind (Cell, Counter, Map, Set, List,
  MerkleTree, HistoricMerkleTree), and fully-resolved type tree.
  This enables language-agnostic tooling to discover a contract's
  ledger layout from the compiler output alone. Both exported and
  non-exported fields are included since the full layout is required
  to navigate the on-chain state tree and construct initial states.

## [Toolchain 0.30.105, language 0.22.101, runtime 0.15.101]

### Added

- Adds `--line-length` flag to fixup.

### Fixed

- JubjubPoint equality is now component-wise; it previously was reference
  equality.

## [Toolchain 0.30.104, language 0.22.101, runtime 0.15.101]

### Changed

- Renames `doc/lang-ref.mdx` and `compiler/lang-ref-proto.mdx` to
  `doc/compact-reference.mdx` and `compiler/compact-reference-proto.mdx`,
  respectively.  It also adopts some changes from midnight-docs PR changes
  for lang-ref 1.0.

## [Toolchain 0.30.103, language 0.22.101, runtime 0.15.101]

### Changed

- The language reference `doc/lang-ref.mdx` is now been fully revised and
  is completely up-to-date with the Compact Version 1.0 language.  Grammar
  snippets are automatically inserted into the document directly from parser.ss,
  and several changes have been made to the presentation of the grammar to
  make it more readable.

## [Toolchain 0.30.102, language 0.22.101, runtime 0.15.101]

### Changed

- Extends the `for (const i of start..end) stmt` syntax to allow `start` and
  `end` to be references to generic parameters.

## [Toolchain 0.30.101, language 0.22.0, runtime 0.15.101]

- Changes the format of the first argument passed to `convertBytesToUint` in `print-typescript` 
- Improves format of error messages for `convertBytesToUint` and `convertBytesToField`
- Changes the type of `maxval` to `bigint` to avoid JavaScript silently losing precision
  when comparing `x > maxval` for larg `Uint`s

## [Toolchain 0.30.0, language 0.22.0, runtime 0.15.0]

This release includes all changes for compiler versions in the range between
0.29.100 and 0.30.0; language versions in the range between 0.21.100 and 0.22.0;
and Compact runtime versions in the range between 0.14.100 and 0.15.0.

## [Unreleased toolchain 0.29.114, language 0.21.101, runtime 0.14.102]

### Changed

- The language reference `doc/lang-ref.mdx` is now largely up-to-date with
  the Compact 0.21.0 language.
- The HTML version of the formal grammar in `doc/Compact.html` has been
  replaced with a markdown (mdx) version in `doc/compact-grammar.mdx`.

### Added

- A list of Compact's keywords and reserved words, including those reserved
  for future use, is given in `doc/compact-keywords.mdx`.

## [Unreleased toolchain 0.29.113, language 0.21.101, runtime 0.14.102]

### Changed

- It is now a compiler error to pass Compact values containing opaque JS values
  (`Opaque<'string'>` or `Opaque<'Uint8Array'>`) to the standard library
  circuits `persistentHash` and `persistentCommit`.  Hashing such values does
  not work in circuit due to the representation of these types.  Previously,
  such code would crash the `zkir` process if it tried to generate prover and
  verifier keys.  Now it is a compiler error instead.
  
  This also affects the standard library operation `merkleTreePathRoot` (because
  it calls `persistentHash` in its implementation), and ledger `MerkleTree`
  insertion operations, because they implicitly use `persistentHash`.
  
  This is a **breaking** change because the error is signaled early, and so it
  is now an error to use any of these circuits or ADT operations, even for
  circuits that don't need prover and verifier key generation which would
  compile successfully before.

## [Unreleased toolchain 0.29.112, language 0.21.101, runtime 0.14.102]

### Changed

- The fixup tool now replaces references to the old standard-library type names
  `CurvePoint` and `NativePoint` with `JubJubPoint`.  It also does a better job
  of renaming standard-library circuits when it is safe to do so and explaining
  why when it is not safe to do so.

### Internal notes

- The expand-modules-and-types code for function lookup is more modular and
  easier to read.

## [Unreleased toolchain 0.29.111, language 0.21.101, runtime 0.14.102]

### Fixed

- The `<=` and `>` operand evaluation order in the proof circuit is incorrect
  (right-to-left rather than left-to-right).  It also differs from the evaluation
  order in the generated JavaScript code, which can result in proof failures
  when the operands are non-trivial.  This fix modifies the common upstream path
  `infer-types` to enforce the correct evaluation order.

## [Unreleased toolchain 0.29.110, language 0.21.101, runtime 0.14.102]

### Fixed

- There was an unreleased bug in ZKIR circuits (not in JS) where the
  representation of the default `JubjubPoint` was wrong.  Fixing this entailed
  allowing `default` in compiler IR from `Lflattened` and downstream in both
  ZKIR v2 and v3 backends.

## [Unreleased toolchain 0.29.109, language 0.21.101, runtime 0.14.102]

### Changed

- The compiler binary can now report `--ledger-version` (and
  `--feature-zkir-v3 --ledger-version`).  This is the version of the ledger that
  is targeted by the generated code and used to produce the generated prover and
  verifier keys.

## [Unreleased toolchain 0.29.108, language 0.21.101, runtime 0.14.102]

### Fixed

- Type declarations of `Uint<n>` and `Uint<0..n>` where `n` is a free type variable
  are now accepted by the compiler.

## [Unreleased toolchain 0.29.107, language 0.21.101, runtime 0.14.102]

### Changed

- The ZKIR v3 format, behind the feature flag `--feature-zkir-v3`, has changed
  so that:
  - circuit inputs are correctly typed as either `Scalar<BLS12-381>` or
    `Point<Jubjub>` (before they were always scalars, with `encode` instructions
    for curve points), and
  - `private_input` and `public_input` instructions are typed (before they
    always read scalars, with `encode` instructions for curve points)

## [Unreleased toolchain 0.29.106, language 0.21.101, runtime 0.14.102]

### Changed

- The Compact compiler now targets `midnight-ledger` version 8.0.0.  The Compact
  runtime now imports `onchain-runtime-v3` (instead of `-v2`) at version
  compatible with 3.0.0-rc.2.

## [Unreleased toolchain 0.29.105, language 0.21.101, runtime 0.14.101]

### Fixed

- [Breaking Change] The search order for include and external module files
  specified with non-absolute paths has been fixed so that (a) the compiler looks
  first relative to the directory of the including or importing file, and (b)
  the compiler does not automatically look in the directory where the compiler
  was invoked.

### Added

- compactc and fixup-compact support two new options: --compact-path to
  set the compact path and --trace-search to cause the compiler to say where
  it looks for include and external module files.  If the `--compact-path`
  command-line option is present, the environment variable `COMPACT_PATH`
  is ignored.

## [Unreleased toolchain 0.29.104, language 0.21.101, runtime 0.14.101]

### Added

- The generated TypeScript now includes a `ProvableCircuits<PS>` type and a
  `provableCircuits` field on the `Contract` class.  `ProvableCircuits` contains
  only the circuits that have verifier keys (i.e., circuits that appear in the
  flattened circuit IR and produce ZKIR files).  This distinguishes them from
  impure circuits that only call witnesses without touching the ledger.

### Fixed

- `setOperation` is now emitted only for provable circuits (those in
  `proof-circuit-name*`) rather than for all impure circuits.  Previously,
  witness-only impure circuits caused the runtime to look
  for a verifier key that does not exist.

## [Unreleased toolchain 0.29.103, language 0.21.101, runtime 0.14.101]

### Changed

- The standard library type `NativePoint` has been removed.  The standard
  library type `JubjubPoint` is now a `new type` alias for
  `Opaque<'JubjubPoint'>`.  This way `Opaque<'JubjubPoint'>` isn't really
  hidden, but it's not shown in error messages.
- `NativePoint` circuits in the standard library and the corresponding
  same-named functions in the Compact runtime have been renamed, and they now
  take or produce `JubjubPoint` values.
  - `nativePointX` -> `jubjubPointX`
  - `nativePointY` -> `jubjubPointY`
  - `constructNativePoint` -> `constructJubjubPoint`
- Signatures of elliptic curve operations in the standard library now use
  `JubjubPoint` in place of `NativePoint`.

### Internal notes

- The `compact fixup` tool can do these renamings except it cannot currently
  rename types (e.g. `NativePoint` to `JubjubPoint`).

## [Unreleased toolchain 0.29.102, language 0.21.100, runtime 0.14.100]

### Added

- There is a new builtin type `Opaque<'JubjubPoint'>`.  Unlike the other opaque
  types, this is intended to be a crypto backend (ZKIR) native type (not a JS
  type).  The standard library exports the type `JubjubPoint` which is a
  (transparent) `type` alias for the opaque type.

### Changed

- The standard library's (opaque) `new type` alias `NativePoint` now has
  underlying type `Opaque<'JubjubPoint'>`.
- The Compact runtime's types `CompactTypeNativePoint` and `NativePoint` are
  renamed to `CompactTypeJubjubPoint` and `JubjubPoint`.
- The runtime has TS (instead of Compact) implementations of the now-builtin
  `NativePointX` and `NativePointY` circuits.
- The feature flag `--zkir-v3` is changed to `--feature-zkir-v3` to fit a
  proposed standard naming convention, and to make crystal clear that it is
  still an experimental feature.

### Internal notes

- When the flag `--feature-zkir-v3` is enabled, `Opaque<'JubjubPoint'>` is
  represented natively in ZKIR v3.  Without the flag, it is still represented as
  a pair of field elements in ZKIR v2.
- This is implemented as a "pseudo"-alignment tag after flattening.  The tag
  looks like `(anative "JubjubPoint")` and it's interpreted as a `midnight-zk`
  JubjubPoint for ZKIR operations, converted to a pair of field values for
  the Impact code embedded in the ZKIR circuit.
- ZKIR v3 has new `encode` and `decode` gates for converting from ZKIR
  representations to Impact representations and back.
- ZKIR v3's `ec_add` has been eliminated; regular `add` is polymorphic,
  operating on either a pair of scalars or a pair of Jubjub curve points.
- ZKIR v3 has type annotations on circuit inputs and on `decode` instructions.
- ZKIR v3 has two types: `Scalar<BLS12-381>` and `Point<Jubjub>`.
- For both ZKIR v3 and ZKIR v2 modes, the JS representation of is still as a pair
  of field elements.

## [Unreleased toolchain version 0.29.101, language version 0.21.0]

### Changed

- In the formal grammar, the `stmt0` grammar production for one-armed
  `if` expressions has been removed.  It was unnecessary and made the grammar
  ambiguous.

## [Unreleased toolchain 0.29.100, language 0.21.0]

### Changed

The compiler binary can now report `--runtime-version`, the version of the
Compact runtime JS package that it will import in generated contract code.

## [Toolchain version 0.29.0, language version 0.21.0]

This release includes all changes for compiler versions in the range
0.28.100 and 0.29.0; and language versions in the range 0.20.100 and
0.21.0.  It uses Compact runtime 0.14.0 and on-chain runtime
compatible with 2.0.0.

## [Unreleased compiler 0.28.109, language 0.20.102]

### Fixed

- The fixup tool fixup-compact.ss failed to look for include files and modules
  relative to the directory of the source pathname.

## [Unreleased compiler 0.28.108, language 0.20.102]

### Removed

- The syntax for external circuits, i.e., circuit definitions with no body,
  has been removed.  This syntax was used exclusively for declaring built-in
  natives and was not useful outside of the compiler.

### Internal notes

- The compiler now injects natives directly into the standard library module.
  This is simpler and gives us a single source of truth for natives.

## [Unreleased compiler 0.28.107, language 0.20.101]

### Fixed

- An issue that caused transactions involving `mintShieldedToken`, `sendShielded`, `mintUnshieldedToken`, or
  `sendUnshielded` to fail validation with `RealUnshieldedSpendsSubsetCheckFailure` when the caller was also the 
  recipient of the newly minted token.

## [Unreleased compiler 0.28.106, language 0.20.100]

### Fixed

- An issue that caused the compiler to take an excessive amount of time to compile
  certain `for` loops, `fold` expression, and `map` expressions.

- An bug that caused the compiler to miss some of certain repeated disclosures
  of a witness value and to overstate the nature of certain other disclosures.

### Changed

- Messages about undeclared witness-value disclosures are now produced in an order
  that attempts, for each disclosure point and witness value, to put the most severe
  disclosures along the shortest paths first, since understanding these is easier
  and properly declaring them often addresses the others.

### Internal notes

- The underlying issue was the representation and maintenance of paths in the
  witness-protection program, and this has been replaced by a simpler mechanism
  with some careful crafting of the code to reduce computational complexity and
  generally make the compiler more efficient.

## [Unreleased compiler 0.28.105, language 0.20.100]

### Added

- The file compiler/contract-info.json that compactc generates in the output
  directory now includes some extra information: (1) version strings for the
  compiler, language, and runtime, and (2) for each circuit, a flag saying whether
  the circuit requires a proof (and therefore whether compactc has produced zkir
  code and prooving keys for it in the zkir and keys subdirectories of the output
  directory).

- ARM Linux artifact is added.

### Internal notes

- Adding the proof flag involved moving the pass that saves the contract-info file
  later in the compiler.  This in turn uncovered a couple of bugs in the preliminary
  handling of (as yet unsupported) cross-contract calls.  These have been fixed,
  though the code remains largely untested.  The zkir passes now recognize
  cross-contract calls and explicitly reject them as unsupported.

## [Unreleased compiler 0.28.104, language 0.20.100]

### Fixed

- A bug reported in issue [#34](https://github.com/LFDT-Minokawa/compact/issues/34) in which 
  `ChargedState` was not properly copied resulting in junk metadata being passed to contract deployments.

## [Unreleased compiler 0.28.103, language 0.20.100]

### Fixed

- A bug in the experimental `--zkir-v3` feature.  The on-chain representation of
  coin commitments changed between ledger version 6.1 and 6.2.  The domain
  separator string is changed, and the inputs to `persistentHash` are in a
  different order.

  This was already implemented for the default ZKIR v2, but the corresponding
  change was not implemented in the ZKIR v3 compiler passes.

## [Unreleased compiler 0.28.102, language 0.20.100]

### Changed

- For any circuit that returns something other than `[]` and for which some path through
  the circuit does not end in `return` form or ends in a `return` form without
  a return-value expression, the resulting error message now clearly states that
  this is the problem.

## [Unreleased compiler 0.28.101, language 0.20.100]

### Added

- Added a constructor, `constructNativePoint`, for `NativePoint` values

### Changed

- Renamed the existing accessors `NativePointX` and `NativePointY` to `nativePointX`
  and `nativePointY` for consistency with our conventions for circuit names.

## [Unreleased compiler 0.28.100, language 0.20.0]

There are no user-visible changes.

### Internal notes

- Instead of pulling test contracts from the separate (private) repository
  `midnight-contracts`, they are added to this repository under
  `test-center/test-contracts`.
  
## [Unreleased compiler 0.28.100, language 0.20.0]

### Changed

- The informal parser rule that "else" clauses belong to the innermost "if"
  expression is now explicit in the grammar.  Previously, we were relying on a
  shaky assumption about how the parser generator treats grammar ambiguities.
  This change is reflected in the formal grammar specification in doc/Compact.html
  but has no impact on how programs are compiled.

## [Compiler version 0.28.0, language version 0.20.0]

This release includes all changes for compiler versions in the range 0.27.100
(inclusive) and 0.28.0 (exclusive); and language versions in the range 0.19.100
(inclusive) and 0.20.0.  It uses compact-runtime 0.14.0-rc.0 and 
on-chain runtime 2.0.0-alpha.1.

## [Unreleased compiler version 0.27.113, language version 0.19.103]

### Changed

- The formatter's handling of several forms has been improved:
  - When the signature of a function needs to be broken up into multiple lines,
    the parameter list is also broken up into multiple lines (even if it would itself
    fit on one line), and the return-type declaration appears on a line following
    the last parameter declaration. This change applies to circuit definitions,
    external declarations, witness declarations, the constructor, and anonymous
    circuit definitions.
  - When a call expression needs to be broken up into multiple lines, the argument
    list is also broken up into multiple lines (even if it would itself
    fit on one line), and the closing parenthesis of the call appears on a line
    following the last argument expression.
  - When an anonymous circuit needs to be broken up into multiple lines, the body
    of the circuit is indented a few spaces in from the start of the parameter
    list rather than all the way out beyond the circuit's signature.
  - When the "else" expression of an "if" expression is itself an "if" expression,
    the inner "if" expression begins on the same line as the "else" and appears at
    at the same level of indentation as the outer "if" expression, in a case-like
    structure.  This special treatment is inhibited by end-of-line comments between
    the outer "else" keyword and the inner "if" keyword.

- The formatter now accepts a --line-length <n> parameter that sets the target line
  length to <n>.  The default line length currently defaults to 100.  The target line
  length can be exceeded in cases where the formatter considers the portion of input
  to be fit on a line to be unbreakable.

### Internal notes

- Configuration parameters have been collected into a single new library, (config-params)

- The formatter line length is now a configuration parameter, set to 100 by default.

- compiler/go now catches keyboard interrupts while running the tests and aborts the tests.

- compiler.md now more accurately describes the composition of the token stream.

- The formatter improvements are supported by the following changes:
  - add-block (appropriately renamed make-Qblock, since it returns a block)
    has been simplified to take a header rather than a proc that produces a header
  - make-Qsep has been split into two routines, one that expects a closer and one
    that doesn't.
  - make-Qsep and make-Qconcat now take an inherit-break? flag whose value is
    recorded in the resulting Qconcat record.  Processing a Qconcat with this
    flag set in the context in which lines are being broken causes the contents
    of the Qconcat itself to be broken into multiple lines.  The contents of a
    Qconcat q with this flag set are still indented relative to q.
  - The code for handling function signatures is now commonized into a single
    constructor make-Qsignature.

## [Unreleased compiler version 0.27.112, language version 0.19.103]

### Changed

- The Compact standard library structure type NativePoint (nee CurvePoint)
  is now a nominal type alias for an unexported internal type.  The standard
  library also now exports two new circuits, NativePointX and NativePointY,
  that can be used to access the x and y coordinates of a native point as Fields.
  This is a breaking change because the internal representation of NativePoint
  is no longer exposed.

- In type errors produced by the Compact compiler, Nominal type aliases are
  now shown simply as TypeName rather than as TypeName=Type.

## [Unreleased compiler version 0.27.111, language version 0.19.102]

### Changed

- Changes `CurvePoint` to `NativePoint`

## [Unreleased compiler version 0.27.110, language version 0.19.101]

### Changed

- Fixes PM-19299 by having `createZswapInput` and `createZswapOutput` return
  an empty array to represent the `[]` type in Compact.

## [Unreleased compiler version 0.27.109, language version 0.19.101]

### Changed

- The compiler now targets ledger version 7.0 instead of 6.2.  There are no API
  changes between 6.2 and 7.0 so it is only necessary to pull in a new
  implementation of the on-chain runtime and bump version numbers.  This is
  **not** a breaking change.

## [Unreleased compiler version 0.27.108, language version 0.19.101]

### Added

- The reserved words from TypeScript and JavaScript are now included in our
  future reserved words.

## [Unreleased compiler version 0.27.107, language version 0.19.100]

### Changed

- The compiler now targets ledger version 6.2 instead of 6.1.  This ledger
  version has changes to Zswap hashing made in response to ledger audit
  feedback.

- There are standard library changes to **non-exported** structs and circuits,
  so this is **not** a breaking change.

## [Unreleased compiler version 0.27.106, language version 0.19.100]

### Fixed

- Bugs in unreleased code preventing proper behavior of type aliases for certain
  uses of ADT types, including ledger operations that treat parameters of type
  QualifiedCoinInfo differently and the += and -= operators for incrementing
  Counters.

## [Unreleased compiler version 0.27.105, language version 0.19.100]

### Changed

- The compiler no longer generates zkir code or proving keys for circuits that
  do not directly touch the ledger.  Previously, it generated zkir code and
  proving keys for all impure circuits, so merely calling a witness or invoking
  one of the witness-like external circuits (`ownPublicKey`, `createZswapInput`,
  `createZswapOutput`) would also trigger zkir and proving-key generation.

## [Unreleased compiler version 0.27.104, language version 0.19.100]

### Fixed

- The compiler now rejects programs whose constructors contain array-reference,
  and bytes-reference, and slice expressions with out-of-bounds indices.
  Previously, such errors could lead to these expressions producing undefined
  values at run time.

## [Unreleased compiler version 0.27.103 language version 0.19.100]

### Added

- Compact now supports the definition of type aliases:
  Structually typed aliases:
    `type Name = Type;` defines `Name` to be an alias for `Type`.  For example,
    `type U32 = Uint<32>` defines `U32` to be the equivalent of and interchangeable
    with `Uint<32>`.

  Nominally typed aliases:
    `new type Name = Type;` is similar, but `Name` is defined as a distinct type
    compatible with `Type` but neither a subtype of nor a supertype of `Type` or
    any other type.  It is compatible in the senses that (a) values of type `Name`
    can be used by primitive operations that require a value of type `Type`, and
    (b) values of type `Name` can be explicitly cast to and from type `Type`.
    For example, within the scope of `type V3U16 = Vector<3, Uint<16>>`, a value
    of type `V3U16` can be referenced or sliced just like a vector of type
    `Vector<3, Uint<16>>`, but it cannot, for example, be passed to a function
    that expects a value of type `Vector<3, Uint<16>>` without an explicit cast.

    When one operand of an arithmetic operations (e.g., `+`) receives a value
    of some nominally typed alias T, the other operand must also be of type T,
    and the result is cast to type T, which might cause a run-time error if the
    result cannot be represented by type T.

    Values of some nominally typed alias T cannot be directly compared (using,
    e.g., `<`, or `==`) with values of any other type without an explicit cast.

  Both types of aliases can take type parameters, e.g.:
  `type V3<T> = Vector<3, T>`
  `new type VField<#N> = Vector<N, Field>`

  This is a breaking change due to the reservation of the `new` and `type` keywords.

### Changed

- Out-of-range constant Bytes value indices are now detected earlier in the
  compiler, which means that additional such errors might be caught, specifically
  those in code that is later discarded.  This is a breaking change.

- Upward casts no longer prevent tuple references and slices from recognizing
  constant indices, which allows more programs with references to non-vector tuple
  types to pass type checking.

### Fixed

- A bug that caused a misleading source location to be reported for some type
  errors, e.g., for invalid arguments to some calls to `map` and `fold`.

### Internal notes

- The Public-ledger ADT (`public-adt`) form, which describes the type of a
  public-ledger ADT, has been replaced by a new Type `tadt` throughout the compiler.
  This simplifies and regularizes the representation of types and allows type
  aliases to be used for ADT types as well as for non-ADT types.

- Equality testing in the unit-test framework has been tightened up to avoid
  false positives when the expected output uses different symbols to represent
  what turns out to be the same id or gensym in the actual output.  This can
  occur when the expected output is wrong or the compiler actually generates
  code that uses the same id or gensym for different purposes.  Several instances
  of the first have been fixed in the unit tests.

- A new checker, `pass-returns`, as been added to the unit-test framework.  It
  is like `returns` but checks the output of a specific pass.  This is intended
  to allow us to move toward having a single occurrence with checks for multiple
  passes rather than having to put multiple copies of the same test in different
  test groups.

- A new form `(assertf expr format-string arg ...)` has been added to utils.ss.
  Like `(assert expr)`, it returns the value of `expr` if `expr` evaluates to a
  true value and raises an exception if `expr` evaluates to #f.  Its error message
  includes the source location of the `assertf` form, as with `assert`, and also
  the result of applying `format` to `format-string` and `arg ...`.  `assertf`
  is useful in preference to `assert` when the assertion expression does not
  already indicate the problem and the problem is not otherwise obvious from the
  context.

- internal-errorf now also includes the source location in the error message.

## [Unreleased compiler version 0.27.102, language version 0.19.0]

### Changed

- The unique variable names in the ZKIR v3 output are now produced in such a way
  that they are stable in the face of changes in the order or set of circuits
  generated.  That is, if the generated zkir for a circuit doesn't otherwise
  change, the variable names should also be identical.

### Internal notes

- Running the unit tests in test.ss now produces the file replacement-results.ss
  containing one entry for each result that differs from the expected result,
  e.g., each returns form when the returned result is different, each oops
  form when the condition is different, each output-file result with the
  output is different, etc.  No entry is included for unexpected exceptions,
  e.g., no entry is included for a return form if an exception occurs instead.
  If replacement-results.ss would be empty, it is deleted and not created.
  The new program compiler/update-test.ss takes as input the pathname of the
  test file (usually compiler/test.ss), the pathname of the replacements file
  (usually replacement-results.ss), and the pathname of an output file (e.g.,
  /tmp/test.ss).  Bad things will happen if the output pathname identifies that
  same file as the input pathname.  update-test.ss applies the replacements in
  the replacements file to the input file and puts the result in the output file.
  The output file can then be manually copied over the input file.  This is useful
  primarily when making cosmetic changes that affect a large number of tests and
  only after spot-checking to make sure that the cosmetic change is doing no harm.

## [Unreleased compiler version 0.27.101, language version 0.19.0]

### Changed

- The ZKIR v3 format (behind the feature flag --zkir-v3) is changed to coalesce
  an Impact instructions encoding into a guarded array.  Previously they were
  multiple unguarded instructions followed by a guarded "skip" instruction.

## [Unreleased compiler version 0.27.100, language version 0.19.0]

### Fixed

- Use of `return` statements among the statements comprising the body of a `for`
  loop are not supported.  Previously, such uses resulted in strange run-time
  behavior or confusing compile-time error messages.  The compiler now explicitly
  flags such uses as static errors with an appropriate error message.

## [Compiler version 0.27.0, language version 0.19.0] - Branched 2025-11-19

This release includes all changes for compiler versions in the range 0.26.100
(inclusive) and 0.27.0 (exclusive); and language versions in the range 0.18.100
(inclusive) and 0.19.0.

## [Unreleased compiler version 0.26.121 language version 0.18.103]

### Changed

- Changed the intermediate languages leading up to Lexpr to reflect that circuit
  and constructor bodies must be blocks rather than arbitrary statements.  reworked
  hoist-local-variables to avoid a dependency on a fluid variable.  These are not
  user-visible changes.

## [Unreleased compiler version 0.26.120 language version 0.18.103]

### Changed

- Changed the (experimental, not yet announced) ZKIR v3 format to use symbolic
  names instead of indexes for instruction inputs and ouputs.

## [Unreleased compiler version 0.26.119 language version 0.18.103]

### Fixed

- The type checker was not raising an exception for casts from Bytes<0> values
  to Field or Uint values and vice versa, which led to confusing downstream errors
  in some cases.

## [Unreleased compiler version 0.26.118 language version 0.18.103]

### Added

- Four new kernel operations, `mintUnshielded`, `claimUnshieldedCoinSpend`, `incUnshieldedOutputs`, and
  `incUnshieldedInputs`.
- Eight new standard library functions, `mintUnshieldedToken`, `sendUnshielded`, `receiveUnshielded`,
  `unshieldedBalance`, `unshieldedBalanceLt`, `unshieldedBalanceGte`, `unshieldedBalanceGt`, `unshieldedBalanceLte`.

### Changed

- Updates the repository to use ledger `6.1.0-alpha.5`, i.e., `@midnight-ntwrk/onchain-runtime-v1` version `1.0.0-alpha.5`.
- Changes names like `QualifiedCoinInfo` and `CoinInfo` to be `QualifiedShieldedCoinInfo` and `ShieldedCoinInfo` to
  match the names in the new on-chain runtime.
- Renames standard library functions to distinguish between shielded and unshielded token utilities.

## [Unreleased compiler version 0.26.117 language version 0.18.102]

### Fixed

- A bug in which types other than tuple, vector, and bytes do not result in an internal
  error when checking the bounds of an index.  This was an unreleased bug, that is,
  the bug was created in an unreleased version of the compiler.

## [Unreleased compiler version 0.26.116 language version 0.18.102]

### Fixed

- A bug in which unimported modules enclosed in unimported modules are not processed
  to detect and report certain errors, including type errors.  While it is
  essentially harmless not to process unimported modules since code in unimported
  modules is never run, this fix potentially allows some issues to be detected
  earlier in the application development process.

## [Unreleased compiler version 0.26.115 language version 0.18.102]

### Fixed

- A bug in which the compiler sometimes mentioned the same incompatible function
  more than once in the error message produced when no function with compatible
  generic or run-time parameters is found at a call site.

## [Unreleased compiler version 0.26.114 language version 0.18.102]

### Changed

- The maximum representable unsigned integer has been reduced from the maximum value
  that fits in the number of _bits_ in a field to the maximum value that fits in the
  number of _bytes_ in a field.  This change is necessary because values that do not
  fit in the number of bytes in a field do not have a valid representation in the
  ledger.  Given that the maximum field value at present is between 2^254 and 2^255,
  the number of whole bytes representable by a field is 31, and the maximum unsigned
  value is (2^8)^31-1 = 2^248-1.

  This is a breaking change because programs that used unsigned integers between
  2^248 (inclusive) and 2^254 (exclusive) will no longer compile.  Though while they
  would previously have compiled, they would not necessarily have worked properly.

## [Unreleased compiler version 0.26.113 language version 0.18.101]

### Fixed

- A bug in which some obviously unreachable statements were not being reported as such.
  This should be considered a breaking change since some programs that previously compiled
  will no longer compile due to this fix.

## [Unreleased compiler version 0.26.112 language version 0.18.101]

### Changed

- `Uint` range end points are now exclusive rather than inclusive to match the
  range syntax for `for` ranges.  That is, `Uint<0..n>` is now interpreted as the
  set of all unsigned integers in the range 0 through `n-1`, e.g., `Uint<0..3>`
  represents the set {0, 1, 2} rather than the set {0, 1, 2, 3}.

- The runtime version has been bumped to 0.10.2.

- when passed the `--update-Uint-ranges` flag, `fixup-compact` now adjusts the
  end point of each Uint whose size is given by a range with a constant end point
  and issues a warning for each Uint whose size is given by a range when the end
  point is a generic-variable reference.

## [Unreleased compiler version 0.26.111 language version 0.18.100]

### Fixed
- A bug in which Compact enums were generated as CJS enums instead of ESM enums. Previously, `index.js` might contain:

  ```javascript
  var Status;
  (function (Status) {
  Status[Status['Pending'] = 0] = 'Pending';
  // ...
  })(Status = exports.Status || (exports.Status = {}));
  ```

  for an enum `Status`. Now, `index.js` contains:

  ```javascript
  export var Status;
  (function (Status) {
    Status[Status['Pending'] = 0] = 'Pending';
    // ...
  })(Status || (Status = {}));
  ```

## [Unreleased compiler version 0.26.110 language version 0.18.100]

### Fixed
- An unreleased bug that was created during putting bounds on vectors/tuples/bytes

## [Unreleased compiler version 0.26.109 language version 0.18.100]

### Fixed

- A bug that could cause ledger operations or witness calls occurring
  in the test part of an `if` expresssion not to be reflected in the
  generated zkir circuit.

## [Unreleased compiler version 0.26.108 language version 0.18.100]

### Fixed

- A bug in unreleased code that caused an internal error message
  about an invalid source object.
- Internal language version is now properly bumped to 0.18.100.

## [Unreleased compiler version 0.26.107 language version 0.18.1]

### Fixed

- A bug that allowed const statements binding patterns or multiple variables
  to appear in a single-statement context, e.g., the consequent or alternative
  of an `if` statement.

## [Unreleased compiler version 0.26.106 language version 0.18.1]

### Added

- Selective module import and renaming, e.g.:
    `import { getMatch, putMatch as $putMatch } from Matching;`
      imports `getMatch` as `getMatch`, `putMatch` as `$putMatch`
    `import { getMatch, putMatch as originalPutMatch } from Matching prefix M$;`
      imports `getMatch` as `M$getMatch`, `putMatch` as `M$originalPutMatch`
  The original form of import is still supported:
    `import Matching;`
      imports everything from `Matching` under their unchanged export names
    `import Matching prefix M$;`
      imports everything from `Matching` with prefix M$

### Fixed

- A bug that sometimes caused impure circuits to be identified as pure
