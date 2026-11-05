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

"""Refuse either historical or current source mutation; never normalize input bytes."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import historical_source_licenses as owner


class HistoricalSourceLicenseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source = self.root / 'source.compact'
        self.raw = self.root / 'source.compact.raw'
        self.old = b'export circuit value(): Boolean { return true; }\n'
        rows = json.loads((owner.ROOT / owner.MANIFEST).read_text())
        name = next(iter(rows))
        actual_raw = (owner.ROOT / (name + '.raw')).read_bytes()
        notice = (owner.ROOT / name).read_bytes()[len(actual_raw):]
        self.assertEqual(hashlib.sha256(notice).hexdigest(), owner.LICENSE_SUFFIX_SHA256)
        roster = patch.object(owner, "LICENSED_SOURCES", frozenset({"source.compact"}))
        roster.start()
        self.addCleanup(roster.stop)
        self.current = self.old + notice
        self.raw.write_bytes(self.old)
        self.source.write_bytes(self.current)
        self.sha = hashlib.sha256(self.old).hexdigest()
        manifest = self.root / owner.MANIFEST
        manifest.parent.mkdir(parents=True)
        manifest.write_text(json.dumps({'source.compact': {
            'historical_sha256': self.sha,
            'current_sha256': hashlib.sha256(self.current).hexdigest()}}))

    def test_current_and_historical_are_independent_exact_identities(self):
        owner.verify_source(self.source, self.sha, root=self.root)
        self.assertEqual(set(owner.inputs(self.root)),
                         {self.root / owner.MANIFEST, self.source, self.raw})
        for path in (self.source, self.raw):
            original = path.read_bytes()
            path.write_bytes(original + b' ')
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, 'identities differ'):
                owner.verify_source(self.source, self.sha, root=self.root)
            path.write_bytes(original)

    def test_wrong_historical_pin_and_unknown_current_source_refuse(self):
        with self.assertRaisesRegex(ValueError, 'identities differ'):
            owner.verify_source(self.source, '0' * 64, root=self.root)
        other = self.root / 'other.compact'
        other.write_bytes(self.current)
        with self.assertRaisesRegex(ValueError, 'identities differ'):
            owner.verify_source(other, self.sha, root=self.root)

    def test_current_or_historical_symlink_refuses(self):
        for path in (self.source, self.raw):
            original = path.read_bytes()
            target = path.with_name(path.name + '.target')
            target.write_bytes(original)
            path.unlink()
            path.symlink_to(target)
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, 'symlink|identities differ'):
                owner.verify_source(self.source, self.sha, root=self.root)
            path.unlink()
            path.write_bytes(original)

    def test_unchanged_legacy_source_keeps_its_literal_binding(self):
        legacy = self.root / 'legacy.compact'
        legacy.write_bytes(self.old)
        owner.verify_source(legacy, self.sha, root=self.root)
        self.source.write_bytes(self.old)
        with self.assertRaisesRegex(ValueError, 'identities differ'):
            owner.verify_source(self.source, self.sha, root=self.root)

    def test_changed_body_and_repinned_current_manifest_still_refuse(self):
        changed = self.current.replace(b'true', b'eurt')
        self.assertNotEqual(changed, self.current)
        self.assertEqual(len(changed), len(self.current))
        self.source.write_bytes(changed)
        self.repin_current(changed)
        with self.assertRaisesRegex(ValueError, 'identities differ'):
            owner.verify_source(self.source, self.sha, root=self.root)

    def test_changed_notice_and_repinned_current_manifest_still_refuse(self):
        changed = self.current + b'// another comment\n'
        self.source.write_bytes(changed)
        self.repin_current(changed)
        with self.assertRaisesRegex(ValueError, 'identities differ'):
            owner.verify_source(self.source, self.sha, root=self.root)

    def repin_current(self, changed):
        manifest = self.root / owner.MANIFEST
        rows = json.loads(manifest.read_text())
        rows['source.compact']['current_sha256'] = hashlib.sha256(changed).hexdigest()
        manifest.write_text(json.dumps(rows))

    def test_removed_manifest_row_and_reverted_current_source_refuse(self):
        (self.root / owner.MANIFEST).write_text('{}')
        self.source.write_bytes(self.old)
        with self.assertRaisesRegex(ValueError, 'membership differs'):
            owner.verify_source(self.source, self.sha, root=self.root)
        with self.assertRaisesRegex(ValueError, 'membership differs'):
            owner.inputs(self.root)

    def test_extra_manifest_member_refuses_before_input_admission(self):
        manifest = self.root / owner.MANIFEST
        rows = json.loads(manifest.read_text())
        rows['unexpected.compact'] = rows['source.compact']
        manifest.write_text(json.dumps(rows))
        with self.assertRaisesRegex(ValueError, 'membership differs'):
            owner.inputs(self.root)
