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

"""Complete proof IR pairing, including changes outside the selected call."""
import json
from pathlib import Path
import tempfile
import unittest

from check_compactc_target import check_complete_ir_binding


class CompleteProofSourceBindingTests(unittest.TestCase):
    def test_current_source_is_accepted_and_any_original_ir_change_is_refused(self):
        source = Path(__file__).with_name('tests') / 'mixed-width-operand-schema20-ir.json'
        original = json.loads(source.read_text())
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            (output / 'contract').mkdir()
            actual = output / 'contract/compact-rust-ir.json'
            actual.write_bytes(source.read_bytes())
            check_complete_ir_binding(output, source)
            # Changes outside the selected proof circuit must also refuse.
            # There is no cast stripping, subset or origin normalization.
            for key in ['schema_version', 'constructor', 'circuits', 'stateful_circuits']:
                changed = dict(original)
                changed[key] = None
                actual.write_text(json.dumps(changed))
                with self.assertRaisesRegex(ValueError, 'complete proof source IR differs'):
                    check_complete_ir_binding(output, source)
            actual.write_bytes(source.read_bytes())
            check_complete_ir_binding(output, source)
            self.assertFalse((output / 'keys').exists())


if __name__ == '__main__':
    unittest.main()
