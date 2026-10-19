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

"""Fast orchestration controls; never build tools or execute native proofs."""
import importlib.util
import json
import tempfile
from types import SimpleNamespace
from pathlib import Path
import unittest
from unittest.mock import patch
import subprocess
import run_native_ci_forge as owner


class OwnerTests(unittest.TestCase):
    def test_one_owner_plan_and_run_keep_uncached_qualification_and_private_cap(self):
        prepared = {name: Path('/actual')/name for name in ('compiler','forge','proof','params','policy_path','native_helper','zkir')}
        commands = owner.owner_commands(prepared, Path('/private/admitted'), Path('/actual/helper'))
        self.assertEqual(len(commands), 2)
        self.assertEqual(commands[0][3], 'plan')
        self.assertEqual(commands[1][3], 'run')
        self.assertEqual(sum(str(x).endswith('check_native_ci.sh') for command in commands for x in command), 1)
        self.assertIn('--uncached-zkir', commands[1])
        self.assertEqual(commands[1][commands[1].index('--max-bytes')+1], 268435456)
        self.assertIn('--snapshot-out', commands[1])
        self.assertNotIn('qualify_linux_native.py', ' '.join(map(str, commands[0]+commands[1])))

    def test_subprocess_failure_propagates_without_fallback(self):
        with patch.object(owner.subprocess, 'run', side_effect=subprocess.CalledProcessError(7, ['owner'])) as called:
            with self.assertRaises(subprocess.CalledProcessError):
                owner.execute(['owner'], {})
            self.assertEqual(called.call_count, 1)
            self.assertTrue(called.call_args.kwargs['check'])

    def test_default_direct_workflow_command_remains_and_experiment_is_mutually_exclusive(self):
        workflow = (owner.ROOT/'.github/workflows/build-compiler.yml').read_text()
        direct = workflow.split('      - name: Check and test native Rust backend', 1)[1].split('      - name:', 1)[0]
        self.assertIn("if: github.event_name != 'workflow_dispatch' || !inputs.native-forge-experiment", direct)
        self.assertIn('run: bash tools/compact-rust-backend/check_native_ci.sh', direct)
        self.assertIn('default: false', workflow)
        self.assertEqual(workflow.count('run_native_ci_forge.py'), 1)
        experiment = workflow.split('      - name: Explicit experimental Forge route', 1)[1].split('      - name:', 1)[0]
        self.assertIn("if: github.event_name == 'workflow_dispatch' && inputs.native-forge-experiment", experiment)
        self.assertNotIn('actions/cache', experiment)
        source = Path(owner.__file__).read_text()
        self.assertIn('qualify_linux_forge.py', source)
        self.assertIn('prepared["compiler"] == compiler and prepared["proof"] == proof', source)
        self.assertIn('prepared["live"]()', source)


    def test_qualified_mode_never_reprovisions_or_repeats_finite_laps(self):
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary).resolve()
            paths = {}
            for name in ('compiler', 'proof', 'helper', 'finite'):
                paths[name] = base/name
                paths[name].write_text('current tool')
            with patch.object(owner.platform, 'system', return_value='Linux'), patch.object(owner.platform, 'machine', return_value='x86_64'), patch.object(owner, 'execute') as execute, patch.object(owner, 'run_qualified', return_value=0) as qualified:
                result = owner.main(['--compiler',str(paths['compiler']), '--qualification-dir',str(paths['finite']),
                    '--proof-runner',str(paths['proof']), '--helper',str(paths['helper']), '--output',str(base/'new-output')])
                self.assertEqual(result, 0)
                execute.assert_not_called()
                self.assertEqual(qualified.call_count, 1)
                self.assertEqual(qualified.call_args.args[:4], (paths['compiler'], paths['proof'], paths['finite'], paths['helper']))
            with self.assertRaisesRegex(ValueError, 'together'):
                owner.main(['--compiler',str(paths['compiler']), '--qualification-dir',str(paths['finite']), '--output',str(base/'missing-group')])
            self.assertFalse((base/'missing-group').exists())

    def test_linux_owner_workflow_is_linux_only_and_preserves_two_lap_qualification(self):
        workflow = (owner.ROOT/'.github/workflows/forge-local-ci.yml').read_text()
        apple = workflow.split('  proof:',1)[1].split('  forge-linux-material:',1)[0]
        self.assertNotIn("inputs.qualification == 'linux-owner'", apple)
        self.assertIn("if: inputs.qualification == 'linux-native'", workflow)
        self.assertIn("if: inputs.qualification == 'linux-owner'", workflow)
        self.assertEqual(workflow.count('run_native_ci_forge.py'), 1)
        command = workflow.split('      - name: Execute the single original native owner',1)[1].split('      # Deliberately',1)[0]
        self.assertIn('--qualification-dir "$LINUX_REPORT"',command)
        self.assertIn('--proof-runner "$LINUX_PROOF_RUNNER"',command)
        self.assertNotIn('qualify_linux_native.py',command)
        self.assertIn("('linux-native', 'linux-owner')", workflow)
        push = workflow.split('  push:',1)[1].split('  workflow_call:',1)[0]
        for dependency in ('.github/scripts/**', '.github/workflows/compiler-producer.yml', '.github/workflows/build-compiler.yml'):
            self.assertIn(dependency,push)


