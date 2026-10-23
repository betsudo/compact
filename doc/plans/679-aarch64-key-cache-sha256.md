# ARM key-cache hashing (#679)

Status: independently reviewed local macOS functional qualification accepted.
Signed local delivery and exact-commit review precede performance measurement.

## Decision and scope

The accepted complete native DID benchmark achieved a 23.49% warm time reduction
and missed the >50% target. A separate diagnostic observed 1.832 seconds in
fourteen unchanged identity-guard spans. These spans overlap other measurements;
they are neither guaranteed removable time nor an optimization result.

The locked `sha2` 0.10.9 ARM SHA256 path requires its `asm` feature. Enable it only
for aarch64 Linux/macOS. The single optional `sha2-asm` 0.6.4 dependency has an
authenticated crates.io checksum and MIT license; every existing locked package
version/source/checksum remains unchanged. Cargo feature unification also affects
SHA512 and dependent crypto crates in both coordinator and helper. These are new
executable identities, requiring real consumers and new measurements.

Keep every original input/executable/parameter hash and mutation boundary. No
provider protocol, cache default, tiny-circuit threshold, backend, serialization,
recipe, verifier identity or parameter change is part of this candidate. Automatic
caching remains optional and bypasses k <= 8; forced tiny caching exists only in
private qualification controls.

## CPU dispatch and limitations

The selected upstream SHA2 ARM implementation uses CPU dispatch. Linux uses
HWCAP detection; upstream Apple detection assumes SHA256 instructions are present.
This ARM macOS host explicitly advertises SHA256 support. The optional dependency
compiles assembly, but its unguarded wrapper is not the selected SHA2 ARM call path.
A separate `sha2/force-soft` build tests software implementation equivalence. It
does not emulate a CPU without SHA instructions or prove the Linux detection path.
The x86 Linux and ARM Windows metadata controls preserve their prior default/std
features; metadata is not cross-platform build or runtime qualification.

## Actual qualification

Both new Nix Rust 1.92 release builds passed all 22 helper tests, including 18
independent SHA256/SHA512 known-answer vectors, padding/block/64KiB boundaries,
chunked verified copy, and actual-file identity checks. Scheme is byte-identical.

The maintained complete DID gate passed intact and changed source across ordinary,
cold, warm, authenticated restored, and direct Scheme execution: all 53 compiler
files and 12 exports match per variant. Ten fresh selected deactivate proofs and
verifications, 20 negative controls and 60 assertions ran. Original and changed
full snapshots contain 12 ready bundles and remain below 256 MiB. Independent
receipt review accepts this scoped functional result, not every-export proofs,
the full DID API/localnet, Linux, CI or comparative performance.

The corrected Rust/Counter JavaScript matrix has terminal success in five lanes:
uncached, tiny automatic bypass, private forced cold, warm and exact restored.
The initial missing-rustfmt controller failure remains recorded. The new attempt
pins the actual Nix formatter; this changes neither the candidate Rust toolchain
nor any proof obligation. Three post-acquisition fault controls also terminated
successfully: an owned ZKIR executable, parameter, or Forge proxy was mutated only
after a genuine restored hit; the coordinator refused each case while preserving
all 53 previous output files. Independent original-receipt review accepts these
scoped consumer and post-acquisition controls.

Original review found missing initial source/key binding in the unexecuted fault
driver. The corrected driver binds all 459 source files and the complete key
object to the authenticated DID receipt before creating work. Both original and
corrected sources/review outputs are retained. Independent and authentic external
reviews accept the corrected plans; separate independent receipt reviews accept
the actual subsequent execution results.

## Evidence and next measurement

The adjacent evidence index identifies original local receipts by exact hash.
Evidence is retained under `rc2-aarch64-key-cache-sha256` in the initiative's local
evidence directory; none of these local research paths is a runtime dependency.
The coordinator is SHA256 `9fde2a9b7ee71632e7dfba5a365f7fce3a97cd58123ab50cb507752c3739400e`;
the cache helper is `2eb114576c5edbfce9595c100ee470803230a40b1244be8ecf07590eabe10b02`.
A final formatter correction only reorders the same three test-module imports;
production patch sections and all test expressions/vectors remain unchanged from
the actual builds and consumer qualification. The final source-integrity receipt
and successful formatting check retain that distinction.
The actual installed Forge RC2 binary remains `7afd88f9ca6fa499af91c678bb94366f38cdf5da99181fe9ec22e9a69f503a74`.

After final receipt/delivery review and a signed local commit, use the existing
maintained native benchmark for three repetitions of the same intact DID workload.
Each repetition has a separate cold store and rotated ordinary/warm/restored
lanes, complete output equality and fresh selected proofs/assertions/negatives.
Cold includes full snapshot creation; restored includes local authenticated
restore. Report medians and a >50% miss honestly. No unchanged campaign rerun,
proof/corpus reduction, silent trimming, cap increase or check removal is allowed.

Keep this branch local for owner review. Exact current-consumer Linux qualification
is still mandatory before enabling Linux reuse. No hosted trial, push/release,
shared installation, merge, activation, settings change or cache deletion is part
of this delivery. Preserve all earlier accepted sources, binaries and receipts.
