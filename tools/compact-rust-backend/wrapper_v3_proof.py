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

"""Measure complete wrapper models and bind the selected Ledger 9 proof artifacts."""
import json
from pathlib import Path
import re

from did_proof_gate import hashes, require, unchanged


def prepare_material(execute, receipt, generated, output, zkir, runner, cache, geometry, selected):
    version = output / "zkir-version.stdout"
    execute([zkir, "--version"], "zkir-version", version)
    require(version.read_text().strip() == "midnight-zkir 3.2.0", "expected published ZKIR v3 3.2.0")
    ir_dir = generated / "zkir"
    require({p.name for p in ir_dir.iterdir()} == {f"{name}.zkir" for name in geometry},
            "complete wrapper ZKIR export roster differs")
    require(selected and len(selected) == len(set(selected))
            and all(1 <= geometry[name][0] <= 8 for name in selected), "wrapper selection exceeds tiny policy")
    receipt["models"] = {}
    for name, expected in geometry.items():
        execute([zkir, "mock-compile", ir_dir / f"{name}.zkir"], f"model-{name}")
        match = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
        require(match is not None and tuple(map(int, match.groups())) == expected, f"{name} model differs")
        receipt["models"][name] = dict(k=expected[0], rows=expected[1], selected=name in selected)
    ks = sorted({geometry[name][0] for name in selected})
    material = output / "proof-material.json"
    execute([runner, "--prepare-proof-material", "--verify-only", *[arg for k in ks for arg in ("--parameter-k", k)]],
            "verify-builtins-and-selected-parameters", material)
    prepared = json.loads(material.read_text())
    require(prepared["mode"] == "verify-only"
            and prepared["verification"] == "all files checked against locked upstream SHA256 tables"
            and Path(prepared["cache_directory"]).resolve() == Path(cache).resolve()
            and set(ks).issubset(p["k"] for p in prepared["parameters"]), "wrapper proof material receipt differs")
    parameters = [Path(cache) / p["name"] for p in prepared["assets"] + prepared["parameters"]]
    pinned = hashes(parameters)
    keys = generated / "keys"
    keys.mkdir()
    receipt["keygen"] = []
    artifacts = []
    for name in selected:
        prover, verifier = keys / f"{name}.prover", keys / f"{name}.verifier"
        execute([zkir, "compile", ir_dir / f"{name}.zkir", prover, verifier], f"keygen-{name}")
        match = re.search(r"\(k=(\d+), rows=(\d+)\)", Path(receipt["commands"][-1]["log"]).read_text())
        require(match is not None and tuple(map(int, match.groups())) == geometry[name], f"{name} keygen model differs")
        k, rows = geometry[name]
        receipt["keygen"].append(dict(mode="fresh direct native", circuit=name, k=k, rows=rows))
        artifacts.extend((prover, verifier, ir_dir / f"{name}.zkir", ir_dir / f"{name}.bzkir"))
    require(all(p.is_file() and p.stat().st_size > 0 for p in parameters + artifacts), "wrapper proof material missing")
    unchanged(pinned)
    return pinned | hashes(artifacts)


def verify_artifacts(execute, receipt, generated, runner, selector, prefix, calls):
    for index, row in enumerate(calls):
        stem = generated / f"{prefix}-{index}"
        standalone = Path(f"{stem}.proof").read_bytes()
        transaction = Path(f"{stem}-transaction.proof").read_bytes()
        require(len(standalone) == row["proof_bytes"] and len(transaction) == row["transaction_proof_bytes"]
                and standalone != transaction, "distinct wrapper proof artifact binding differs")
        after = Path(f"{stem}-ledger-after.bin").read_bytes()
        require(after and after == Path(f"{stem}-ledger-replay.bin").read_bytes(), "serialized wrapper replay differs")
        tx = Path(f"{stem}-transaction.bin")
        require(tx.is_file() and tx.stat().st_size > 0, "missing applied wrapper transaction")
        if "transaction_bytes" in row:
            require(tx.stat().st_size == row["transaction_bytes"], "applied wrapper transaction size differs")
    receipt["result_hashes"] = hashes(p for p in generated.glob(f"{prefix}-*") if p.is_file())
    execute([runner, selector, generated], "independent-standalone-verification")
    log = Path(receipt["commands"][-1]["log"]).read_text()
    for index, row in enumerate(calls):
        name = row.get("circuit", "inferred")
        require(f"{name}/{index}: standalone v3 proof verified and corruption refused" in log,
                "wrapper verifier omitted a selected case")
    unchanged(receipt["result_hashes"])