class AdmissionTests(unittest.TestCase):
    def fixture(self, base):
        native = owner.native
        qdir, output, params = base/'finite', base/'admitted', base/'params'
        for path in (qdir, output, params):
            path.mkdir()
        tools = {}
        for name in ('compiler', 'proof_runner', 'forge', 'zkir', 'helper', 'compact-zkir-cache', 'source'):
            tools[name] = base/('midnight-zkir-forge-ci' if name == 'helper' else name)
            tools[name].write_text('actual current '+name)
        hashes = {name:native.adapter.digest(path) for name,path in tools.items()}
        config = json.loads((owner.ROOT/'tools/compact-rust-backend/forge_native_qualification.json').read_text())
        for k in config['parameters']:
            path = params/('bls_midnight_2p'+k)
            path.write_text('actual parameter '+k)
            config['parameters'][k] = native.adapter.digest(path)
        config_path = base/'config.json'
        config_path.write_text(json.dumps(config))
        force = dict(selection='force', profile=native.adapter.PROFILE, ledger_revision=native.adapter.REVISION,
            serialization=native.adapter.SERIALIZATION, keygen_recipe='library-keygen/default-options/no-rng',
            unavailable='refuse', build_fingerprint=native.FINGERPRINT,
            forge=dict(path=str(tools['forge']),sha256=hashes['forge']),
            zkir=dict(path=str(tools['zkir']),sha256=hashes['zkir']), parameter_directory=str(params),
            parameters={k:config['parameters'][k] for k in ('6','8')})
        qualified = dict(passed=True, public_source=native.finite.PUBLIC,
            expected_build_fingerprint=native.FINGERPRINT,
            source_manifest_sha256=native.adapter.digest(Path(native.__file__).with_name('forge_linux_source.json')),
            tools={name:hashes[name] for name in ('compiler','proof_runner','forge','zkir')},
            consumer_inputs={str(tools['source']):hashes['source']})
        for name,value in [('qualification.json',qualified),('force-policy.json',force),
            ('cold.command.json',dict(exit_code=0,argv=['run','--compiler',str(tools['compiler']),'--proof-runner',str(tools['proof_runner'])]))]:
            (qdir/name).write_text(json.dumps(value))
        for label in ('cold','warm','restored','auto','uncached'):
            (qdir/(label+'.json')).write_text(json.dumps(dict(compiler_sha256=hashes['compiler'],
                proof_runner_sha256=hashes['proof_runner'], artifact='same-current-bytes')))
        return SimpleNamespace(qualification_dir=qdir, config=config_path, helper=tools['helper'], output=output), tools

    def test_current_finite_admission_and_live_mutation_controls(self):
        for changed in (None, 'stale', 'tool', 'source', 'policy', 'finite-artifact', 'wrong-helper'):
            with self.subTest(changed=changed), tempfile.TemporaryDirectory() as temporary:
                args, tools = self.fixture(Path(temporary).resolve())
                if changed == 'stale':
                    path = args.qualification_dir/'qualification.json'
                    value = json.loads(path.read_text()); value['passed'] = False; path.write_text(json.dumps(value))
                elif changed in ('tool','source'):
                    tools['compiler' if changed == 'tool' else 'source'].write_text('live mutation')
                elif changed == 'wrong-helper':
                    args.helper = Path(temporary)/'unqualified-helper'
                    args.helper.write_text('different current helper bytes')
                elif changed == 'policy':
                    path = args.qualification_dir/'force-policy.json'
                    value = json.loads(path.read_text()); value['profile'] = 'unqualified'; path.write_text(json.dumps(value))
                elif changed == 'finite-artifact':
                    path = args.qualification_dir/'warm.json'
                    value = json.loads(path.read_text()); value['artifact'] = 'changed'; path.write_text(json.dumps(value))
                # Simulate only the already separately tested finite artifact decoder;
                # all extracted admission/file hashing below is the real function.
                with patch.object(owner.native.finite, 'checked_lap', side_effect=lambda lap, route: lap['artifact']), patch.object(owner, 'owner_commands') as plan:
                    if changed:
                        with self.assertRaises(ValueError):
                            owner.native.prepare_native(args)
                        plan.assert_not_called()
                    else:
                        prepared = owner.native.prepare_native(args)
                        self.assertEqual(prepared['compiler'], tools['compiler'])
                        self.assertEqual(prepared['policy']['tiny_max_k'], 16)
                        prepared['live']()
                        tools['source'].write_text('later live mutation')
                        with self.assertRaisesRegex(ValueError, 'live qualification premise'):
                            prepared['live']()


if __name__ == '__main__':
    unittest.main()
