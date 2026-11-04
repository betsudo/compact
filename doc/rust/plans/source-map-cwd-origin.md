# Default source-map invocation-root publication repair

## Current scope and preservation

The outer-Vector semantic branch and backup remain at aa2be69c9. This isolated
local branch, codex/rust-ast-source-map-cwd-origin, reuses the clean worktree;
canonical Rust AST, accepted metadata/component proof refs and deferred Forge
remain unchanged. The broader goal and outer-Vector qualification are incomplete.

The complete original/family compiler189 and190 SDK maps point into removed
staging directories. A separate complete existing Counter source (two exports,
no extraction) confirms that relative source + absolute or parent output fails
already in the original Scheme entrypoint. Relative/relative and absolute-source
controls resolve correctly. Explicit URL roots survive; an escaped explicit root
produces invalid JSON directly and refuses combined publication. All raw outputs
and failed comparisons remain retained; no map has been edited or normalized.

## Gap, owners and public API

compiler/sourcemaps.ss::compute-source-root currently returns an empty root when
source and target paths have no common head or a remaining target contains `..`.
Its unchanged relative source operands then lose their invocation-directory
base. The Rust source_maps relocation helper correctly treats that empty root as
map-relative and therefore preserves the wrong base into final publication.
Repair the Scheme default fallback to retain the invocation cwd explicitly,
without rewriting source operands or guessing a filesystem origin in the Rust
CLIs. Preserve existing computed roots and explicit user overrides.

Use the existing compiler/json.ss path-string serializer for sourceRoot/sources
so quoted, backslash and Unicode values remain valid JSON. Source-map path fields
are the intended changed metadata; IR/origins/Programs/ZKIR/JS bodies and non-path
map values must remain identical at their declared source epochs.

Public APIs and flags are unchanged. compactc/compact-rustc keep staging,
rollback, manifests and final publication ownership. No new wrapper, compiler
option, runtime/schema/ABI change, source canonicalization, conversion or framework.
The retained source strings and map paths do not attest consumed bytes, every
import, adversarial filesystem continuity or a deployed URL source tree.

## Declared controls and acceptance

Retain the18 initial original190 probes and2 additional literal-quote cwd probes
with their exact scopes (the initial quoted-cwd label used Unicode/space only).
Add the actual standalone compact-rustc eight default path probes, using its
maintained Cargo binary and paired original configured frontend. The Nix
compact-rust-cli package installs compactc only; do not fabricate a packaged
compact-rustc identity or alter its installation surface for this repair.

After the repair, execute all28 cases across direct Scheme, combined Rust/TS
and standalone Rust publication. All default maps must parse and resolve to the
exact declared physical full Counter source bytes; explicit roots must retain
their literal intended values (physical binding N/A). Exercise relative/absolute
source operands, absolute/relative/parent outputs, Unicode/space and actual
quoted cwd, URL roots and JSON-escaped root strings.

Preserve all existing source-map unit tests for both Rust CLIs; add a maintained
current-recipe integration gate with absent outputs and raw stdout/stderr, source,
compiler/tool and map/manifest identities. Verify manifests match actual maps.
Run focused local checks, original full compiler recipe, owner-generated version
documentation and all literal version steps. Obtain independent source and
separate receipt review before accepting the bounded publication repair.

Requalify both complete outer-Vector sources/all16 exports without trimming:
strict SDKs, exact independent IR and raw Program/ZKIR/JS/carrier continuity.
Only declared map path fields, version/derived hashes and the previously intended
two190 attributes may differ from the preserved189 outputs. Then resume native/
TS/static/IR/recorded/model/proof/strict-application qualification. The metadata
repair itself supplies no proof acceptance and does not complete those gates.

## Exclusions

No source/geometry/golden normalization, rewritten source operand, proof/result
cache, relaxed exact binding, cap raise, CI optimization project, hosted run,
push/merge/release/shared installation, cleanup or Forge work. No universal
filesystem/deployment/Windows/native-Linux/fullCI/integration claim from local
macOS arm64 controls. Keep the complete required integration/proof gates.

## Standalone manifest finding and implementation

All eight actual standalone190 probes retain stale source-map manifest hashes.
The existing compactc manifest_tree/refresh_manifest bodies are moved unchanged
into bin_support/output_manifest.rs and reused by compact-rustc after map/library
publication changes. Existing main-CLI behavior remains required-manifest; the
standalone entry keeps missing manifests optional and rejects a present malformed
or non-regular manifest before final publication. Version191 owner documentation
is generated through the maintained ledger.ss owner after the original build;
actual publication and semantic/native/proof acceptance remain open.

## Review corrections and CI retention

The first independent source review found two qualification-harness defects:
assuming Cargo's default output path could select a stale executable under a
configured target, and text-mode diagnostic capture could alter CRLF/CR bytes.
The gate now selects the sole binary artifact reported by the current Cargo JSON
build stream for the exact package manifest, then preserves its copied identity.
Child streams are redirected to exclusive binary files before exec while the
existing bounded process-group owner retains timeout/kill ownership. Diagnostics
are interpreted separately from retained bytes. Real-child controls cover raw
undecodable bytes, CRLF/CR, cwd and timeout retention; artifact controls reject
missing, duplicate and wrong-package/non-binary records while a stale default
binary remains present.

The Linux compiler workflow calls the full publication gate using the existing
authenticated compiler and original-recipe recorder; failure and success evidence
are retained. The three new public controls use existing unittest discovery.
All existing compiler, native consumer and proof gates remain required. These
are local workflow-source changes, with no hosted execution or Linux acceptance.
Explicit URL and absolute escaped roots are tested literally; relative explicit
roots retain the existing Rust relocation semantics and are not a new guarantee.

## Current local evidence

Signed implementation bf6b14560 and harness correction0739a65e3 retain all prior
branches and raw outputs. Original compiler191 built from frozen0739a65e3 in
153.17s using the unchanged .#compactc recipe. Independent corrected source review
completed in174.59s with no actionable findings; first-review findings and failed
workflow-control output remain retained. The maintained ledger owner generated
version191 documentation with only its version line changed; its literal version
workflow check passes. Actual publication matrix and separate receipt review
remain prerequisites, followed by the complete outer-Vector semantic gates.
