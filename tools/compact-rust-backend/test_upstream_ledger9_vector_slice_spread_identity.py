#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Admission refusals and receipt wiring for the slice and spread gates."""

import ast
import importlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock


CHECKERS = ("check_upstream_ledger9_vector_slice",
            "check_upstream_ledger9_vector_spread")


class SliceSpreadIdentityControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        base = Path(self.temporary.name)
        package = base / "compiler-package"
        (package / "bin").mkdir(parents=True)
        self.compiler = package / "bin/compactc"
        self.compiler.write_bytes(b"compiler")
        (package / "bin/compactc-scheme").write_bytes(b"scheme")
        self.runtime = base / "runtime"
        self.runtime.mkdir()
        source_package = Path(__file__).resolve().parents[2] / "runtime/package.json"
        (self.runtime / "package.json").write_bytes(source_package.read_bytes())
        self.recipe = base / "recipe.json"
        self.recipe.write_text(json.dumps({"status": "stale"}))
        self.output = base / "output"

    def invoke_admission(self, module, *, source_error=None, signed=True,
                         runtime_error=None, recipe_error=None):
        argv = [module.__file__, "--compiler", str(self.compiler),
                "--compiler-recipe", str(self.recipe),
                "--ts-runtime", str(self.runtime), "--output", str(self.output)]
        signature = "G\nSigned-off-by: Test <test@example.com>\nAssisted-by: AI\n"
        if not signed:
            signature = "N\nSigned-off-by: Test <test@example.com>\nAssisted-by: AI\n"
        source = ({"head": "expected-head", "tree": "expected-tree"}
                  if source_error is None else ValueError(source_error))
        runtime = ({"path": str(self.runtime)} if runtime_error is None
                   else ValueError(runtime_error))
        recipe = ({"head": "expected-head"} if recipe_error is None
                  else ValueError(recipe_error))
        with (mock.patch.object(sys, "argv", argv),
              mock.patch.object(module, "source_state", side_effect=[source]),
              mock.patch.object(module.subprocess, "check_output", return_value=signature),
              mock.patch.object(module.shutil, "which", return_value=sys.executable),
              mock.patch.object(module, "runtime_recipe_identity", side_effect=[runtime]),
              mock.patch.object(module, "runtime_identity", return_value={"dist_files": {}}),
              mock.patch.object(module, "compiler_recipe_identity", side_effect=[recipe])):
            module.main()

    def test_dirty_or_unsigned_source_refused_before_output(self):
        for name in CHECKERS:
            module = importlib.import_module(name)
            with self.subTest(name=name, reason="dirty"):
                with self.assertRaisesRegex(ValueError, "clean committed source"):
                    self.invoke_admission(module, source_error="clean committed source required")
                self.assertFalse(self.output.exists())
            with self.subTest(name=name, reason="unsigned"):
                with self.assertRaises(SystemExit):
                    self.invoke_admission(module, signed=False)
                self.assertFalse(self.output.exists())

    def test_wrong_runtime_and_stale_compiler_recipe_refused_before_output(self):
        for name in CHECKERS:
            module = importlib.import_module(name)
            with self.subTest(name=name, reason="runtime"):
                with self.assertRaisesRegex(ValueError, "wrong exact-head runtime"):
                    self.invoke_admission(module, runtime_error="wrong exact-head runtime")
                self.assertFalse(self.output.exists())
            with self.subTest(name=name, reason="compiler"):
                with self.assertRaisesRegex(ValueError, "stale compiler recipe"):
                    self.invoke_admission(module, recipe_error="stale compiler recipe")
                self.assertFalse(self.output.exists())

    def test_final_identity_check_precedes_receipt_publication(self):
        for name in CHECKERS:
            module = importlib.import_module(name)
            main = next(node for node in ast.parse(Path(module.__file__).read_text()).body
                        if isinstance(node, ast.FunctionDef) and node.name == "main")
            calls = [(node.lineno, node.func.id) for node in ast.walk(main)
                     if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)]
            def line(function):
                return next(number for number, called in calls if called == function)
            receipt = next(node for node in ast.walk(main)
                           if isinstance(node, ast.Assign) and
                           any(isinstance(target, ast.Name) and target.id == "receipt"
                               for target in node.targets))
            self.assertLess(line("source_state"), line("runtime_recipe_identity"))
            self.assertLess(line("runtime_recipe_identity"), line("compiler_recipe_identity"))
            self.assertLess(line("compiler_recipe_identity"), line("require_unchanged"))
            self.assertLess(line("require_unchanged"), receipt.lineno)
            self.assertIn("source_clean", ast.unparse(receipt.value))
            for key in ("runtime_recipe", "compiler_recipe", "checker_sha256",
                        "identity_library_sha256"):
                self.assertIn(key, ast.unparse(receipt.value))


if __name__ == "__main__":
    unittest.main()
