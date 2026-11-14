#!/usr/bin/env python3
# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Mutation controls for the Vector casts exact-head qualification receipt."""

import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


CHECKER = Path(__file__).with_name("check_upstream_ledger9_vector_casts.py")
SPEC = importlib.util.spec_from_file_location("vector_casts_checker", CHECKER)
assert SPEC and SPEC.loader
checker = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(checker)


class IdentityControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        base = Path(self.temporary.name)
        self.root = base / "repo"
        self.root.mkdir()
        self.package = base / "package"
        bin_dir = self.package / "bin"
        bin_dir.mkdir(parents=True)
        self.compiler = bin_dir / "compactc"
        self.scheme = bin_dir / "compactc-scheme"
        self.compiler.write_bytes(b"compiler")
        self.scheme.write_bytes(b"scheme")
        self.runtime = base / "runtime"
        (self.runtime / "dist").mkdir(parents=True)
        (self.runtime / "package.json").write_text('{"version":"0.18.0"}\n')
        (self.runtime / "dist/index.js").write_text("export const value = 1;\n")
        (self.runtime / "dist/helper.js").write_text("export const helper = 1;\n")
        self.source_paths = ["flake.nix", "flake.lock", "Cargo.lock",
                             "tools/compact-rust-backend/Cargo.lock",
                             "compiler/compiler-version.ss",
                             "tools/compact-rust-backend/record_compiler_recipe.py"]
        for name in self.source_paths:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(name + "\n")
        self.git("init", "-q")
        self.git("config", "user.name", "Test")
        self.git("config", "user.email", "test@example.com")
        self.git("add", ".")
        self.git("-c", "commit.gpgsign=false", "commit", "-qm", "initial")
        self.source = checker.source_state(self.root)
        self.runtime_snapshot = checker.runtime_identity(self.runtime)
        self.recipe_path = base / "recipe.json"
        self.recipe = {
            "format": "compact-original-existing-compiler-recipe/v1",
            "status": "passed", **self.source,
            "store": str(self.package),
            "compactc": {"path": str(self.compiler), "sha256": checker.sha256(self.compiler)},
            "compactc-scheme": {"path": str(self.scheme), "sha256": checker.sha256(self.scheme)},
            "inputs_before_use": {str((self.root / name).resolve(strict=True)):
                                  checker.sha256(self.root / name)
                                  for name in self.source_paths},
        }
        self.write_recipe()
        self.recipe_snapshot = self.read_recipe_identity()

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True).strip()

    def write_recipe(self):
        self.recipe_path.write_text(json.dumps(self.recipe) + "\n")

    def read_recipe_identity(self):
        return checker.compiler_recipe_identity(self.recipe_path, self.source,
                                                self.compiler, self.scheme, self.root)

    def check_unchanged(self):
        checker.require_unchanged(self.source, self.runtime_snapshot,
                                  self.recipe_snapshot, self.runtime,
                                  self.recipe_path, self.compiler, self.scheme, self.root)

    def test_exact_inputs_pass(self):
        self.check_unchanged()

    def test_dirty_source_refused(self):
        (self.root / "flake.nix").write_text("changed\n")
        with self.assertRaisesRegex(ValueError, "clean committed source"):
            self.check_unchanged()

    def test_changed_head_refused(self):
        self.git("-c", "commit.gpgsign=false", "commit", "--allow-empty", "-qm", "next")
        with self.assertRaisesRegex(ValueError, "source head or tree changed"):
            self.check_unchanged()

    def test_runtime_dist_mutation_refused(self):
        (self.runtime / "dist/helper.js").write_text("export const helper = 2;\n")
        with self.assertRaisesRegex(ValueError, "runtime dist changed"):
            self.check_unchanged()

    def test_runtime_path_substitution_refused(self):
        other = self.runtime.parent / "other-runtime"
        (other / "dist").mkdir(parents=True)
        for name in ("package.json", "dist/index.js", "dist/helper.js"):
            path = other / name
            path.write_bytes((self.runtime / name).read_bytes())
        with self.assertRaisesRegex(ValueError, "runtime dist changed"):
            checker.require_unchanged(self.source, self.runtime_snapshot,
                                      self.recipe_snapshot, other, self.recipe_path,
                                      self.compiler, self.scheme, self.root)

    def test_recipe_head_substitution_refused(self):
        self.recipe["head"] = "0" * 40
        self.write_recipe()
        with self.assertRaisesRegex(ValueError, "recipe does not bind"):
            self.check_unchanged()

    def test_recipe_source_pin_substitution_refused(self):
        self.recipe["inputs_before_use"][str((self.root / "flake.nix").resolve(strict=True))] = "0" * 64
        self.write_recipe()
        with self.assertRaisesRegex(ValueError, "source pin differs"):
            self.check_unchanged()

    def test_compiler_binary_mutation_refused(self):
        self.compiler.write_bytes(b"other compiler")
        with self.assertRaisesRegex(ValueError, "compactc identity differs"):
            self.check_unchanged()


if __name__ == "__main__":
    unittest.main()
