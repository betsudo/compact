# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Qualify fresh proofs and funded transactions for all original unused casts."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/stdlib/secp256r1/casts/mixed-original"
BASE = ROOT / "tools/compact-rust-backend/check_upstream_ledger9_mixed_k1_r1_proofs.py"
CASES = {"unusedK1": 8, "unusedR1": 8, "unusedC25519": 9,
         "unusedFieldK1": 8, "unusedFieldR1": 8}
WRONG_K8_SHA256 = "a5ccce332c0d515b9e886e8ea07d2b1e008e26728a2663275363d9c311546d5a"
WRONG_K9_SHA256 = "0c1a1af8b47096a6d07ca3a06962d0d65fa2ea6c2a568c43cc4a7015bf22bc3b"
K9_PARAM_SHA256 = "b9009f1098bcefffec3c461ab3a5e3a17f7e5599f0f08c70fcdc55a89227bcbd"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(command, *, env=None, log=None):
    if log is None:
        subprocess.run(command, cwd=ROOT, env=env, check=True)
    else:
        with log.open("xb") as output:
            subprocess.run(command, cwd=ROOT, env=env, stdout=output,
                           stderr=subprocess.STDOUT, check=True)


def git(*args):
    return subprocess.check_output(("git", *args), cwd=ROOT, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("compiler", "scheme-compiler", "ts-runtime", "zkir", "zkir-wasm",
                 "proof-runner", "proof-source", "material", "wrong-verifier",
                 "wrong-verifier-k10", "wrong-verifier-k11", "wrong-verifier-k12",
                 "wrong-verifier-k9", "output",
                 "cargo-target"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    source_head = git("rev-parse", "HEAD")
    source_tree = git("rev-parse", "HEAD^{tree}")
    if git("status", "--porcelain=v1"):
        parser.error("source checkout must be clean")
    for name in ("compiler", "scheme_compiler", "ts_runtime", "zkir", "zkir_wasm",
                 "proof_runner", "proof_source", "material", "wrong_verifier",
                 "wrong_verifier_k10", "wrong_verifier_k11", "wrong_verifier_k12",
                 "wrong_verifier_k9"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    output.mkdir(parents=True)
    cargo_target = args.cargo_target.resolve()
    cargo_target.mkdir(parents=True, exist_ok=True)
    prior = output / "prior"
    options = []
    for name in ("compiler", "scheme-compiler", "ts-runtime", "zkir", "zkir-wasm",
                 "proof-runner", "proof-source", "material", "wrong-verifier",
                 "wrong-verifier-k10", "wrong-verifier-k11", "wrong-verifier-k12"):
        options.extend(("--" + name, str(getattr(args, name.replace("-", "_")))))
    options.extend(("--output", str(prior), "--cargo-target", str(cargo_target)))
    run(("python3", str(BASE), *options), log=output / "prior-run.log")
    prior_receipt = json.loads((prior / "receipt.json").read_text())
    base = prior / "prior/base"
    base_toolchain = json.loads((base / "toolchain.json").read_text())
    if (base_toolchain["source_head"] != source_head or base_toolchain["source_tree"] != source_tree
            or prior_receipt["head"] != source_head or prior_receipt["tree"] != source_tree):
        raise RuntimeError("nested family source does not match this checkout")

    capture = output / "unused-capture"
    run(("node", str(FAMILY / "capture-unused.mjs"),
         str(base / "both/unused/contract/index.js"), str(args.zkir_wasm), str(capture)),
        log=output / "capture-run.log")
    captures = json.loads((capture / "capture.json").read_text())
    if set(captures) != set(CASES):
        raise RuntimeError("unused-cast case inventory changed")
    material_receipt = json.loads((args.material.parent / "material-receipt.json").read_text())
    if sha(args.material / "bls_midnight_2p9") != K9_PARAM_SHA256:
        raise RuntimeError("pinned SDK k=9 parameters changed")
    wrong_k8 = base / "proof/statement.verifier"
    if base_toolchain["checks"]["proof_k"] != 8 or sha(wrong_k8) != WRONG_K8_SHA256:
        raise RuntimeError("k=8 different-circuit castNative verifier changed")
    if sha(args.wrong_verifier_k9) != WRONG_K9_SHA256:
        raise RuntimeError("k=9 archived different-circuit verifier changed")
    archived_k9_prove = args.wrong_verifier_k9.with_name("prove.stdout")
    if not archived_k9_prove.read_text().startswith("proved k=9,"):
        raise RuntimeError("k=9 verifier provenance is missing")
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(cargo_target)
    runner = cargo_target / "debug/secp256k1-unary-proof"
    if not runner.is_file() or sha(runner) != prior_receipt["proof_runner"]["sha256"]:
        raise RuntimeError("nested source-built proof runner changed")
    proofs = output / "unused-proofs"
    proofs.mkdir()
    proof_receipts = {}
    for case, k in CASES.items():
        circuit = case
        row = captures[case]
        if row["circuit"] != circuit or row["honest_check"] != "passed" or row["changed_write_check"] != "refused":
            raise RuntimeError(f"{case} preimage check failed")
        raw = base / f"both/unused/zkir/{circuit}.zkir"
        preimage = capture / case / "honest.preimage"
        changed = capture / case / "changed-write.preimage"
        params = args.material / f"bls_midnight_2p{k}"
        if sha(raw) != row["raw_sha256"] or sha(preimage) != row["preimage_sha256"] or sha(changed) != row["changed_write_sha256"]:
            raise RuntimeError(f"{case} source or preimage changed")
        if sha(params) != material_receipt["files"][params.name]["sha256"]:
            raise RuntimeError(f"{case} proof parameters changed")
        target = proofs / case
        target.mkdir()
        wrong = args.wrong_verifier_k9 if k == 9 else wrong_k8
        for mode, auxiliary in (("prove", params), ("verify", wrong)):
            run((str(runner), mode, str(raw), str(preimage), str(target), str(auxiliary)),
                log=target / f"{mode}.stdout")
        if not (target / "prove.stdout").read_text().startswith(f"proved k={k},"):
            raise RuntimeError(f"{case} proof summary mismatch")
        if "verified in separate process; rejected different verifier, changed binding and corrupt proof" not in (target / "verify.stdout").read_text():
            raise RuntimeError(f"{case} proof controls missing")
        proof_receipts[case] = {"circuit": circuit, "k": k, "raw_sha256": sha(raw),
                                "preimage_sha256": sha(preimage), "params_sha256": sha(params),
                                "proof_sha256": sha(target / "statement.proof"),
                                "prover_sha256": sha(target / "statement.prover"),
                                "wrong_verifier_sha256": sha(wrong),
                                "verifier_sha256": sha(target / "statement.verifier")}

    consumer = output / "unused-funded"
    consumer.mkdir()
    old_manifest = (base / "funded-consumer/Cargo.toml").read_text()
    package_name = 'name = "ledger9-mixed-foreign-casts-funded"\nversion = "0.0.0"'
    if old_manifest.count(package_name) != 1:
        raise RuntimeError("base funded package changed")
    original = 'name = "ledger9-mixed-foreign-casts-funded"\npath = "main.rs"'
    if old_manifest.count(original) != 1:
        raise RuntimeError("base funded manifest changed")
    unused_dependency = f'constant = {{ package = "compact-contract-constant", path = "{base / "both/constant/contract"}", features = ["ledger-transaction"] }}'
    if old_manifest.count(unused_dependency) != 1:
        raise RuntimeError("base funded dependency changed")
    updated_manifest = old_manifest.replace(
        package_name, 'name = "ledger9-mixed-unused-funded"\nversion = "0.0.0"').replace(
        original, 'name = "ledger9-mixed-unused-funded"\npath = "main.rs"').replace(
        unused_dependency,
        f'unused = {{ package = "compact-contract-unused", path = "{base / "both/unused/contract"}", features = ["ledger-transaction"] }}')
    (consumer / "Cargo.toml").write_text(updated_manifest)
    base_lock = base / "funded-consumer/Cargo.lock"
    shutil.copyfile(base_lock, consumer / "Cargo.lock")
    shutil.copyfile(FAMILY / "funded-unused.rs", consumer / "main.rs")
    run(("cargo", "+1.99.0", "build", "--offline", "--manifest-path",
         str(consumer / "Cargo.toml")), env=env, log=output / "funded-build.log")
    def registry_packages(path):
        packages = tomllib.loads(path.read_text())["package"]
        return sorted((row["name"], row["version"], row.get("source"), row.get("checksum"))
                      for row in packages if row.get("source", "").startswith("registry+"))
    if registry_packages(base_lock) != registry_packages(consumer / "Cargo.lock"):
        raise RuntimeError("funded consumer changed the pinned registry lock")
    run(("cargo", "+1.99.0", "build", "--locked", "--offline", "--manifest-path",
         str(consumer / "Cargo.toml")), env=env, log=output / "funded-locked-build.log")
    funded_env = env.copy()
    funded_env.update(MIDNIGHT_PP=str(args.material), MIDNIGHT_LEDGER_TEST_STATIC_DIR=str(args.material),
                      RAYON_NUM_THREADS="4")
    result_dir = output / "funded-result"
    with (output / "funded-run.json").open("xb") as stdout, (
        output / "funded-run.stderr").open("xb") as stderr:
        subprocess.run((str(cargo_target / "debug/ledger9-mixed-unused-funded"), str(base),
                        str(capture), str(proofs), str(result_dir),
                        str(wrong_k8), str(args.wrong_verifier_k9)),
                       cwd=ROOT, env=funded_env, stdout=stdout, stderr=stderr, check=True)
    results = json.loads((output / "funded-run.json").read_text())
    if len(results) != len(CASES) or {row["case"] for row in results} != set(CASES):
        raise RuntimeError("funded result inventory mismatch")
    for row in results:
        if row["status"] != "passed" or not all(row[key] for key in (
            "default_strict", "ledger_matches_native", "typescript_preimage_fields_match",
            "wrong_verifier_refused", "replay_unchanged")):
            raise RuntimeError(f"{row['case']} funded transaction failed")
        if row["positive_fee_dust_spends"] != 1 or row["replay_refusal"] != "IntentAlreadyExists":
            raise RuntimeError(f"{row['case']} fee or replay check failed")
        after = result_dir / row["case"] / "ledger-after.bin"
        replay = result_dir / row["case"] / "ledger-replay.bin"
        if after.read_bytes() != replay.read_bytes():
            raise RuntimeError(f"{row['case']} replay changed ledger bytes")
        proof_receipts[row["case"]]["transaction_sha256"] = sha(
            result_dir / row["case"] / "applied-transaction.bin")
        proof_receipts[row["case"]]["transaction_proof_sha256"] = sha(
            result_dir / row["case"] / "transaction-bound.proof")
        proof_receipts[row["case"]]["ledger_after_sha256"] = sha(after)
    if git("status", "--porcelain=v1") or git("rev-parse", "HEAD") != source_head:
        raise RuntimeError("source changed during unused-cast qualification")
    receipt = {"platform": f"{platform.system()} {platform.machine()}",
               "head": source_head, "tree": source_tree,
               "prior_receipt_sha256": sha(prior / "receipt.json"),
               "base_toolchain_sha256": sha(base / "toolchain.json"),
               "capture_sha256": sha(capture / "capture.json"),
               "proof_runner": {"path": str(runner), "sha256": sha(runner),
                                "source_main_sha256": prior_receipt["proof_runner"]["source_main_sha256"],
                                "source_manifest_sha256": prior_receipt["proof_runner"]["source_manifest_sha256"],
                                "source_lock_sha256": prior_receipt["proof_runner"]["source_lock_sha256"]},
               "different_circuit_wrong_verifier_sha256": {"k8": sha(wrong_k8),
                                                              "k9": sha(args.wrong_verifier_k9)},
               "cases": proof_receipts, "funded_results": results,
               "limitations": ["Local platform only; native Linux remains required.",
                               "The original casts are unused in the fixture: their generated APIs execute, but the proof does not constrain their cast values.",
                               "The archived k=9 wrong-verifier key must be provisioned for Linux."]}
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"mixed unused-cast proofs: {len(CASES)} fresh proofs and funded replay checks passed; {output}")


if __name__ == "__main__":
    main()
