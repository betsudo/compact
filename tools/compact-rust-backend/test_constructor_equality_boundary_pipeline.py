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

"""HOST orchestration controls only; no source/model/key/proof admission."""
from contextlib import ExitStack, redirect_stdout
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_constructor_equality_proof as owner


class CompositionControls(unittest.TestCase):
    def exercise(self, *, boundaries=True, tuple_guards=False, key_free=False, fail=None):
        with tempfile.TemporaryDirectory(prefix="HOST equality pipeline ") as temporary:
            root = Path(temporary).resolve()
            source = root / "source"
            source.mkdir()
            output = root / "output"
            zkir = root / "HOST-not-executable"
            zkir.write_text("HOST, never executed")
            recipe = root / "recipe.json"
            recipe.write_text(json.dumps(dict(status="passed", head="head", tree="tree", format="HOST-no-authority")))
            count = 105 if tuple_guards else 30 if boundaries else 16
            calls, arguments = [], {}
            def stage(name):
                def run(**kwargs):
                    actual = "tuple-scope" if name == "scope" and kwargs.get("tuple_guards") else name
                    calls.append(actual)
                    arguments[actual] = list(sys.argv[1:])
                    expected = {"tuple_guards": True} if tuple_guards and actual != "scope" else {"boundaries": True} if boundaries and name != "scope" else {}
                    self.assertEqual(kwargs, expected)
                    if fail == actual:
                        raise RuntimeError("HOST stop " + actual)
                    destination = Path(sys.argv[sys.argv.index("--output") + 1])
                    destination.mkdir()
                    value = dict(status="passed", head="head", inputs={}, format="HOST-no-authority")
                    if name == "source":
                        value["sources"] = {str(i): {} for i in range(16 if tuple_guards else 8)}
                    if name == "preflight":
                        value.update(selected_cases=count, prerequisites={"zkir": str(zkir)},
                                     frozen_inputs={}, frozen_directories={}, dependency_sources={},
                                     verified_material_by_k={"6": {"format": "HOST-no-material-authority"}})
                    if name == "models":
                        value["observed_k"] = [6]
                    (destination / "receipt.json").write_text(json.dumps(value))
                return run
            def prove(selected, environment, **kwargs):
                calls.append("proofs")
                self.assertEqual(selected, output)
                self.assertEqual(kwargs, {"tuple_guards": True} if tuple_guards else {"boundaries": True} if boundaries else {})
                if fail == "proofs":
                    raise RuntimeError("HOST stop proofs")
                value = dict(status="passed", fresh_standalone_proofs=count,
                             separately_proved_applications=count, format="HOST-no-proof-authority")
                (selected / "proofs").mkdir()
                (selected / "proofs/receipt.json").write_text(json.dumps(value))
                return value
            argv = ["HOST-gate", "--compiler-receipt", str(recipe), "--ts-runtime", str(root), "--zkir", str(zkir), "--output", str(output)]
            if key_free:
                argv.append("--preflight-only")
            with ExitStack() as stack:
                stack.enter_context(patch.object(owner, "W", source))
                stack.enter_context(patch.object(owner, "git", side_effect=lambda *a: {("rev-parse", "HEAD"): "head", ("rev-parse", "HEAD^{tree}"): "tree", ("status", "--porcelain"): ""}[a]))
                stack.enter_context(patch.object(owner.sdk, "cargo_configuration", return_value={}))
                stack.enter_context(patch.object(owner.source_owner, "main", stage("source")))
                stack.enter_context(patch.object(owner.scope_owner, "main", stage("scope")))
                stack.enter_context(patch.object(owner.model_owner, "main", stage("models")))
                stack.enter_context(patch.object(owner.preflight_owner, "main", stage("preflight")))
                stack.enter_context(patch.object(owner, "prove_selected", side_effect=prove))
                stack.enter_context(patch.object(sys, "argv", argv))
                stack.enter_context(patch.dict(os.environ, {"CARGO_TARGET_DIR": str(root / "target")}))
                before_env, before_argv = dict(os.environ), sys.argv
                with redirect_stdout(io.StringIO()):
                    if fail:
                        with self.assertRaisesRegex(RuntimeError, "HOST stop"):
                            owner.main(boundaries=boundaries, tuple_guards=tuple_guards)
                    else:
                        owner.main(boundaries=boundaries, tuple_guards=tuple_guards)
                self.assertEqual(dict(os.environ), before_env)
                self.assertIs(sys.argv, before_argv)
            return calls, arguments, json.loads((output / "receipt.json").read_text())

    def test_historical_default_keeps_three_stages_and_sixteen_cases(self):
        calls, args, receipt = self.exercise(boundaries=False)
        self.assertEqual(calls, ["source", "models", "preflight", "proofs"])
        self.assertNotIn("--scope-receipt", args["models"])
        self.assertEqual(receipt["standalone_proof_cases"], 16)
        self.assertEqual(receipt["separate_default_strict_applications"], 16)

    def test_boundary_full_path_preserves_all_thirty_cases_after_scope(self):
        calls, args, receipt = self.exercise()
        self.assertEqual(calls, ["source", "scope", "models", "preflight", "proofs"])
        self.assertIn("--scope-receipt", args["models"])
        self.assertEqual(receipt["standalone_proof_cases"], 30)
        self.assertEqual(receipt["separate_default_strict_applications"], 30)

    def test_tuple_path_keeps_both_scopes_and_all_cases(self):
        calls, args, receipt = self.exercise(boundaries=False, tuple_guards=True)
        self.assertEqual(calls, ["source", "scope", "tuple-scope", "models", "preflight", "proofs"])
        self.assertIn("--scope-receipt", args["models"])
        self.assertIn("--tuple-scope-receipt", args["models"])
        self.assertEqual(receipt["fresh_keypairs"], 32)
        self.assertEqual(receipt["standalone_proof_cases"], 105)
        self.assertEqual(receipt["separate_default_strict_applications"], 105)

    def test_tuple_key_free_path_never_invokes_proving(self):
        calls, _, receipt = self.exercise(boundaries=False, tuple_guards=True, key_free=True)
        self.assertEqual(calls, ["source", "scope", "tuple-scope", "models", "preflight"])
        self.assertEqual(receipt["status"], "passed-key-free-preflight")
        self.assertFalse(receipt["proof_execution"])
        self.assertFalse(receipt["key_generation"])
        self.assertEqual(receipt["selected_cases"], 105)
        self.assertIn("tuple_scope_receipt", receipt)

    def test_tuple_failure_stops_later_stages(self):
        order = ["source", "scope", "tuple-scope", "models", "preflight", "proofs"]
        for name in order:
            with self.subTest(stage=name):
                calls, _, receipt = self.exercise(boundaries=False, tuple_guards=True, fail=name)
                self.assertEqual(calls, order[:order.index(name) + 1])
                self.assertEqual(receipt["status"], "failed")
                self.assertFalse(receipt["proof_execution"])

    def test_overlapping_modes_refuse_before_any_effect(self):
        with patch.object(owner, "git") as git, patch.object(owner.process_owner, "bounded_command") as child:
            with self.assertRaisesRegex(AssertionError, "one finite cohort"):
                owner.main(boundaries=True, tuple_guards=True)
            with self.assertRaisesRegex(AssertionError, "one finite cohort"):
                owner.prove_selected(Path("HOST never created"), {}, boundaries=True, tuple_guards=True)
        git.assert_not_called()
        child.assert_not_called()

    def test_boundary_key_free_path_never_invokes_proving(self):
        calls, _, receipt = self.exercise(key_free=True)
        self.assertEqual(calls, ["source", "scope", "models", "preflight"])
        self.assertEqual(receipt["status"], "passed-key-free-preflight")
        self.assertFalse(receipt["proof_execution"])
        self.assertFalse(receipt["key_generation"])
        self.assertEqual(receipt["selected_cases"], 30)
        self.assertEqual(receipt["observed_k"], [6])
        self.assertEqual(receipt["verified_material_k"], [6])
        self.assertEqual(set(receipt) & {"source_receipt", "scope_receipt", "model_receipt", "preflight_receipt"},
                         {"source_receipt", "scope_receipt", "model_receipt", "preflight_receipt"})

    def test_each_failure_prevents_all_later_stages(self):
        order = ["source", "scope", "models", "preflight", "proofs"]
        for name in order:
            with self.subTest(stage=name):
                calls, _, receipt = self.exercise(fail=name)
                self.assertEqual(calls, order[:order.index(name) + 1])
                self.assertEqual(receipt["status"], "failed")
                self.assertFalse(receipt["proof_execution"])

    def test_invalid_model_policy_refuses_before_key_directory_or_child(self):
        for k in (0, 9, True, "6"):
            with self.subTest(k=k), tempfile.TemporaryDirectory(prefix="HOST no keys ") as temporary:
                root = Path(temporary)
                (root / "preflight").mkdir()
                (root / "models").mkdir()
                (root / "preflight/receipt.json").write_text(json.dumps(dict(status="passed", sources={str(i): {} for i in range(8)}, selected_cases=30)))
                (root / "models/receipt.json").write_text(json.dumps(dict(status="passed", observed_models=24, observed_k=[k])))
                with patch.object(owner.process_owner, "bounded_command") as child:
                    with self.assertRaises(AssertionError):
                        owner.prove_selected(root, {}, boundaries=True)
                child.assert_not_called()
                self.assertFalse((root / "proofs").exists())

    def test_duplicate_model_membership_refuses_before_keys(self):
        with tempfile.TemporaryDirectory(prefix="HOST whole cohort ") as temporary:
            root = Path(temporary)
            (root / "preflight").mkdir()
            (root / "models").mkdir()
            roles = [str(i) for i in range(8)]
            rows = [dict(source=role, circuit=entry, k=6) for role in roles
                    for entry in ("read", "advance", "advance_other")]
            rows[-1] = rows[0].copy()
            (root / "preflight/receipt.json").write_text(json.dumps(dict(status="passed", sources={r: {} for r in roles}, selected_cases=30)))
            (root / "models/receipt.json").write_text(json.dumps(dict(status="passed", observed_models=24, observed_k=[6], inventory={"circuits": rows})))
            with patch.object(owner.process_owner, "bounded_command") as child:
                with self.assertRaises(AssertionError):
                    owner.prove_selected(root, {}, boundaries=True)
            child.assert_not_called()
            self.assertFalse((root / "proofs").exists())

    def test_tuple_invalid_or_incomplete_models_refuse_before_keys(self):
        roles = [str(i) for i in range(16)]
        valid = [dict(source=role, circuit=entry, k=6) for role in roles
                 for entry in ("read", "advance", "advance_other")]
        mutants = []
        for k in (0, 9, True, "6"):
            rows = [dict(row) for row in valid]
            rows[-1]["k"] = k
            mutants.append(("bad-row-k-" + str(k), rows, [6]))
        mutants += [("duplicate", valid[:-1] + [dict(valid[0])], [6]),
                    ("missing", valid[:-1], [6]),
                    ("extra", valid + [dict(valid[0])], [6]),
                    ("observed-k-set-differs", valid, [7])]
        for name, rows, ks in mutants:
            with self.subTest(mutant=name), tempfile.TemporaryDirectory(prefix="HOST tuple no keys ") as temporary:
                root = Path(temporary)
                (root / "preflight").mkdir()
                (root / "models").mkdir()
                (root / "preflight/receipt.json").write_text(json.dumps(dict(status="passed", sources={r: {} for r in roles}, selected_cases=105)))
                (root / "models/receipt.json").write_text(json.dumps(dict(status="passed", observed_models=48, observed_k=ks, inventory={"circuits": rows})))
                with patch.object(owner.process_owner, "bounded_command") as child:
                    with self.assertRaises(AssertionError):
                        owner.prove_selected(root, {}, tuple_guards=True)
                child.assert_not_called()
                self.assertFalse((root / "proofs").exists())


if __name__ == "__main__":
    unittest.main()
