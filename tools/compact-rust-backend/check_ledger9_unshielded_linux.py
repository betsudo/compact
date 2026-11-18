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

"""Run the original token Linux integration gate on a signed PR772 descendant.

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

from verify_ledger9_unshielded_tokens_original import (
    PINS, require_compiler_source_tree, source_compiler,
)

ROOT = Path(__file__).resolve().parents[2]
SOURCE_HEAD = "06d27825249d3122e9ba6d597204161acda77e0b"
UPSTREAM = "672bd0f226f47a7d036c5c074ce71fbe1af1925e"
SOURCE = ROOT / "test-center/compact/unshielded-tokens.compact"
TEST = ROOT / "tools/compact-rust-backend/fixtures/unshielded-tokens-original-672bd0f/original-unshielded-tokens.ts"
PROOF_MARKERS = (
    "mintUnshieldedToSelfTest: original source fresh proof and funded strict replay gate passed",
    "getUnshieldedBalanceTest: original 100-unit result, 100-unit seeded balance proof and funded strict replay gate passed",
    "getUnshieldedBalanceTest: original 0-unit result, 100-unit seeded balance proof and funded strict replay gate passed",
    "getUnshieldedBalanceGtTest(50): original true result, funded strict application and replay gate passed",
    "getUnshieldedBalanceGtTest(150): original false result, funded strict application and replay gate passed",
    "getUnshieldedBalanceLtTest(50): original false result, funded strict application and replay gate passed",
    "getUnshieldedBalanceLtTest(150): original true result, funded strict application and replay gate passed",
    "sendUnshieldedToContractTest+receiveUnshieldedTest: original 100-unit two-contract transaction, funded strict apply and replay gate passed",
    "sendUnshieldedToSelfTest: original 100-unit auto-receive proof, funded strict apply and replay gate passed",
    "mintUnshieldedToContractTest: original 100-unit destination proof, funded strict apply and replay gate passed",
    "mintUnshieldedToUserTest: original 100-unit destination proof, funded strict apply and replay gate passed",
    "sendUnshieldedToUserTest: original 100-unit User send, funded strict apply and replay gate passed",
)
SHIELDED_MARKERS = (
    "mintShieldedToSelfTest: unmatched-output commitment refusal passed",
    "mintShieldedToSelfTest: original mint100, matching self spend/receive commitment, funded strict apply and replay passed",
)
SHIELDED_BATCHES = (
    ("mint-self", "--shielded-original-mint-self-proof", SHIELDED_MARKERS),
    ("mint-contract", "--shielded-original-mint-contract-proof", (
        "mintShieldedToContractTest: original mint100, matching recipient contract output without auto-receive, funded strict apply and replay passed",
        "receiveShieldedTest: original self-color receive Effects and proof passed",
    )),
    ("mint-user", "--shielded-original-mint-user-proof", (
        "mintShieldedToUserTest: original mint100, matching user output without auto-receive, funded strict apply and replay passed",
    )),
    ("send-self", "--shielded-original-send-self-proof", (
        "sendShieldedToSelfTest: original send 60 from 100, 2 self output(s), funded strict apply and replay passed",
        "sendShieldedToSelfTest: original send 100 from 100, 1 self output(s), funded strict apply and replay passed",
    )),
    ("send-contract", "--shielded-original-send-contract-proof", (
        "sendShieldedToContractTest: original send 60 from 100, one recipient and one change output, funded strict apply and replay passed",
    )),
    ("send-user", "--shielded-original-send-user-proof", (
        "sendShieldedToUserTest: original send 60 from 100, one user output and 40 self change, funded strict apply and replay passed",
    )),
    ("immediate-self", "--shielded-original-immediate-self-proof", (
        "sendImmediateShieldedToSelfTest: pinned original 100→100 Effects and both source-bound standalone proofs passed",
        "sendImmediateShieldedToSelfTest: real wallet-funded singleton transient, two original calls, strict apply and replay passed",
    )),
    ("immediate-contract", "--shielded-original-immediate-contract-proof", (
        "sendImmediateShieldedToContractTest: pinned original 100→60+40 Effects and all three source-bound standalone proofs passed",
        "sendImmediateShieldedToContractTest: real wallet-funded singleton transient, three original calls, funded strict apply and replay passed",
    )),
    ("immediate-user", "--shielded-original-immediate-user-proof", (
        "sendImmediateShieldedToUserTest: pinned original 100→60 User+40 change Effects and both source-bound standalone proofs passed",
        "sendImmediateShieldedToUserTest: real wallet-funded singleton transient, two original calls, strict apply and replay passed",
    )),
)
QUALIFIED_CIRCUITS = (
    "mintUnshieldedToSelfTest", "getUnshieldedBalanceTest",
    "getUnshieldedBalanceGtTest", "getUnshieldedBalanceLtTest",
    "sendUnshieldedToContractTest", "receiveUnshieldedTest",
    "sendUnshieldedToSelfTest", "mintUnshieldedToContractTest",
    "mintUnshieldedToUserTest", "sendUnshieldedToUserTest",
    "mintShieldedToSelfTest", "mintShieldedToContractTest",
    "receiveShieldedTest", "mintShieldedToUserTest",
    "sendShieldedToSelfTest", "sendShieldedToContractTest",
    "sendShieldedToUserTest",
    "sendImmediateShieldedToSelfTest", "sendImmediateShieldedToContractTest",
    "sendImmediateShieldedToUserTest",
)
PROOF_STEMS = (
    "unshielded-original-mint-self",
    "unshielded-original-balance-100",
    "unshielded-original-balance-other-color-0",
    "unshielded-original-balance-gt-50",
    "unshielded-original-balance-gt-150",
    "unshielded-original-balance-lt-50",
    "unshielded-original-balance-lt-150",
    "unshielded-original-send-contract-100",
    "unshielded-original-receive-contract-100",
    "unshielded-original-send-self-100",
    "unshielded-original-mint-contract-100",
    "unshielded-original-mint-contract-receive-100",
    "unshielded-original-mint-user-100",
    "unshielded-original-send-user-100",
    "unshielded-original-mint-shielded-self-100",
    "unshielded-original-mint-shielded-contract-100",
    "unshielded-original-mint-shielded-contract-100-recipient-receive",
    "unshielded-original-mint-shielded-user-100",
    "unshielded-original-send-shielded-self-change-60",
    "unshielded-original-send-shielded-self-full-100",
    "unshielded-original-send-shielded-contract-change-60",
    "unshielded-original-send-shielded-contract-change-60-recipient-receive",
    "unshielded-original-send-shielded-user-change-60",
    "unshielded-original-immediate-shielded-self-100",
    "unshielded-original-immediate-shielded-self-100-receive",
    "unshielded-original-immediate-shielded-contract-change-60",
    "unshielded-original-immediate-shielded-contract-change-60-receive",
    "unshielded-original-immediate-shielded-contract-change-60-recipient-receive",
    "unshielded-original-immediate-shielded-user-change-60",
    "unshielded-original-immediate-shielded-user-change-60-receive",
)
REPLAY_STEMS = tuple(stem for stem in PROOF_STEMS if stem not in (
    "unshielded-original-receive-contract-100",
    "unshielded-original-mint-contract-receive-100",
    "unshielded-original-mint-shielded-contract-100-recipient-receive",
    "unshielded-original-send-shielded-contract-change-60-recipient-receive",
    "unshielded-original-immediate-shielded-self-100-receive",
    "unshielded-original-immediate-shielded-contract-change-60-receive",
    "unshielded-original-immediate-shielded-contract-change-60-recipient-receive",
    "unshielded-original-immediate-shielded-user-change-60-receive",
))
REFUSAL_REASONS = {
    "unshielded-original-mint-shielded-self-100-without-offer-refusal.txt":
        "CommitmentsNEClaimedShieldedReceives",
    "unshielded-original-mint-shielded-contract-100-without-offer-refusal.txt":
        "AllCommitmentsSubsetCheckFailure",
    "unshielded-original-mint-shielded-contract-100-without-recipient-refusal.txt":
        "CommitmentsNEClaimedShieldedReceives",
    "unshielded-original-mint-shielded-user-100-without-offer-refusal.txt":
        "AllCommitmentsSubsetCheckFailure",
    "unshielded-original-send-shielded-self-change-60-without-offer-refusal.txt":
        "NullifiersNEClaimedNullifiers",
    "unshielded-original-send-shielded-self-full-100-without-offer-refusal.txt":
        "NullifiersNEClaimedNullifiers",
    "unshielded-original-send-shielded-contract-change-60-without-offer-refusal.txt":
        "NullifiersNEClaimedNullifiers",
    "unshielded-original-send-shielded-contract-change-60-without-recipient-refusal.txt":
        "CommitmentsNEClaimedShieldedReceives",
    "unshielded-original-send-shielded-user-change-60-without-offer-refusal.txt":
        "NullifiersNEClaimedNullifiers",
    "unshielded-original-immediate-shielded-self-100-wrong-index-refusal.txt":
        "Public transcript input mismatch for input 13;",
    "unshielded-original-immediate-shielded-contract-change-60-without-offer-refusal.txt":
        "NullifiersNEClaimedNullifiers",
    "unshielded-original-immediate-shielded-contract-change-60-without-recipient-refusal.txt":
        "CommitmentsNEClaimedShieldedReceives",
    "unshielded-original-immediate-shielded-contract-change-60-wrong-index-refusal.txt":
        "Public transcript input mismatch for input 13;",
    "unshielded-original-immediate-shielded-user-change-60-without-offer-refusal.txt":
        "NullifiersNEClaimedNullifiers",
    "unshielded-original-immediate-shielded-user-change-60-wrong-index-refusal.txt":
        "Public transcript input mismatch for input 13;",
}
IMMEDIATE_SOURCES = {
    "sendImmediateShieldedToSelfTest": (
        "shielded_original_immediate_self.rs",
        "unshielded-original-immediate-shielded-self-100",
    ),
    "sendImmediateShieldedToContractTest": (
        "shielded_original_immediate_contract.rs",
        "unshielded-original-immediate-shielded-contract-change-60",
    ),
    "sendImmediateShieldedToUserTest": (
        "shielded_original_immediate_user.rs",
        "unshielded-original-immediate-shielded-user-change-60",
    ),
}


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def require_qualified_plan() -> None:
    """Refuse an incomplete selector before reporting source-only readiness."""
    require(len(QUALIFIED_CIRCUITS) == len(set(QUALIFIED_CIRCUITS)) == 20 and
            len(PROOF_STEMS) == len(set(PROOF_STEMS)) == 30 and
            len(REPLAY_STEMS) == len(set(REPLAY_STEMS)) == 22 and
            len(SHIELDED_BATCHES) == 9 and len(REFUSAL_REASONS) == 15,
            "original 20-circuit proof/replay/refusal inventory differs")
    source = SOURCE.read_text()
    dispatch = (ROOT / "tools/compact-rust-proof-smoke/src/direct_dispatch.rs").read_text()
    require(all(name in source for name in QUALIFIED_CIRCUITS) and
            all(flag in dispatch and markers for _, flag, markers in SHIELDED_BATCHES),
            "original circuit source or proof-smoke dispatch differs")
    smoke = ROOT / "tools/compact-rust-proof-smoke/src"
    immediate_batches = {name: markers for name, _, markers in SHIELDED_BATCHES
                         if name.startswith("immediate-")}
    for circuit, (file, stem) in IMMEDIATE_SOURCES.items():
        code = (smoke / file).read_text()
        batch = "immediate-" + circuit.removeprefix("sendImmediateShieldedTo").removesuffix("Test").lower()
        markers = immediate_batches.get(batch, ())
        pair_stems = (stem, f"{stem}-receive")
        if batch == "immediate-contract":
            pair_stems += (f"{stem}-recipient-receive",)
        require(f'const NAME: &str = "{circuit}"' in code and
                f'const STEM: &str = "{stem}"' in code and
                "wrong-index-refusal.txt" in code and
                all(member in PROOF_STEMS for member in pair_stems) and
                stem in REPLAY_STEMS and len(markers) == 2 and
                all(marker.startswith(f"{circuit}: ") and
                    marker.partition(": ")[2] in code for marker in markers),
                f"immediate original source/selector differs: {circuit}")


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


def signed_checkout_identity(head: str, signed_checkout_head: str | None = None) -> str:
    if signed_checkout_head is not None:
        require(re.fullmatch(r"[0-9a-f]{40}", signed_checkout_head) is not None,
                "--signed-checkout-head must be a full Git commit ID")
        require(git("rev-parse", f"{signed_checkout_head}^{{tree}}") ==
                git("rev-parse", f"{head}^{{tree}}"),
                "signed checkout tree differs from current HEAD")
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
        for commit in (SOURCE_HEAD, signed_checkout_head or head):
            signature = git("log", "-1", "--format=%G?%n%GK%n%B", commit, env=env)
            require(signature.startswith("G\nB75A9FB8C6327844\n") and
                    "Signed-off-by:" in signature,
                    f"signed DCO source required: {commit}")
    return signed_checkout_head or head


def source_state(signed_checkout_head: str | None = None) -> dict[str, str]:
    head = git("rev-parse", "HEAD")
    require(git("status", "--porcelain") == "", "exact-head source must be clean")
    require_compiler_source_tree(SOURCE_HEAD)
    signed_head = signed_checkout_identity(head, signed_checkout_head)
    return {"head": head, "tree": git("rev-parse", "HEAD^{tree}"),
            "signed_checkout_head": signed_head, "compiler_source_head": SOURCE_HEAD,
            "upstream": UPSTREAM}


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
    parser.add_argument("--signed-checkout-head", help="signed equal-tree commit for a GitHub squash merge")
    args = parser.parse_args()
    require_qualified_plan()
    inputs = pinned_inputs(args.runtime_tar, args.onchain_tar)
    state = source_state(args.signed_checkout_head)
    if args.preflight_only:
        print(json.dumps({"format": "ledger9-original-token-linux/v4", "status": "prepared",
                          "native_linux": False, "source": state, "inputs_sha256": inputs,
                          "original_tests": 19, "proof_circuits": len(QUALIFIED_CIRCUITS),
                          "proof_pairs": len(PROOF_STEMS),
                          "replay_pairs": len(REPLAY_STEMS),
                          "refusal_artifacts": len(REFUSAL_REASONS)}, sort_keys=True))
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
    receipt = {"format": "ledger9-original-token-linux/v4", "status": "failed",
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
        # passes.ss probes `zkir` before invoking `zkir-v3` for v3 circuits.
        (tool_bin / "zkir").symlink_to(zkir)
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
        require(set(QUALIFIED_CIRCUITS) <= set(circuits),
                "original qualified circuits differ from generated ZKIR")
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
        unshielded_log = output / "unshielded-proof-run.log"
        unshielded = run(["cargo", "run", "--locked", "--offline", "-p",
                     "compact-rust-proof-smoke", "--features", "unshielded-original-proof",
                     "--", "--unshielded-original-proof", str(generated)], unshielded_log, env)
        require(all(marker in unshielded for marker in PROOF_MARKERS),
                "one or more original unshielded proof/strict markers missing")
        proof_logs = {"unshielded": digest(unshielded_log)}
        for name, flag, markers in SHIELDED_BATCHES:
            log = output / f"shielded-{name}-proof-run.log"
            text = run(["cargo", "run", "--locked", "--offline", "-p",
                        "compact-rust-proof-smoke", "--features", "unshielded-original-proof",
                        "--", flag, str(generated)], log, env)
            require(all(marker in text for marker in markers),
                    f"original shielded {name} proof/strict marker missing")
            proof_logs[name] = digest(log)
        for stem in PROOF_STEMS:
            standalone = generated / f"{stem}.proof"
            transaction = generated / f"{stem}-transaction.proof"
            require(standalone.is_file() and standalone.stat().st_size > 0 and
                    transaction.is_file() and transaction.stat().st_size > 0 and
                    digest(standalone) != digest(transaction),
                    f"fresh standalone and transaction-bound proofs differ for {stem}")
        for stem in REPLAY_STEMS:
            after = generated / f"{stem}-ledger-after.bin"
            replay = generated / f"{stem}-ledger-replay.bin"
            require(after.is_file() and replay.is_file() and digest(after) == digest(replay),
                    f"{stem} replay changed serialized Ledger")
        for name, reason in REFUSAL_REASONS.items():
            refusal = generated / name
            require(refusal.is_file() and reason in refusal.read_text(),
                    f"original shielded refusal differs: {name}")
        self_color = generated / "unshielded-original-mint-shielded-contract-100-recipient-self-color.proof"
        require(self_color.is_file() and self_color.stat().st_size > 0,
                "original shielded recipient self-color control proof absent")
        receipt["proof_circuits"] = {"unshielded": 10, "shielded": 10}
        receipt["proof_pairs"] = len(PROOF_STEMS)
        receipt["replay_pairs"] = len(REPLAY_STEMS)
        receipt["refusal_artifacts"] = len(REFUSAL_REASONS)
        receipt["proof_log_sha256"] = proof_logs
        receipt["ledger_static_provenance_sha256"] = digest(static_receipt)
        proof_artifacts = files(generated)
        require(len(proof_artifacts) == 303 and
                all(proof_artifacts.get(name) == sha for name, sha in artifacts.items()),
                "proof batch changed generated files or exact artifact count")
        receipt["proof_artifacts_sha256"] = proof_artifacts
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
