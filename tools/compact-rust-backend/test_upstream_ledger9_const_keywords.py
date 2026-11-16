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

"""Negative controls for the const-keyword source admission gate."""

import unittest

from check_upstream_ledger9_const_keywords import valid_source_signature


class ConstKeywordSourceAdmission(unittest.TestCase):
    def test_final_author_matching_dco_and_assistance_admitted(self):
        body = ('Port const originals\n\nPreserve pinned assertions.\n\n'
                'Signed-off-by: Author <author@example.org>\n\nAssisted-by: AI\n')
        self.assertTrue(valid_source_signature('G', 'Author <author@example.org>', body))

    def test_mismatched_dco_and_prose_spoof_refused(self):
        body = ('Port const originals\n\nPreserve pinned assertions.\n\n'
                'Signed-off-by: Other <other@example.org>\n\nAssisted-by: AI\n')
        self.assertFalse(valid_source_signature('G', 'Author <author@example.org>', body))
        self.assertFalse(valid_source_signature('G', 'Other <other@example.org>',
                                                'Signed-off-by: Other <other@example.org>\n'
                                                'Assisted-by: AI\n'))
        self.assertFalse(valid_source_signature('N', 'Other <other@example.org>', body))


if __name__ == '__main__':
    unittest.main()
