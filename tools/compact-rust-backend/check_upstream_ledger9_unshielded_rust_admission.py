#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Require both-target emission of the exact original unshielded contract.

This is a generated artifact gate. Proof, funded application, and execution
parity are qualified separately.
"""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "test-center/compact/unshielded-tokens.compact"
ORIGINAL = (ROOT / "tools/compact-rust-backend/fixtures/"
            "unshielded-tokens-original-672bd0f/original-unshielded-tokens.ts")
SOURCE_SHA = "581c2485dad88e71550f612921735a9bbf0b9b00b04b5f6bb680e2651c73e556"
ORIGINAL_SHA = "6fc3737182878c3a31bcba5eae1dec70377097901791fc681adca294f576c913"
def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    output = args.output_dir.resolve()
    if output.exists():
        parser.error("output directory must be absent")
    if sha(SOURCE) != SOURCE_SHA or sha(ORIGINAL) != ORIGINAL_SHA:
        raise ValueError("pinned original source or assertions changed")
    output.mkdir(parents=True)
    command = [str(compiler), "--target", "rust", "--target", "ts",
               "--feature-zkir-v3", "--skip-zk", "--rust-runtime-root",
               str(ROOT), str(SOURCE), str(output / "generated")]
    process = subprocess.run(command, cwd=ROOT, capture_output=True)
    (output / "compile.stdout").write_bytes(process.stdout)
    (output / "compile.stderr").write_bytes(process.stderr)
    (output / "compile.exit").write_text(f"{process.returncode}\n")
    generated = output / "generated"
    required = [generated / "contract/lib.rs", generated / "contract/index.js",
                generated / "contract/compact-rust-ir.json",
                generated / "compiler/contract-manifest.json"]
    if (process.returncode != 0 or process.stdout or process.stderr or
            not all(path.is_file() for path in required) or
            "pub mod recorded" not in required[0].read_text()):
        raise RuntimeError("original source both-target emission failed; inspect retained compile logs")
    artifacts = {str(path.relative_to(generated)): sha(path)
                 for path in sorted(generated.rglob("*")) if path.is_file()}
    result = {
        "status": "both-target-emitted",
        "upstream": "672bd0f226f47a7d036c5c074ce71fbe1af1925e",
        "source_sha256": SOURCE_SHA,
        "original_assertions_sha256": ORIGINAL_SHA,
        "compiler": str(compiler),
        "compiler_sha256": sha(compiler),
        "command": command,
        "exit": process.returncode,
        "stdout_sha256": sha(output / "compile.stdout"),
        "stderr_sha256": sha(output / "compile.stderr"),
        "artifacts_sha256": artifacts,
        "output_published": True,
        "limits": ["--skip-zk emission only", "no generated Rust/TypeScript execution parity, proof, funded transaction or Linux claim"],
    }
    (output / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
