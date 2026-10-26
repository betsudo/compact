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

"""Exercise the fixture updater entrypoint without a compiler or Cargo process."""
import contextlib
import io
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

import check_fixture_outputs as checker


class FunctionalFixtureUpdateTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / "examples/demo.compact"
        self.source.parent.mkdir()
        self.source.write_text("source placeholder")
        self.fixture = self.root / "fixtures/demo/lib.rs"
        self.programs = {"program-0.json": b'{"complete":true}\n'}

    def invoke(self, update=False):
        row = SimpleNamespace(source=self.source, frontend_flags=("--rust-functional-ledger8",))
        def execute(argv, **kwargs):
            if argv[0] == "fixture-test-compiler":
                contract = Path(argv[-1]) / "contract"
                contract.mkdir()
                (contract / "lib.rs").write_bytes(b"fresh library")
                (contract / "rust-capabilities.json").write_text('{"schema_version":3,"circuits":[]}')
                (contract / "checked-programs").mkdir()
                for name, contents in self.programs.items():
                    (contract / "checked-programs" / name).write_bytes(contents)
            elif argv[0] != "rustfmt":
                raise AssertionError(f"unexpected external command: {argv}")
            return subprocess.CompletedProcess(argv, 0, "", "")
        with mock.patch.object(checker, "ROOT", self.root), \
             mock.patch.object(checker.fixture_inventory, "source_registry", return_value=[row]), \
             mock.patch.object(checker.fixture_inventory, "emission_fixture", return_value=self.fixture), \
             mock.patch.object(checker, "run", side_effect=execute), \
             mock.patch.dict(os.environ, {"COMPACTC": "fixture-test-compiler"}), \
             mock.patch.object(sys, "argv", ["check_fixture_outputs.py"] + (["--update"] if update else [])), \
             contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return checker.main()

    def retain_old(self):
        directory = self.fixture.parent / "checked-programs"
        directory.mkdir(parents=True)
        self.fixture.write_bytes(b"old library")
        (directory / "old.json").write_bytes(b"old program")
        (directory / "README").write_bytes(b"unrelated retained note")
        return directory

    def test_check_refuses_stale_programs_without_changing_retained_files(self):
        directory = self.retain_old()
        self.assertEqual(self.invoke(), 1)
        self.assertEqual(self.fixture.read_bytes(), b"old library")
        self.assertEqual((directory / "old.json").read_bytes(), b"old program")
        self.assertFalse((directory / "program-0.json").exists())

    def test_update_replaces_complete_program_roster_and_keeps_unrelated_file(self):
        directory = self.retain_old()
        self.assertEqual(self.invoke(update=True), 0)
        self.assertEqual(self.fixture.read_bytes(), b"fresh library")
        self.assertEqual({p.name: p.read_bytes() for p in directory.glob("*.json")}, self.programs)
        self.assertEqual((directory / "README").read_bytes(), b"unrelated retained note")
        self.assertEqual(self.invoke(), 0)

    def test_update_creates_first_registration_with_program_assets(self):
        self.assertEqual(self.invoke(update=True), 0)
        self.assertEqual((self.fixture.parent / "checked-programs/program-0.json").read_bytes(),
                         self.programs["program-0.json"])
        self.assertEqual(self.invoke(), 0)

    def test_update_refuses_missing_fresh_programs_without_mutation(self):
        directory = self.retain_old()
        self.programs = {}
        self.assertEqual(self.invoke(update=True), 1)
        self.assertEqual(self.fixture.read_bytes(), b"old library")
        self.assertEqual((directory / "old.json").read_bytes(), b"old program")


if __name__ == "__main__":
    unittest.main()
