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
import unittest

from check_constant_vector_projection import compile_failure


class ProjectionApiRefusalTests(unittest.TestCase):
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
