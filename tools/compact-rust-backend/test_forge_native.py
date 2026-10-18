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

"""Pinned single-circuit native adapter controls, without model/key generation."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import forge_native as native


class NativeAdapterTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.source = self.root / 'actual.zkir'
        self.source.write_bytes(b'current source')
        self.sibling = self.root / 'other.zkir'
        self.sibling.write_bytes(b'untouched sibling')
        self.helper, self.zkir, self.forge = [self.root / name for name in ['helper', 'zkir', 'forge']]
        for tool in [self.helper, self.zkir, self.forge]:
            tool.write_bytes(b'pinned executable')
        self.params = self.root / 'params'
        self.params.mkdir()
        (self.params / 'bls_midnight_2p8').write_bytes(b'parameters')
        self.policy = self.root / 'policy.json'
        self.policy.write_text(json.dumps({'zkir': {'path': str(self.zkir), 'sha256': native.digest(self.zkir)},
            'forge': {'path': str(self.forge)}, 'parameter_directory': str(self.params),
            'parameters': {'8': native.digest(self.params / 'bls_midnight_2p8')}, 'selection': 'auto'}))
        self.env = dict(zip(native.PINS, [str(self.helper), native.digest(self.helper), str(self.policy), native.digest(self.policy)]))
        self.env.update(COMPACT_ZKIR_MODE='forge', MIDNIGHT_PP=str(self.params), MIDNIGHT_ZKIR_FORGE_DIR=str(self.root / 'cache'))
        self.prover, self.verifier = self.root / 'arbitrary/proof.key', self.root / 'different/verify.key'
        self.private = None

    def response(self, argv, *, env, **kwargs):
        self.assertEqual(argv, [str(self.helper)])
        self.assertNotIn('shell', kwargs)
        request = json.loads(env['COMPACT_ZKIR_CACHE_REQUEST'])
        inputs, keys = Path(request['inputs']), Path(request['keys'])
        self.private = inputs.parent
        self.assertEqual([p.name for p in inputs.iterdir()], ['actual.zkir'])
        self.assertEqual((inputs / 'actual.zkir').read_bytes(), self.source.read_bytes())
        self.assertTrue(request['report'])
        self.assertIsNone(request['final_output'])
        keys.mkdir()
        artifacts = []
        for kind, file in [('binary_ir', inputs / 'actual.bzkir'), ('prover', keys / 'actual.prover'), ('verifier', keys / 'actual.verifier')]:
            file.write_bytes(kind.encode())
            artifacts.append({'kind': kind, 'digest': native.digest(file), 'size': file.stat().st_size})
        self.row = {'name': 'actual', 'source_sha256': native.digest(self.source), 'k': 8, 'rows': 19,
                    'parameter_sha256': native.digest(self.params / 'bls_midnight_2p8'),
                    'acquisition_path': self.route, 'artifacts': artifacts}
        if self.mutate:
            self.mutate(self.row)
        kwargs['stdout'].write(json.dumps({'schema': 1, 'circuits': [self.row]}).encode())
        return subprocess.CompletedProcess(argv, 0)

    def run_case(self, route='auto-bypass', mutate=None):
        self.route, self.mutate = route, mutate
        with patch.object(native.subprocess, 'run', side_effect=self.response) as run:
            value = native.compile_circuit(self.source, self.prover, self.verifier, self.zkir, self.env)
        self.assertEqual(run.call_count, 1)  # No second CLI version/model/keygen process.
        self.assertFalse(self.private.exists())
        self.assertEqual(self.sibling.read_bytes(), b'untouched sibling')
        self.assertFalse(self.sibling.with_suffix('.bzkir').exists())
        return value

    def test_auto_force_and_actual_route_are_preserved(self):
        for route in ['auto-bypass', 'forge-hit', 'forge-miss', 'unavailable-keygen']:
            policy = json.loads(self.policy.read_text())
            policy['selection'] = 'auto' if route == 'auto-bypass' else 'force'
            self.policy.write_text(json.dumps(policy))
            self.env[native.PINS[3]] = native.digest(self.policy)
            with self.subTest(route=route):
                self.assertEqual(self.run_case(route)['acquisition_path'], route)
                self.assertEqual(self.prover.read_bytes(), b'prover')
                self.assertEqual(self.verifier.read_bytes(), b'verifier')

    def test_complete_pins_and_provider_mismatch_have_no_process_effects(self):
        for key in native.PINS:
            env = dict(self.env)
            del env[key]
            with self.assertRaises(ValueError):
                native.selected(env)
        self.env[native.PINS[1]] = '0' * 64
        with patch.object(native.subprocess, 'run', side_effect=AssertionError('must not invoke')):
            with self.assertRaises(ValueError):
                native.compile_circuit(self.source, self.prover, self.verifier, self.zkir, self.env)
        self.assertFalse(self.prover.exists())

    def test_malformed_receipt_and_changed_source_refuse_before_outputs(self):
        mutations = [lambda row: row.update(name='other'), lambda row: row.update(extra=True),
                     lambda row: row.update(k=True), lambda row: row.update(rows=True),
                     lambda row: row.update(rows=2**64), lambda row: row.update(acquisition_path='unknown'),
                     lambda row: row['artifacts'].append(row['artifacts'][0]),
                     lambda row: row['artifacts'][0].update(digest='0' * 64),
                     lambda row: self.source.write_bytes(b'changed original')]
        for mutate in mutations:
            self.source.write_bytes(b'current source')
            with self.subTest(mutate=mutate), self.assertRaises(ValueError):
                self.run_case(mutate=mutate)
            self.assertFalse(self.prover.exists())
            self.assertFalse(self.private.exists())

    def test_output_parameter_policy_tool_and_store_aliases_refuse(self):
        for target in [self.params / 'new.key', self.policy, self.helper, self.root / 'cache/key']:
            with self.subTest(target=target), patch.object(native.subprocess, 'run', side_effect=AssertionError('must not invoke')):
                with self.assertRaises(ValueError):
                    native.compile_circuit(self.source, target, self.verifier, self.zkir, self.env)
        self.assertEqual(self.policy.read_text()[0], '{')

    def test_actual_zkir_and_parameter_selection_must_match_policy(self):
        other = self.root / 'different-zkir'
        other.write_bytes(b'pinned executable')
        with patch.object(native.subprocess, 'run', side_effect=AssertionError('must not invoke')):
            with self.assertRaises(ValueError):
                native.compile_circuit(self.source, self.prover, self.verifier, other, self.env)
            environment = dict(self.env, MIDNIGHT_PP=str(self.root))
            with self.assertRaises(ValueError):
                native.compile_circuit(self.source, self.prover, self.verifier, self.zkir, environment)
        self.assertFalse(self.prover.exists())

    def test_original_parent_alias_retarget_refuses_without_source_mutation(self):
        import os
        for kind in ['source', 'policy', 'helper', 'zkir', 'params', 'forge', 'store']:
            with self.subTest(kind=kind):
                alias = self.root / ('alias-' + kind)
                alias.symlink_to(self.root, target_is_directory=True)
                other = self.root / ('other-' + kind)
                other.mkdir()
                source, zkir, environment = self.source, self.zkir, dict(self.env)
                policy = json.loads(self.policy.read_text())
                if kind == 'source':
                    source = alias / self.source.name
                elif kind == 'policy':
                    environment[native.PINS[2]] = str(alias / self.policy.name)
                elif kind == 'helper':
                    environment[native.PINS[0]] = str(alias / self.helper.name)
                elif kind == 'zkir':
                    zkir = alias / self.zkir.name
                elif kind == 'params':
                    environment['MIDNIGHT_PP'] = str(alias / self.params.name)
                elif kind == 'forge':
                    policy['forge']['path'] = str(alias / self.forge.name)
                else:
                    environment['MIDNIGHT_ZKIR_FORGE_DIR'] = str(alias / 'cache')
                self.policy.write_text(json.dumps(policy))
                environment[native.PINS[3]] = native.digest(self.policy)
                self.route = 'auto-bypass'
                def retarget(row):
                    alias.unlink()
                    alias.symlink_to(other, target_is_directory=True)
                self.mutate = retarget
                with patch.object(native.subprocess, 'run', side_effect=self.response):
                    with self.assertRaises((ValueError, OSError)):
                        native.compile_circuit(source, self.prover, self.verifier, zkir, environment)
                self.assertFalse(self.private.exists())
                self.assertFalse(self.prover.exists())
                self.assertEqual(self.source.read_bytes(), b'current source')
                alias.unlink()
                policy['forge']['path'] = str(self.forge)
                self.policy.write_text(json.dumps(policy))
                self.env[native.PINS[3]] = native.digest(self.policy)

    def test_source_policy_and_child_json_bounds_before_output_copy(self):
        for kind in ['source', 'policy']:
            target = self.source if kind == 'source' else self.policy
            saved = target.read_bytes()
            with target.open('wb') as stream:
                stream.truncate(native.MAX_JSON + 1)
            with patch.object(native.subprocess, 'run', side_effect=AssertionError('must not execute')):
                with self.assertRaises(ValueError):
                    native.compile_circuit(self.source, self.prover, self.verifier, self.zkir, self.env)
            target.write_bytes(saved)
        def huge(argv, *, env, stdout, stderr, **kwargs):
            request = json.loads(env['COMPACT_ZKIR_CACHE_REQUEST'])
            self.private = Path(request['inputs']).parent
            stdout.seek(native.MAX_JSON)
            stdout.write(b'x')
            return subprocess.CompletedProcess(argv, 0)
        with patch.object(native.subprocess, 'run', side_effect=huge):
            with self.assertRaises(ValueError):
                native.compile_circuit(self.source, self.prover, self.verifier, self.zkir, self.env)
        self.assertFalse(self.private.exists())
        self.assertFalse(self.prover.exists())

    def test_stream_copy_refuses_oversized_artifact_without_final_replace(self):
        artifact, destination = self.root / 'growing', self.root / 'private-copy'
        artifact.write_bytes(b'oversized')
        expected = {'size': 1, 'digest': native.digest(artifact)}
        with self.assertRaises(ValueError):
            native.copy_verified(artifact, destination, expected)
        self.assertFalse(destination.exists())
        self.assertFalse(self.prover.exists())

    def test_duplicate_json_and_helper_failure_refuse(self):
        for raw in ['{"schema":1,"schema":1}', '{"schema":NaN}', '{"schema":Infinity}']:
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                native.decode(raw)
        with patch.object(native.subprocess, 'run', side_effect=subprocess.CalledProcessError(2, ['helper'])):
            with self.assertRaises(subprocess.CalledProcessError):
                native.compile_circuit(self.source, self.prover, self.verifier, self.zkir, self.env)
        self.assertFalse(self.prover.exists())


if __name__ == '__main__':
    unittest.main()
