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

from check_ledger9_unshielded_linux import (
    require_qualified_plan, signed_checkout_identity,
)
from verify_ledger9_unshielded_tokens_original import require_compiler_source_tree


SIGNED_PR_HEAD = "24f7df322876c839cbceadefebb41cbd4dce7103"
SQUASH_MERGE_HEAD = "785eddfc5ec07036c98fdc988c6138e1c8cc834e"
STALE_COMPILER_HEAD = "cdb4cb64629af69c2b48ea1f3edfae638e7369a6"


class OriginalTokenLinuxSourceTests(unittest.TestCase):
    def test_qualified_plan_matches_source_and_dispatch(self):
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
        require_compiler_source_tree(SIGNED_PR_HEAD)
        with self.assertRaisesRegex(ValueError, "compiler source files differ"):
            require_compiler_source_tree(STALE_COMPILER_HEAD)


if __name__ == "__main__":
    unittest.main()
