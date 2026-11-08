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

"""Authenticate complete historical raw/source fixtures, never current output."""
import hashlib
import json
from pathlib import Path
import unittest


class HistoricalTupleFixtures(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
