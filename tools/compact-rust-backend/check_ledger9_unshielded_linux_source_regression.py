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

"""Source provenance checks for the manual original-token Linux gate."""

import unittest
from unittest.mock import patch

import check_ledger9_unshielded_linux as gate

from check_ledger9_unshielded_linux import (
    require_qualified_plan, signed_checkout_identity,
)
from verify_ledger9_unshielded_tokens_original import require_compiler_source_tree


SIGNED_PR_HEAD = "24f7df322876c839cbceadefebb41cbd4dce7103"
SIGNED_INTEGRATED_HEAD = "ec76228a9853a35ebbdcad39fe0487f897db1bfe"
PRE_FOUNDATION_HEAD = "06d27825249d3122e9ba6d597204161acda77e0b"
SQUASH_MERGE_HEAD = "785eddfc5ec07036c98fdc988c6138e1c8cc834e"
STALE_COMPILER_HEAD = "cdb4cb64629af69c2b48ea1f3edfae638e7369a6"


class OriginalTokenLinuxSourceTests(unittest.TestCase):
    def test_qualified_plan_matches_source_and_dispatch(self):
        require_qualified_plan()
        self.assertEqual((len(gate.QUALIFIED_CIRCUITS), len(gate.PROOF_STEMS),
                          len(gate.REPLAY_STEMS), len(gate.SHIELDED_BATCHES),
                          len(gate.REFUSAL_REASONS)), (20, 30, 22, 9, 15))

    def test_incomplete_immediate_selectors_refuse_preflight(self):
        for name, replacement in (
            ("QUALIFIED_CIRCUITS", gate.QUALIFIED_CIRCUITS[:-1]),
            ("PROOF_STEMS", gate.PROOF_STEMS[:-1]),
            ("SHIELDED_BATCHES", gate.SHIELDED_BATCHES[:-1]),
            ("REFUSAL_REASONS", dict(list(gate.REFUSAL_REASONS.items())[:-1])),
        ):
            with self.subTest(name=name), patch.object(gate, name, replacement):
                with self.assertRaisesRegex(ValueError,
                                            "original 20-circuit proof/replay/refusal inventory differs"):
                    require_qualified_plan()

    def test_immediate_marker_must_match_source(self):
        batches = list(gate.SHIELDED_BATCHES)
        name, flag, markers = batches[-1]
        batches[-1] = (name, flag, (markers[0] + " changed", markers[1]))
        with patch.object(gate, "SHIELDED_BATCHES", tuple(batches)):
            with self.assertRaisesRegex(ValueError,
                                        "immediate original source/selector differs"):
                require_qualified_plan()

    def test_squash_merge_requires_signed_equal_tree_provenance(self):
        self.assertEqual(signed_checkout_identity(SIGNED_PR_HEAD), SIGNED_PR_HEAD)
        self.assertEqual(signed_checkout_identity(SQUASH_MERGE_HEAD, SIGNED_PR_HEAD),
                         SIGNED_PR_HEAD)
        with self.assertRaisesRegex(ValueError, "signed DCO source required"):
            signed_checkout_identity(SQUASH_MERGE_HEAD)
        with self.assertRaisesRegex(ValueError, "signed checkout tree differs"):
            signed_checkout_identity(SQUASH_MERGE_HEAD, STALE_COMPILER_HEAD)

    def test_current_compiler_tree_matches_signed_pin_and_refuses_stale_pin(self):
        self.assertEqual(gate.SOURCE_HEAD, SIGNED_INTEGRATED_HEAD)
        require_compiler_source_tree(SIGNED_INTEGRATED_HEAD)
        with self.assertRaisesRegex(ValueError, "compiler source files differ"):
            require_compiler_source_tree(PRE_FOUNDATION_HEAD)
        with self.assertRaisesRegex(ValueError, "compiler source files differ"):
            require_compiler_source_tree(SIGNED_PR_HEAD)
        with self.assertRaisesRegex(ValueError, "compiler source files differ"):
            require_compiler_source_tree(STALE_COMPILER_HEAD)


if __name__ == "__main__":
    unittest.main()
