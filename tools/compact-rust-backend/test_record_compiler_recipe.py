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

"""Recorder boundary controls using real children and a private Git/tool fixture.

No fixture receipt is CI authentication and no host Nix command/store is used.
"""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import record_compiler_recipe as owner


class ExistingRecipeControls(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.base = Path(self.directory.name).resolve()
        self.root = self.base / "source"
        self.root.mkdir()
        for name in ["flake.nix", "flake.lock", "Cargo.lock", "tools/compact-rust-backend/Cargo.lock",
                     "compiler/compiler-version.ss"]:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("private fixture input\n")
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "-c", "commit.gpgsign=false", "commit", "-qm", "private fixture"], cwd=self.root, check=True)
        self.package = self.base / "package"
        (self.package / "bin").mkdir(parents=True)
        self.compiler = self.package / "bin/compactc"
        self.compiler.write_text("#!/usr/bin/env python3\nprint('0.31.fixture')\n")
        self.compiler.chmod(0o755)
        scheme = self.package / "bin/compactc-scheme"
        scheme.write_text("#!/bin/sh\nexit 0\n")
        scheme.chmod(0o755)
        self.tools = self.base / "tools"
        self.tools.mkdir()
        self.argv = self.base / "actual-argv.json"
        self.output = self.base / "receipt"
        self.action()
        self.environment = patch.dict(os.environ, PATH=str(self.tools) + os.pathsep + os.environ['PATH'])
        self.environment.start()
        self.addCleanup(self.environment.stop)

    def action(self, script=""):
        nix = self.tools / "nix"
        nix.write_text("#!/usr/bin/env python3\nimport json,sys\nfrom pathlib import Path\n"
                       + f"Path({str(self.argv)!r}).write_text(json.dumps(sys.argv[1:]))\n"
                       + script + f"\nprint({str(self.package)!r})\n")
        nix.chmod(0o755)

    def execute(self):
        return owner.record(self.compiler, self.output, root=self.root)

    def failed(self):
        value = json.loads((self.output / "receipt.json").read_text())
        self.assertEqual(value['status'], 'failed')
        self.assertNotIn('compactc', value)
        return value

    def test_actual_children_require_existing_only_recipe_and_matching_package(self):
        value = self.execute()
        self.assertEqual(value['status'], 'passed')
        self.assertEqual(value['version'], '0.31.fixture')
        self.assertEqual(value['store'], str(self.package))
        self.assertEqual(json.loads(self.argv.read_text()), ["build", "--offline", "--no-link", "--print-out-paths",
            "--max-jobs", "0", "--builders", "", "--option", "substitute", "false", ".#compactc"])
        self.assertEqual([row['exit'] for row in value['commands']], [0, 0])
        self.assertTrue(all(owner.digest(p) == sha for p, sha in value['inputs_before_use'].items()))
        for row in value['commands']:
            self.assertEqual(owner.digest(row['stdout']), row['stdout_sha256'])
            self.assertEqual(owner.digest(row['stderr']), row['stderr_sha256'])

    def test_failed_realization_keeps_actual_failure_and_raw_diagnostics(self):
        self.action("print('actual-realization-failure', file=sys.stderr)\nsys.exit(7)")
        with self.assertRaisesRegex(ValueError, "existing-recipe failed"):
            self.execute()
        value = self.failed()
        self.assertEqual(value['commands'][0]['exit'], 7)
        self.assertIn('actual-realization-failure', Path(value['commands'][0]['stderr']).read_text())
        self.assertEqual(len(value['commands']), 1)

    def test_wrong_or_multiple_output_packages_refuse_before_version_execution(self):
        self.action("print('/different/package')")
        with self.assertRaisesRegex(ValueError, "output differs"):
            self.execute()
        self.failed()
        self.assertFalse((self.output / 'compiler-version.stdout').exists())

    def test_changed_source_refuses_before_version_execution(self):
        self.action(f"Path({str(self.root / 'flake.nix')!r}).write_text('changed source')")
        with self.assertRaisesRegex(ValueError, "clean source required"):
            self.execute()
        self.failed()
        self.assertFalse((self.output / 'compiler-version.stdout').exists())

    def test_changed_compiler_refuses_before_using_new_tool_bytes(self):
        self.action(f"with Path({str(self.compiler)!r}).open('a') as stream: stream.write('# changed tool\\n')")
        with self.assertRaisesRegex(ValueError, "tool bytes changed"):
            self.execute()
        self.failed()
        self.assertFalse((self.output / 'compiler-version.stdout').exists())

    def test_failed_version_command_cannot_publish_passed_recipe(self):
        self.compiler.write_text("#!/bin/sh\nexit 9\n")
        with self.assertRaisesRegex(ValueError, "compiler-version failed"):
            self.execute()
        value = self.failed()
        self.assertEqual([row['exit'] for row in value['commands']], [0, 9])

    def test_existing_output_is_never_overwritten_or_reused(self):
        self.execute()
        sha = owner.digest(self.output / 'receipt.json')
        with self.assertRaisesRegex(ValueError, "must be absent"):
            self.execute()
        self.assertEqual(owner.digest(self.output / 'receipt.json'), sha)

    def test_missing_scheme_sibling_never_runs_realization(self):
        (self.package / 'bin/compactc-scheme').unlink()
        with self.assertRaises(FileNotFoundError):
            self.execute()
        self.failed()
        self.assertFalse(self.argv.exists())


if __name__ == '__main__':
    unittest.main()
