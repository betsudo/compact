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

"""Fixture binding controls only; no model or cryptographic proof authority."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = Path(__file__).with_name("tests") / "constructor_equality_boundaries"


class FixtureBindingControls(unittest.TestCase):
    def test_full_sources_bind_all_eight_roles_and_three_original_exports(self):
        roster = json.loads((FIXTURE / "sources.json").read_text())
        actual = {p.stem: p for p in (FIXTURE / "sources").glob("*.compact")}
        self.assertEqual(len(roster), 8)
        self.assertEqual(len({r["role"] for r in roster}), 8)
        self.assertEqual({r["role"] for r in roster}, set(actual))
        for spec in roster:
            source = ROOT / spec["path"]
            self.assertEqual(source, actual[spec["role"]])
            self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), spec["sha256"])
            self.assertEqual(spec["entries"], ["read", "advance", "advance_other"])
            self.assertEqual(re.findall(r"export circuit (\w+)\(", source.read_text()), spec["entries"])
            self.assertEqual(spec["constructor_consumer"], spec["role"])
            self.assertTrue((FIXTURE / "constructors" / (spec["role"] + ".rs")).is_file())

    def test_all_thirty_proof_cases_preserve_exact_api_data_and_expected_state(self):
        api = json.loads((FIXTURE / "cases.json").read_text())
        proofs = json.loads((FIXTURE / "proof-cases.json").read_text())
        self.assertEqual(len(api), 30)
        self.assertEqual(len(proofs), len(api))
        self.assertEqual(len({c["id"] for c in proofs}), 30)
        metadata = {"id", "privateState", "trace", "error"}
        self.assertEqual([{k: v for k, v in c.items() if k not in metadata} for c in proofs], api)
        for case in proofs:
            self.assertEqual(case["id"], case["role"] + "_" + case["case"])
            self.assertEqual(case["privateState"], 42)
            self.assertEqual(case["trace"], [])
            self.assertIsNone(case["error"])


class ModelEntryAdmissionControls(unittest.TestCase):
    def test_historical_model_cli_keeps_its_original_arguments(self):
        script = Path(__file__).with_name("check_constructor_equality_models.py")
        result = subprocess.run([sys.executable, "-B", script, "--help"],
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0)
        self.assertIn("--producer-root", result.stdout)
        self.assertIn("--producer-receipt", result.stdout)
        self.assertNotIn("--scope-receipt", result.stdout)

    def test_boundary_model_entry_requires_scope_before_any_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "out"
            script = Path(__file__).with_name("check_constructor_equality_boundaries_models.py")
            result = subprocess.run([sys.executable, "-B", script, "--producer-root", root / "missing",
                "--producer-receipt", root / "missing.json", "--output", output],
                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, timeout=30)
            self.assertEqual(result.returncode, 2)
            self.assertIn("--scope-receipt", result.stderr)
            self.assertFalse(output.exists())

    def test_optimized_model_entry_refuses_before_argument_or_source_effects(self):
        script = Path(__file__).with_name("check_constructor_equality_boundaries_models.py")
        result = subprocess.run([sys.executable, "-O", "-B", script, "--help"],
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, timeout=30)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("optimized Python invocation is not admitted", result.stderr)

    def test_new_proof_entries_refuse_optimization_before_help(self):
        for name, message in (("check_constructor_equality_boundaries_proof.py", "optimized Python invocation is not admitted"),
                              ("check_constructor_equality_boundaries_proof_preflight.py", "optimized preflight is not admitted")):
            with self.subTest(script=name):
                script = Path(__file__).with_name(name)
                result = subprocess.run([sys.executable, "-O", "-B", script, "--help"],
                    env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, timeout=30)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(message, result.stderr)


if __name__ == "__main__":
    unittest.main()
