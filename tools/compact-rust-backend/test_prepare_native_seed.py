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

"""Pilot publication planning controls; never upload or execute proofs."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import prepare_native_seed as owner


class SeedTests(unittest.TestCase):
    def test_event_ref_mode_window_and_restore_authorization(self):
        owner.authorize('workflow_dispatch',owner.TRUSTED_REF,'linux-owner',True,'owner-approved window',False)
        for event,ref,mode,publish,approval,restore in (
            ('pull_request',owner.TRUSTED_REF,'linux-owner',True,'window',False),
            ('push',owner.TRUSTED_REF,'linux-owner',True,'window',False),
            ('workflow_dispatch','refs/heads/other','linux-owner',True,'window',False),
            ('workflow_dispatch',owner.TRUSTED_REF,'apple',True,'window',False),
            ('workflow_dispatch',owner.TRUSTED_REF,'linux-owner',True,'',False),
            ('workflow_dispatch',owner.TRUSTED_REF,'linux-owner',True,'window',True),
            ('workflow_dispatch',owner.TRUSTED_REF,'linux-owner',False,'window',False)):
            with self.subTest(event=event,ref=ref,mode=mode),self.assertRaises(ValueError):
                owner.authorize(event,ref,mode,publish,approval,restore)

    def test_real_gate_status_and_fixed_path_before_official_planning(self):
        for failed in (None,'run','native-script','uncached','uncached-controls','live-postguard','path'):
            with self.subTest(failed=failed), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);plan=root/'native-plan';plan.mkdir();snapshot=root/owner.SNAPSHOT_NAME
                values={'run.json':dict(passed=failed!='run',mode='native-script'),
                    'native-script.command.json':dict(exit_code=1 if failed=='native-script' else 0),
                    'uncached.command.json':dict(exit_code=1 if failed=='uncached' else 0),
                    'uncached-report.json':{},'invocation.json':dict(native=True),'key.json':dict(primary='official',partition='a'*64)}
                for name,value in values.items():(plan/name).write_text(json.dumps(value))
                env=dict(GITHUB_EVENT_NAME='workflow_dispatch',GITHUB_REF=owner.TRUSTED_REF,RUNNER_TEMP=str(root))
                if failed=='path':snapshot=root/'wrong'
                with patch.object(owner.finite,'checked_lap',side_effect=ValueError('negative incomplete') if failed=='uncached-controls' else None), patch.object(owner.forge_ci,'validate_invocation'), patch.object(owner.forge_ci,'check_tools',side_effect=ValueError('live mutation') if failed=='live-postguard' else None), patch.object(owner.forge_ci,'helper',return_value=dict(publish=False,reason='no_generation_misses')) as helper:
                    if failed:
                        with self.assertRaises(ValueError):owner.prepare(plan,snapshot,'approved',env)
                        helper.assert_not_called()
                    else:
                        self.assertEqual(owner.prepare(plan,snapshot,'approved',env)['reason'],'no_generation_misses')
                        self.assertEqual(helper.call_count,1)
                        self.assertIn('prepare-save',helper.call_args.args)
                        self.assertIn(snapshot,helper.call_args.args)
                        self.assertIn('--tests-passed',helper.call_args.args)


    def test_guard_cli_is_read_only_and_rejects_preparation_arguments(self):
        with patch('sys.argv',['prepare_native_seed.py','--guard-plan','/private/plan']), \
                patch.object(owner,'guard') as guard, \
                patch.object(owner,'prepare') as prepare, \
                patch.object(owner.forge_ci,'write') as write, \
                patch.object(owner.forge_ci,'helper') as helper:
            owner.main()
            guard.assert_called_once_with(Path('/private/plan'))
            prepare.assert_not_called()
            write.assert_not_called()
            helper.assert_not_called()
        with patch('sys.argv',['prepare_native_seed.py','--guard-plan','/private/plan','--approval','window']), \
                patch.object(owner,'guard') as guard:
            with self.assertRaises(ValueError):
                owner.main()
            guard.assert_not_called()

    def test_publication_carries_only_exact_official_key_identity_path_and_cap(self):
        for changed in (None,'key','partition','path','budget','id'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);plan=root/'plan';plan.mkdir();snapshot=root/owner.SNAPSHOT_NAME
                for name,value in {'run.json':dict(passed=True,mode='native-script'),
                    'native-script.command.json':dict(exit_code=0),'uncached.command.json':dict(exit_code=0),
                    'uncached-report.json':{},'invocation.json':dict(native=True),
                    'key.json':dict(primary='official-primary',partition='a'*64)}.items():
                    (plan/name).write_text(json.dumps(value))
                value=dict(publish=True,key='official-primary',snapshot=dict(path=str(snapshot),partition='a'*64,snapshot_id='b'*64,bytes=77))
                if changed=='key':value['key']='prefix-match'
                elif changed=='partition':value['snapshot']['partition']='c'*64
                elif changed=='path':value['snapshot']['path']=str(root/'different')
                elif changed=='budget':value['snapshot']['bytes']=owner.LIMIT+1
                elif changed=='id':value['snapshot']['snapshot_id']='not-an-id'
                env=dict(GITHUB_EVENT_NAME='workflow_dispatch',GITHUB_REF=owner.TRUSTED_REF,RUNNER_TEMP=str(root))
                with patch.object(owner.finite,'checked_lap'),patch.object(owner.forge_ci,'validate_invocation'),patch.object(owner.forge_ci,'check_tools'),patch.object(owner.forge_ci,'helper',return_value=value):
                    if changed:
                        with self.assertRaises(ValueError):owner.prepare(plan,snapshot,'approved',env)
                    else:self.assertEqual(owner.prepare(plan,snapshot,'approved',env),value)


if __name__=='__main__':unittest.main()
