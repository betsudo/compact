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

"""Finite fixture and early-refusal controls; no generated execution authority."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_constructor_equality as owner


class BoundaryControls(unittest.TestCase):
    def test_literal_cases_distinguish_both_operators_and_aggregate_components(self):
        fixtures = owner.FIXTURES.with_name("constructor_equality_boundaries")
        rows = json.loads((fixtures / "cases.json").read_text())
        self.assertEqual(len(rows), 30)
        self.assertEqual({row["role"] for row in rows}, set(owner.BOUNDARY_ROLES))
        self.assertEqual(len({(r["role"], r["case"]) for r in rows}), 30)
        for row in rows:
            values, right = row["args"]
            self.assertEqual(len(values), 2)
            matches = values[-1] == right
            condition = matches if row["role"].endswith("_eq") else not matches
            self.assertEqual(row["expected"], 7 if condition else 11)
        for role in owner.BOUNDARY_ROLES:
            selected = {r["case"]: r for r in rows if r["role"] == role}
            self.assertEqual({r["expected"] for r in selected.values()}, {7, 11})
            if role.startswith("mixed_uint_"):
                self.assertEqual({r["args"][1] for r in selected.values()}, {0, 2, 255, 256, 65535})
                for row in selected.values():
                    self.assertTrue(all(0 <= n <= 255 for n in row["args"][0]))
                # These exact final elements expose truncation to the source width.
                self.assertEqual(selected["no_narrow_256"]["args"][0][-1], 0)
                self.assertEqual(selected["no_narrow_65535"]["args"][0][-1], 255)
            else:
                self.assertEqual(set(selected), {"last_match", "first_component_differs", "second_component_differs"})
                for label in ("first_component_differs", "second_component_differs"):
                    values, right = selected[label]["args"]
                    self.assertEqual(values[0], right)
                    left = values[1]
                    a, b = ("value", "tag") if role.startswith("struct_") else (0, 1)
                    self.assertEqual(left[a] == right[a], label == "second_component_differs")
                    self.assertEqual(left[b] == right[b], label == "first_component_differs")

    def test_all_misuses_have_a_marked_call_and_unmarked_valid_twin(self):
        fixtures = owner.FIXTURES.with_name("constructor_equality_boundaries")
        self.assertEqual(len(owner.BOUNDARY_NEGATIVES), 5)
        for name in owner.BOUNDARY_NEGATIVES:
            bad = (fixtures / (name + ".rs")).read_text()
            good = (fixtures / (name + "_twin.rs")).read_text()
            self.assertEqual(bad.count("// expected-error"), 1)
            self.assertNotIn("// expected-error", good)
            self.assertIn("::ledger_contract::initial_state(", bad)
            self.assertIn("::ledger_contract::initial_state(", good)
            self.assertNotEqual(bad.replace(" // expected-error", ""), good)

    def test_dirty_or_stale_recipe_refuses_before_output_or_compiler_effects(self):
        for boundaries in (False, True):
            for dirty in (False, True):
                with self.subTest(boundaries=boundaries, dirty=dirty), tempfile.TemporaryDirectory() as temporary:
                    root = Path(temporary)
                    receipt = root / "recipe.json"
                    receipt.write_text(json.dumps(dict(status="passed", head="stale", tree="tree")))
                    out = root / "output"
                    argv = ["gate", "--compiler-receipt", str(receipt), "--ts-runtime", str(root), "--output", str(out)]
                    def git(command, **kwargs):
                        return {("status", "--porcelain"): "dirty" if dirty else "", ("rev-parse", "HEAD"): "head", ("rev-parse", "HEAD^{tree}"): "tree"}[tuple(command[1:])]
                    with patch.object(owner, "ROOT", root), patch.object(sys, "argv", argv), patch.object(owner.subprocess, "check_output", side_effect=git), patch.object(owner.subprocess, "run") as effects:
                        with self.assertRaisesRegex(ValueError, "clean source|current clean original"):
                            owner.main(boundaries=boundaries)
                    effects.assert_not_called()
                    self.assertFalse(out.exists())


if __name__ == "__main__":
    unittest.main()
