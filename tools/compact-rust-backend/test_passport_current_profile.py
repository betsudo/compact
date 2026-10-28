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

"""Adversarial identity, isolated-consumer and false-success gate controls."""
import copy
import json
from pathlib import Path
import re
import shutil
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import check_passport_current_profile as gate


class PassportControlTests(unittest.TestCase):
    def test_capture_retains_current_profile_and_all_original_behavior(self):
        expected = {"profile": "historical", "rows": [{"value": 42}], "vectors": {"root": "ab"}}
        actual = dict(copy.deepcopy(expected), profile="current")
        before = copy.deepcopy(actual)
        gate.compare_capture(actual, expected, "current")
        self.assertEqual(actual, before)
        for change in [dict(actual, profile="historical"), dict(actual, rows=[]),
                       dict(actual, vectors={"root": "cd"}), dict(actual, extra=True),
                       {"profile": "current", "rows": actual["rows"]}]:
            with self.subTest(capture=change), self.assertRaises(ValueError):
                gate.compare_capture(change, expected, "current")

    def test_original_import_closure_rejects_missing_extra_or_changed_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "source"
            shutil.copytree(gate.SOURCE, source)
            with patch.object(gate, "SOURCE", source):
                pins = gate.original_source_files()
                original = source / pins[0]["path"]
                saved = original.read_bytes()
                original.write_bytes(saved + b"\n")
                with self.assertRaisesRegex(ValueError, "source differs"):
                    gate.original_source_files()
                original.unlink()
                with self.assertRaisesRegex(ValueError, "membership differs"):
                    gate.original_source_files()
                original.write_bytes(saved)
                (source / "unexpected.compact").write_text("export pure circuit unrelated(): Boolean { return true; }")
                with self.assertRaisesRegex(ValueError, "membership differs"):
                    gate.original_source_files()

    def test_lock_and_local_dependency_controls_reject_escape_and_identity_drift(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            manifest = root / "Cargo.toml"
            lock = root / "Cargo.lock"
            lock.write_text('version=4\n[[package]]\nname="remote"\nversion="1.0.0"\nsource="registry+qualified"\nchecksum="abc"\n')
            remote = {"name": "remote", "version": "1.0.0", "source": "registry+qualified"}
            local = {"name": "local", "version": "0.1.0", "source": None, "manifest_path": str(manifest)}
            metadata = {"packages": [local, remote]}
            qualified = {("remote", "1.0.0", "registry+qualified"): "abc"}
            gate.validate_dependencies(metadata, lock, {manifest.resolve()}, qualified)
            for changed in [dict(local, manifest_path=str(root.parent / "Cargo.toml")),
                            dict(remote, version="2.0.0"), dict(remote, source="git+unapproved#other")]:
                with self.subTest(package=changed), self.assertRaises(ValueError):
                    gate.validate_dependencies({"packages": [changed]}, lock, {manifest.resolve()}, qualified)
            lock.write_text(lock.read_text().replace('checksum="abc"', 'checksum="other"'))
            with self.assertRaisesRegex(ValueError, "checksum differs"):
                gate.validate_dependencies(metadata, lock, {manifest.resolve()}, qualified)

    def test_refusal_requires_one_exact_type_error_and_real_failure(self):
        def result(code="E0308", exit_code=101):
            row = {"reason": "compiler-message", "message": {"level": "error", "code": {"code": code}}}
            return SimpleNamespace(returncode=exit_code, stdout=json.dumps(row) + "\n")
        self.assertEqual(gate.typed_refusal(result()), ["E0308"])
        for process in [result(exit_code=0), result(code="E0432"),
                        SimpleNamespace(returncode=101, stdout=""),
                        SimpleNamespace(returncode=101, stdout=result().stdout * 2)]:
            with self.subTest(process=process), self.assertRaises(ValueError):
                gate.typed_refusal(process)

    def test_dirty_checkout_cannot_start_a_compiler_or_consumer(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            output = root / "output"
            with patch.object(gate.subprocess, "check_output", side_effect=["head\n", " M runtime-rs/src/lib.rs\n"]), patch.object(gate.subprocess, "run") as execute:
                with self.assertRaisesRegex(RuntimeError, "clean source checkout required"):
                    gate.main(["--compiler", str(root / "missing"), "--ts-runtime", str(root), "--output", str(output)])
                execute.assert_not_called()
            receipt = json.loads((output / "receipt.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertTrue(receipt["source_dirty"])

    def test_existing_destination_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sentinel = root / "retained"
            sentinel.write_text("preserve this evidence")
            with self.assertRaisesRegex(ValueError, "destination must be absent"):
                gate.main(["--compiler", "missing", "--ts-runtime", "missing", "--output", str(root)])
            self.assertEqual(sentinel.read_text(), "preserve this evidence")

    def test_compiler_version_prefix_or_prose_is_not_selected_identity(self):
        version = re.search(r"make-version 'compiler (\d+) (\d+) (\d+)",
                            (gate.ROOT / "compiler/compiler-version.ss").read_text())
        selected = ".".join(version.groups())
        for reported in [selected + "0\n", "unselected " + selected + "\n"]:
            with self.subTest(reported=reported), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                compiler = root / "compiler"
                compiler.write_bytes(b"must not execute")
                (root / "package.json").write_bytes((gate.ROOT / "runtime/package.json").read_bytes())
                output = root / "result"
                process = SimpleNamespace(returncode=0, stdout=reported, stderr="")
                with patch.object(gate.subprocess, "check_output", side_effect=["head\n", ""]), patch.object(
                        gate.subprocess, "run", return_value=process) as execute, patch.object(
                        gate.shutil, "which", return_value=str(compiler)):
                    with self.assertRaisesRegex(ValueError, "compiler version differs"):
                        gate.main(["--compiler", str(compiler), "--ts-runtime", str(root), "--output", str(output)])
                    execute.assert_called_once()
                    self.assertEqual(execute.call_args.args[0], [str(compiler.resolve()), "--version"])
                self.assertEqual(json.loads((output / "receipt.json").read_text())["status"], "failed")


if __name__ == "__main__":
    unittest.main()

