#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#  http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Check the pinned source and test inventory for the 37 codegen-rust oracles.

This checks provenance and the presence of executable Rust/TypeScript fixture
links. It does not claim result, state, gas, or transcript parity: those claims
belong to the individual executing tests and the M2 acceptance matrix.
"""

import hashlib
import json
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).with_name("oracle_acceptance.json")
LEDGER9_BINDINGS = Path(__file__).with_name("oracle_ledger9_bindings.json")


def non_comment_lines(data: bytes) -> bytes:
    """Ignore only full-line Compact comments, preserving executable text."""
    return "\n".join(
        line for line in data.decode().splitlines() if not line.lstrip().startswith("//")
    ).strip().encode()


def relative_file(name: str) -> Path:
    path = ROOT / name
    if path.resolve().is_relative_to(ROOT.resolve()) and path.is_file():
        return path
    raise ValueError(f"missing or invalid repository file: {name}")



def ledger9_bindings() -> dict:
    bindings = json.loads(LEDGER9_BINDINGS.read_text())
    compatibility = json.loads(relative_file("runtime-rs/compatibility.json").read_text())
    if (bindings["format"] != "compact-oracle-ledger9-bindings/v1"
            or bindings["ledger_version"] != compatibility["ledger_version"]):
        raise ValueError("Ledger 9 oracle binding version differs")

    reviewed = {}
    for filename, field in (
        ("oracle_direct_behavior_review.json", "reviewed_rust_test_sha256"),
        ("oracle_call_registry_behavior_review.json", "artifact_sha256"),
        ("oracle_behavior_review.json", "reviewed_file_sha256"),
    ):
        for path, digest in json.loads(MANIFEST.with_name(filename).read_text())[field].items():
            reviewed.setdefault(path, set()).add(digest)

    used_captures = set()
    for name, binding in bindings["tests"].items():
        path = Path(name)
        is_test = (path.parts[:1] == ("tests-rust-backend",) and path.suffix == ".rs"
                   and ("tests" in path.parts or name == "tests-rust-backend/support/oracle_recorded_trace.rs"))
        if not is_test or reviewed.get(name) != {binding["historical_sha256"]}:
            raise ValueError(f"unreviewed Ledger 9 test binding: {name}")
        contents = relative_file(name).read_bytes()
        if hashlib.sha256(contents).hexdigest() != binding["current_sha256"]:
            raise ValueError(f"Ledger 9 reviewed Rust test changed: {name}")
        for historical in binding["capture_paths"]:
            current = bindings["captures"][historical]["current"]
            if (current + '"').encode() not in contents:
                raise ValueError(f"Ledger 9 test does not reference {current}: {name}")
            used_captures.add(historical)
    if used_captures != set(bindings["captures"]):
        raise ValueError("Ledger 9 reviewed capture membership differs")

    for historical, binding in bindings["captures"].items():
        path = Path(historical)
        current = str(path.with_name("ledger9-" + path.name))
        if path.parent != Path("runtime-rs/tests/fixtures") or binding["current"] != current:
            raise ValueError(f"invalid Ledger 9 capture path: {historical}")
        for name, field in ((historical, "historical_sha256"), (current, "current_sha256")):
            if hashlib.sha256(relative_file(name).read_bytes()).hexdigest() != binding[field]:
                raise ValueError(f"reviewed capture changed: {name}")
    return bindings


def reviewed_test_matches(path: str, digest: str, bindings: dict) -> bool:
    # The loader has already checked both historical authority and active bytes.
    binding = bindings["tests"].get(path)
    if binding is not None:
        return digest == binding["historical_sha256"]
    return hashlib.sha256(relative_file(path).read_bytes()).hexdigest() == digest


def reviewed_behavior_failures(bindings: dict) -> list[str]:
    """Check reviewed artifact identity/links, never infer semantic coverage from text."""
    review = json.loads(Path(__file__).with_name("oracle_direct_behavior_review.json").read_text())
    failures = []
    for path, digest in review["source_hashes"].items():
        if hashlib.sha256(relative_file(path).read_bytes()).hexdigest() != digest:
            failures.append(f"reviewed behavior artifact changed; re-review required: {path}")
    for path, digest in review["reviewed_rust_test_sha256"].items():
        if not reviewed_test_matches(path, digest, bindings):
            failures.append(f"reviewed behavior test changed; re-review required: {path}")
    capture_path = "runtime-rs/tests/fixtures/oracle-direct-behavior.json"
    for path, digest in [(capture_path, review["capture_sha256"]),
                         (review["capture_script"], review["capture_script_sha256"])]:
        if hashlib.sha256(relative_file(path).read_bytes()).hexdigest() != digest:
            failures.append(f"reviewed behavior capture changed; re-review required: {path}")
    capture = json.loads(relative_file(capture_path).read_text())
    rows = review["rows"]
    identities = {(row["source"], row["export"]) for row in rows}
    if len(identities) != len(rows) or len(rows) != review["reviewed_export_count"]:
        failures.append("duplicate or incomplete reviewed export identities")
    pure_names = set(capture["literal"]["exports"])
    if {row["export"] for row in rows if row["kind"] == "pure"} != pure_names:
        failures.append("literal reviewed export set differs from independent capture")
    for row in rows:
        relative_file(row["rust_test"])
        if row["kind"] == "pure":
            ids = [case["id"] for case in capture["literal"]["cases"] if case["export"] == row["export"]]
        else:
            ids = [case["export"] + "/" + ("default" if not case["args"] else str(case["args"][0]).lower())
                   for case in capture["stateful"] if case["export"] == row["export"]]
        if not ids or ids != row["case_ids"] or len(set(ids)) != len(ids):
            failures.append(f"reviewed case identity mismatch: {row['export']}")
    return failures


def reviewed_ternary_failures() -> list[str]:
    """Validate the manually reviewed pure case matrix and its pinned artifacts."""
    review = json.loads(Path(__file__).with_name("oracle_ternary_behavior_review.json").read_text())
    failures = []
    for path, digest in review["artifact_sha256"].items():
        if hashlib.sha256(relative_file(path).read_bytes()).hexdigest() != digest:
            failures.append(f"reviewed ternary artifact changed; re-review required: {path}")
    capture = json.loads(relative_file(review["capture"]).read_text())
    rows, cases = review["rows"], capture["cases"]
    names = [row["export"] for row in rows]
    if len(names) != len(set(names)) or len(names) != review["reviewed_export_count"]:
        failures.append("duplicate or incomplete reviewed ternary exports")
    if set(names) != set(capture["exports"]):
        failures.append("reviewed ternary exports differ from independent capture")
    ids = [case["id"] for case in cases]
    if len(ids) != len(set(ids)) or len(ids) != review["case_count"]:
        failures.append("duplicate or incomplete reviewed ternary cases")
    successes = sum(case["ok"] for case in cases)
    if successes != review["success_count"] or len(cases) - successes != review["error_count"]:
        failures.append("reviewed ternary outcome counts differ")
    for row in rows:
        relative_file(row["rust_test"])
        expected = [case["id"] for case in cases if case["export"] == row["export"]]
        if not expected or expected != row["case_ids"]:
            failures.append(f"reviewed ternary case identity mismatch: {row['export']}")
    return failures


def reviewed_call_registry_failures(bindings: dict) -> list[str]:
    review = json.loads(Path(__file__).with_name("oracle_call_registry_behavior_review.json").read_text())
    failures = []
    for path, digest in review["artifact_sha256"].items():
        if not reviewed_test_matches(path, digest, bindings):
            failures.append(f"reviewed call/registry artifact changed: {path}")
    capture = json.loads(relative_file(review["capture"]).read_text())
    cases = capture["cases"]
    ids = [case["id"] for case in cases]
    if len(ids) != len(set(ids)) or len(ids) != review["pure_case_count"]:
        failures.append("duplicate or incomplete call/registry cases")
    if sum(c["ok"] for c in cases) != review["pure_success_count"] or sum(not c["ok"] for c in cases) != review["pure_error_count"]:
        failures.append("call/registry outcomes differ")
    identities = {(row["group"], row["export"]) for row in review["rows"]}
    if len(identities) != len(review["rows"]) or len(identities) != review["reviewed_export_count"]:
        failures.append("duplicate or incomplete call/registry export identities")
    for row in review["rows"]:
        relative_file(row["rust_test"])
        expected = ([case["id"] for case in cases if (case["group"], case["export"]) == (row["group"], row["export"])]
                    if row["kind"] == "pure" else [capture["close"][kind]["id"] for kind in ["success", "repeat"]])
        if not expected or expected != row["case_ids"]:
            failures.append(f"call/registry case identity mismatch: {row['export']}")
    return failures


def reviewed_inventory_failures(fixtures: list[dict], bindings: dict) -> list[str]:
    """The full source review records limits, not a claim of full behavior coverage."""
    review = json.loads(Path(__file__).with_name("oracle_behavior_review.json").read_text())
    failures = []
    rows = review["rows"]
    if len(rows) != review["candidate_row_count"] or len({(r["source"], r["export"]) for r in rows}) != len(rows):
        failures.append("reviewed inventory has duplicate/missing identities")
    if {r["source"] for r in rows} != {f["source"] for f in fixtures}:
        failures.append("reviewed source inventory differs from pinned manifest")
    if review["behavioral_coverage_complete"] or review["proof_ledger_audit_complete"]:
        failures.append("reviewed inventory exceeds its explicit acceptance scope")
    for path, digest in review["reviewed_file_sha256"].items():
        if not reviewed_test_matches(path, digest, bindings):
            failures.append(f"behavior review evidence changed; re-review required: {path}")
    for row in rows:
        if not row["limits"] or not row["dimensions"] or not row["source_review"]:
            failures.append(f"missing reviewed limits/dimensions: {row['source']}::{row['export']}")
        for path in row["rust_assertion_files"]:
            relative_file(path)
        if "case_matrix" in row:
            relative_file(row["case_matrix"])
    return failures


def main() -> int:
    manifest = json.loads(MANIFEST.read_text())
    fixtures = manifest["fixtures"]
    failures = []
    try:
        bindings = ledger9_bindings()
    except (KeyError, ValueError, OSError) as exc:
        print(f"FAIL Ledger 9 oracle binding: {exc}", file=sys.stderr)
        return 1
    if len(fixtures) != 37:
        failures.append(f"expected 37 oracle entries, found {len(fixtures)}")
    names = [entry["oracle_source"] for entry in fixtures]
    if len(set(names)) != len(names):
        failures.append("duplicate oracle source")

    for entry in fixtures:
        name = entry["oracle_source"]
        try:
            source = relative_file(entry["source"])
            contents = source.read_bytes()
            if hashlib.sha256(contents).hexdigest() != entry["source_sha256"]:
                failures.append(f"{name}: local source bytes changed")
            if hashlib.sha256(non_comment_lines(contents)).hexdigest() != entry["non_comment_sha256"]:
                failures.append(f"{name}: executable source differs from the pinned oracle")

            tests = entry["rust_tests"]
            references = entry["typescript_fixtures"]
            if not tests or not references:
                failures.append(f"{name}: missing Rust test or TypeScript fixture link")
            test_contents = [relative_file(path).read_text() for path in tests]
            for path in references:
                relative_file(path)
                active = bindings["captures"].get(path, {}).get("current", path)
                if not any(Path(active).name in text for text in test_contents):
                    failures.append(f"{name}: test does not reference {path}")
        except (KeyError, ValueError, UnicodeDecodeError) as exc:
            failures.append(f"{name}: {exc}")

    try:
        failures.extend(reviewed_behavior_failures(bindings))
        failures.extend(reviewed_ternary_failures())
        failures.extend(reviewed_call_registry_failures(bindings))
        failures.extend(reviewed_inventory_failures(fixtures, bindings))
    except (KeyError, ValueError, OSError) as exc:
        failures.append(f"reviewed behavior matrix: {exc}")

    for failure in failures:
        print(f"FAIL {failure}", file=sys.stderr)
    print(f"Checked {len(fixtures)} pinned oracle sources; {len(failures)} failed")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
