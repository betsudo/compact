# Historical inputs and licensed current sources

The adjacent JSON table binds eleven named current sources to two independent
SHA256 identities: the original historical bytes and the fully licensed current
source. The `.compact.raw` files are literal historical test data from signed
merge720fb4717. They are retained byte for byte, including their original SPDX
notice; the repository's Apache2 license continues to apply. They are not newly
generated compiler output or proof artifacts.

Current `.compact` sources carry the complete license notice after the original
bytes, preserving every original source position. The historical compiler180/
184 refusal tables and their literal hash guards remain unchanged. Header-policy
tests consume the original historical data; current compiler gates consume the
separately pinned fully licensed source and compare exact current diagnostics.

`historical_source_licenses.verify_source` accepts only the explicit current hash
AND the original historical hash; it does not strip comments, normalize bytes,
infer a hash from matching prefixes or accept an arbitrary replacement source.
The original bytes must also be an exact prefix and the remaining suffix must
have the code-literal SHA256 of the one permitted full license notice. These are
additional constraints; prefix equality alone never admits an input. A changed
program body or notice with a coordinated new manifest hash still refuses.
The complete manifest/current/raw inputs are pinned before execution by each
affected gate. Unknown current paths, changed historical or current bytes and
symlinks refuse. New proofs keep their actual current whole source/IR/Program
and immutable verifier identities. No historical proof receipt is relabeled.

The eleven permitted manifest paths are also a code-literal roster. Missing or
extra manifest entries refuse before input admission, including removal of a
row combined with reverting its current source to old bytes. Historical
unchanged sources outside that roster retain their original literal hash check.
