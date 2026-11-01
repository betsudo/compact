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

"""Require static refusal at the actual external-consumer misuse."""
import copy
import json
import subprocess
import shutil
from pathlib import Path
import tempfile
import unittest

from check_constant_vector_projection import compile_failure


class ProjectionApiRefusalTests(unittest.TestCase):
    def test_numeric_capture_rejects_wrong_success_and_preserves_positive_twin(self):
        node = shutil.which("node")
        self.assertIsNotNone(node, "Node is required for the maintained JS capture controls")
        script = Path(__file__).with_name("tests") / "vector_projection/capture_numeric_aliases.mjs"
        positive = (
            "function failure(message){throw Object.assign(Error('failed assert: '+message),"
            "{name:'CompactError'});}\n"
            "export const pureCircuits={zero(){return 0n;},"
            "checked(x){if(x!==0n)throw Error('checked cast');return 0n;},"
            "ordered(x){throw Error(x===0n?'second failure':'checked cast');},"
            "index_guard_left(){failure('result of subtraction would be negative');},"
            "index_guard_right(){failure('result of subtraction would be negative');},"
            "index_guard_wide(){failure('result of subtraction would be negative');},"
            "index_guard_success(items){return items[0];},"
            "index_total_wide(items){return items[0];},"
            "index_guard_wide_success(items){return items[0];},"
            "index_order(items,fail){failure(fail?'Vector operand before index':"
            "'result of subtraction would be negative');}};"
        )
        controls = {
            "positive": positive,
            "wrong_zero": positive.replace("zero(){return 0n;}", "zero(){return 1n;}"),
            "cast_did_not_fail": positive.replace(
                "checked(x){if(x!==0n)throw Error('checked cast');return 0n;}",
                "checked(){return 0n;}"),
            "later_did_not_fail": positive.replace(
                "ordered(x){throw Error(x===0n?'second failure':'checked cast');}",
                "ordered(){return 0n;}"),
            "index_guard_erased": positive.replace(
                "index_guard_left(){failure('result of subtraction would be negative');}",
                "index_guard_left(items){return items[0];}"),
            "index_operand_order_reversed": positive.replace(
                "fail?'Vector operand before index':'result of subtraction would be negative'",
                "'result of subtraction would be negative'"),
        }
        with tempfile.TemporaryDirectory() as folder:
            for name, source in controls.items():
                with self.subTest(name=name):
                    module = Path(folder) / (name + ".mjs")
                    output = Path(folder) / (name + ".json")
                    module.write_text(source)
                    process = subprocess.run([node, str(script), str(module), str(output)],
                                             capture_output=True, text=True, timeout=30)
                    if name == "positive":
                        self.assertEqual(process.returncode, 0, process.stderr)
                        self.assertEqual(len(json.loads(output.read_text())["zero"]), 4)
                    else:
                        self.assertNotEqual(process.returncode, 0)
                        self.assertIn("AssertionError", process.stderr)
                        self.assertFalse(output.exists(), "wrong behavior must not publish a capture")

    def test_only_primary_error_at_the_marked_expression_counts(self):
        source = "fn main() {\n    misuse(); // expected-error\n}\n"
        span = {"file_name": "src/bin/double_context.rs", "is_primary": True,
                "line_start": 2, "line_end": 2, "text": [{"text": source.splitlines()[1]}]}
        message = {"reason": "compiler-message", "message": {
            "level": "error", "code": {"code": "E0382"}, "spans": [span]}}
        def process(row, code=101):
            return subprocess.CompletedProcess([], code, json.dumps(row), "")
        self.assertEqual(len(compile_failure(process(message), "double_context", "E0382", source)), 1)
        mutations = [
            ("file_name", "dependency.rs"), ("line_start", 3), ("line_end", 1),
            ("is_primary", False), ("text", [{"text": "different source"}]),
        ]
        for field, value in mutations:
            row = copy.deepcopy(message)
            row["message"]["spans"][0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                compile_failure(process(row), "double_context", "E0382", source)
        row = copy.deepcopy(message)
        row["message"]["code"]["code"] = "E0308"
        with self.assertRaises(ValueError):
            compile_failure(process(row), "double_context", "E0382", source)
        with self.assertRaises(ValueError):
            compile_failure(process(message, 0), "double_context", "E0382", source)
        with self.assertRaises(ValueError):
            compile_failure(subprocess.CompletedProcess([], 101, "", "missing dependency"),
                            "double_context", "E0382", source)
        for invalid in [source.replace("// expected-error", ""), source + "// expected-error\n"]:
            with self.assertRaises(ValueError):
                compile_failure(process(message), "double_context", "E0382", invalid)


if __name__ == "__main__":
    unittest.main()
