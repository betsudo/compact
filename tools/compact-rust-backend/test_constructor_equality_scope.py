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

"""HOST early-refusal controls for the scope gate, not generation authority."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_constructor_equality_scope as owner


class ScopeAdmissionControls(unittest.TestCase):
    def test_dirty_or_stale_recipe_refuses_before_outputs_and_compiler(self):
        for dirty in (False, True):
            with self.subTest(dirty=dirty), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary).resolve()
                source = root / "source"
                source.mkdir()
                recipe = root / "recipe.json"
                recipe.write_text(json.dumps(dict(status="passed", head="stale", tree="tree")))
                output = root / "out"
                argv = ["gate", "--compiler-receipt", str(recipe), "--output", str(output)]
                def git(command, **kwargs):
                    return {("rev-parse", "HEAD"): "head", ("rev-parse", "HEAD^{tree}"): "tree",
                            ("status", "--porcelain"): "dirty" if dirty else ""}[tuple(command[1:])]
                with patch.object(owner, "ROOT", source), patch.object(sys, "argv", argv), patch.object(owner.subprocess, "check_output", side_effect=git), patch.object(owner, "bounded_command") as execute:
                    with self.assertRaisesRegex(ValueError, "clean source|current clean original"):
                        owner.main()
                execute.assert_not_called()
                self.assertFalse(output.exists())

    def test_existing_and_inside_source_outputs_refuse_before_git_or_compiler(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            source = root / "source"
            source.mkdir()
            existing = root / "existing"
            existing.mkdir()
            for output in (existing, source / "out"):
                with self.subTest(output=output), patch.object(owner, "ROOT", source), patch.object(sys, "argv", ["gate", "--compiler-receipt", str(root / "missing"), "--output", str(output)]), patch.object(owner.subprocess, "check_output") as git, patch.object(owner, "bounded_command") as execute:
                    with self.assertRaisesRegex(ValueError, "fresh output outside source"):
                        owner.main()
                git.assert_not_called()
                execute.assert_not_called()

    def test_optimized_invocation_refuses_before_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            output = root / "out"
            p = subprocess.run([sys.executable, "-O", "-B", owner.__file__, "--compiler-receipt", str(root / "missing"), "--output", str(output)], env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, timeout=30)
            self.assertNotEqual(p.returncode, 0)
            self.assertIn("optimized Python is not admitted", p.stderr)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
