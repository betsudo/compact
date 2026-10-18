# CoPS-003 — Output manifest path handling

Status: local mitigation under qualification; not a release or upstream runtime claim
Date: 2026-10-19
Owner: shared compiler artifact preparation
Issues: [#682](https://github.com/betsudo/compact/issues/682), [#679](https://github.com/betsudo/compact/issues/679)

## Problem and affected use case

The shared Scheme manifest pass tests bare directory names against the caller's
working directory, rather than the generated output root. A normal TypeScript
compile from the repository root can therefore emit `contract/` and `zkir/`
without including them in `compiler/contract-manifest.json`. When keys are
generated, their directory can also be omitted. Nested directory classification
has the same problem: bare child names are resolved against the caller's cwd.

The checksum command separately interpolates pathnames inside single quotes.
Literal quotes need correct shell argument encoding; backslash/newline filenames
also need unambiguous checksum output handling. These concerns exist after the
new Forge helper's safe argument-array/JSON boundary, in shared manifest creation.

The JSON writer also emitted object keys without JSON string escaping. A nested
pathname containing a quote, backslash or newline could therefore produce an
invalid manifest even after path resolution and checksum quoting were corrected.

This is missing or unreliable manifest coverage. It is not evidence of a
successful proof bypass or a demonstrated remote exploit.

## Observed evidence

Using the source manifest pass present at `ba5f053f30acbb74a75528e036ee1f71f4a26f51`:

```sh
compactc --target ts --skip-zk \
  examples/rust_backend/effect_plan_counter_only.compact ./generated-ts
```

From the repository root, output contained `compiler/`, `contract/` and `zkir/`,
while the manifest covered only `compiler/`. The opt-in local-cache path also
generated `keys/` but covered only `compiler/`. Both observations used the same
native compiler executable, SHA256
`226bba75b1604bd32709ddefe0c9c4bc42a06f90e2511e228f1c82d1d14930e0`.
The first command intentionally skips keys; it demonstrates a manifest problem
independent of Forge or key generation.

## Backend and upstream comparison

| Target | Pinned implementation | Finding | Evidence limits |
|---|---|---|---|
| Rust-AST native command | Shared pass at `ba5f053f3`; native `refresh_manifest` | Native Rust publication already rebuilds complete trees, masking the shared omission | Does not fix direct internal Scheme/TS publication |
| TypeScript, default and opt-in local cache | Same shared Scheme pass and executable above | Missing generated directory coverage observed | Two bounded counter compilations, not an SDK/network audit |
| Mainstream ledger8 | [LFDT-Minokawa/compact at `eb72a5ab6085bf9b8564cc80c98c973e04825c2d`](https://github.com/LFDT-Minokawa/compact/blob/eb72a5ab6085bf9b8564cc80c98c973e04825c2d/compiler/manifest-passes.ss) | Both cwd-relative filtering and interpolated checksum command are present in source | Source comparison; upstream runtime not rerun |

The pinned upstream file SHA256 is
`46cbad33712d15be298fd43629b4ccf6e6dc4132de421deac8704a132c0987ef`.
Moving branch names alone are not comparison authority.
The same pinned tree's `compiler/json.ss` contains both the unescaped object-key
writer and the malformed control-character escape; its SHA256 is
`eadefc32393b499a58f8a53911ea98618a46b1d75e888824c75761a7d43a812b`.
This is source evidence for the shared TS path, not an upstream runtime test.

## Mitigation and ownership

Resolve manifest directory existence and nested file/directory classification
against the actual output path. Encode the checksum pathname as one POSIX shell
argument and request zero-terminated checksum output, which keeps the first 64
hexadecimal characters stable for escaped filenames. Retain the original SHA256
algorithm, manifest version fields and generated artifact bytes.
Encode object keys using the existing JSON string-value escaper, retaining the
historical conversion of non-string keys. The inherited string escaper also
formatted otherwise-unhandled U+0000–U+001F controls as `uXXXX` without the required
backslash and passed a character to an integer formatter. Emit standard
`\uXXXX` using the character's integer value. The same existing escaper serves
keys and values; the regression includes U+0001 in both and retains all previous
path, hash and byte-size checks. This does not change number parsing.

Before: `file-exists?("contract")` depends on cwd.
After: `file-exists?(output_root + "/contract")` describes the actual output.

This is a shared compiler fix; it does not belong to the Rust runtime, generated
domain model, Forge cache identity, or ledger cryptography. Correct coverage can
add previously missing manifest entries. Consumers must not interpret that as a
different prover/verifier format. Default cache selection remains off.

## Verification and acceptance

Retain every existing `save-manifest` test and add a focused test under both v2
and v3 parameterizations. Verify actual generated trees, known SHA256 and byte
lengths, a nested directory, a leaf named `compiler` that collides with the
caller's cwd, omitted absent directories, and literal special-character paths.

The public compiler must then pass TS-only, Rust-only and combined-target
manifest checks, output-preserving refusal controls, and unchanged real
proof/application/corrupt-verifier tests. Evidence for a native cache hit does
not establish general upstream or hosted qualification. Final acceptance is
recorded in #682 and the local milestone report after these gates complete.
