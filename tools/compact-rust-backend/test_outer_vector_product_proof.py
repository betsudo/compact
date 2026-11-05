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

"""Failure-path controls for raw proof-gate command evidence, not proof verdicts."""
import hashlib
import copy
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_outer_vector_product_proof as gate


class CommandEvidence(unittest.TestCase):
    def test_raw_carriage_returns_and_separate_streams_survive(self):
        with tempfile.TemporaryDirectory() as folder:
            report = {"commands": []}
            command = [sys.executable, "-c", "import os;os.write(1,b'a\\r\\nb\\r');os.write(2,b'c\\r\\nd\\r')"]
            result, row = gate.captured(command, Path(folder), "raw", report, dict(os.environ))
            self.assertEqual(result.stdout, "a\r\nb\r")
            self.assertEqual(result.stderr, "c\r\nd\r")
            for stream, expected in [("stdout", b"a\r\nb\r"), ("stderr", b"c\r\nd\r")]:
                self.assertEqual(Path(row[stream]["path"]).read_bytes(), expected)
                self.assertEqual(row[stream]["sha256"], hashlib.sha256(expected).hexdigest())

    def test_undecodable_bytes_fail_but_keep_original_evidence(self):
        with tempfile.TemporaryDirectory() as folder:
            report = {"commands": []}
            with self.assertRaises(UnicodeDecodeError):
                gate.captured([sys.executable, "-c", "import os;os.write(1,b'\\xff\\x00\\r')"],
                              Path(folder), "undecodable", report, dict(os.environ))
            rows = report["captured_commands"]
            self.assertEqual(len(rows), 1)
            self.assertEqual(Path(rows[0]["stdout"]["path"]).read_bytes(), b"\xff\x00\r")
            self.assertEqual(rows[0]["exit_code"], 0)
            self.assertNotIn("status", rows[0])

    def test_timeout_keeps_partial_streams_and_explicit_timeout(self):
        def fail(command, environment, cwd, stdout, stderr):
            stdout.write_bytes(b"partial\r\n")
            stderr.write_bytes(b"diagnostic\x00")
            raise subprocess.TimeoutExpired(command, 1)
        with tempfile.TemporaryDirectory() as folder:
            report = {"commands": []}
            with patch.object(gate.main_gate, "capture_command", fail), self.assertRaises(subprocess.TimeoutExpired):
                gate.captured(["controlled-timeout"], Path(folder), "timeout", report, {})
            row = report["captured_commands"][0]
            self.assertTrue(row["timeout"])
            self.assertNotIn("exit_code", row)
            self.assertEqual(Path(row["stdout"]["path"]).read_bytes(), b"partial\r\n")
            self.assertEqual(Path(row["stderr"]["path"]).read_bytes(), b"diagnostic\x00")
            self.assertEqual(Path(row["log"]).read_bytes(), b"partial\r\ndiagnostic\x00")


class PreKeyRefusalMembership(unittest.TestCase):
    def report(self):
        # Synthetic validator input only: no subprocess, key or proof verdict.
        expected = (["family-changed-ir"]
                    + [f"family-changed-program-{index}" for index in range(15)]
                    + ["family-missing-program-0", "family-missing-program-14",
                       "original-changed-ir", "original-changed-program-0",
                       "original-missing-program-0", "selected-model-over-cap"])
        return {"raw_binding_refusals": [{"label": label} for label in expected[:-1]],
                "refusal_invocations": [{"label": label, "runner_sha256": "controlled-runner",
                                         "started_monotonic": 1, "ended_monotonic": 2}
                                        for label in expected]}

    def test_complete_actual_cohort_requires_twenty_two_refusals(self):
        report = self.report()
        self.assertEqual(len(report["refusal_invocations"]), 22)
        self.assertEqual(gate.validate_pre_key_refusals(report, "controlled-runner"),
                         [row["label"] for row in report["refusal_invocations"]])

    def test_missing_duplicate_and_reordered_controls_refuse(self):
        for change in ("missing", "duplicate", "reordered", "raw-member"):
            with self.subTest(change=change):
                report = copy.deepcopy(self.report())
                rows = report["refusal_invocations"]
                if change == "missing": rows.pop(15)
                elif change == "duplicate": rows[15] = rows[14].copy()
                elif change == "reordered": rows[14], rows[15] = rows[15], rows[14]
                else: report["raw_binding_refusals"].pop()
                with self.assertRaisesRegex(ValueError, "refusal.*differs"):
                    gate.validate_pre_key_refusals(report, "controlled-runner")

    def test_different_runner_or_reversed_time_refuses(self):
        for change in ("runner", "time"):
            with self.subTest(change=change):
                report = self.report()
                if change == "runner": report["refusal_invocations"][0]["runner_sha256"] = "other"
                else: report["refusal_invocations"][0]["ended_monotonic"] = 0
                with self.assertRaisesRegex(ValueError, "runner or timing differs"):
                    gate.validate_pre_key_refusals(report, "controlled-runner")


if __name__ == "__main__":
    unittest.main()
