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

import copy
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
import did_primitive_reducer_gate as gate


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.row={'kind':'alias-set','operations':[{'operation':'mutate'}],'proof_cases':[{'case':'insert','operation':'mutate'},{'case':'remove','operation':'mutate'}]}
        self.good={'format':'compact-did-primitive-reducer-proof/v1','kind':'alias-set',
                   'status':'passed','strictness':'default','constructor_data_deployed':True,
                   'constructor_execution_proved':False,'calls':[]}
        for case in self.row['proof_cases']:
            self.good['calls'].append({**case,'proof_bytes':2912,
                                      'state_bytes':100,'changed_binding_rejected':True,
                                      'applied':True,'recorded_state_matches':True,
                                      'replay_unchanged':True,'replay_refusal':'IntentAlreadyExists'})

    def test_complete_rows_pass(self):
        gate.validate_summary(self.good,self.row)

    def test_current_summary_requires_v3_funding_separate_verification_and_corruption_refusal(self):
        with self.assertRaises(gate.common.GateError):
            gate.validate_summary(self.good, self.row, require_v3=True)
        current = copy.deepcopy(self.good)
        current["format"] = "compact-did-primitive-reducer-proof/v2"
        for row in current["calls"]:
            row.update(transaction_proof_bytes=10, transaction_proof_version=3,
                       funded_dust_spends=1, separate_verification=True, corrupt_proof_rejected=True)
        gate.validate_summary(current, self.row, require_v3=True)
        for field, bad in (("transaction_proof_version", 2), ("funded_dust_spends", True),
                           ("separate_verification", False), ("corrupt_proof_rejected", False)):
            changed = copy.deepcopy(current)
            changed["calls"][0][field] = bad
            with self.subTest(field=field), self.assertRaises(gate.common.GateError):
                gate.validate_summary(changed, self.row, require_v3=True)

    def test_historical_capture_requires_exact_ledger9_bytes_and_frozen_source(self):
        fixture = "tests-rust-backend/did-point-digest-reducer"
        oracle = gate.ROOT / fixture / "oracle"
        with tempfile.TemporaryDirectory() as temp:
            self.assertEqual(set(gate.validate_historical_capture(fixture)),
                             {str(oracle / name) for name in
                              ("ledger9-binding.json", "ledger9-cases.json", "cases.json",
                               "reviewed-rust-binding.json")} |
                             {str(source) for source, _ in gate.FROZEN_LEDGER9_RUNTIME.values()})
            frozen = gate.FROZEN_LEDGER9_RUNTIME['runtime/src/built-ins.ts']
            changed_source = Path(temp) / 'changed-built-ins.ts'
            changed_source.write_text('not the reviewed source')
            with patch.dict(gate.FROZEN_LEDGER9_RUNTIME,
                            {'runtime/src/built-ins.ts': (changed_source, frozen[1])}):
                with self.assertRaisesRegex(gate.common.GateError, 'reducer capture source changed'):
                    gate.validate_historical_capture(fixture)
            with patch.object(gate.common, 'sha256', side_effect=lambda path:
                              'changed' if Path(path) == oracle / 'ledger9-cases.json' else
                              __import__('hashlib').sha256(Path(path).read_bytes()).hexdigest()):
                with self.assertRaisesRegex(gate.common.GateError, "exact reviewed Ledger 9 bytes"):
                    gate.validate_historical_capture(fixture)

    def test_current_capture_records_preserve_all_nine_historical_case_sets(self):
        count = 0
        for row in gate.reviewed_rows():
            oracle = gate.ROOT / row['fixture'] / 'oracle'
            binding = gate.read_json(oracle / 'ledger9-runtime0.18-binding.json')
            capture = gate.read_json(oracle / 'ledger9-runtime0.18-cases.json')
            historical = gate.read_json(oracle / 'ledger9-cases.json')
            with self.subTest(kind=row['kind']):
                expected = dict(binding, source_sha256=gate.common.sha256(gate.ROOT / row['source']),
                    capture_script_sha256=gate.common.sha256(gate.CURRENT_CAPTURE_SCRIPT),
                    runtime_source_sha256=gate.runtime_source_hash(),
                    historical_ledger9_binding_sha256=gate.common.sha256(oracle / 'ledger9-binding.json'),
                    capture_sha256=gate.common.sha256(oracle / 'ledger9-runtime0.18-cases.json'))
                gate.validate_current_capture_record(binding, expected, capture, historical)
                self.assertEqual(binding['kind'], row['kind'])
                self.assertEqual(binding['compact_runtime'], '0.18.0')
                self.assertEqual(binding['source_sha256'],
                                 gate.common.sha256(gate.ROOT / row['source']))
                self.assertEqual(binding['capture_script_sha256'],
                                 gate.common.sha256(gate.CURRENT_CAPTURE_SCRIPT))
                self.assertEqual(binding['runtime_source_sha256'], gate.runtime_source_hash())
                self.assertEqual(binding['historical_ledger9_binding_sha256'],
                                 gate.common.sha256(oracle / 'ledger9-binding.json'))
                self.assertEqual(binding['capture_sha256'],
                                 gate.common.sha256(oracle / 'ledger9-runtime0.18-cases.json'))
                for provenance in capture['provenance']:
                    relative = gate.reviewed.relative(provenance['path'])
                    self.assertEqual(provenance['sha256'], gate.common.sha256(gate.ROOT / relative),
                                     f"{row['kind']}: {relative}")
                changed = binding.copy()
                changed['generated_javascript_sha256'] = 'changed'
                with self.assertRaisesRegex(gate.common.GateError, 'binding differs'):
                    gate.validate_current_capture_record(changed, expected, capture, historical)
                changed = copy.deepcopy(capture)
                changed['cases'][0]['result'] = 'changed'
                with self.assertRaisesRegex(gate.common.GateError, 'retained behavior'):
                    gate.validate_current_capture_record(binding, expected, changed, historical)
            count += len(capture['cases'])
        self.assertEqual(count, 55)

    def test_runtime_closure_hash_covers_dist_lock_and_nested_dependencies_without_store_prefix(self):
        def make_runtime(root):
            runtime = root / 'runtime'
            (runtime / 'dist').mkdir(parents=True)
            (runtime / 'dist/index.js').write_text('export const v = 1;')
            (runtime / 'package.json').write_text(json.dumps({'name':'runtime','dependencies':{'outer':'1'}}))
            (runtime / 'package-lock.json').write_text('{}')
            outer = runtime / 'node_modules/outer'
            outer.mkdir(parents=True)
            (outer / 'package.json').write_text(json.dumps({'name':'outer','dependencies':{'nested':'1'}}))
            (outer / 'index.js').write_text('outer')
            nested = outer / 'node_modules/nested'
            nested.mkdir(parents=True)
            (nested / 'package.json').write_text(json.dumps({'name':'nested'}))
            (nested / 'index.js').write_text('nested')
            return runtime, nested
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            runtime, nested = make_runtime(root / 'first')
            other, _ = make_runtime(root / 'second')
            baseline = gate.runtime_closure_hash(runtime)
            self.assertEqual(baseline, gate.runtime_closure_hash(other))
            for path in (runtime / 'dist/index.js', runtime / 'package-lock.json',
                         nested / 'index.js'):
                with self.subTest(path=path):
                    original = path.read_text()
                    path.write_text(original + 'changed')
                    self.assertNotEqual(baseline, gate.runtime_closure_hash(runtime))
                    path.write_text(original)
            (nested / 'package.json').unlink()
            with self.assertRaisesRegex(gate.common.GateError, 'dependency missing'):
                gate.runtime_closure_hash(runtime)

    def test_current_runtime_path_requires_matching_source_and_nix_package(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            source = root / 'runtime/package.json'
            source.parent.mkdir()
            package = root / 'nix-out/lib/node_modules/@midnight-ntwrk/compact-runtime/package.json'
            package.parent.mkdir(parents=True)
            (package.parent / 'dist').mkdir()
            (package.parent / 'dist/built-ins.js').write_text('builtins')
            metadata = {'name':'@midnight-ntwrk/compact-runtime','version':'0.18.0',
                        'dependencies':{'@midnightntwrk/onchain-runtime-v4':'4.0.1'}}
            source.write_text(json.dumps(metadata))
            package.write_text(json.dumps(metadata))
            result = SimpleNamespace(returncode=0, stdout=str(root / 'nix-out'))
            with patch.object(gate, 'ROOT', root), patch.object(gate.subprocess, 'run', return_value=result):
                self.assertEqual(gate.current_runtime_package(), package.parent.resolve())
                package.write_text(json.dumps(dict(metadata, version='0.17.0')))
                with self.assertRaisesRegex(gate.common.GateError, 'differs from source'):
                    gate.current_runtime_package()
                package.write_text(json.dumps(metadata))
                source.write_text(json.dumps(dict(metadata, version='0.19.0')))
                with self.assertRaisesRegex(gate.common.GateError, 'differs from source'):
                    gate.current_runtime_package()

    def test_missing_duplicate_extra_or_wrong_operation_refuse(self):
        for change in ('missing','duplicate','extra','reordered','operation'):
            value=copy.deepcopy(self.good)
            if change=='missing':value['calls'].pop()
            elif change=='duplicate':value['calls'][1]=copy.deepcopy(value['calls'][0])
            elif change=='extra':value['calls'].append(copy.deepcopy(value['calls'][0]))
            elif change=='reordered':value['calls'].reverse()
            else:value['calls'][1]['operation']='another'
            with self.subTest(change=change), self.assertRaises(gate.common.GateError):
                gate.validate_summary(value,self.row)

    def test_each_required_flag_and_nonempty_proof_is_required(self):
        for key in ('changed_binding_rejected','applied','recorded_state_matches','replay_unchanged',
                    'proof_bytes','state_bytes','replay_refusal'):
            for bad in (None,False,0):
                value=copy.deepcopy(self.good);value['calls'][1][key]=bad
                with self.subTest(key=key,bad=bad), self.assertRaises(gate.common.GateError):
                    gate.validate_summary(value,self.row)
        value=copy.deepcopy(self.good);value['calls'][0]['proof_bytes']=True
        with self.assertRaises(gate.common.GateError):gate.validate_summary(value,self.row)

    def test_wrong_profile_and_constructor_claim_refuse(self):
        for key,bad in [('kind','point-guard'),('status','failed'),('strictness','relaxed'),
                        ('constructor_data_deployed',False),('constructor_execution_proved',True)]:
            value=copy.deepcopy(self.good);value[key]=bad
            with self.subTest(key=key), self.assertRaises(gate.common.GateError):
                gate.validate_summary(value,self.row)

    def test_failed_command_retains_failed_receipt_and_no_completed_rows(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);out=root/'gate'
            (root/'package.json').write_text('{}')
            (root/'dist').mkdir()
            (root/'dist/built-ins.js').write_text('')
            frozen_tool = root / 'frozen'
            frozen_tool.write_text('stub tool snapshot')
            row={**self.row,'package':'fixture'}
            def failed(*args,**kwargs):raise gate.common.GateError('simulated failed consumer compile')
            with patch.object(gate,'reviewed_rows',return_value=[row]), \
                 patch.object(gate,'source_inventory',return_value={}), \
                 patch.object(gate.ledger_static,'FIXTURE_HASHES',{}), \
                 patch.object(gate.shutil,'which',return_value='/usr/bin/true'), \
                 patch.object(gate,'current_runtime_package',return_value=root), \
                 patch.object(gate,'current_runtime_identity',return_value={'runtime_source_sha256':'source','runtime_closure_sha256':'closure'}), \
                 patch.object(gate.common,'git_head',return_value='frozen'), \
                 patch.object(gate.common,'stable_copy',return_value={'snapshot':str(frozen_tool),'sha256':'hash'}), \
                 patch.object(gate.common,'run',side_effect=failed):
                result=gate.run_gate(out,root,root,root,[],
                                    {'MIDNIGHT_PP':str(root),'MIDNIGHT_LEDGER_TEST_STATIC_DIR':str(root)})
            self.assertEqual(result['status'],'failed')
            self.assertEqual(result['reducers'],[])
            self.assertIn('simulated failed',result['error'])
            self.assertEqual(json.loads((out/'receipt.json').read_text()),result)


if __name__=='__main__':
    unittest.main()
