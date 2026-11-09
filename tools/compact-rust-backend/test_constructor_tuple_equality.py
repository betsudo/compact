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

"""Fixture identity and entry admission controls, never current execution proof."""
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_constructor_equality_scope as scope_owner
import constructor_tuple_scope as tuple_scope
import check_constructor_equality as api_owner


class HistoricalTupleFixtures(unittest.TestCase):
    def test_test_worker_artifact_requires_current_source_profile_and_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            consumer = root / "consumer"
            consumer.mkdir()
            (consumer / "consumer.rs").write_text("// HOST source, never compiled\n")
            target = root / "target"
            target.mkdir()
            executable = target / "HOST-never-executed"
            executable.write_bytes(b"HOST fixture, not an ELF")
            outside = root / "HOST-outside-target"
            outside.write_bytes(b"HOST fixture, never executed")
            metadata = dict(resolve=dict(root="HOST-package"))
            original = dict(reason="compiler-artifact", package_id="HOST-package",
                executable=str(executable), fresh=True,
                target=dict(kind=["lib"], src_path=str(consumer / "consumer.rs")),
                profile=dict(test=True, debug_assertions=True, overflow_checks=True))
            admitted = api_owner.test_worker_artifact(json.dumps(original), metadata, consumer, target)
            self.assertEqual(admitted["path"], str(executable))
            self.assertTrue(admitted["compiled_cache_reused"])
            for mutation in ("missing", "duplicate", "wrong-package", "bin-target",
                             "not-test", "wrong-source", "unchecked", "outside-target"):
                with self.subTest(mutation=mutation):
                    row = deepcopy(original)
                    if mutation == "wrong-package": row["package_id"] = "HOST-other"
                    elif mutation == "bin-target": row["target"]["kind"] = ["bin"]
                    elif mutation == "not-test": row["profile"]["test"] = False
                    elif mutation == "wrong-source": row["target"]["src_path"] = str(consumer / "other.rs")
                    elif mutation == "unchecked": row["profile"]["overflow_checks"] = False
                    elif mutation == "outside-target": row["executable"] = str(outside)
                    stdout = "" if mutation == "missing" else json.dumps(row)
                    if mutation == "duplicate": stdout += "\n" + json.dumps(row)
                    with self.assertRaises(ValueError):
                        api_owner.test_worker_artifact(stdout, metadata, consumer, target)

    def test_all_current_cases_keep_exact_proof_metadata_and_complete_source_roster(self):
        fixtures = tuple_scope.FIXTURES
        cases = json.loads((fixtures / "cases.json").read_bytes())
        proofs = json.loads((fixtures / "proof-cases.json").read_bytes())
        sources = json.loads((fixtures / "sources.json").read_bytes())
        self.assertEqual(len(cases), 105)
        self.assertEqual(len({(r["role"], r["case"]) for r in cases}), 105)
        self.assertEqual(tuple(r["role"] for r in sources), api_owner.TUPLE_ROLES)
        self.assertEqual({r["role"] for r in cases}, set(api_owner.TUPLE_ROLES))
        self.assertEqual(len(sources), 16)
        self.assertEqual({p.stem for p in (fixtures / 'constructors').glob('*.rs')},
                         set(api_owner.TUPLE_ROLES))
        self.assertEqual(proofs, [dict(r, id=r["role"] + "_" + r["case"],
                                      privateState=42, trace=[], error=None) for r in cases])
        for row in sources:
            source = api_owner.ROOT / row["path"]
            self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), row["sha256"])
            self.assertEqual(row["entries"], ["read", "advance", "advance_other"])
            self.assertEqual(row["constructor_consumer"], row["role"])
        for name in api_owner.TUPLE_NEGATIVES:
            self.assertEqual((fixtures / (name + ".rs")).read_text().count("// expected-error"), 1)
            self.assertNotIn("// expected-error", (fixtures / (name + "_twin.rs")).read_text())

    def test_tuple_api_dirty_stale_recipe_refuses_before_tools_or_output(self):
        for dirty in (False, True):
            with self.subTest(dirty=dirty), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                recipe = root / "recipe.json"
                recipe.write_text(json.dumps(dict(status="passed", head="stale", tree="tree")))
                output = root / "output"
                argv = ["gate", "--compiler-receipt", str(recipe), "--ts-runtime", str(root), "--output", str(output)]
                def git(command, **kwargs):
                    return {("status", "--porcelain"): "dirty" if dirty else "", ("rev-parse", "HEAD"): "head", ("rev-parse", "HEAD^{tree}"): "tree"}[tuple(command[1:])]
                with patch.object(api_owner, "ROOT", root), patch.object(sys, "argv", argv), patch.object(api_owner.subprocess, "check_output", side_effect=git), patch.object(api_owner.subprocess, "run") as execute:
                    with self.assertRaisesRegex(ValueError, "clean source|current clean original"):
                        api_owner.main(tuple_guards=True)
                execute.assert_not_called()
                self.assertFalse(output.exists())

    def test_current_scope_expectations_cover_all_complete_sources(self):
        rows = tuple_scope.expectations()
        self.assertEqual(set(rows), set(tuple_scope.ROLES))
        self.assertEqual(len(rows), 18)
        for row in rows.values():
            if "raw_refusal" in row:
                continue
            paths = [tuple(r["path"]) for r in row["condition_origins"]]
            self.assertEqual(len(paths), len(set(paths)))
            self.assertTrue(paths)
            for r in row["condition_origins"]:
                self.assertTrue(r["source_fragment"])
                self.assertEqual(r["origin_role"], "expression")

    def test_full_209_source_ir_pairs_keep_exact_hashes_and_all_exports(self):
        fixtures = Path(__file__).with_name("tests") / "constructor_equality_tuple_guards"
        rows = json.loads((fixtures / "historical209/manifest.json").read_bytes())
        self.assertEqual(set(rows), {"flat_eq", "flat_ne", "nested_eq", "mixed_literal_eq",
                                    "reversed_eq", "asymmetric_eq", "owned_capture_eq"})
        for role, row in rows.items():
            source = fixtures / (role + ".compact")
            raw = fixtures / "historical209" / (role + ".json")
            self.assertEqual(hashlib.sha256(source.read_bytes()).hexdigest(), row["source_sha256"])
            self.assertEqual(hashlib.sha256(raw.read_bytes()).hexdigest(), row["raw_ir_sha256"])
            self.assertEqual(row["compiler"], "0.31.209")
            self.assertEqual(row["compiler_source"], "7f690b7a68f054048fad74406e4f622a6a03af48")
            ir = json.loads(raw.read_bytes())
            self.assertEqual(ir["schema_version"], 20)
            self.assertEqual([c["name"] for c in ir["stateful_circuits"]],
                             ["read", "advance", "advance_other"])
            self.assertEqual((row["raw_generation_exit"], row["ts_generation_exit"], row["rust_generation_exit"]), (0, 0, 1))
            self.assertNotEqual(row["type_mismatch_expected"], row["type_mismatch_actual"])

    def test_tuple_scope_stale_and_dirty_recipe_refuse_before_compiler_effects(self):
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
                with patch.object(scope_owner, "ROOT", source), patch.object(sys, "argv", argv), patch.object(scope_owner.subprocess, "check_output", side_effect=git), patch.object(scope_owner, "bounded_command") as execute:
                    with self.assertRaisesRegex(ValueError, "clean source|current clean original"):
                        scope_owner.main(tuple_guards=True)
                execute.assert_not_called()
                self.assertFalse(output.exists())

    def test_optimized_tuple_scope_entrypoint_refuses_before_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            output = root / "out"
            script = Path(scope_owner.__file__).with_name("check_constructor_tuple_equality_scope.py")
            result = subprocess.run([sys.executable, "-O", "-B", str(script),
                "--compiler-receipt", str(root / "missing"), "--output", str(output)],
                capture_output=True, text=True, timeout=30,
                env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("optimized Python is not admitted", result.stderr)
            self.assertFalse(output.exists())

    def test_tuple_model_entry_requires_both_scope_receipts_before_output(self):
        for supplied, missing in (([], '--scope-receipt'),
                                  (['--scope-receipt', 'HOST-missing.json'], '--tuple-scope-receipt')):
            with self.subTest(missing=missing), tempfile.TemporaryDirectory() as temporary:
                output = Path(temporary) / 'out'
                script = Path(__file__).with_name('check_constructor_tuple_equality_models.py')
                result = subprocess.run([sys.executable, '-B', script, '--producer-root', temporary,
                    '--producer-receipt', 'HOST-missing.json', '--output', output, *supplied],
                    capture_output=True, text=True, timeout=30,
                    env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'))
                self.assertEqual(result.returncode, 2)
                self.assertIn(missing, result.stderr)
                self.assertFalse(output.exists())

    def test_tuple_qualification_entries_reject_optimized_python(self):
        for suffix in ('models', 'proof_preflight', 'proof'):
            with self.subTest(suffix=suffix):
                script = Path(__file__).with_name('check_constructor_tuple_equality_' + suffix + '.py')
                result = subprocess.run([sys.executable, '-O', '-B', script, '--help'],
                    capture_output=True, text=True, timeout=30,
                    env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'))
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('is not admitted', result.stderr)


if __name__ == "__main__":
    unittest.main()
