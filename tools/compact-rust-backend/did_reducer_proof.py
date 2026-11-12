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

"""Current Ledger 9 proof material and result checks for the DID reducers."""

from pathlib import Path
import re

import did_proof_gate as did
import local_parity_gate as common


def checked_file(path):
    did.require(path.is_file() and not path.is_symlink() and path.stat().st_size > 0,
                f"missing, linked or empty DID reducer artifact: {path}")
    return path


def require_zkir_version(execute, zkir, path):
    execute([zkir, "--version"], "zkir-version", path)
    did.require(path.read_text().strip() == "midnight-zkir 3.2.0",
                "expected published ZKIR v3 3.2.0")


def model(execute, receipt, zkir, output, geometry):
    raw = {checked_file(path).stem for path in (output / "zkir").glob("*.zkir")}
    did.require(raw == geometry.keys(), f"{output.name}: raw v3 circuit inventory differs")
    actual = did.model_circuits(execute, receipt, zkir, output,
                                {name: shape[0] for name, shape in geometry.items()})
    did.require(actual == geometry, f"{output.name}: exact v3 circuit geometry changed")
    return actual


def prepare_parameters(execute, receipt, runner, path, cache, selected_k):
    flags = [argument for k in sorted(selected_k) for argument in ("--parameter-k", k)]
    execute([runner, "--prepare-proof-material", *flags], "prepare-" + path.stem, path)
    prepared = did.read_json(path)
    did.require(prepared.get("format") == "compact-proof-material/v1"
                and prepared.get("mode") == "prepare"
                and prepared.get("verification") == "all files checked against locked upstream SHA256 tables"
                and prepared.get("requested_parameter_k") == sorted(selected_k)
                and selected_k <= {row["k"] for row in prepared["parameters"]}
                and Path(prepared["cache_directory"]).resolve() == cache.resolve(),
                "DID reducer proof material or parameter recipe differs")
    receipt[path.stem.replace("-", "_")] = prepared
    return did.hashes(checked_file(cache / row["name"])
                      for row in prepared["assets"] + prepared["parameters"])


def generated_hashes(output):
    contract = output / "contract"
    program_dir = contract / "checked-programs"
    programs = []
    # The maintained reducers use the ordinary Rust target. Functional
    # profiles additionally emit checked-program files.
    if program_dir.exists() or program_dir.is_symlink():
        did.require(program_dir.is_dir() and not program_dir.is_symlink(),
                    f"{output.name}: invalid checked-program directory")
        programs = sorted(program_dir.iterdir())
        did.require(programs and [p.name for p in programs] ==
                    sorted(f"program-{i}.json" for i in range(len(programs))),
                    f"{output.name}: checked-program inventory differs")
    paths = [contract / "lib.rs", contract / "compact-rust-ir.json",
             contract / "rust-capabilities.json", output / "compiler/contract-info.json",
             output / "compiler/contract-manifest.json",
             *programs, *sorted((output / "zkir").glob("*.zkir"))]
    return did.hashes(checked_file(path) for path in paths)


def keygen(execute, receipt, zkir, output, operation, expected, label):
    (output / "keys").mkdir(exist_ok=True)
    execute([zkir, "compile", output / f"zkir/{operation}.zkir",
             output / f"keys/{operation}.prover", output / f"keys/{operation}.verifier"], label)
    match = re.search(r"\(k=(\d+), rows=(\d+)\)",
                      Path(receipt["commands"][-1]["log"]).read_text())
    did.require(match is not None and tuple(map(int, match.groups())) == expected,
                f"{output.name}/{operation}: key generation differs from v3 model")


def validate_call(call):
    did.validate_call(call)
    for flag in ("separate_verification", "corrupt_proof_rejected"):
        did.require(call.get(flag) is True, f"DID reducer result missing {flag}")


def proof_artifacts(output, stem, call, state_bytes):
    proof = checked_file(output / f"{stem}.proof")
    transaction_proof = checked_file(output / f"{stem}-transaction.proof")
    inputs = checked_file(output / f"{stem}.inputs")
    transaction = checked_file(output / f"{stem}-transaction.bin")
    state = checked_file(output / f"{stem}-state.bin")
    after = checked_file(output / f"{stem}-ledger-after.bin")
    replay = checked_file(output / f"{stem}-ledger-replay.bin")
    did.require(proof.stat().st_size == call["proof_bytes"]
                and transaction_proof.stat().st_size == call["transaction_proof_bytes"]
                and state.stat().st_size == state_bytes
                and common.sha256(proof) != common.sha256(transaction_proof),
                f"{stem}: standalone proof, transaction proof or final state differs from receipt")
    did.require(common.sha256(after) == common.sha256(replay), f"{stem}: replay bytes differ")
    return [proof, transaction_proof, inputs, transaction, state, after, replay]
