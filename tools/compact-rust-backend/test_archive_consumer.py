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

"""Retained archive acceptance evidence and failure controls; no Cargo run."""

import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location(
    "archive_consumer", Path(__file__).with_name("check_archive_consumer.py")
)
archive = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(archive)


class RetainedArchiveTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.output = self.root / "retained"
        self.source = self.root / "source"
        self.source.write_text("original source")
        self.inputs = {"sha256": {str(self.source): archive.sha256(self.source)}}

    def receipt(self):
        return json.loads((self.output / "receipt.json").read_text())

    def test_success_retains_raw_artifacts_and_command_streams(self):
        with archive.qualification_directory(self.output, self.inputs) as work:
            (work / "raw.rs").write_bytes(b"// untouched raw bytes\n")
            result = archive.execute([sys.executable, "-c", "print('actual command')"], cwd=work)
            self.assertEqual(result.returncode, 0)
        result = self.receipt()
        self.assertEqual(result["status"], "passed")
        self.assertEqual((work / "raw.rs").read_bytes(), b"// untouched raw bytes\n")
        self.assertIn("actual command", (self.output / "commands/00.stdout").read_text())
        for path, digest in result["artifacts"].items():
            self.assertEqual(archive.sha256(self.output / path), digest)
        self.assertIsNone(archive.RETAINED)

    def test_child_failure_preserves_partial_outputs_and_original_error(self):
        with self.assertRaisesRegex(RuntimeError, "failed with 17"):
            with archive.qualification_directory(self.output, self.inputs) as work:
                (work / "partial.rs").write_text("partial output")
                archive.run([sys.executable, "-c", "import sys; print('diagnostic', file=sys.stderr); sys.exit(17)"], cwd=work)
        result = self.receipt()
        self.assertEqual(result["status"], "failed")
        self.assertEqual(result["commands"][0]["exit"], 17)
        self.assertEqual((work / "partial.rs").read_text(), "partial output")
        self.assertIn("diagnostic", (self.output / "commands/00.stderr").read_text())
        self.assertIsNone(archive.RETAINED)

    def test_existing_output_refuses_without_changing_previous_evidence(self):
        self.output.mkdir()
        marker = self.output / "receipt.json"
        marker.write_text("previous evidence")
        with self.assertRaises(FileExistsError):
            with archive.qualification_directory(self.output, self.inputs):
                self.fail("must refuse before execution")
        self.assertEqual(marker.read_text(), "previous evidence")
        self.assertEqual(list(self.output.iterdir()), [marker])

    def test_changed_input_refuses_success_and_retains_failure(self):
        with self.assertRaisesRegex(RuntimeError, "qualification input changed"):
            with archive.qualification_directory(self.output, self.inputs):
                self.source.write_text("changed source")
        self.assertEqual(self.receipt()["status"], "failed")
        self.assertEqual(self.receipt()["inputs"], self.inputs)


if __name__ == "__main__":
    unittest.main()
