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

"""Refusal controls for the isolated SDK qualification boundary."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_functional_source_sdk as gate


class IsolatedSdkControls(unittest.TestCase):
    def test_clippy_library_requires_exact_nonfresh_owner(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "lib.rs"
            source.write_text("pub fn correct() {}\n")
            target = {"kind": ["lib"], "src_path": str(source)}
            metadata = {"packages": [{"name": "selected", "id": "selected-id", "targets": [target]}]}
            artifact = {"reason": "compiler-artifact", "package_id": "selected-id",
                        "target": target, "fresh": False}
            result = gate.clippy_library_artifact(json.dumps(artifact), metadata, "selected")
            self.assertEqual(result["source"], str(source.resolve()))
            with self.assertRaisesRegex(ValueError, "required proof feature"):
                gate.clippy_library_artifact(json.dumps(artifact), metadata, "selected", required_feature="proof")
            gate.clippy_library_artifact(json.dumps(dict(artifact, features=["proof"])), metadata,
                                        "selected", required_feature="proof")
            for invalid in (dict(artifact, fresh=True), dict(artifact, package_id="dependency-id"),
                            dict(artifact, target={"kind": ["bin"], "src_path": str(source)})):
                with self.assertRaisesRegex(ValueError, "actually compile"):
                    gate.clippy_library_artifact(json.dumps(invalid), metadata, "selected")
            with self.assertRaisesRegex(ValueError, "actually compile"):
                gate.clippy_library_artifact("", metadata, "selected")
            with self.assertRaisesRegex(ValueError, "actually compile"):
                gate.clippy_library_artifact(json.dumps(artifact)+"\n"+json.dumps(artifact), metadata, "selected")

    def test_external_library_clippy_detects_lint_after_warm_plain_check(self):
        rustup = shutil.which("rustup")
        if rustup is None:
            self.skipTest("Rustup and pinned Rust1.99 Clippy required for actual lint regression")
        selected = subprocess.run([rustup, "which", "--toolchain", "1.99.0", "clippy-driver"],
                                  capture_output=True, text=True)
        if selected.returncode:
            self.skipTest("Pinned Rust1.99 Clippy unavailable; synthetic artifact controls still run")
        driver = Path(selected.stdout.strip()).resolve(strict=True)
        cargo = driver.with_name("cargo")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            consumer, dependency = root / "consumer", root / "dependency"
            for folder in (consumer, dependency):
                (folder / "src").mkdir(parents=True)
            manifest = consumer / "Cargo.toml"
            manifest.write_text('[workspace]\n[package]\nname="lint-consumer"\nversion="0.0.0"\n'
                                'edition="2024"\n[dependencies]\nlint-dependency={path="../dependency"}\n'
                                '[features]\nproof=["lint-dependency/proof"]\n')
            (consumer / "src/lib.rs").write_text("pub fn call() -> bool { lint_dependency::call(&[]) }\n")
            (dependency / "Cargo.toml").write_text('[package]\nname="lint-dependency"\nversion="0.0.0"\nedition="2024"\n'
                                                   '[features]\nproof=[]\n')
            source = dependency / "src/lib.rs"
            source.write_text('pub fn call(xs: &[u8]) -> bool { xs.len() == 0 }\n')
            env = dict(os.environ, RUSTUP_TOOLCHAIN="1.99.0", CARGO_TARGET_DIR=str(root / "target"))
            gate.cargo_configuration(env)
            warm = subprocess.run([cargo, "check", "--offline", "--manifest-path", manifest],
                                  env=env, capture_output=True, text=True, timeout=60)
            self.assertEqual(warm.returncode, 0, warm.stderr)
            def metadata(selected_manifest, *features):
                result = subprocess.run([cargo, "metadata", "--offline", "--format-version", "1",
                                         "--manifest-path", selected_manifest, *features], env=env,
                                        capture_output=True, text=True, check=True, timeout=60)
                return json.loads(result.stdout)
            original = metadata(manifest, "--features", "proof")
            lint_manifest = gate.library_lint_root(manifest, root / "library-lints")
            lint_graph = metadata(lint_manifest)
            gate.check_library_lint_graph(original, lint_graph, lint_manifest)
            for field in ("packages", "resolve"):
                changed = copy.deepcopy(lint_graph)
                if field == "packages":
                    owner = next(p for p in changed[field] if p["id"] == original["resolve"]["root"])
                    owner["version"] = "999.0.0"
                else:
                    node = next(n for n in changed[field]["nodes"] if n["id"] == original["resolve"]["root"])
                    node["features"] = []
                with self.assertRaisesRegex(ValueError, "identities or features"):
                    gate.check_library_lint_graph(original, changed, lint_manifest)
            command = gate.clippy_library_command(cargo, lint_manifest, "lint-dependency")
            lint_env = dict(env, RUSTC=str(driver))
            warning = subprocess.run(command[:-2], env=lint_env, capture_output=True, text=True, timeout=60)
            self.assertEqual(warning.returncode, 0, warning.stderr)
            warnings = [json.loads(line) for line in warning.stdout.splitlines()]
            self.assertTrue(any(r.get("reason") == "compiler-message" and r["message"]["level"] == "warning"
                                and (r["message"].get("code") or {}).get("code") == "clippy::len_zero"
                                for r in warnings))
            refused = subprocess.run(command, env=lint_env, capture_output=True, text=True, timeout=60)
            diagnostics = [json.loads(line) for line in refused.stdout.splitlines()]
            self.assertNotEqual(refused.returncode, 0, refused.stderr)
            errors = [r["message"] for r in diagnostics if r.get("reason") == "compiler-message"
                      and r["message"]["level"] == "error"]
            self.assertEqual([r["code"]["code"] for r in errors], ["clippy::len_zero"])
            self.assertTrue(any(Path(s["file_name"]).resolve() == source.resolve()
                                for s in errors[0]["spans"] if s["is_primary"]))
            source.write_text('pub fn call(xs: &[u8]) -> bool { xs.is_empty() }\n')
            correct = subprocess.run(command, env=lint_env, capture_output=True, text=True, timeout=60)
            self.assertEqual(correct.returncode, 0, correct.stderr)
            result = gate.clippy_library_artifact(correct.stdout, original, "lint-dependency", required_feature="proof")
            self.assertFalse(result["fresh"])
            gate.cargo_configuration(env)

    def test_ignored_build_output_and_lock_refuse_tracked_source_admission(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            consumer = root / "consumer"
            (consumer / "src").mkdir(parents=True)
            (consumer / "Cargo.toml").write_text('[package]\nname="fixture"\nversion="0.0.0"\n')
            (consumer / "src/lib.rs").write_text("pub fn fixture() {}\n")
            (root / ".gitignore").write_text("target/\nCargo.lock\n")
            environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
            subprocess.run(["git", "init", "-q", root], env=environment, check=True)
            subprocess.run(["git", "add", ".gitignore", "consumer"], cwd=root, env=environment, check=True)
            with patch.object(gate, "ROOT", root):
                expected = {consumer / "Cargo.toml", consumer / "src/lib.rs"}
                self.assertEqual(set(gate.tracked_source_files(consumer)), expected)
                for relative in ("target/build/generated.rs", "Cargo.lock"):
                    injected = consumer / relative
                    injected.parent.mkdir(parents=True, exist_ok=True)
                    injected.write_text("ignored build output")
                    self.assertEqual(subprocess.run(["git", "check-ignore", "--quiet", injected],
                        cwd=root, env=environment).returncode, 0)
                    with self.assertRaisesRegex(ValueError, "untracked/ignored"):
                        gate.tracked_source_files(consumer)
                    injected.unlink()
                self.assertEqual(set(gate.tracked_source_files(consumer)), expected)

    def test_runtime_source_override_refuses_before_prerequisites_and_commands(self):
        for value in (str(gate.ROOT), ""):
            with self.subTest(value=value), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "qualification"
                with patch.dict(os.environ, {"COMPACT_RUST_RUNTIME_DIR": value}, clear=True), \
                     patch.object(gate.common, "git_head", return_value="selected-source"), \
                     patch.object(gate.common, "dirty_paths", return_value=[]), \
                     patch.object(gate.did_proof_gate, "prerequisites") as prerequisites, \
                     patch.object(gate, "bounded_command") as command:
                    with self.assertRaisesRegex(ValueError, "forbids COMPACT_RUST_RUNTIME_DIR"):
                        gate.main(["--compiler", str(Path(directory) / "not-needed"),
                                   "--material-runner", str(Path(directory) / "not-needed-either"),
                                   "--output", str(output)])
                    prerequisites.assert_not_called()
                    command.assert_not_called()
                failed = json.loads((output / "receipt.json").read_text())
                self.assertEqual(failed["status"], "failed")
                self.assertEqual(failed["commands"], [])

    def test_proof_server_refuses_before_prerequisites(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "qualification"
            with patch.dict(os.environ, {"MIDNIGHT_PROOF_SERVER": "http://localhost:9"}, clear=True), \
                 patch.object(gate.common, "git_head", return_value="head"), \
                 patch.object(gate.common, "dirty_paths", return_value=[]), \
                 patch.object(gate.did_proof_gate, "prerequisites") as prerequisites:
                with self.assertRaisesRegex(ValueError, "requires local proving"):
                    gate.main(["--compiler", "compiler", "--material-runner", "runner",
                               "--output", str(output)])
                prerequisites.assert_not_called()

    def test_installed_runtime_selection_matches_cli_prefix_order_and_refuses_fallback(self):
        for layout in ("flat", "bin", "both", "missing", "first-is-file"):
            with self.subTest(layout=layout), tempfile.TemporaryDirectory() as directory:
                package = Path(directory)
                compiler = package / ("compactc" if layout == "flat" else "bin/compactc")
                compiler.parent.mkdir(parents=True, exist_ok=True)
                compiler.write_text("selected compiler fixture")
                first = compiler.parent / "share/compactc"
                second = compiler.parent.parent / "share/compactc"
                if layout == "missing":
                    with self.assertRaisesRegex(ValueError, "lacks an installed runtime"):
                        gate.installed_runtime_source_root(compiler)
                    continue
                if layout == "first-is-file":
                    first.parent.mkdir(parents=True)
                    first.write_text("invalid earlier prefix must not silently fall through")
                    second.mkdir(parents=True)
                    with self.assertRaisesRegex(ValueError, "not a directory"):
                        gate.installed_runtime_source_root(compiler)
                    continue
                expected = second if layout == "bin" else first
                expected.mkdir(parents=True)
                if layout == "both":
                    second.mkdir(parents=True)
                self.assertEqual(gate.installed_runtime_source_root(compiler), expected.resolve())

    def test_missing_material_refuses_before_any_external_command(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "material").mkdir()
            (root / "static").mkdir()
            zkir = root / "zkir"
            zkir.write_text("#!/bin/sh\nexit 0\n")
            zkir.chmod(0o700)
            selected = {"MIDNIGHT_PP": str(root / "material"),
                        "MIDNIGHT_LEDGER_TEST_STATIC_DIR": str(root / "static"),
                        "PATH": str(root)}
            self.assertEqual(gate.did_proof_gate.prerequisites(selected)["zkir"], str(zkir.resolve()))
            selected.pop("MIDNIGHT_PP")
            with patch.dict(os.environ, selected, clear=True), \
                 patch.object(gate.common, "git_head", return_value="selected-source"), \
                 patch.object(gate.common, "dirty_paths", return_value=[]), \
                 patch.object(gate, "bounded_command") as command:
                with self.assertRaisesRegex(gate.common.GateError, "MIDNIGHT_PP"):
                    gate.main(["--compiler", str(root / "not-yet-needed"),
                               "--material-runner", str(root / "not-yet-needed-either"),
                               "--output", str(root / "qualification")])
                command.assert_not_called()
            failed = json.loads((root / "qualification/receipt.json").read_text())
            self.assertEqual(failed["status"], "failed")
            self.assertEqual(failed["commands"], [])

    def test_extra_file_and_symlink_refuse_after_positive_inventory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "lib.rs").write_text("raw SDK")
            expected = gate.inventory(root)
            gate.equal_inventory(root, expected)
            (root / "injected.rs").write_text("extra input")
            with self.assertRaisesRegex(ValueError, "membership"):
                gate.equal_inventory(root, expected)
            (root / "injected.rs").unlink()
            (root / "link").symlink_to(root / "lib.rs")
            with self.assertRaisesRegex(ValueError, "symlink"):
                gate.inventory(root)

    def test_staged_manifest_lock_readme_and_license_drift_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = ("Cargo.toml", "Cargo.lock", "README.md", "LICENSE")
            for name in names:
                (root / name).write_text("staged:"+name)
            staged = gate.inventory(root)
            gate.equal_inventory(root, staged)
            for name in names:
                original = (root / name).read_bytes()
                (root / name).write_bytes(original+b" drift")
                with self.assertRaisesRegex(ValueError, "membership or bytes"):
                    gate.equal_inventory(root, staged)
                (root / name).write_bytes(original)
            gate.equal_inventory(root, staged)

    def test_library_recipe_preserves_original_and_refuses_ambiguous_path(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "original"
            (source / "src").mkdir(parents=True)
            (source / "src/lib.rs").write_text("original")
            original = b'[dependencies]\nruntime={path = "../runtime-rs"}\n'
            (source / "Cargo.toml").write_bytes(original)
            for name in ("README.md", "LICENSE"):
                (source / name).write_text(name)
            recipe = gate.source_testkit(source, root / "testkit-rs")
            self.assertEqual((root / "original-testkit-Cargo.toml").read_bytes(), original)
            self.assertEqual((source / "Cargo.toml").read_bytes(), original)
            self.assertEqual(recipe["transformation"]["count"], 1)
            self.assertNotEqual(recipe["original_sha256"], recipe["derived_sha256"])
            (source / "Cargo.toml").write_bytes(original * 2)
            with self.assertRaisesRegex(ValueError, "exactly once"):
                gate.source_testkit(source, root / "ambiguous")

    def test_repository_license_is_copied_when_testkit_has_no_own_license(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "testkit-source"
            (source / "src").mkdir(parents=True)
            (source / "src/lib.rs").write_text("source")
            (source / "Cargo.toml").write_text('runtime={path = "../runtime-rs"}')
            (source / "README.md").write_text("readme")
            (root / "LICENSE").write_text("repository license")
            with patch.object(gate, "ROOT", root):
                recipe = gate.source_testkit(source, root / "package")
            self.assertEqual((root / "package/LICENSE").read_bytes(), (root / "LICENSE").read_bytes())
            self.assertEqual(recipe["license_source"], str(root / "LICENSE"))

    def test_dependency_graph_refuses_checkout_path_and_checksum_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            packages = []
            for name, relative in (("consumer", "consumer"), ("testkit", "testkit-rs"),
                                   ("contract", "generated/contract"),
                                   ("midnight-compact-runtime", "generated/contract/runtime-rs"),
                                   ("macros", "generated/contract/runtime-rs-macros")):
                packages.append({"name": name, "version": "0.0.0", "source": None,
                                 "manifest_path": str(root / relative / "Cargo.toml")})
            lock = root / "Cargo.lock"
            lock.write_text('[[package]]\nname="external"\nversion="1"\nsource="registry+test"\nchecksum="good"\n')
            cohort = {("external", "1", "registry+test"): "good"}
            gate.dependency_graph({"packages": packages}, lock, root, cohort)
            bad = copy.deepcopy(packages)
            bad[0]["manifest_path"] = str(root.parent / "checkout/Cargo.toml")
            with self.assertRaisesRegex(ValueError, "escapes"):
                gate.dependency_graph({"packages": bad}, lock, root, cohort)
            lock.write_text(lock.read_text().replace('"good"', '"bad"'))
            with self.assertRaisesRegex(ValueError, "checksum"):
                gate.dependency_graph({"packages": packages}, lock, root, cohort)

    def test_rust_error_needs_single_expected_code_and_intended_primary_span(self):
        path = Path("consumed_context_negative.rs")
        row = {"reason": "compiler-message", "message": {"level": "error", "code": {"code": "E0382"},
               "spans": [{"is_primary": True, "file_name": str(path), "line_start": 8, "line_end": 8}]}}
        def process(rows, status=101):
            return subprocess.CompletedProcess([], status, "\n".join(map(json.dumps, rows)), "")
        gate.typed_refusal(process([row]), "E0382", path, 8)
        for rows, code, line, status in (([row], "E0053", 8, 101), ([row], "E0382", 7, 101),
                                          ([row, row], "E0382", 8, 101), ([row], "E0382", 8, 0)):
            with self.assertRaises(ValueError):
                gate.typed_refusal(process(rows, status), code, path, line)

    def test_proof_artifact_binds_actual_source_and_checked_profile(self):
        consumer = Path("/isolated/consumer")
        row = {"reason": "compiler-artifact", "package_id": "source-package", "fresh": True,
               "executable": "/target/proof-checks/functional-sdk-proof",
               "target": {"name": "functional-sdk-proof", "src_path": str(consumer / "proof/prove.rs")},
               "profile": {"debug_assertions": True, "overflow_checks": True}}
        self.assertTrue(gate.proof_artifact(json.dumps(row), consumer)["compiled_cache_reused"])
        for key in ("debug_assertions", "overflow_checks"):
            bad = copy.deepcopy(row)
            bad["profile"][key] = False
            with self.assertRaisesRegex(ValueError, "checked profile"):
                gate.proof_artifact(json.dumps(bad), consumer)
        bad = copy.deepcopy(row)
        bad["target"]["src_path"] = "/old-checkout/prove.rs"
        with self.assertRaisesRegex(ValueError, "different consumer"):
            gate.proof_artifact(json.dumps(bad), consumer)
        with self.assertRaisesRegex(ValueError, "one actual"):
            gate.proof_artifact(json.dumps(row)+"\n"+json.dumps(row), consumer)

    def test_relaxed_missing_or_wrong_replay_results_cannot_pass(self):
        result = dict(status="passed", strict_application="default", replay_refusal="IntentAlreadyExists",
                      witness_private_state=[7, 8], skip_sequence=[None]*6,
                      standalone_proof_bytes=2912, transaction_bytes=6607, final_state_hex="00",
                      final_data_hex="00", expected_data_hex="00",
                      final_cell_le_hex=(42).to_bytes(32, "little").hex(),
                      preparation_refusals=["ReplayStateMismatch", "NonCanonicalPrivateOutput(0,0)"],
                      changed_input_new_statement_check="valid; witness return is unconstrained relative to input")
        names = ("changed_binding_rejected", "changed_input_rejected", "changed_output_rejected",
                 "changed_private_witness_rejected", "changed_proof_rejected", "same_geometry_controls", "replay_unchanged",
                 "changed_input_new_statement_valid")
        result.update({name: True for name in names})
        gate.validate_result(result)
        for name in names:
            bad = dict(result)
            bad.pop(name)
            with self.assertRaises(ValueError):
                gate.validate_result(bad)
        for key, value in (("strict_application", "relaxed"), ("replay_refusal", "any error"),
                           ("skip_sequence", [None, 1]), ("transaction_bytes", 0),
                           ("preparation_refusals", ["any error"]),
                           ("changed_input_new_statement_valid", False),
                           ("final_data_hex", "01"), ("expected_data_hex", "01"),
                           ("final_cell_le_hex", (43).to_bytes(32, "little").hex()),
                           ("final_state_hex", ""), ("changed_input_new_statement_check", "unqualified")):
            with self.assertRaises(ValueError):
                gate.validate_result(dict(result, **{key: value}))

    def test_ledger9_result_requires_distinct_funded_v3_proof(self):
        result = dict(transaction_proof_version=3, funded_dust_spends=1,
                      standalone_proof_bytes=2, transaction_proof_bytes=2,
                      transaction_bytes=3)
        with tempfile.TemporaryDirectory() as directory:
            application = Path(directory)
            (application / "standalone.proof").write_bytes(b"aa")
            (application / "transaction-bound.proof").write_bytes(b"bb")
            (application / "applied-transaction.bin").write_bytes(b"txn")
            with patch.object(gate, "validate_result") as base:
                gate.validate_ledger9_result(result, application)
                base.assert_called_once_with(result)
                for key, value in (("transaction_proof_version", 2), ("funded_dust_spends", True),
                                   ("funded_dust_spends", 0), ("transaction_proof_bytes", 3)):
                    with self.assertRaises(ValueError):
                        gate.validate_ledger9_result(dict(result, **{key: value}), application)
                (application / "transaction-bound.proof").write_bytes(b"aa")
                with self.assertRaisesRegex(ValueError, "must be distinct"):
                    gate.validate_ledger9_result(result, application)

    def test_selected_config_refuses_compiler_substitution_and_unchecked_profiles(self):
        for key in ("RUSTC_WRAPPER", "RUSTC_BOOTSTRAP", "CARGO_PROFILE_PROOF_CHECKS_DEBUG_ASSERTIONS",
                    "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS", "CLIPPY_ARGS"):
            with self.assertRaisesRegex(ValueError, "override"):
                gate.cargo_configuration({key: "false"})
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text('[profile.proof-checks]\noverflow-checks=false\n')
            with patch.object(gate, "ROOT", root), self.assertRaisesRegex(ValueError, "profile"):
                gate.cargo_configuration({"CARGO_HOME": str(root / "cargo-home")})

    @unittest.skipUnless(os.name == "posix", "owned process group control")
    def test_actual_timeout_preserves_partial_output(self):
        with patch.object(gate, "TIMEOUT", 0.1):
            with self.assertRaises(subprocess.TimeoutExpired) as result:
                gate.bounded_command([sys.executable, "-c", "import time;print('started',flush=True);time.sleep(30)"], dict(os.environ))
        text = result.exception.stdout
        self.assertIn("started", text.decode() if isinstance(text, bytes) else text)


if __name__ == "__main__":
    unittest.main()
