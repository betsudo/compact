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

"""Exact historical and active oracle bindings for the Ledger 9 preflight."""

import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_oracle_acceptance as gate


class Ledger9OracleBindingsTests(unittest.TestCase):
    def test_active_bindings_keep_historical_authority(self):
        bindings = gate.ledger9_bindings()
        self.assertTrue(bindings["captures"])
        for path, row in bindings["tests"].items():
            self.assertTrue(gate.reviewed_test_matches(path, row["historical_sha256"], bindings))
            self.assertFalse(gate.reviewed_test_matches(path, row["current_sha256"], bindings))

    def test_changed_rust_or_capture_bytes_refuse(self):
        bindings = gate.ledger9_bindings()
        legacy, capture = next(iter(bindings["captures"].items()))
        original = gate.relative_file
        for name in (next(iter(bindings["tests"])), legacy, capture["current"]):
            with self.subTest(path=name), tempfile.TemporaryDirectory() as tmp:
                changed = Path(tmp) / "changed"
                changed.write_bytes(original(name).read_bytes() + b" ")
                with patch.object(gate, "relative_file",
                                  side_effect=lambda path: changed if path == name else original(path)):
                    with self.assertRaisesRegex(ValueError, "changed"):
                        gate.ledger9_bindings()

    def test_unreviewed_test_identity_refuses(self):
        bindings = gate.ledger9_bindings()
        next(iter(bindings["tests"].values()))["historical_sha256"] = "0" * 64
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "bindings.json"
            path.write_text(json.dumps(bindings))
            with patch.object(gate, "LEDGER9_BINDINGS", path):
                with self.assertRaisesRegex(ValueError, "unreviewed Ledger 9 test binding"):
                    gate.ledger9_bindings()
