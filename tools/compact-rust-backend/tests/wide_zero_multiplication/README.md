<!-- This file is part of Compact. -->
<!-- Copyright (C) 2026 Midnight Foundation -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Wide-zero multiplication consumers

The complete source is `examples/rust_backend/wide_zero_multiplication.compact`.
Run `check_wide_zero_multiplication.py` with an original compiler and explicit
TS runtime to qualify fresh, unedited Rust/TS emission. The gate requires six
consumer tests, both ordinary compile-fail controls and their positive twin.

The fixtures cover both operand orders at 0, 1, 2^128 and 2^200−1; aliases,
zero-typed parameters, exact cast errors and distinct first-error order; native,
recorded and both functional routes; Counter transition and transcript replay;
fallible witnesses observed once after the Counter, private state/transcript;
and nonzero data rejected by the zero singleton at an untrusted decode boundary.

`capture.mjs` runs the genuine wrapper, checks the exact runtime error class and
cast value/maximum/family, and exercises wrong-family/value/maximum controls.
The Rust consumer independently checks its actual output. A changed TS result
must make that consumer fail. Compile-fail errors must be E0308 at the marked
source expression, with no unrelated diagnostic accepted as success.

Fresh selected ordinary call proofs are a separate maintained gate. Neither
these behavior controls nor parser-summary tests claim cryptographic execution.
The broader stateful nonliteral-zero frontend limitation remains documented in
the slice plan. Existing compatibility tests and full integration gates remain.
