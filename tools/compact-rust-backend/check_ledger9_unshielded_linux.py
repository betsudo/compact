#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Run the original unshielded-token Linux integration gate on a signed PR771 descendant.

The input tarballs are the released packages named by the pinned original test
lockfile. This manual gate does not start hosted CI. Its output is fresh, local
evidence; a prepared source-only check is never a Linux qualification.
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
import sys
import tempfile

from verify_ledger9_unshielded_tokens_original import PINS, source_compiler

ROOT = Path(__file__).resolve().parents[2]
SOURCE_HEAD = "29e6e6d8796cd144a8bc44fc08012d5b6ab19896"
UPSTREAM = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"
SOURCE = ROOT / "test-center/compact/unshielded-tokens.compact"
TEST = ROOT / "tools/compact-rust-backend/fixtures/unshielded-tokens-original-672bd0f/original-unshielded-tokens.ts"
PROOF_MARKERS = (
    "mintUnshieldedToSelfTest: original source fresh proof and funded strict replay gate passed",
    "getUnshieldedBalanceTest: original 100-unit balance proof and funded strict replay gate passed",
    "sendUnshieldedToContractTest+receiveUnshieldedTest: original 100-unit two-contract transaction, funded strict apply and replay gate passed",
)


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def git(*args: str, env: dict[str, str] | None = None) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, env=env, text=True).strip()


def pinned_inputs(runtime: Path, onchain: Path) -> dict[str, str]:
    paths = {"source": SOURCE, "test": TEST, "runtime": runtime, "onchain": onchain}
    found = {}
    for name, path in paths.items():
        require(path.is_file() and not path.is_symlink(), f"missing physical {name}: {path}")
        found[name] = digest(path)
        require(found[name] == PINS[name], f"{name} SHA-256 differs from pinned original")
    require(len(re.findall(r"^test\('", TEST.read_text(), re.M)) == 19,
            "original 19-case inventory changed")
    return found


def source_state() -> dict[str, str]:
    head = git("rev-parse", "HEAD")
    require(git("status", "--porcelain") == "", "exact-head source must be clean")
    subprocess.run(["git", "merge-base", "--is-ancestor", SOURCE_HEAD, head],
                   cwd=ROOT, check=True)
    key = ROOT / ".github/keys/ledger9-vector-source-signing.asc"
    require(key.is_file() and not key.is_symlink(), "pinned public signing key missing")
    with tempfile.TemporaryDirectory(prefix="ledger9-source-key-") as home:
        os.chmod(home, 0o700)
        env = os.environ.copy()
        env["GNUPGHOME"] = home
        fingerprint = subprocess.check_output(
            ["gpg", "--batch", "--with-colons", "--show-keys", str(key)],
            env=env, text=True)
        require(re.findall(r"^fpr:{9}([^:]+):", fingerprint, re.M) ==
                ["F8851C118C66BF98DB9B1B5DB75A9FB8C6327844"],
                "source signing key fingerprint changed")
        subprocess.run(["gpg", "--batch", "--import", str(key)], env=env,
                       check=True, capture_output=True)
        subprocess.run(["gpg", "--batch", "--import-ownertrust"], env=env,
                       input="F8851C118C66BF98DB9B1B5DB75A9FB8C6327844:6:\n",
                       text=True, check=True, capture_output=True)
        signature = git("log", "-1", "--format=%G?%n%GK%n%B", env=env)
        require(signature.startswith("G\nB75A9FB8C6327844\n") and
                "Signed-off-by:" in signature and "Assisted-by: AI" in signature,
                "signed DCO+Assisted source required")
    return {"head": head, "tree": git("rev-parse", "HEAD^{tree}"),
            "compiler_source_head": SOURCE_HEAD, "upstream": UPSTREAM}


def run(argv: list[str], log: Path, env: dict[str, str]) -> str:
    with log.open("w") as output:
        result = subprocess.run([str(arg) for arg in argv], cwd=ROOT, env=env,
                                stdout=output, stderr=subprocess.STDOUT, text=True)
    require(result.returncode == 0, f"{argv[0]} exited {result.returncode}; see {log}")
    return log.read_text()


