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

"""Host-only release version controls using the unchanged pinned shell scripts."""

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path(__file__).with_name("ledger9-host-version-scripts.sha256.json")
READ = ROOT / "scripts/read-version.sh"
STAMP = ROOT / "scripts/stamp-compiler-version.sh"
COMMIT = "0123456789abcdef" * 2 + "01234567"
DATE = "2026-11-14"


class UpstreamVersionScripts(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name)
        self.source = self.base / "compiler-version.ss"
        self.source.write_text("(make-version 'compiler 0 31 213)\n")
        self.config = self.base / "version-config.ss"
        self.original = "(library (version-config) (export version))\n"
        self.config.write_text(self.original)

    def run_script(self, script: Path, *args: str):
        return subprocess.run([str(script), *map(str, args)], cwd=self.base,
                              capture_output=True, text=True, check=False)

    def stamp(self, tag: str, commit: str = COMMIT, date: str = DATE):
        return self.run_script(STAMP, tag, commit, date, self.config, self.source)

    def test_original_script_bytes_and_executable_modes(self):
        manifest = json.loads(MANIFEST.read_text())
        self.assertEqual(manifest["upstream_commit"],
                         "672bd0f226f47a7d036c5c074ce71fbe1af1925e")
        self.assertEqual(set(manifest["files"]),
                         {"scripts/read-version.sh", "scripts/stamp-compiler-version.sh"})
        for name, digest in manifest["files"].items():
            source = ROOT / name
            self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), digest)
            self.assertTrue(source.stat().st_mode & 0o111)
            upstream = subprocess.run(["git", "show", f'{manifest["upstream_commit"]}:{name}'],
                                      cwd=ROOT, capture_output=True, check=False)
            if upstream.returncode == 0:  # A shallow CI checkout may omit this object.
                self.assertEqual(source.read_bytes(), upstream.stdout)

    def test_read_committed_compiler_and_language_versions(self):
        for feature, expected in (("compiler", "0.31.213"), ("language", "0.23.105")):
            result = subprocess.run([str(READ), feature], cwd=ROOT,
                                    capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout, expected + "\n")

    def test_read_custom_source_and_refuse_missing_feature(self):
        result = self.run_script(READ, "compiler", self.source)
        self.assertEqual((result.returncode, result.stdout), (0, "0.31.213\n"))
        result = self.run_script(READ, "language", self.source)
        self.assertEqual(result.returncode, 1)
        self.assertIn("could not read the language version", result.stderr)

    def test_read_requires_feature(self):
        result = self.run_script(READ)
        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr)

    def test_release_tag_stamps_complete_version_and_provenance(self):
        result = self.stamp("v0.31.213")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, f"0.31.213 ({COMMIT[:9]} {DATE})\n")
        config = self.config.read_text()
        self.assertIn('(define compiler-version-tag "")', config)
        self.assertIn(f'(define compiler-version-commit "{COMMIT}")', config)
        self.assertIn(f'(define compiler-version-commit-date "{DATE}")', config)

    def test_prerelease_and_development_tags(self):
        prerelease = self.stamp("v0.31.213-rc.2")
        self.assertEqual(prerelease.returncode, 0, prerelease.stderr)
        self.assertEqual(prerelease.stdout,
                         f"0.31.213-rc.2 ({COMMIT[:9]} {DATE})\n")
        self.assertIn('(define compiler-version-tag "-rc.2")', self.config.read_text())
        development = self.stamp("branch/name")
        self.assertEqual(development.returncode, 0, development.stderr)
        self.assertEqual(development.stdout,
                         f"0.31.213-dev ({COMMIT[:9]} {DATE})\n")
        self.assertIn('(define compiler-version-tag "-dev")', self.config.read_text())

    def test_invalid_inputs_refuse_before_writing(self):
        cases = (
            ("v0.31.213", "short", DATE, "not a full 40-character hash"),
            ("v0.31.213", COMMIT, "bad-date", "not YYYY-MM-DD"),
            ("v0.32.0", COMMIT, DATE, "but"),
            ("v0.31.213-rc.01", COMMIT, DATE, "not a valid semver prerelease"),
            ('v0.31.213-rc.1"', COMMIT, DATE, "not a valid semver prerelease"),
        )
        for tag, commit, date, diagnostic in cases:
            with self.subTest(tag=tag, commit=commit, date=date):
                self.config.write_text(self.original)
                result = self.stamp(tag, commit, date)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(diagnostic, result.stderr)
                self.assertEqual(self.config.read_text(), self.original)

    def test_wrong_library_refuses_without_overwrite(self):
        self.config.write_text("(library (other-config))\n")
        result = self.stamp("v0.31.213")
        self.assertEqual(result.returncode, 1)
        self.assertIn("not the version-config library", result.stderr)
        self.assertEqual(self.config.read_text(), "(library (other-config))\n")

    def test_missing_default_config_refuses_without_creating_it(self):
        # A missing default config is refused even when the version source exists.
        (self.base / "compiler").mkdir()
        (self.base / "compiler/compiler-version.ss").write_text(self.source.read_text())
        result = self.run_script(STAMP, "v0.31.213", COMMIT, DATE)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.base / "compiler/version-config.ss").exists())


if __name__ == "__main__":
    unittest.main()
