#!/usr/bin/env python3

# This file is part of Compact.
# Copyright (C) 2026 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0

"""Focused pinned-if source and fail-closed Rust refusal controls."""

import json
from pathlib import Path
from types import SimpleNamespace
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_upstream_ledger9_if_keywords as gate  # noqa: E402


class IfKeywordGateControls(unittest.TestCase):
    def test_all_six_pinned_originals(self):
        self.assertEqual(len(gate.original_inventory()), 6)
        self.assertEqual(len(gate.CASES), 2)

    def test_changed_manifest_refused(self):
        record = json.loads(gate.MANIFEST.read_text())
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'manifest.json'
            record['files'].pop(next(iter(record['files'])))
            path.write_text(json.dumps(record))
            with patch.object(gate, 'MANIFEST', path):
                with self.assertRaisesRegex(Exception, 'pinned roster changed'):
                    gate.original_inventory()

    def test_proof_required_capability(self):
        accepted = {'circuits': [{'name': 'if_empty_else_branch',
                                  'pure': False, 'proof': True}]}
        gate.check_capability(accepted, 'empty_else_branch')
        for changed in ({'proof': False}, {'pure': True}, {'name': 'other'}):
            with self.assertRaisesRegex(Exception, 'capability changed'):
                gate.check_capability({'circuits': [dict(accepted['circuits'][0], **changed)]},
                                      'empty_else_branch')

    def test_nested_insert_refusal_requires_empty_product(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / 'product'
            result = SimpleNamespace(returncode=255, stdout='',
                                     stderr=f'line 38 char 17:\n  {gate.RUST_REFUSAL}\n')
            gate.check_rust_refusal(result, destination)
            destination.mkdir()
            with self.assertRaisesRegex(Exception, 'published output'):
                gate.check_rust_refusal(result, destination)
            destination.rmdir()
            result.stderr = 'Rust backend accepted nested Set.insert'
            with self.assertRaisesRegex(Exception, 'diagnostic changed'):
                gate.check_rust_refusal(result, destination)

    def test_generated_symlink_and_mutation_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            artifact = root / 'index.js'
            artifact.write_text('export const value = 1;\n')
            snapshot = gate.generated_files(root)
            artifact.write_text('export const value = 2;\n')
            self.assertNotEqual(gate.generated_files(root), snapshot)
            (root / 'borrowed.js').symlink_to(artifact)
            with self.assertRaisesRegex(Exception, 'generated symlink'):
                gate.generated_files(root)


if __name__ == '__main__':
    unittest.main()
