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

"""Pure accounting controls; actual native script/proofs remain integration gates."""
import copy
import json
import unittest
from pathlib import Path

import qualify_linux_native as native


class AccountingTests(unittest.TestCase):
    def setUp(self):
        self.config = native.configuration(native.read(Path(__file__).with_name("forge_native_qualification.json")))
        self.rows = [self.row(5, "auto-bypass") for _ in range(289)] + [self.row(17, "forge-miss")]

    def row(self, k, route):
        return dict(k=k, rows=10, source_sha256="a" * 64,
                    parameter_digest=self.config["parameters"][str(k)], acquisition_path=route,
                    artifacts=[dict(kind=kind, digest="b" * 64, size=12)
                               for kind in ("binary_ir", "prover", "verifier")])

    def test_cold_restored_same_actual_identities(self):
        restored = copy.deepcopy(self.rows)
        restored[-1]["acquisition_path"] = "forge-hit"
        self.assertEqual(native.acquisitions(self.rows, self.config, "forge-miss"),
                         native.acquisitions(restored, self.config, "forge-hit"))

    def test_changed_artifact_changes_multiset(self):
        changed = copy.deepcopy(self.rows)
        changed[0]["artifacts"][0]["digest"] = "c" * 64
        self.assertNotEqual(native.acquisitions(self.rows, self.config, "forge-miss"),
                            native.acquisitions(changed, self.config, "forge-miss"))

    def test_missing_case_refuses(self):
        with self.assertRaisesRegex(ValueError, "count"):
            native.acquisitions(self.rows[:-1], self.config, "forge-miss")

    def test_parameter_mutation_refuses(self):
        self.rows[0]["parameter_digest"] = "c" * 64
        with self.assertRaisesRegex(ValueError, "parameter"):
            native.acquisitions(self.rows, self.config, "forge-miss")

    def test_route_and_admitted_cohort_refuse(self):
        self.rows[-1]["acquisition_path"] = "auto-bypass"
        with self.assertRaisesRegex(ValueError, "route"):
            native.acquisitions(self.rows, self.config, "forge-miss")

    def test_duplicate_artifact_refuses(self):
        self.rows[0]["artifacts"][0]["kind"] = "prover"
        with self.assertRaisesRegex(ValueError, "artifact"):
            native.acquisitions(self.rows, self.config, "forge-miss")

    def test_configuration_unknown_field_refuses(self):
        with self.assertRaisesRegex(ValueError, "configuration"):
            native.configuration({**self.config, "extra": 1})

    def printed(self, rows):
        return dict(stdout="noise\n" + "\n".join("COMPACT_FORGE " + json.dumps(row) for row in rows), stderr="")

    def test_printed_records_are_only_canonical_marker_lines(self):
        command = self.printed(self.rows)
        command["stderr"] = "unrelated diagnostic\n"
        self.assertEqual(native.printed_acquisitions(command), self.rows)

    def test_missing_and_duplicate_printed_record_accounting(self):
        for rows in (self.rows[:-1], self.rows + [self.rows[0]]):
            with self.assertRaisesRegex(ValueError, "count"):
                native.acquisitions(native.printed_acquisitions(self.printed(rows)), self.config, "forge-miss")

    def test_malformed_or_ambiguous_printed_json_refuses(self):
        for text in ('{', '{"k": 17, "k": 16}', '{"rows": NaN}', '[]'):
            with self.assertRaises((ValueError, json.JSONDecodeError)):
                native.printed_acquisitions(dict(stdout="COMPACT_FORGE " + text, stderr=""))

    def test_printed_cohorts_require_names_and_no_failures(self):
        text = "test domain::actual ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n"
        self.assertEqual(native.rust_cohorts(text)["passed"], 1)
        with self.assertRaises(ValueError):
            native.rust_cohorts(text.replace("0 failed", "1 failed"))
        with self.assertRaises(ValueError):
            native.rust_cohorts(text.split("\n", 1)[1])


if __name__ == "__main__":
    unittest.main()
