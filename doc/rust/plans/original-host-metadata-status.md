# Original host metadata qualification

The pinned upstream host tests are separate from generated Rust and transaction qualification. The local implementation checkout and the canonical documentation checkout also have different source inventories. The signed coverage audit `audit.json` (SHA-256 `082391e8e7de9afdc006b5b6f1e67f5c9fb9a7bc216d3d000098dd2b6f3a4249`) records the exact heads: accepted `3eafaf3eb` contains all 943 pinned `test-contracts` files byte-for-byte; among 45 E2E test modules, three are exact, 37 adapted and five absent. Presence does not establish execution. Focused receipts do not qualify the complete root lint/test or E2E package. [#727](https://github.com/betsudo/compact/issues/727), [#728](https://github.com/betsudo/compact/issues/728) and [#732](https://github.com/betsudo/compact/issues/732) retain coverage, frontend metadata and host/Linux ownership.

## Current-source expectedVk

The unchanged original five-case verifier-key host body passes at independently reviewed signed `94b97c691f51924fa02640735614ff951c3f6edb`, on a separate branch from accepted `3eafaf3eb`. It uses the private source Scheme profile selecting ZKIR v3, released Compact runtime 0.21 and physical verifier-key metadata. Three circuits have retained real key bytes, generated JavaScript and compiler manifests; the released onchain SDK 4.0.1 setter case passes. Thirty-one focused metadata guards also pass.

The metadata-enabled emitter now uses computed literal keys and immediately sets the table prototype to null. This preserves an own `__proto__` data entry and the null-prototype property while allowing the unchanged original Acorn reader to inspect the literal initializer. The original test and fixture remain byte-identical to the pinned upstream revision. Bare public CLI output is explicitly tested to omit metadata, as before; this slice does not change that default.

Signed `qualification-94b.json` has SHA-256 `06c4a3ce56ae8ed8c390d71cedd54db72854eb71aa1e83fe0e344a75e9d3619a`. The independent review `independent-review-94b.json` has SHA-256 `ad4803846a89ce2494f9c9028a89e5412648ac9782a48a5e056bba8838418cdb`, verifying six source, 85 artifact, ten log and five command hashes. An initial failed Node prototype-check command is retained; its corrected command passes. The older released Compact 0.36.0 five-case receipt remains a separate baseline, not additional current-source coverage.

These are private-profile Darwin arm64 host results. There is no public-default metadata, Rust descriptor, proof/funded or Linux claim. The changed emitter is not integrated into the accepted v4 branch and does not requalify immutable Events v4/v5 or Caller packages.

## Original compiler version and combined gate

Reviewed signed `72b54c22734f59a67163fb6680e37d940070437c` restores the original compiler-commit assertion and original counter fixture. The unchanged case passes on released 0.36.0, private source v3/released0.21 and source default Ledger 8. An unstamped source build reports empty metadata/unknown verbose commit explicitly. Full manifest member/size/hash checks, six tamper controls and six first-exit flag regressions pass. Signed receipt SHA-256 is `c385f895a93608d597cdca45881518913f68299591fdeb9c2811f53f748c5b9e`. Release stamping and the full compiler gate remain separate.

Reviewed signed `414a86ae2a1fc9205fc099f7a353bbe90a11a184` ports five original communications-commitment, zero-bit and secp arithmetic cases. Both profiles pass 5/5. V3 zero-bit descriptors/alignment use one byte and the v3 communications flag follows its parsed option. The feature guards preserve the default Ledger 8 zero-byte descriptor and `[0,8]` hash alignment; the trusted Events no-communications refusal remains. Final toolchain receipt SHA-256 is `f82d053344f7dded78b5c8f500a929198f082029d0bd5ab9c06b3453cbd118a7`. A signed supersession records an earlier receipt-signature bookkeeping correction; source and output were unchanged.

The separate combined branch at reviewed `99d08ec897be6b771fb18232dd614bd4791b0ef5` integrates eight reviewed commits, preserving all helper behaviors and emitter changes. Six unchanged modules pass **21/21 on each of released and private source profiles**: expectedVk 5, version 1, the five cases above and [ECDSA 10](original-ecdsa-parity.md). Eight actual compiler outputs per profile are captured before consumption, including three WPP key pairs. All 13 original test/fixture blobs and 416 SDK files are verified. The original counter hash is used, rather than the adapted default fixture. Signed `qualification-99d-host21.json` SHA-256 is `35f14e82ead10ef4ea01926157d63b5e44b28fb31ec508293c866b1bf15188cb`; independent review SHA-256 is `5e1f3bff344df9f6c7b3626f2e15292800fa2ea47a137e2d059dee4cba451299`.

This is a private-profile local host gate. Compiler metadata changes require new physical Events packages and fresh admission qualification; the frozen v4/v5 identities are not silently reused. The accepted v4 source branch remains distinct. Full root prepare/lint/test, all 45 E2E modules, default fuzzer corpus, remaining editor/formal tooling and native Linux remain open. Owned targets, test dependencies and finished worktrees are cleaned; all final evidence and source refs remain.


## Intact original External module

Reviewed signed harness `bee4994cc0fd7b733572ffd695ece6e047b3d005` executes
the unchanged original External module once per profile: released Compact0.36.0
1/1 and private source99d 1/1. The original JubjubScalar fixture is retained;
the accepted adapted Field fixture is not substituted. Both original assertions
check successful compilation and generated JavaScript syntax. This is host-only
qualification with no Rust, proof, funded or Linux claim.

Original test SHA-256 is
`c7bf7fce6112ad286c31068b9d733ca145247fd301e93789410c332c39e92e1c`;
fixture SHA-256 is
`d8a44938ea1c1384251d387ad9eec3297886ffeb688bc8715a195b3f2230454c`.
Signed qualification SHA-256 is
`fc87aa1fa468fff1464a612037675b4dd87c2c0902200a136d602af5f081b4d2`;
independent evidence review SHA-256 is
`e67ff28694173464b98815edbf6dd0c7c3907a859ba7ca5a7f6a696301af4cef`.
The review checks raw Vitest results, both complete run inventories, exact
source/tool inputs and six same-run compiler captures per profile; it does not
claim an independent Vitest rerun.

The offline 183-package tool graph uses direct Vitest4.1.11, with an explicit
std-env4.2 to4.3 override and other-platform optional packages omitted. It does
not reproduce the entire upstream installation. Full45-module/root lint/test
qualification remains open. All owned tool/stage/run copies and the clean
worktree are removed after review; signed source, raw captures and receipts remain.


## Next intact CamelCase source preparation

Signed local source `bd709e9b868bf810e3e63a9478909e66211c8ccc` preserves the
unchanged two-case CamelCase test and both complete fixtures under
`examples/camelCase/all`, including original Field and JubjubScalar casts.
Source-only staging and trust controls pass with independent review; no Node,
compiler or original-case execution is claimed. PREP SHA-256 is
`ff8e503786287007b63f197a91f72dc7a334c4a3c854e8105b89c1651680cf6f`;
root independent review SHA-256 is
`a5ab4dd05f20f8527bb904e483abac4ded8e7fb2eff9fc45e24f5594130cf1e2`.
An isolated Git index creates the small local source commit without allocating
a worktree or changing existing checkouts. Future execution still needs its
own reviewed tool graph, source/compiler closure, output budget and fresh
50 GiB disk guard.

The six-module host21 gate and single-module External gate establish a union
of seven modules/22 cases per profile across two separate qualifications.
They do not establish one combined22 gate or the full45-module/root lint suite.
The next tool proposal reconstructs 3,000 package files/27 links from 183
checksum-pinned archives, measuring about40.54 MiB of file payload. Projection
and Node execution were unrun at that source-only proposal. The prior439 MiB
whole-tool footprint remains
historical; payload estimates are not measured CI or runtime savings.


## CamelCase projection attempt before original host execution

The signed local capture/executor source at `bd6235243` and `24ff2bbfa`
passes independent source and plan review, including raw compiler streams,
original relative arguments, generated-tree POSIX modes and bounded process
cleanup. A real 7ec projector run creates the expected 3,000 regular files,
572 directories and 27 links from the retained 183 compressed inputs; its
identity matches and is signed. That projection phase takes 103.5 seconds.

The owner then stops the run before Node starts: the runtime-copy helper
repeats full worktree/resource scans for every file, producing avoidable
quadratic work. The partial run remains **aborted and unqualified**, with
zero original CamelCase cases on either profile. Signed abort receipt SHA-256
is `932913f4a858912415d6b36e1f936f7caa0514d54c57abd42491169d51f258a2`.
Raw phases, source inventory, partial-tree map and process cleanup evidence
remain; no other job is interrupted. A new source revision will batch Git
object reads and retain continuous resource monitoring with bounded copy
checks. Its real retry still requires a new reviewed launch plan. No complete
runtime or CI speedup is measured from this partial attempt.

## Efficient retries and original output transport

The efficient executor `833bc7fa5` batches framed Git object reads and checks
whole-copy boundaries while retaining continuous disk and process guards.
Its real projection passes in 96.24 seconds, and copying reaches Vitest
without the earlier per-file full-worktree scans. The released original two
cases then fail before either compiler runs: the input guard rejects the
original logger's data file. This is an instrumentation failure, with zero
host qualification credit. Signed failure receipt SHA-256 is
`0f1d2792aac027676d32f27dff5a79636939b4fb99479be9c9e215dbf32cc179`.

The next exact retry at `d183d86df` preserves the original logger and permits
only its bounded data file and two directories. Released Vitest passes both
unchanged original tests, with two successful compiler calls, empty raw
compiler streams and valid JavaScript. Collection then refuses a transport
mismatch: unchanged Node `fs.cpSync` gives its captured tree a `0755` root,
while the wrapper's original tree has a `0700` root. All five generated files,
their bytes and modes, and both child-directory modes match in each call.
The source profile is unrun, and this partial result does not qualify the
complete host lane. Signed failure receipt SHA-256 is
`eb7343cd128410d8bbcf6cc107621c9f4d72eace6caf60662402cb12b2fa63d4`;
independent failure review SHA-256 is
`51bb4a2fad8ee33e4cf4ed711f5542f082193fa286ec38132c70e6d570681a78`.
A focused capture correction will retain both root modes and validate only
that exact copy-root transformation; other root pairs, child modes, file
bytes and membership remain strict. Original tests, fixtures, assertions,
logging and the bridge remain unchanged.

Both failed attempts retain raw streams and source inventories. Only d183
retains complete generated call and bridge trees; 833 invokes no compiler. Finished owned resources are cleaned after review or explicitly
deferred for the immediate follow-up. The retained 183-archive input mirror
is separate from disposable outputs. The measured framed Git-only read is
1.792 seconds for 9,883 paths; it is not a complete host, physical-copy or CI
speedup measurement. Native Linux and the full 45-module gate remain open.


## Qualified original CamelCase and artifact finalization

Executed signed source `27d94b147126e8296ea0bf1600005a60d80839ae` with capture
`d097eb176` passes both byte-exact original CamelCase tests on each profile:
released Compact 0.36.0 and the private source99d ZKIR-v3/released0.21 compiler.
All four compiler calls exit zero with empty raw streams and valid JavaScript;
both original collectors pass. Full generated file/child modes, the exact
0700-to-0755 copy-root relation, complete closed original logs, source/material
binding and process cleanup pass. Original fixtures, including Field and
JubjubScalar casts, and original assertions remain unchanged.

The restricted source PATH adds only the physically pinned `/sbin/sha256sum`
last, retaining prior precedence and the released profile's PATH. Its GNU
binary/zero-terminated `abc` vector is checked before projection. This corrects
the prior source setup failure; it makes no OS-vendor signature claim.

After both profiles pass, the original runner's final source check raises an
arity TypeError before receipt creation. Signed local correction `d8daf6372`
changes only `source(repo, head)` to `source(head)` and is **not executed**.
A separately signed artifact-only finalizer authenticates the original run's
full 9,145-file/1,789-directory/81-link tree, nine retained phase records,
raw outputs, both pure collectors, source/material/logs/modes and absence of
owned processes. It writes new receipts and transfer maps without rerunning
Node, either compiler or projection, and preserves the original executed head.
Independent final review rehashes the complete original tree after finalization.

Signed final qualification SHA-256 is
`c9dfcfc105ce7908cd850f6374b2827b5fa08c7e308d99410e5d3f48b6fffa42`;
independent actual review SHA-256 is
`faa15e5f4e08cb1e276d2e6ab672834267e8815f08bd12a79a7ec77230c5ac1e`.
The original TypeError and all earlier failed attempts remain retained.

After review and active-use checks, the owned worktree and disposable graph
copies are removed. The 145-file/6,217,271-byte capture retains raw compiler,
Node, generated/bridge, log, phase, source and identity data, including both
134-byte runtime cache reports and their original paths/modes. One authentic
42,504,224-byte pristine graph (3,000 files/572 directories/27 links) is retained
as an explicitly needed input for subsequent original host tests; it and the
183-archive mirror are separate from disposable outputs.

This adds one module/two distinct original cases per profile. With the six-module
host21 and one-module External gates, the union is eight modules/24 cases per
profile across three separate qualifications. It does not establish a combined
24-case gate, the full 45 modules, Runtime SDK execution, generated Rust,
proof/funding, public defaults or native Linux. The private 183-package graph
retains its disclosed std-env 4.2-to-4.3 override and platform omissions;
no exact original installation metadata or controlled CI speedup is claimed.

## Original ADT and exports attempt, 20 November 2026

The environment-clean executor at signed local source
`4ee44a22d799a54cc51fdaa76e7972e9482d41a0` executes the unchanged
original ADT and exports modules with their original fixtures and assertions.
Raw Vitest results show released Compact 0.36.0 passing ADT **27/27** and
exports **7/7**. Private source99d passes **3/27** ADT cases and fails 24;
source exports is unrun because the controller stops at the failed phase.
The first failing source case, `kernel.compact`, reports an unimplemented
VM instruction; later capture failures still require diagnosis and do not
establish 24 independent compiler defects.

The entire new host lane remains **red and unqualified**. These observations
are not added to the accepted host union of eight modules/24 distinct cases
per profile across three earlier, separate gates. The executor removes
ambient variables only at the compiler process boundary; original Node
tests, arguments and assertions remain unchanged. The retained offline graph
still discloses its std-env override and optional-platform omissions.

Raw report SHA-256 values are:

| Phase | SHA-256 |
| --- | --- |
| Released ADT | `33b3bee8197f7ed4342a52b2584f8497a8f01048606281ba01984d759162854a` |
| Released exports | `d6a4452eed6b93f2a4b24d06db8ee124e243faff2a0ef889e06c204857a43dba` |
| Source ADT | `6f9994157d6aed2b8534a66191e88a9f9178541b2ab46f5c9c39cdf69922911a` |

Failed-run captures and disposable outputs are retained for the immediate
source diagnosis. No full 45-module, Rust/proof or Linux qualification is
implied. The earlier failed inherited-environment run remains historical.
