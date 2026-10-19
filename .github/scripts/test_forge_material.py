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
"""Pure material trust controls; never import a Nix store or access GitHub."""
import argparse
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import forge_material as owner


class MaterialTests(unittest.TestCase):
    def test_service_identity_and_digest_precede_extraction(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive=Path(tmp)/'artifact.zip';archive.write_bytes(b'actual downloaded bytes')
            args=argparse.Namespace(run_id='12',run_attempt='2',producer_attempt='1',artifact_id='34',artifact_zip=archive)
            value=dict(id=34,expired=False,workflow_run=dict(id=12),name='forge-nix-linux-12-1',digest='sha256:'+owner.closure.sha(archive))
            owner.authenticate(value,args)
            for key,new in [('id',35),('expired',True),('workflow_run',dict(id=13)),('name','forge-nix-linux-12-2'),('digest','sha256:'+'0'*64)]:
                with self.subTest(key=key),self.assertRaises(ValueError):owner.authenticate({**value,key:new},args)
            args.producer_attempt='3'
            with self.assertRaises(ValueError):owner.authenticate(value,args)

    def test_file_inventory_mutation_refuses(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);args=argparse.Namespace(directory=root,repository='betsudo/compact',run_id='12',producer_attempt='1')
            for n in owner.FILES:(root/n).write_bytes(b'current')
            manifest=dict(schema=1,payload_source=owner.SOURCE,producer_source=owner.PRODUCER,system='x86_64-linux',producer_repository='betsudo/forge',repository=args.repository,run_id='12',run_attempt='1',max_bytes=owner.LIMIT,files={n:dict(bytes=7,sha256=owner.closure.sha(root/n)) for n in owner.FILES})
            (root/'closure.nar').write_bytes(b'mutated')
            with self.assertRaisesRegex(ValueError,'file changed'):owner.validate(manifest,args)
            with self.assertRaisesRegex(ValueError,'provenance'):owner.validate({**manifest,'producer_source':'0'*40},args)

    def test_complete_inventory_and_root_hashes_are_required(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);args=argparse.Namespace(directory=root,repository='betsudo/compact',run_id='12',producer_attempt='1')
            paths=['/nix/store/'+c*32+'-'+name for c,name in [('a','forge'),('b','rust'),('c','source'),('d','build-source')]]
            import base64
            inventory={p:dict(narHash='sha256-'+base64.b64encode(bytes(32)).decode(),narSize=0) for p in paths}
            for n in owner.FILES:(root/n).write_text('{}')
            (root/'store-paths.json').write_text(json.dumps(inventory))
            manifest=dict(schema=1,payload_source=owner.SOURCE,producer_source=owner.PRODUCER,system='x86_64-linux',producer_repository='betsudo/forge',repository=args.repository,run_id='12',run_attempt='1',max_bytes=owner.LIMIT,store_paths=4,nar_bytes=0,root=paths[0],rust_root=paths[1],source_root=paths[2],build_source_root=paths[3],files={n:dict(bytes=(root/n).stat().st_size,sha256=owner.closure.sha(root/n)) for n in owner.FILES})
            self.assertEqual(owner.validate(manifest,args),inventory)
            for key,value in [('store_paths',3),('nar_bytes',1),('root','/nix/store/'+'e'*32+'-absent')]:
                with self.subTest(key=key),self.assertRaises(ValueError):owner.validate({**manifest,key:value},args)

    def test_owner_route_has_no_forge_rebuild_or_key_publication(self):
        source=Path(owner.__file__).read_text()
        self.assertNotIn("['nix','build'",source)
        workflow=(owner.closure.ROOT/'.github/workflows/forge-local-ci.yml').read_text()
        self.assertIn('needs: [standalone-linux-compiler, forge-linux-material]',workflow)
        self.assertIn("if: inputs.qualification != 'linux-owner'",workflow)
        self.assertIn('producer-source: '+owner.PRODUCER,workflow)
        self.assertIn('compare_forge_contract_loop.py',workflow)


if __name__=='__main__':unittest.main()
