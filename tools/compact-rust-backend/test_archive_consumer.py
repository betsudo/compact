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
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "archive_consumer", Path(__file__).with_name("check_archive_consumer.py")
)
archive = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(archive)


class RetainedArchiveTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name).resolve()
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

    def test_preflight_failure_has_receipt_before_manifest_parsing(self):
        manifest = self.root / "manifest.json"
        manifest.write_text("not JSON")
        with patch.object(archive, "ROOT", self.root), patch.object(sys, "argv", [
            "archive", "--manifest", str(manifest), "--output", str(self.output),
        ]), patch.object(archive.subprocess, "run") as run, self.assertRaises(json.JSONDecodeError):
            archive.main()
        run.assert_not_called()
        self.assertEqual(self.receipt()["status"], "failed")
        self.assertEqual(self.receipt()["inputs"]["sha256"][str(manifest)], archive.sha256(manifest))

    def test_preflight_snapshot_includes_every_safety_fixture(self):
        entries = []
        for name in archive.PACKAGE_NAMES:
            path = self.root / (name + ".crate")
            path.write_bytes(b"archive placeholder; no extraction reached")
            entries.append(dict(name=name, version="0.2.0", archive=str(path), sha256=archive.sha256(path)))
        manifest = self.root / "manifest.json"
        manifest.write_text(json.dumps(dict(schema=1, source=dict(commit="head", tree="tree"), packages=entries)))

        def run(command, **kwargs):
            if command == ["git", "rev-parse", "HEAD"]:
                return "head"
            if command == ["git", "rev-parse", "HEAD^{tree}"]:
                return "tree"
            if "vendor" in command:
                raise RuntimeError("stop before vendor")
            return "tool version"

        with self.assertRaisesRegex(RuntimeError, "stop before vendor"), archive.qualification_directory(self.output, {}), patch.object(archive, "run", side_effect=run), patch.object(
            archive, "selected_rust_tool", return_value=Path(sys.executable).resolve()
        ):
            archive.qualify(SimpleNamespace(manifest=manifest, compiler=sys.executable), self.output / "scratch")
            self.fail("vendor must not run")
        self.assertEqual(self.receipt()["status"], "failed")
        pins = self.receipt()["inputs"]["sha256"]
        for name in (*archive.POSITIVES, *archive.NEGATIVES):
            path = archive.FIXTURES / f"{name}.rs"
            self.assertEqual(pins[str(path)], archive.sha256(path))

    def test_rustup_proxy_records_selected_binary(self):
        proxy = self.root / "rustup"
        proxy.write_text("dispatcher")
        actual = self.root / "selected-cargo"
        actual.write_text("actual Cargo")
        with patch.object(archive.shutil, "which", return_value=str(proxy)), patch.object(
            archive, "run", return_value=str(actual)
        ) as run:
            self.assertEqual(archive.selected_rust_tool("cargo"), actual)
        run.assert_called_once_with([str(proxy), "which", "cargo"])

    def test_recorded_tools_are_invoked_and_drift_refuses_before_child(self):
        cargo = self.root / "cargo"
        rustc = self.root / "rustc"
        cargo.write_text("Cargo")
        rustc.write_text("rustc")
        tools = {name: dict(path=str(path), sha256=archive.sha256(path))
                 for name, path in (("cargo", cargo), ("rustc", rustc))}
        with archive.qualification_directory(self.output, dict(tools=tools)):
            with patch.object(archive.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "", "")) as run:
                archive.execute(["cargo", "--version"])
                self.assertEqual(run.call_args.args[0], [str(cargo), "--version"])
                self.assertEqual(run.call_args.kwargs["env"]["RUSTC"], str(rustc))
                rustc.write_text("changed compiler")
                with self.assertRaisesRegex(RuntimeError, "tool changed"):
                    archive.execute(["cargo", "--version"])
                self.assertEqual(run.call_count, 1)


if __name__ == "__main__":
    unittest.main()
