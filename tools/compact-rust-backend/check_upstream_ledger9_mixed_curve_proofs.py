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

"""Qualify the original mixed Curve25519 branches with proofs and funded replay."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[2]
FAMILY = ROOT / "test-contracts/stdlib/secp256r1/casts/mixed-original"
BASE = ROOT / "tools/compact-rust-backend/check_upstream_ledger9_mixed_foreign_casts.py"
CASES = {"castC25519-false": ("castC25519", 11),
         "castC25519-true": ("castC25519", 11), "castWide": ("castWide", 10)}


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
                 "wrong-verifier-k10", "wrong-verifier-k11", "output",
                 "cargo-target"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    source_head = git("rev-parse", "HEAD")
    source_tree = git("rev-parse", "HEAD^{tree}")
    if git("status", "--porcelain=v1"):
        parser.error("source checkout must be clean")
    for name in ("compiler", "scheme_compiler", "ts_runtime", "zkir", "zkir_wasm",
                 "proof_runner", "proof_source", "material", "wrong_verifier",
                 "wrong_verifier_k10", "wrong_verifier_k11"):
        setattr(args, name, getattr(args, name).resolve(strict=True))
    output = args.output.resolve()
    if args.output.is_symlink() or output.exists():
        parser.error("output must be absent")
    output.mkdir(parents=True)
    cargo_target = args.cargo_target.resolve()
    cargo_target.mkdir(parents=True, exist_ok=True)
    base = output / "base"
    options = []
    for name in ("compiler", "scheme-compiler", "ts-runtime", "zkir", "zkir-wasm",
                 "proof-runner", "material", "wrong-verifier"):
        options.extend(("--" + name, str(getattr(args, name.replace("-", "_")))))
    options.extend(("--output", str(base), "--cargo-target", str(cargo_target)))
    run(("python3", str(BASE), *options), log=output / "base-run.log")
    base_toolchain = json.loads((base / "toolchain.json").read_text())
    if base_toolchain["source_head"] != source_head or base_toolchain["source_tree"] != source_tree:
        raise RuntimeError("base family source does not match this checkout")

    capture = output / "curve-capture"
    run(("node", str(FAMILY / "capture-curve.mjs"),
         str(base / "both/constant/contract/index.js"), str(args.zkir_wasm), str(capture)),
        log=output / "capture-run.log")
    captures = json.loads((capture / "capture.json").read_text())
    if set(captures) != set(CASES):
        raise RuntimeError("Curve25519 case inventory changed")
    material_receipt = json.loads((args.material.parent / "material-receipt.json").read_text())
    archived_proofs = json.loads((args.proof_runner.parent / "receipt.json").read_text())
    for k, case in ((10, "constant-zero"), (11, "constant-false")):
        wrong = getattr(args, f"wrong_verifier_k{k}")
        if sha(wrong) != archived_proofs[case]["pins"]["statement.verifier"]:
            raise RuntimeError(f"k={k} wrong verifier differs from archived proof")
    proof_source = args.proof_source
    manifest = proof_source / "Cargo.toml"
    if not (proof_source / "src/main.rs").is_file() or not (proof_source / "Cargo.lock").is_file():
        raise RuntimeError("proof runner source is incomplete")
    for name, path in {"main": proof_source / "src/main.rs", "manifest": manifest,
                       "lock": proof_source / "Cargo.lock"}.items():
        if sha(path) != base_toolchain["proof_runner_source"][name]["sha256"]:
            raise RuntimeError(f"{name} proof runner source differs from archived binary source")
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(cargo_target)
    run(("cargo", "+1.99.0", "build", "--locked", "--offline", "--manifest-path", str(manifest)),
        env=env, log=output / "proof-runner-build.log")
    runner = cargo_target / "debug/secp256k1-unary-proof"
    if not runner.is_file():
        raise RuntimeError("source-built proof runner missing")
    wrong_sha = sha(args.wrong_verifier)
    proofs = output / "curve-proofs"
    proofs.mkdir()
    proof_receipts = {}
    for case, (circuit, k) in CASES.items():
        row = captures[case]
        if row["circuit"] != circuit or row["honest_check"] != "passed" or row["changed_write_check"] != "refused":
            raise RuntimeError(f"{case} preimage check failed")
        raw = base / f"both/constant/zkir/{circuit}.zkir"
        preimage = capture / case / "honest.preimage"
        changed = capture / case / "changed-write.preimage"
        params = args.material / f"bls_midnight_2p{k}"
        if sha(raw) != row["raw_sha256"] or sha(preimage) != row["preimage_sha256"] or sha(changed) != row["changed_write_sha256"]:
            raise RuntimeError(f"{case} source or preimage changed")
        if sha(params) != material_receipt["files"][params.name]["sha256"]:
            raise RuntimeError(f"{case} proof parameters changed")
        target = proofs / case
        target.mkdir()
        wrong = getattr(args, f"wrong_verifier_k{k}")
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

    consumer = output / "curve-funded"
    consumer.mkdir()
    old_manifest = (base / "funded-consumer/Cargo.toml").read_text()
    original = 'name = "ledger9-mixed-foreign-casts-funded"\npath = "main.rs"'
    if old_manifest.count(original) != 1:
        raise RuntimeError("base funded manifest changed")
    (consumer / "Cargo.toml").write_text(old_manifest.replace(
        original, 'name = "ledger9-mixed-curve-funded"\npath = "main.rs"'))
    shutil.copyfile(base / "funded-consumer/Cargo.lock", consumer / "Cargo.lock")
    shutil.copyfile(FAMILY / "funded-curve.rs", consumer / "main.rs")
    run(("cargo", "+1.99.0", "build", "--locked", "--offline", "--manifest-path",
         str(consumer / "Cargo.toml")), env=env, log=output / "funded-build.log")
    funded_env = env.copy()
    funded_env.update(MIDNIGHT_PP=str(args.material), MIDNIGHT_LEDGER_TEST_STATIC_DIR=str(args.material),
                      RAYON_NUM_THREADS="4")
    result_dir = output / "funded-result"
    with (output / "funded-run.json").open("xb") as stdout, (
        output / "funded-run.stderr").open("xb") as stderr:
        subprocess.run((str(cargo_target / "debug/ledger9-mixed-curve-funded"), str(base),
                        str(capture), str(proofs), str(result_dir),
                        str(args.wrong_verifier_k10), str(args.wrong_verifier_k11)),
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
        raise RuntimeError("source changed during Curve25519 qualification")
    receipt = {"platform": f"{platform.system()} {platform.machine()}",
               "head": source_head, "tree": source_tree,
               "base_toolchain_sha256": sha(base / "toolchain.json"),
               "capture_sha256": sha(capture / "capture.json"),
               "proof_runner": {"path": str(runner), "sha256": sha(runner),
                                "source_main_sha256": sha(proof_source / "src/main.rs"),
                                "source_manifest_sha256": sha(manifest),
                                "source_lock_sha256": sha(proof_source / "Cargo.lock")},
               "base_wrong_verifier_sha256": wrong_sha,
               "curve_wrong_verifier_sha256": {"k10": sha(args.wrong_verifier_k10),
                                                 "k11": sha(args.wrong_verifier_k11)},
               "cases": proof_receipts, "funded_results": results,
               "limitations": ["Local platform only; native Linux remains required.",
                               "The other eight original mixed-cast exports retain open proof and funded coverage."]}
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"mixed Curve25519 proofs: {len(CASES)} fresh proofs and funded replay checks passed; {output}")


if __name__ == "__main__":
    main()
