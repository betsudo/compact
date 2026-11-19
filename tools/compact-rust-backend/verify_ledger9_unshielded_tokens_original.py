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

"""Run pinned original unshielded-token Test Center assertions.

The original test body and contract are byte-exact at upstream 672bd0f. A
private runner supplies the async Test Center startContract bridge and uses
released Runtime 0.21.0. The archive mode retains the released 0.36.0
baseline; compiler mode tests separately identified source-built JavaScript.
Neither mode creates a proof or a funded transaction.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = Path(__file__).resolve().parent / "fixtures/unshielded-tokens-original-672bd0f"
SOURCE = ROOT / "test-center/compact/unshielded-tokens.compact"
PINS = {
    "archive": "a8126b2cc06230ed2c6558059a8945119e370ff222930e6a52f90870e8cc3c5d",
    "runtime": "1b078b5810ce4141efa11990df7e86268fab5e9e2706089fb487715d5c4f1128",
    "onchain": "61c058a34f35ae28d19a878136568a6b7d1f8aa0cb6cf3f030e2402777cf9283",
    "source": "581c2485dad88e71550f612921735a9bbf0b9b00b04b5f6bb680e2651c73e556",
    "test": "6fc3737182878c3a31bcba5eae1dec70377097901791fc681adca294f576c913",
    "package": "ed541cd8d98153689d05e0c0f57d69bbc372c33ce09124ec1c801524eb7ccc46",
    "lock": "7d39a86b9763cbc91847c236b7aadd5704d76429ce3b0f325dc5eda772e8121a",
}
COMPILER_SOURCE_PATHS = ("compiler", "runtime-rs", "tools/compact-rust-backend/src",
                         "tools/compact-rust-cli", "Cargo.lock", "flake.nix", "flake.lock")

HEADER = """import { expect, test } from 'vitest';
import * as runtime from '@midnight-ntwrk/compact-runtime';
import * as ocrt from '@midnightntwrk/onchain-runtime-v4';
import * as generated from './generated/contract/index.js';
const contractCode = { ...generated, contractDir: './generated' };
const startContract = async (module, witnesses, privateState) => {
  const contract = new module.Contract(witnesses);
  const initial = await contract.initialState(runtime.createConstructorContext(privateState, '0'.repeat(64)));
  const context = runtime.createCircuitContext({
    circuitId: 'constructor',
    contractAddress: ocrt.sampleContractAddress(),
    coinPublicKeyOrZswapState: initial.currentZswapLocalState.coinPublicKey,
    contractState: initial.currentContractState,
    privateState: initial.currentPrivateState,
  });
  return [contract, context];
};
"""


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def check(name: str, path: Path) -> None:
    if not path.is_file() or sha256(path) != PINS[name]:
        raise ValueError(f"{name} pin mismatch or file missing: {path}")


def command(argv: list[str], cwd: Path, log: Path) -> None:
    result = subprocess.run(argv, cwd=cwd, text=True, capture_output=True, check=False)
    log.write_text(result.stdout + result.stderr)
    if result.returncode:
        raise RuntimeError(f"{argv[0]} failed ({result.returncode}); see {log}")


def require_compiler_source_tree(head: str) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", head):
        raise ValueError("--compiler-source-head must be a full Git commit ID")
    if subprocess.run(["git", "cat-file", "-e", f"{head}^{{commit}}"], cwd=ROOT,
                      capture_output=True, check=False).returncode:
        raise ValueError("compiler source head is absent from this checkout")
    # The adapter itself may be a later commit. Compiler, runtime, lock and
    # Nix recipe bytes must still be those named by the supplied source head.
    if subprocess.run(["git", "diff", "--quiet", head, "--", *COMPILER_SOURCE_PATHS],
                      cwd=ROOT, check=False).returncode:
        raise ValueError("compiler source files differ from --compiler-source-head")


def source_compiler(compiler: Path, head: str, digest: str, scheme_digest: str | None) -> Path:
    compiler = compiler.resolve(strict=True)
    if not compiler.is_file() or not os.access(compiler, os.X_OK):
        raise ValueError(f"source-built compiler is not executable: {compiler}")
    if not re.fullmatch(r"[0-9a-f]{40}", head):
        raise ValueError("--compiler-source-head must be a full Git commit ID")
    if not re.fullmatch(r"[0-9a-f]{64}", digest) or sha256(compiler) != digest:
        raise ValueError("source-built compiler SHA-256 does not match --compiler-sha256")
    if scheme_digest is not None:
        scheme = compiler.with_name("compactc-scheme")
        if not re.fullmatch(r"[0-9a-f]{64}", scheme_digest) or not scheme.is_file() or sha256(scheme) != scheme_digest:
            raise ValueError("source-built Scheme compiler SHA-256 does not match --compiler-scheme-sha256")
    require_compiler_source_tree(head)
    return compiler


def main() -> int:
    parser = argparse.ArgumentParser()
    compiler_input = parser.add_mutually_exclusive_group(required=True)
    compiler_input.add_argument("--archive", type=Path)
    compiler_input.add_argument("--compiler", type=Path)
    parser.add_argument("--ts-runtime-profile", choices=["released-0.21.0"])
    parser.add_argument("--compiler-source-head")
    parser.add_argument("--compiler-sha256")
    parser.add_argument("--compiler-scheme-sha256")
    parser.add_argument("--runtime-tar", type=Path, required=True)
    parser.add_argument("--onchain-tar", type=Path, required=True)
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    if args.compiler and (not args.compiler_source_head or not args.compiler_sha256):
        parser.error("--compiler-source-head and --compiler-sha256 are required with --compiler")
    if args.ts_runtime_profile and not args.compiler_scheme_sha256:
        parser.error("--compiler-scheme-sha256 is required with --ts-runtime-profile")
    if args.archive and (args.compiler_source_head or args.compiler_sha256 or args.compiler_scheme_sha256):
        parser.error("--compiler-source-head and --compiler-sha256 require --compiler")
    if args.ts_runtime_profile and not args.compiler:
        parser.error("--ts-runtime-profile requires --compiler")
    inputs = {
        "runtime": args.runtime_tar,
        "onchain": args.onchain_tar,
        "source": args.source,
        "test": FIXTURE / "original-unshielded-tokens.ts",
        "package": FIXTURE / "package.json",
        "lock": FIXTURE / "package-lock.json",
    }
    if args.archive:
        inputs["archive"] = args.archive
    for name, path in inputs.items():
        check(name, path)
    built_compiler = (source_compiler(args.compiler, args.compiler_source_head, args.compiler_sha256, args.compiler_scheme_sha256)
                      if args.compiler else None)
    if len(re.findall(r"^test\('", inputs["test"].read_text(), re.M)) != 19:
        raise ValueError("original test inventory changed")
    if args.verify_only:
        if built_compiler:
            print(json.dumps({"pins": {k: v for k, v in PINS.items() if k != "archive"},
                              "compiler_sha256": sha256(built_compiler),
                              "compiler_scheme_sha256": (sha256(built_compiler.with_name("compactc-scheme"))
                                                         if args.compiler_scheme_sha256 else None),
                              "compiler_source_head": args.compiler_source_head,
                              "original_tests": 19, "mode": "verify-only"}, sort_keys=True))
        else:
            print(json.dumps({"pins": PINS, "original_tests": 19, "mode": "verify-only"}, sort_keys=True))
        return 0
    if not args.output_dir:
        parser.error("--output-dir is required for a run")
    host = (platform.system(), platform.machine())
    if host not in (("Darwin", "arm64"), ("Linux", "x86_64")):
        raise ValueError("the pinned host adapter requires Darwin arm64 or Linux x86_64")
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="unshielded-original-", dir=output) as tmp:
        work = Path(tmp)
        if built_compiler:
            compiler_path = built_compiler
        else:
            compiler = work / "compiler"
            compiler.mkdir()
            with zipfile.ZipFile(args.archive) as z:
                if any(Path(n).name != n for n in z.namelist()):
                    raise ValueError("unexpected archive member path")
                z.extractall(compiler)
            for name in ("compactc", "compactc.bin", "zkir-v3"):
                path = compiler / name
                if not path.is_file():
                    raise ValueError(f"release archive missing {name}")
                path.chmod(path.stat().st_mode | 0o111)
            compiler_path = compiler / "compactc"
        source = work / "unshielded-tokens.compact"
        shutil.copyfile(args.source, source)
        compile_options = (["--target", "ts", "--feature-zkir-v3"] if built_compiler else [])
        if args.ts_runtime_profile:
            compile_options += ["--ts-runtime-profile", args.ts_runtime_profile]
        compile_command = [str(compiler_path), *compile_options, "--skip-zk", str(source),
                           str(work / "generated")]
        command(compile_command, work, output / "compile.log")
        for name in ("package.json", "package-lock.json"):
            shutil.copyfile(FIXTURE / name, work / name)
        shutil.copyfile(args.runtime_tar, work / "compact-runtime-0.21.0.tgz")
        shutil.copyfile(args.onchain_tar, work / "onchain-runtime-v4-4.0.1.tgz")
        command(["npm", "ci", "--ignore-scripts", "--no-audit", "--no-fund"], work, output / "npm-ci.log")
        (work / "test.test.ts").write_text(HEADER + inputs["test"].read_text())
        command([str(work / "node_modules/.bin/vitest"), "run", "test.test.ts", "--reporter=verbose"], work, output / "original-19.log")
        log = (output / "original-19.log").read_text()
        if "Tests  19 passed (19)" not in log:
            raise RuntimeError("original 19-case summary missing")
        retained = output / "retained"
        (retained / "contract").mkdir(parents=True, exist_ok=True)
        (retained / "compiler").mkdir(parents=True, exist_ok=True)
        shutil.copyfile(work / "generated/contract/index.js", retained / "contract/index.js")
        shutil.copyfile(work / "generated/compiler/contract-manifest.json", retained / "compiler/contract-manifest.json")
        result = {
            "upstream": "672bd0f226f47a7d036c5c074ce71fbe1af1925e",
            "pr771_base": "d6513a24debac14d43a8199460d9dc2c6d9af2b1",
            "platform": " ".join(host),
            "original_cases_passed": 19,
            "original_cases_total": 19,
            "released_compact": "0.36.0",
            "runtime": "0.21.0",
            "onchain": "4.0.1",
            "generated_js_sha256": sha256(retained / "contract/index.js"),
            "generated_manifest_sha256": sha256(retained / "compiler/contract-manifest.json"),
            "inputs_sha256": PINS,
            "limits": ["released-matrix private harness; original test body and contract bytes unchanged", "--skip-zk execution of generated JavaScript; no fresh proof or funded Ledger transaction", "full Test Center unrun; this adapter does not qualify the proof gate"],
        }
        if built_compiler:
            result.pop("released_compact")
            result["compiler_kind"] = "source-built"
            result["compiler_sha256"] = sha256(built_compiler)
            result["compiler_source_head"] = args.compiler_source_head
            if args.compiler_scheme_sha256:
                result["compiler_scheme_sha256"] = args.compiler_scheme_sha256
            result["compiler_command"] = compile_command
            result["ts_runtime_profile"] = args.ts_runtime_profile or "local-0.18.0"
            result["inputs_sha256"] = {k: v for k, v in PINS.items() if k != "archive"}
            result["limits"] = [
                "source-built compiler; original test body and contract bytes unchanged",
                "--skip-zk generated JavaScript only; Rust, proof and funded Ledger behavior are separate gates",
                "full Test Center unrun; this adapter does not qualify the proof gate",
            ]
        (output / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