def nix_build(attribute: str, output: Path, env: dict[str, str]) -> Path:
    value = run(["nix", "build", "--no-link", "--print-out-paths", attribute], output, env)
    rows = [row for row in value.splitlines() if re.fullmatch(r"/nix/store/[A-Za-z0-9._+-]+", row)]
    require(len(rows) == 1,
            f"Nix did not yield one exact {attribute} store path")
    return Path(rows[0])


def files(root: Path) -> dict[str, str]:
    found = {}
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in physical artifact tree: {path}")
        if path.is_file():
            found[path.relative_to(root).as_posix()] = digest(path)
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-tar", required=True, type=Path)
    parser.add_argument("--onchain-tar", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--preflight-only", action="store_true")
    args = parser.parse_args()
    inputs = pinned_inputs(args.runtime_tar, args.onchain_tar)
    state = source_state()
    if args.preflight_only:
        print(json.dumps({"format": "ledger9-unshielded-linux/v1", "status": "prepared",
                          "native_linux": False, "source": state, "inputs_sha256": inputs,
                          "original_tests": 19}, sort_keys=True))
        return 0
    if not args.output:
        parser.error("--output is required for qualification")
    require((platform.system(), platform.machine()) == ("Linux", "x86_64"),
            "qualification requires native Linux x86_64")
    output = args.output.absolute()
    require(not output.exists() and not output.is_symlink(), "fresh output directory required")
    require(shutil.disk_usage(output.parent).free >= 50 * 1024**3,
            "at least 50 GiB free required before compiler, keys and proof batch")
    output.mkdir(parents=True)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(output / "cargo-target")
    env["MIDNIGHT_PP"] = str(output / "proof-parameters")
    (output / "proof-parameters").mkdir()
    receipt = {"format": "ledger9-unshielded-linux/v1", "status": "failed",
               "native_linux": True, "platform": "Linux x86_64", "source": state,
               "inputs_sha256": inputs, "sdk_matrix": {
                   "contract_semantics": "Ledger 9.1.0.1", "ledger_client": "midnight-ledger-v9 1.0.2",
                   "released_compact": "0.36.0", "released_runtime": "0.21.0",
                   "released_onchain": "4.0.1", "zkir_v3": "3.2.0"},
               "limits": ["offline local TestState, not a shared node or proof server",
                          "the 100-unit balance query uses a seeded ContractState"]}
    try:
        compiler = nix_build(".#compactc", output / "compiler-build.log", env) / "bin/compactc"
        zkir = nix_build(".#zkir-v3", output / "zkir-build.log", env) / "bin/zkir"
        require(zkir.is_file() and os.access(zkir, os.X_OK), "ZKIR v3 executable missing")
        scheme = compiler.with_name("compactc-scheme")
        source_compiler(compiler, SOURCE_HEAD, digest(compiler), digest(scheme))
        receipt["tools_sha256"] = {"compiler_launcher": digest(compiler),
                                    "compiler_scheme": digest(scheme), "zkir_v3": digest(zkir)}
        tool_bin = output / "tool-bin"
        tool_bin.mkdir()
        (tool_bin / "zkir-v3").symlink_to(zkir)
        env["PATH"] = f"{tool_bin}:{env['PATH']}"
        run(["cargo", "fetch", "--locked"], output / "cargo-fetch.log", env)
        compile_args = [str(compiler), "--target", "rust", "--target", "ts",
                        "--rust-require-recording", "--feature-zkir-v3",
                        "--ts-runtime-profile", "released-0.21.0",
                        "--rust-runtime-root", str(ROOT)]
        probe = output / "generated-probe"
        run([*compile_args, "--skip-zk", str(SOURCE), str(probe)],
            output / "compile-probe.log", env)
        circuits = sorted(p.stem for p in (probe / "zkir").glob("*.zkir"))
        require(len(circuits) == 20, "expected all 20 original ZKIR circuits")
        parameters = set()
        for circuit in circuits:
            log = output / f"model-{circuit}.log"
            model = run([str(zkir), "mock-compile", str(probe / "zkir" / f"{circuit}.zkir")],
                        log, env)
            match = re.search(r"k=(\d+), rows=(\d+)", model)
            require(match is not None, f"ZKIR geometry absent for {circuit}")
            parameters.add(int(match.group(1)))
        receipt["zkir_models"] = {name: (output / f"model-{name}.log").read_text().strip()
                                   for name in circuits}
        run(["cargo", "run", "--locked", "--offline", "-p", "compact-rust-proof-smoke",
             "--", "--prepare-proof-material", *[part for k in sorted(parameters)
                                                  for part in ("--parameter-k", str(k))]],
            output / "proof-material.log", env)
        generated = output / "generated-with-keys"
        run([*compile_args, str(SOURCE), str(generated)], output / "compile-full.log", env)
        artifacts = files(generated)
        require(len(artifacts) == 91 and
                sorted(p.stem for p in (generated / "zkir").glob("*.zkir")) == circuits and
                len(list((generated / "keys").glob("*.prover"))) == 20 and
                len(list((generated / "keys").glob("*.verifier"))) == 20,
                "full Rust/TS/ZKIR/key artifact inventory differs")
        require(digest(probe / "contract/index.js") == digest(generated / "contract/index.js"),
                "full and probe generated TypeScript differ")
        receipt["generated_sha256"] = artifacts
        adapter = ROOT / "tools/compact-rust-backend/verify_ledger9_unshielded_tokens_original.py"
        run([sys.executable, str(adapter), "--compiler", str(compiler),
             "--compiler-source-head", SOURCE_HEAD, "--compiler-sha256", digest(compiler),
             "--compiler-scheme-sha256", digest(scheme), "--ts-runtime-profile",
             "released-0.21.0", "--runtime-tar", str(args.runtime_tar.absolute()),
             "--onchain-tar", str(args.onchain_tar.absolute()), "--output-dir",
             str(output / "original-19")], output / "original-19-run.log", env)
        ts = json.loads((output / "original-19/result.json").read_text())
        require(ts["original_cases_passed"] == ts["original_cases_total"] == 19 and
                ts["generated_js_sha256"] == artifacts["contract/index.js"],
                "original 19 cases or generated TypeScript parity failed")
        receipt["original_19_result_sha256"] = digest(output / "original-19/result.json")
        metadata = output / "cargo-metadata.json"
        metadata.write_text(subprocess.check_output(
            ["cargo", "metadata", "--locked", "--format-version", "1"],
            cwd=ROOT, env=env, text=True))
        static_receipt = output / "ledger-static.json"
        static_path = run([sys.executable, str(ROOT / "tools/compact-rust-backend/resolve_ledger_test_static.py"),
                           "--metadata", str(metadata), "--lock", str(ROOT / "Cargo.lock"),
                           "--provenance", str(static_receipt)], output / "ledger-static.log", env).strip()
        env["MIDNIGHT_LEDGER_TEST_STATIC_DIR"] = static_path
        env["COMPACT_UNSHIELDED_ORIGINAL_GENERATED"] = str(generated)
        proof_log = output / "proof-run.log"
        proof = run(["cargo", "run", "--locked", "--offline", "-p",
                     "compact-rust-proof-smoke", "--features", "unshielded-original-proof",
                     "--", "--unshielded-original-proof", str(generated)], proof_log, env)
        require(all(marker in proof for marker in PROOF_MARKERS),
                "mint, balance or two-contract strict proof marker missing")
        require(digest(generated / "unshielded-original-send-contract-100-ledger-after.bin") ==
                digest(generated / "unshielded-original-send-contract-100-ledger-replay.bin"),
                "two-contract replay changed serialized Ledger")
        receipt["proof_log_sha256"] = digest(proof_log)
        receipt["ledger_static_provenance_sha256"] = digest(static_receipt)
        receipt["proof_artifacts_sha256"] = files(generated)
        receipt["status"] = "qualified"
    except Exception as error:
        receipt["failure"] = str(error)
        raise
    finally:
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": receipt["status"], "receipt": str(output / "receipt.json")},
                     sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
