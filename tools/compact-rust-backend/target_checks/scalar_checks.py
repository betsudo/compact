# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Scalar arithmetic, comparison, and byte-conversion target checks.

The CLI owns flag precedence, environment setup, and temporary-directory lifetime.
Sources and command runners are explicit inputs; each check retains its own
acceptance assertions, failure order, and optional proof commands.
"""

from collections.abc import Callable
import json
from pathlib import Path
from subprocess import CompletedProcess


def _prove_source_bound_v3(
    output: Path,
    retained: dict,
    circuit: str,
    selector: str,
    *,
    run: Callable[..., None],
) -> None:
    ir = json.loads((output / "contract/compact-rust-ir.json").read_text())
    keys = output / "keys"
    if json.dumps(ir, sort_keys=True) != json.dumps(retained, sort_keys=True) or keys.exists():
        raise ValueError(f"{selector} source IR differs or preflight produced keys")
    keys.mkdir()
    run("zkir-v3", "compile", str(output / f"zkir/{circuit}.zkir"),
        str(keys / f"{circuit}.prover"), str(keys / f"{circuit}.verifier"))
    for flag in (selector, f"{selector}-verify"):
        run("cargo", "run", "--quiet", "-p", "compact-rust-proof-smoke", "--", flag, str(output))


def check_wide_add(
    compiler: str,
    base: Path,
    *,
    root: Path,
    run: Callable[..., None],
    capture: Callable[..., CompletedProcess[str]],
) -> None:
    source = root / "examples/rust_backend/wide_add_oracle.compact"
    output = base / "wide-add"
    run(compiler, "--target", "rust", "--skip-zk", str(source), str(output))
    generated = (output / "contract/lib.rs").read_text()
    assert "runtime::add_wide_unsigned" in generated
    assert "runtime::narrow_wide_uint" in generated
    compatibility = json.loads((root / "runtime-rs/compatibility.json").read_text())
    abi = compatibility["runtime_abi"]
    assert f"assert!(runtime::RUST_RUNTIME_ABI == {abi});" in generated, (
        "wide add runtime ABI differs from the current compatibility record"
    )
    report = json.loads((output / "contract/rust-capabilities.json").read_text())
    assert len(report["circuits"]) == 1
    row = report["circuits"][0]
    assert row["recorded"] and row["observed_call"]
    assert not row["proof_required"] and row["recording_status"] == "not_applicable"
    original = capture([compiler, "--target", "rust", "--skip-zk", str(root / "test-center/test-contracts/micro-dao.compact"), str(base / "original-dao")], cwd=root, capture_output=True, text=True)
    assert original.returncode == 0, original.stderr
    assert (base / "original-dao/contract/lib.rs").is_file()
    print("wide addition admitted; original micro-dao emits native Rust")


def check_counter_less_than(
    compiler: str,
    base: Path,
    *,
    root: Path,
    proof_requested: bool,
    run: Callable[..., None],
) -> None:
    output = base / "counter-less-than"
    run(compiler, "--target", "rust", "--rust-require-recording", "--skip-zk",
        str(root / "examples/rust_backend/counter_less_than_oracle.compact"), str(output))
    ir = json.loads((output / "contract/compact-rust-ir.json").read_text())
    assert ir["schema_version"] == 20
    report = json.loads((output / "contract/rust-capabilities.json").read_text())
    assert {row["name"] for row in report["circuits"]} == {"compare", "nested", "short_circuit", "checked"}
    assert all(row["recorded"] and row["observed_call"] for row in report["circuits"])
    def kinds(value):
        if isinstance(value, dict):
            yield value.get("kind")
            for child in value.values():
                yield from kinds(child)
        elif isinstance(value, list):
            for child in value:
                yield from kinds(child)
    assert {"counter_less_than", "counter_read"}.issubset(set(kinds(ir)))
    if proof_requested:
        retained = json.loads((root / "tools/compact-rust-backend/tests/counter-less-than-schema15-ir.json").read_text())
        if retained["schema_version"] != 15:
            raise ValueError("unexpected Counter lessThan retained IR schema")
        retained["schema_version"] = 20
        _prove_source_bound_v3(output, retained, "compare", "--counter-less-than", run=run)
    print("typed Counter less-than target gate passed")


def prove_literal_bytes_field(
    compiler: str,
    output: Path,
    *,
    root: Path,
    run: Callable[..., None],
) -> None:
    run(compiler, "--target", "rust", "--rust-require-recording", "--skip-zk",
        str(root / "examples/rust_backend/literal_bytes_field_oracle.compact"), str(output))
    retained = json.loads((root / "tools/compact-rust-backend/tests/literal-bytes-field-schema20-ir.json").read_text())
    _prove_source_bound_v3(output, retained, "snapshot", "--literal-bytes-field", run=run)


def check_literal_bytes_field(
    compiler: str,
    base: Path,
    *,
    root: Path,
    proof_requested: bool,
    run: Callable[..., None],
    python_executable: str,
) -> None:
    run(python_executable, str(root / "tools/compact-rust-backend/check_literal_bytes_field.py"))
    if proof_requested:
        prove_literal_bytes_field(compiler, base / "literal-bytes-field", root=root, run=run)


def check_field_to_bytes32(
    compiler: str,
    base: Path,
    *,
    root: Path,
    field_to_bytes32_source: Path,
    proof_requested: bool,
    run: Callable[..., None],
    capture: Callable[..., CompletedProcess[str]],
) -> None:
    output = base / "field-to-bytes32"
    command = [compiler, "--target", "rust"]
    command.append("--rust-require-recording")
    if not proof_requested:
        command.append("--skip-zk")
    run(*command, str(field_to_bytes32_source), str(output))
    ir = json.loads((output / "contract/compact-rust-ir.json").read_text())
    assert ir["schema_version"] == 20
    assert [(row["name"], row["body"]["kind"])
            for row in ir["circuits"]] == [("encode", "field_to_bytes32")]
    assert [(row["name"], row["return_value"]["value"]["kind"])
            for row in ir["stateful_circuits"]] == [("snapshot", "field_to_bytes32")]
    capabilities = json.loads((output / "contract/rust-capabilities.json").read_text())
    assert [(row["name"], row["proof_required"], row["recorded"], row["observed_call"])
            for row in capabilities["circuits"]] == [
        ("snapshot", True, True, True)]
    if proof_requested:
        for extension in ("prover", "verifier"):
            assert (output / "keys" / f"snapshot.{extension}").is_file()
        for extension in ("zkir", "bzkir"):
            assert (output / "zkir" / f"snapshot.{extension}").is_file()
    rejected_source = base / "unsupported-byte-width.compact"
    rejected_source.write_text("export circuit encode(value: Field): Bytes<31> { return value as Bytes<31>; }\n")
    rejected = capture(
        [compiler, "--target", "rust", "--skip-zk", str(rejected_source),
         str(base / "unsupported-byte-width")],
        cwd=root, capture_output=True, text=True,
    )
    assert rejected.returncode != 0
    assert "Field-to-Bytes<32> casts only" in rejected.stderr
    print("field-to-bytes32 target gate passed")
