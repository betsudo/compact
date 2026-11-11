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

"""HOST orchestration/refusal controls; no source, material or proof authority."""
from contextlib import ExitStack, redirect_stdout
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import check_constructor_vector_proof as owner


class CompositionControls(unittest.TestCase):
    def exercise(self, failure=None, *, stale=False, inside_target=False):
        with tempfile.TemporaryDirectory(prefix='HOST constructor composition ') as temporary:
            root=Path(temporary).resolve()
            source=root/'source';source.mkdir()
            output=root/'output with spaces'
            compiler=root/'compiler';compiler.write_bytes(b'HOST compiler, never executed')
            scheme=root/'scheme';scheme.write_bytes(b'HOST Scheme, never executed')
            zkir=root/'zkir';zkir.write_bytes(b'HOST ZKIR, never executed')
            runtime=root/'runtime';runtime.mkdir()
            head='1'*40;tree='2'*40
            recipe=root/'recipe.json'
            recipe.write_text(
                json.dumps(
                    dict(
                        status='passed',
                        format='HOST-no-authority',
                        head='3' * 40 if stale else head,
                        tree=tree,
                        version='HOST',
                        compactc=dict(path=str(compiler), sha256=owner.sha(compiler)),
                        **{'compactc-scheme': dict(path=str(scheme), sha256=owner.sha(scheme))}
                    )
                )
            )
            calls=[];arguments={}
            def fake_git(*args):
                return {('rev-parse','HEAD'):head,('rev-parse','HEAD^{tree}'):tree,('status','--porcelain'):''}[args]
            def stage(name):
                def run():
                    calls.append(name);arguments[name]=sys.argv[1:]
                    self.assertNotIn('LD_LIBRARY_PATH',os.environ)
                    self.assertEqual(Path(os.environ['CARGO_TARGET_DIR']),root/'shared-target')
                    if failure==name:raise RuntimeError('HOST '+name+' refusal')
                    path=Path(sys.argv[sys.argv.index('--output')+1]);path.mkdir()
                    if name=='source':
                        value = dict(
                            status='passed',
                            head=head,
                            complete_sources=14,
                            successful_constructor_cases=17,
                            inputs={},
                            tools={
                                str(compiler): owner.sha(compiler),
                                str(scheme): owner.sha(scheme),
                            },
                        )
                    elif name=='models':value=dict(status='passed',scope='HOST-no-model-authority')
                    else:value=dict(status='passed',prerequisites=dict(zkir=str(zkir)))
                    (path/'receipt.json').write_text(json.dumps(value))
                return run
            def fake_proofs(selected,env):
                calls.append('proofs')
                self.assertEqual(selected,output)
                self.assertEqual(Path(env['CARGO_TARGET_DIR']),root/'shared-target')
                if failure=='proofs':raise RuntimeError('HOST proof command failure')
                proof=output/'proofs';proof.mkdir()
                (proof/'receipt.json').write_text('{"scope":"HOST-no-proof-authority"}')
                return dict(status='passed',fresh_standalone_proofs=17,
                            separate_standalone_verifications=17,separately_proved_applications=17)
            argv=['gate','--compiler-receipt',str(recipe),'--ts-runtime',str(runtime),'--zkir',str(zkir),'--output',str(output)]
            env=dict(CARGO_TARGET_DIR=str(source/'target' if inside_target else root/'shared-target'),LD_LIBRARY_PATH='HOST-untrusted')
            with ExitStack() as stack:
                stack.enter_context(patch.object(owner,'W',source))
                stack.enter_context(patch.object(owner,'git',fake_git))
                stack.enter_context(patch.object(owner.sdk,'cargo_configuration',return_value={}))
                stack.enter_context(patch.object(owner.source_owner,'main',stage('source')))
                stack.enter_context(patch.object(owner.model_owner,'main',stage('models')))
                stack.enter_context(patch.object(owner.preflight_owner,'main',stage('preflight')))
                stack.enter_context(patch.object(owner,'prove_selected',fake_proofs))
                stack.enter_context(patch.object(sys,'argv',argv))
                stack.enter_context(patch.dict(os.environ,env))
                original_env=dict(os.environ);original_argv=sys.argv
                with redirect_stdout(io.StringIO()):
                    if failure or stale or inside_target:
                        with self.assertRaisesRegex((RuntimeError,ValueError), 'HOST|current clean|outside source'):
                            owner.main()
                    else:owner.main()
                self.assertEqual(dict(os.environ),original_env)
                self.assertIs(sys.argv,original_argv)
            receipt=json.loads((output/'receipt.json').read_text()) if (output/'receipt.json').exists() else None
            return calls,arguments,receipt

    def test_success_forwards_each_maintained_owner_before_proof_stage(self):
        calls,args,receipt=self.exercise()
        self.assertEqual(calls,['source','models','preflight','proofs'])
        self.assertEqual([s['label'] for s in receipt['stages']],['source-api','models','preflight'])
        self.assertEqual(receipt['status'],'passed')
        self.assertTrue(receipt['proof_execution'])
        self.assertEqual(receipt['standalone_proof_cases'],17)
        self.assertEqual(receipt['separate_standalone_verifications'],17)
        self.assertEqual(receipt['separate_default_strict_applications'],17)
        self.assertEqual(args['models'][0],'--producer-root')
        self.assertTrue(args['models'][1].endswith('/protocol/generated'))
        self.assertIn('--compiler-receipt',args['preflight'])
        self.assertIn('--zkir',args['preflight'])
        self.assertNotIn('--prepare-only',sum(args.values(),[]))

    def test_each_failure_stops_later_stages_and_keeps_failure_receipt(self):
        order=['source','models','preflight','proofs']
        for failure in order:
            with self.subTest(stage=failure):
                calls,_,receipt=self.exercise(failure)
                self.assertEqual(calls,order[:order.index(failure)+1])
                self.assertEqual(receipt['status'],'failed')
                self.assertFalse(receipt['proof_execution'])
                self.assertIn('HOST',receipt['error'])
                if failure!='proofs':self.assertEqual(receipt['stages'][-1]['exit'],1)

    def test_stale_recipe_stops_before_owner_or_key_effects(self):
        calls,_,_=self.exercise(stale=True)
        self.assertEqual(calls,[])

    def test_source_target_is_refused_before_owner_or_key_effects(self):
        calls,_,_=self.exercise(inside_target=True)
        self.assertEqual(calls,[])

    def test_remote_prover_configuration_is_refused_before_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'output'
            argv = ['gate', '--compiler-receipt', 'unused', '--ts-runtime', 'unused',
                    '--zkir', 'unused', '--output', str(output)]
            with patch.object(sys, 'argv', argv), patch.dict(os.environ, MIDNIGHT_PROOF_SERVER=''):
                with self.assertRaisesRegex(ValueError, 'requires local proving'):
                    owner.main()
            self.assertFalse(output.exists())

    def test_optimized_python_is_refused_before_local_imports_or_output(self):
        for script in ['check_constructor_vector_models.py','check_constructor_vector_proof.py']:
            for flag in [True,False]:
                with self.subTest(script=script,flag=flag):
                    env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
                    argv=[sys.executable,*(['-O'] if flag else []),'-B',str(Path(owner.__file__).with_name(script)),'--help']
                    if not flag:env['PYTHONOPTIMIZE']='1'
                    p=subprocess.run(argv,env=env,capture_output=True,text=True,timeout=30)
                    self.assertNotEqual(p.returncode,0)
                    self.assertIn('optimized Python invocation is not admitted',p.stderr)


if __name__=='__main__':
    unittest.main()
