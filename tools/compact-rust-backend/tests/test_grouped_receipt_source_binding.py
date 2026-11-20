#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Current aggregate producers may change only with a freshly pinned receipt."""

import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
import unittest


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from check_upstream_ledger9_bytes_map_funded import (  # noqa: E402
    require_receipt_pin, sha, source_bound_to_receipt,
)


class GroupedReceiptSourceBindingTest(unittest.TestCase):
    def test_updated_aggregate_producer_requires_updated_receipt_and_pin(self):
        with TemporaryDirectory() as temp:
            source = Path(temp) / 'producer.py'
            receipt = Path(temp) / 'receipt.json'
            source.write_text('old producer\n')
            old_digest = sha(source)
            receipt.write_text(json.dumps({'gate_source_sha256': old_digest}))
            old_receipt_pin = sha(receipt)

            source.write_text('lineage-repaired producer\n')
            self.assertFalse(source_bound_to_receipt(old_digest, source))
            receipt.write_text(json.dumps({'gate_source_sha256': sha(source)}))
            with self.assertRaisesRegex(RuntimeError, 'independently supplied'):
                require_receipt_pin(receipt, old_receipt_pin, 'proof')

            require_receipt_pin(receipt, sha(receipt), 'proof')
            current_digest = json.loads(receipt.read_text())['gate_source_sha256']
            self.assertTrue(source_bound_to_receipt(current_digest, source))
            self.assertFalse(source_bound_to_receipt(current_digest, source, old_digest))

    def test_post_receipt_source_change_is_refused(self):
        with TemporaryDirectory() as temp:
            source = Path(temp) / 'producer.py'
            source.write_text('current producer\n')
            receipt_digest = sha(source)
            source.write_text('changed after receipt\n')
            self.assertFalse(source_bound_to_receipt(receipt_digest, source))


if __name__ == '__main__':
    unittest.main()
