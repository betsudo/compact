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
import subprocess
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


    def test_cargo_target_defaults_outside_sdk_and_preserves_shared_cache(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary).resolve() / "qualification"
            self.assertEqual(gate.cargo_target({}, output), output / "cargo-target")
            shared = output.parent / "shared-target"
            self.assertEqual(gate.cargo_target({"CARGO_TARGET_DIR": str(shared)}, output), shared)
            self.assertEqual(gate.cargo_target({"CARGO_TARGET_DIR": "target"}, output),
                             (gate.ROOT / "target").resolve())
            for path in [output / "generated", output / "generated/contract/target"]:
                with self.subTest(path=path), self.assertRaisesRegex(ValueError, "outside the generated"):
                    gate.cargo_target({"CARGO_TARGET_DIR": str(path)}, output)

    def test_formatting_uses_copy_and_rejects_original_source_mutation(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            original = root / "lib.rs"
            copy = root / "formatted.rs"
            raw = "fn original(){true}\n"
            original.write_text(raw)
            def formatter(label, command):
                self.assertEqual(Path(command[-1]), copy)
                copy.write_text("fn original() { true }\n")
            self.assertEqual(gate.formatted_copy(original, copy, formatter), gate.digest(copy))
            self.assertEqual(original.read_text(), raw)
            def corrupting_formatter(label, command):
                formatter(label, command)
                original.write_text("changed original SDK\n")
            with self.assertRaisesRegex(ValueError, "changed the original"):
                gate.formatted_copy(original, copy, corrupting_formatter)

    def test_original_oracle_pin_authority_cannot_silently_lose_an_entry(self):
        with tempfile.TemporaryDirectory() as temporary:
            oracle = Path(temporary) / "oracle"
            shutil.copytree(gate.ORACLE, oracle)
            with patch.object(gate, "ORACLE", oracle):
                provenance = gate.original_oracle_provenance()
                self.assertEqual(len(provenance["oracle_files"]), 39)
                self.assertEqual(len(provenance["codec_sources"]), 2)
                provenance["oracle_files"].pop(next(iter(provenance["oracle_files"])))
                (oracle / "provenance.json").write_text(json.dumps(provenance))
                with self.assertRaisesRegex(ValueError, "provenance identity differs"):
                    gate.original_oracle_provenance()

    def test_short_or_missing_execution_accounting_cannot_pass(self):
        valid = "test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured"
        self.assertEqual(gate.rust_test_count(valid, 18), 18)
        for text in ["", valid.replace("18 passed", "17 passed"),
                     valid.replace("0 ignored", "1 ignored"), valid.replace("0 failed", "1 failed")]:
            with self.subTest(stdout=text), self.assertRaises(ValueError):
                gate.rust_test_count(text, 18)
        rows = [{"family": family, "cases": 202 if i == 0 else 0}
                for i, family in enumerate(gate.FAMILIES)]
        self.assertEqual(gate.capture_case_count(rows), 202)
        for changed in [rows[:-1], [dict(row, cases=201) if i == 0 else row for i, row in enumerate(rows)],
                        [dict(row, family="unknown") if i == 0 else row for i, row in enumerate(rows)]]:
            with self.subTest(rows=changed), self.assertRaises(ValueError):
                gate.capture_case_count(changed)

    def test_timeout_retains_partial_diagnostics_and_cannot_be_a_refusal(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            compiler = root / "compiler"
            compiler.write_bytes(b"must not execute")
            (root / "package.json").write_bytes((gate.ROOT / "runtime/package.json").read_bytes())
            output = root / "result"
            timeout = subprocess.TimeoutExpired([str(compiler), "--version"], gate.COMMAND_TIMEOUT_SECONDS,
                                                output=b"partial output\n", stderr=b"partial diagnostic\n")
            with patch.dict(gate.os.environ, {"COMPACTC_SCHEME": ""}), patch.object(
                    gate.subprocess, "check_output", side_effect=["head\n", ""]), patch.object(
                    gate.subprocess, "run", side_effect=timeout), patch.object(
                    gate.shutil, "which", return_value=str(compiler)), patch.object(gate, "typed_refusal") as refusal:
                with self.assertRaisesRegex(RuntimeError, "timed out: compiler-version"):
                    gate.main(["--compiler", str(compiler), "--ts-runtime", str(root), "--output", str(output)])
                refusal.assert_not_called()
            receipt = json.loads((output / "receipt.json").read_text())
            self.assertEqual(receipt["status"], "failed")
            self.assertIsNone(receipt["commands"][0]["exit"])
            self.assertTrue(receipt["commands"][0]["timed_out"])
            self.assertEqual((output / "compiler-version.stdout").read_text(), "partial output\n")
            self.assertEqual((output / "compiler-version.stderr").read_text(), "partial diagnostic\n")

    def test_scheme_override_cannot_impersonate_original_package(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.dict(gate.os.environ, {"COMPACTC_SCHEME": "unselected-compiler"}), patch.object(
                    gate.subprocess, "check_output", side_effect=["head\n", ""]), patch.object(
                    gate.subprocess, "run") as execute:
                with self.assertRaisesRegex(ValueError, "no COMPACTC_SCHEME override"):
                    gate.main(["--compiler", "missing", "--ts-runtime", "missing", "--output", str(root / "output")])
                execute.assert_not_called()


if __name__ == "__main__":
    unittest.main()

