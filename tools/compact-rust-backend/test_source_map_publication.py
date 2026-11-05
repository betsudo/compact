#!/usr/bin/env python3
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

"""Real child-byte retention and Cargo executable-selection controls."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_passport_current_profile as process_owner
import check_source_map_publication as gate


class PublicationControlTests(unittest.TestCase):
    def test_capture_keeps_crlf_cr_and_undecodable_bytes_and_actual_cwd(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            out, err = root / "stdout", root / "stderr"
            command = [sys.executable, "-c", "import os;assert os.getcwd()==os.environ['EXPECTED_CWD'];os.write(1,b'line\\r\\ncarriage\\r\\xff');os.write(2,b'error\\r\\n\\xfe')"]
            result = gate.capture_command(command, dict(os.environ, EXPECTED_CWD=str(root.resolve())), root, out, err)
            self.assertEqual(result.returncode, 0)
            self.assertEqual(out.read_bytes(), b"line\r\ncarriage\r\xff")
            self.assertEqual(err.read_bytes(), b"error\r\n\xfe")
            self.assertEqual(result.stdout, out.read_bytes())
            self.assertEqual(result.stderr, err.read_bytes())

    def test_timeout_preserves_raw_bytes_without_overwriting_with_decoded_pipes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            out, err = root / "stdout", root / "stderr"
            command = [sys.executable, "-c", "import os,time;os.write(1,b'retained\\r\\n\\xff');os.write(2,b'failure\\r');time.sleep(30)"]
            with patch.object(process_owner, "COMMAND_TIMEOUT_SECONDS", 1), self.assertRaises(subprocess.TimeoutExpired):
                gate.capture_command(command, dict(os.environ), root, out, err)
            self.assertEqual(out.read_bytes(), b"retained\r\n\xff")
            self.assertEqual(err.read_bytes(), b"failure\r")

    def test_cargo_report_selects_configured_target_instead_of_stale_default(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / "Cargo.toml"
            actual = root / "target/host-triple/debug/compact-rustc"
            stale = root / "target/debug/compact-rustc"
            for path, value in ((actual, b"current"), (stale, b"stale")):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(value)
            message = {"reason": "compiler-artifact", "manifest_path": str(manifest), "target": {"name": "compact-rustc", "kind": ["bin"]}, "executable": str(actual)}
            output = (json.dumps({"reason": "build-finished", "success": True}) + "\n" + json.dumps(message) + "\n").encode()
            self.assertEqual(gate.cargo_executable(output, manifest), actual.resolve())
            self.assertEqual(stale.read_bytes(), b"stale")
            for changed in ({**message, "manifest_path": str(root / "other/Cargo.toml")}, {**message, "target": {"name": "compact-rustc", "kind": ["lib"]}}, {**message, "executable": None}):
                with self.subTest(message=changed), self.assertRaisesRegex(ValueError, "exactly one"):
                    gate.cargo_executable(json.dumps(changed).encode(), manifest)
            for invalid in (b"", (json.dumps(message) + "\n" + json.dumps(message)).encode()):
                with self.subTest(output=invalid), self.assertRaisesRegex(ValueError, "exactly one"):
                    gate.cargo_executable(invalid, manifest)


if __name__ == "__main__":
    unittest.main()
