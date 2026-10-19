#!/usr/bin/env python3
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

"""Exercise the real functional frontend's original-occurrence source boundary.

Private wrappers run the configured official Scheme compiler first, then mutate
only that invocation's metadata or private source fixture. Native compilation
must remain valid while experimental publication refuses. This checks optional
metadata enforcement and atomic output, not consumed-source authentication.
No keys, proofs, network services or result caches are involved.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    "import CompactStandardLibrary;\n"
    "export ledger value: Field;\n"
    "export circuit update(input: Field): [] { value = disclose(input); }\n"
    "export circuit read(): Field { return value; }\n"
)

# Address rows identify original frontend occurrences. Never use candidate
# diagnostic paths or a caller's proposed source point to select a root span.
WRAPPER = r'''#!/usr/bin/env python3
import hashlib, json, os, subprocess, sys
from pathlib import Path
status = subprocess.run([SCHEME, *sys.argv[1:]], check=False)
if status.returncode:
    raise SystemExit(status.returncode)
source, stage = Path(sys.argv[-2]), Path(sys.argv[-1])
metadata = stage / "contract/compact-rust-origins.raw.json"
ir_path = stage / "contract/compact-rust-ir.json"
data = json.loads(metadata.read_text())
ir = json.loads(ir_path.read_text())
arrays = [i for i, row in enumerate(data["addresses"])
          if row[0] == 0 and row[1] == {"field": "stateful_circuits"}]
assert len(arrays) == 1
roots = [(i, row) for i, row in enumerate(data["addresses"])
         if row[0] == arrays[0] and "index" in row[1]]
assert len(roots) == len(ir["stateful_circuits"]) == 2
points = {}
for _, row in roots:
    index = row[1]["index"]
    span = data["spans"][data["origins"][row[3]][0]]
    assert span[7]["kind"] == "physical_source"
    points[ir["stateful_circuits"][index]["name"]] = {
        "file": data["files"][span[0]]["path"],
        "line": span[1], "column": span[2],
    }
mode = os.environ["COMPACT_FUNCTIONAL_ORIGIN_CASE"]
audit = {"mode": mode, "bound_root_points": points,
         "metadata_before_sha256": hashlib.sha256(metadata.read_bytes()).hexdigest()}
if mode == "missing":
    metadata.unlink()
elif mode == "malformed-json":
    metadata.write_text("{not-json")
elif mode == "wrong-node-kind":
    roots[0][1][2] = "wrong_original_kind"
    metadata.write_text(json.dumps(data))
elif mode == "detached-origin":
    # Remove this original occurrence's source attachment without changing the
    # valid typed address protocol. Native publication tolerates absent origins.
    roots[0][1][3] = None
    metadata.write_text(json.dumps(data))
elif mode == "nonphysical":
    contexts = [i for i, span in enumerate(data["spans"])
                if span[7]["kind"] == "compiler_context"]
    assert contexts
    nonphysical_origins = [i for i, origin in enumerate(data["origins"])
                           if origin[0] in contexts]
    assert nonphysical_origins
    roots[0][1][3] = nonphysical_origins[0]
    metadata.write_text(json.dumps(data))
elif mode == "snapshot-truncate":
    # Break endpoint correspondence, without claiming that same-size content
    # mutations would be detected by this weaker existing snapshot boundary.
    source.write_text("")
elif mode == "source-annotation":
    for circuit in ir["stateful_circuits"]:
        assert circuit.get("source") is not None
        circuit["source"] = {"file": "candidate-decoy.compact", "line": 999, "column": 88}
    ir_path.write_text(json.dumps(ir))
elif mode != "positive":
    raise AssertionError(mode)
Path(os.environ["COMPACT_FUNCTIONAL_ORIGIN_AUDIT"]).write_text(json.dumps(audit))
'''


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def snapshot(directory: Path) -> dict[str, str]:
    return {str(path.relative_to(directory)): digest(path.read_bytes())
            for path in directory.rglob("*") if path.is_file()}


def program_metadata(library: Path) -> list[dict]:
    # This deliberately ASCII-only fixture uses ordinary JSON-compatible Rust
    # string escapes. This is a test extractor, not a general Rust parser.
    from check_compactc_target import check_manifest
    check_manifest(library.parent.parent)
    source = library.read_text()
    strings = re.findall(r'pub const CHECKED_PROGRAM:\s*&str\s*=\s*("(?:[^"\\]|\\.)*")\s*;', source)
    if strings:
        return [json.loads(json.loads(literal)) for literal in strings]
    paths = re.findall(
        r'pub const CHECKED_PROGRAM:\s*&str\s*=\s*include_str!\s*\(\s*("[^"\n]+")\s*\)\s*;', source)
    programs = []
    for literal in paths:
        relative = Path(json.loads(literal))
        assert not relative.is_absolute() and ".." not in relative.parts
        programs.append(json.loads((library.parent / relative).read_bytes()))
    return programs


def run(compiler: Path, scheme: Path, report: dict) -> None:
    with tempfile.TemporaryDirectory(prefix="compact-functional-origins-") as private:
        directory = Path(private)
        report["private_fixture"] = str(directory)
        wrapper = directory / "frontend.py"
        wrapper.write_text(WRAPPER.replace("SCHEME", repr(str(scheme)), 1))
        wrapper.chmod(0o700)
        source = directory / "origin-control.compact"
        audit_path = directory / "audit.json"

        def compile_case(mode: str, functional: bool, output: Path) -> tuple[subprocess.CompletedProcess, dict]:
            source.write_text(SOURCE)
            audit_path.unlink(missing_ok=True)
            command = [str(compiler), "--target", "rust", "--skip-zk",
                       "--rust-runtime-root", str(ROOT)]
            if functional:
                command.append("--rust-functional-ledger8")
            command.extend([str(source), str(output)])
            environment = dict(os.environ, COMPACTC_SCHEME=str(wrapper),
                               COMPACT_FUNCTIONAL_ORIGIN_CASE=mode,
                               COMPACT_FUNCTIONAL_ORIGIN_AUDIT=str(audit_path))
            result = subprocess.run(command, env=environment, capture_output=True,
                                    text=True, check=False, cwd=directory)
            audit = json.loads(audit_path.read_text()) if audit_path.exists() else None
            report["invocations"].append({
                "case": mode, "functional": functional, "command": command,
                "returncode": result.returncode, "stdout": result.stdout,
                "stderr": result.stderr, "frontend_audit": audit,
                "source_after_sha256": digest(source.read_bytes()),
            })
            assert audit is not None, f"official frontend did not finish for {mode}: {result.stderr}"
            return result, audit

        for mode in ("positive", "source-annotation"):
            output = directory / f"{mode}-accepted"
            result, audit = compile_case(mode, True, output)
            assert result.returncode == 0, result.stderr
            programs = program_metadata(output / "contract/lib.rs")
            assert {program["root"] for program in programs} == {"read", "update"}
            for program in programs:
                origin = program["root_origin"]
                assert origin["kind"] == "bound_original_root_fallback"
                assert origin["source"] == audit["bound_root_points"][program["root"]]
                assert origin["consumed_source_bytes"] == "unknown"
                assert origin["snapshot_correspondence"] == "unknown"
                assert all(effect["privacy"] == "unknown" for effect in program["effects"])
            assert source.read_text() == SOURCE
            report["cases"].append({"case": mode, "accepted_programs": len(programs),
                                    "actual_bound_coordinates": True})

        for mode in ("missing", "detached-origin", "nonphysical", "snapshot-truncate"):
            output = directory / f"{mode}-existing"
            native, _ = compile_case(mode, False, output)
            assert native.returncode == 0, f"native must remain valid for {mode}: {native.stderr}"
            assert (output / "contract/lib.rs").is_file()
            manifest = output / "compiler/contract-manifest.json"
            assert manifest.is_file()
            (output / "preserved-sentinel.txt").write_text("previous complete publication\n")
            before = snapshot(output)
            rejected, _ = compile_case(mode, True, output)
            assert rejected.returncode != 0, f"functional accepted {mode}"
            assert "no retained original source binding" in rejected.stderr, rejected.stderr
            assert snapshot(output) == before, f"refusal changed existing output for {mode}"
            assert not list(directory.glob(f".{output.name}.compactc-stage-*"))
            fresh = directory / f"{mode}-fresh"
            refused, _ = compile_case(mode, True, fresh)
            assert refused.returncode != 0, f"fresh functional accepted {mode}"
            assert "no retained original source binding" in refused.stderr, refused.stderr
            assert not fresh.exists(), f"refusal published fresh output for {mode}"
            assert not list(directory.glob(f".{fresh.name}.compactc-stage-*"))
            expected_source = "" if mode == "snapshot-truncate" else SOURCE
            assert source.read_text() == expected_source
            report["cases"].append({"case": mode, "native_accepted": True,
                                    "existing_refused_unchanged": True, "fresh_refused_absent": True,
                                    "staging_absent": True, "previous_file_count": len(before),
                                    "previous_manifest_sha256": before[str(manifest.relative_to(output))]})
        # Malformed transport and a corrupted typed-address binding are already
        # shared publication refusals. Do not misreport them as native-valid or
        # relax their existing protocol gate to manufacture an opt-in contrast.
        for mode in ("malformed-json", "wrong-node-kind"):
            output = directory / f"{mode}-existing"
            accepted, _ = compile_case("positive", False, output)
            assert accepted.returncode == 0, accepted.stderr
            (output / "preserved-sentinel.txt").write_text("previous complete publication\n")
            before = snapshot(output)
            for functional in (False, True):
                rejected, _ = compile_case(mode, functional, output)
                assert rejected.returncode != 0, f"invalid protocol accepted: {mode}"
                expected = ("no retained original source binding" if functional
                            else "origin metadata protocol problem")
                assert expected in rejected.stderr, rejected.stderr
                assert snapshot(output) == before
                assert not list(directory.glob(f".{output.name}.compactc-stage-*"))
            fresh = directory / f"{mode}-fresh"
            rejected, _ = compile_case(mode, True, fresh)
            assert rejected.returncode != 0 and not fresh.exists()
            assert not list(directory.glob(f".{fresh.name}.compactc-stage-*"))
            report["cases"].append({"case": mode, "shared_protocol_refusal": True,
                                    "existing_refused_unchanged": True,
                                    "fresh_refused_absent": True, "staging_absent": True})
    report["private_fixture_removed"] = not directory.exists()


def main() -> int:
    if not __debug__:
        raise RuntimeError("functional origin checks require Python assertions; omit -O")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--compiler", type=Path, required=True)
    parser.add_argument("--scheme", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    compiler, scheme = args.compiler.resolve(strict=True), args.scheme.resolve(strict=True)
    report = {"schema": 1, "compiler_sha256": digest(compiler.read_bytes()),
              "harness_sha256": digest(Path(__file__).read_bytes()),
              "scheme_sha256": digest(scheme.read_bytes()), "source_sha256": digest(SOURCE.encode()),
              "scope": "original binding and endpoint correspondence; not consumed-source authentication",
              "cases": [], "invocations": []}
    try:
        run(compiler, scheme, report)
    except (AssertionError, OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        report["error"] = str(error)
        report["passed"] = False
    else:
        report["passed"] = True
    if "private_fixture" in report:
        report["private_fixture_removed"] = not Path(report["private_fixture"]).exists()
        if not report["private_fixture_removed"]:
            report["passed"] = False
            report["error"] = "private fixture cleanup failed"
    report["compiler_after_sha256"] = digest(compiler.read_bytes())
    report["scheme_after_sha256"] = digest(scheme.read_bytes())
    if (report["compiler_after_sha256"] != report["compiler_sha256"]
            or report["scheme_after_sha256"] != report["scheme_sha256"]):
        report["passed"] = False
        report["error"] = "compiler or Scheme binary changed during qualification"
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(f"Functional origin controls: {len(report['cases'])}/8 completed; passed={report['passed']}")
    if not report["passed"]:
        print(report["error"], file=sys.stderr)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
