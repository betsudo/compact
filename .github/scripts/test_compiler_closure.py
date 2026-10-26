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
"""Mock-boundary tests; never export/import the host Nix store."""
import copy
import argparse
import base64
import json
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('compiler_closure', Path(__file__).with_name('compiler_closure.py'))
owner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owner)


class ReceiptTests(unittest.TestCase):
    def test_build_failure_streams_diagnostics_and_preserves_machine_stdout(self):
        # Exercise a real failed child without reading or modifying a Nix store.
        script = (
            "import importlib.util, sys; "
            f"s=importlib.util.spec_from_file_location('handoff', {str(Path('.github/scripts/compiler_closure.py').resolve())!r}); "
            "m=importlib.util.module_from_spec(s); s.loader.exec_module(m); "
            "m.text([sys.executable, '-c', "
            "\"import sys; print('/nix/store/machine-output'); "
            "print('compiler-build-diagnostic', file=sys.stderr); sys.exit(7)\"] )"
        )
        result = owner.subprocess.run([owner.sys.executable, '-c', script],
                                      capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('compiler-build-diagnostic', result.stderr.splitlines())
        self.assertEqual(result.stdout, '')

    def test_receipt_accepts_only_exact_identity_archive_and_roster(self):
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory)/'closure'
            archive.write_bytes(b'official export stream placeholder')
            output = '/nix/store/' + 'a'*32 + '-compactc'
            expected = {'source_sha': 'b'*40, 'system': 'x86_64-linux'}
            receipt = dict(schema=1, identity=expected, derivation=output+'.drv', output=output,
                           closure={output:'sha256-' + base64.b64encode(bytes(32)).decode()}, archive_sha256=owner.sha(archive))
            owner.validate_receipt(receipt, expected, archive)
            for field, replacement in [('schema', 2), ('identity', {}), ('output', '/tmp/compiler'),
                                       ('derivation', '/tmp/derivation'), ('closure', {}),
                                       ('archive_sha256', '0'*64)]:
                with self.subTest(field=field):
                    changed = copy.deepcopy(receipt)
                    changed[field] = replacement
                    with self.assertRaises(ValueError):
                        owner.validate_receipt(changed, expected, archive)
            changed = dict(receipt, unexpected=True)
            with self.assertRaises(ValueError):
                owner.validate_receipt(changed, expected, archive)
            alias = Path(directory)/'symlink'
            alias.symlink_to(archive)
            with self.assertRaises(ValueError):
                owner.validate_receipt(receipt, expected, alias)

    def test_rejected_material_never_imports(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            output = '/nix/store/' + 'a'*32 + '-compactc'
            expected = {'source_sha': 'b'*40, 'system': 'x86_64-linux', 'run_id':'1', 'run_attempt':'2'}
            archive = directory/'closure.nar-export'
            archive.write_bytes(b'export')
            valid = dict(schema=1, identity=expected, derivation=output+'.drv', output=output,
                         closure={output:'sha256-' + base64.b64encode(bytes(32)).decode()},
                         archive_sha256=owner.sha(archive))
            for changed in ('source_sha', 'system', 'run_id', 'run_attempt', 'archive', 'roster', 'nar', 'derivation', 'output'):
                value = copy.deepcopy(valid)
                if changed in expected:
                    value['identity'][changed] = 'wrong'
                elif changed == 'archive':
                    value['archive_sha256'] = '0'*64
                elif changed == 'roster':
                    value['closure'] = {}
                elif changed == 'nar':
                    value['closure'][output] = 'sha256-example'
                else:
                    value[changed] = '/nix/store/' + 'c'*32 + '-different' + ('.drv' if changed == 'derivation' else '')
                (directory/'manifest.json').write_text(json.dumps(value))
                with self.subTest(changed=changed), patch.object(owner, 'unpack_artifact'), patch.object(owner, 'text_import') as imported:
                    with self.assertRaises(ValueError):
                        owner.consume(argparse.Namespace(), expected, directory, valid['derivation'], output)
                    imported.assert_not_called()

    def test_runtime_requisite_omission_is_detected_even_on_warm_store(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            output = '/nix/store/' + 'a'*32 + '-compactc'
            archive = directory/'closure.nar-export'
            archive.write_bytes(b'export')
            receipt = dict(schema=1, identity={}, derivation=output+'.drv', output=output,
                closure={output:'sha256-' + base64.b64encode(bytes(32)).decode()}, archive_sha256=owner.sha(archive))
            (directory/'manifest.json').write_text(json.dumps(receipt))
            with patch.object(owner, 'unpack_artifact'), patch.object(owner, 'identity', return_value={}), patch.object(owner, 'text_import', return_value=[output]), patch.object(owner, 'text', return_value=output+'\n/nix/store/'+'d'*32+'-dependency'), patch.object(owner, 'smoke') as smoke:
                with self.assertRaisesRegex(ValueError, 'requisite roster'):
                    owner.consume(argparse.Namespace(), {}, directory, receipt['derivation'], output)
                smoke.assert_not_called()

    def test_same_run_service_identity_and_digest_are_required_before_extraction(self):
        args = argparse.Namespace(artifact_id='42', run_id='1', run_attempt='2', producer_attempt='2', system='x86_64-linux', source_sha='b'*40)
        for changed in ('id', 'expired', 'workflow_run', 'name', 'digest'):
            value = dict(id=42, expired=False, workflow_run={'id':1},
                         name='compiler-1-2-x86_64-linux-'+'b'*40, digest='sha256:'+'a'*64)
            value[changed] = {'id':2} if changed == 'workflow_run' else ('wrong' if changed != 'expired' else True)
            with self.subTest(changed=changed), patch.object(owner, 'service_artifact', return_value=value), patch.object(owner, 'sha', return_value='a'*64), patch.object(owner.zipfile, 'ZipFile') as unzip:
                args.artifact_zip = Path('unused.zip')
                with self.assertRaises(ValueError):
                    owner.unpack_artifact(args, Path('never-created'))
                unzip.assert_not_called()

    def test_smoke_checks_actual_layout_and_keeps_stdout_for_machine_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            package = Path(temporary)
            for name in ('compactc', 'compactc-scheme'):
                path = package/'bin'/name
                path.parent.mkdir(exist_ok=True)
                path.write_text('executable')
                path.chmod(0o755)
            for name in ('runtime-rs', 'runtime-rs-macros'):
                path = package/'share/compactc'/name/'Cargo.toml'
                path.parent.mkdir(parents=True)
                path.write_text('manifest')
            def fake_run(argv, **kwargs):
                if '--target' in argv:
                    target = Path(argv[-1])
                    (target/'contract').mkdir(parents=True)
                    (target/'contract/lib.rs').write_text('generated')
                    (target/'compiler').mkdir()
                    (target/'compiler/contract-manifest.json').write_text('{}')
            with patch.object(owner, 'run', side_effect=fake_run):
                owner.smoke(str(package))
        with patch.object(owner.subprocess, 'run') as child:
            owner.run(['compiler', '--help'])
            self.assertIs(child.call_args.kwargs['stdout'], owner.sys.stderr)

    def test_prior_producer_attempt_preserves_same_run_source_platform_binding(self):
        self.assertEqual(owner.producer_attempt('3', '1'), '1')
        self.assertEqual(owner.producer_attempt('3', '3'), '3')
        for bad in ('4', '0', '', 'bad'):
            with self.subTest(attempt=bad), self.assertRaises(ValueError):
                owner.producer_attempt('3', bad)
        args = argparse.Namespace(artifact_id='42', run_id='10', run_attempt='3', producer_attempt='1',
                                  system='x86_64-linux', source_sha='b'*40, artifact_zip=Path('unused.zip'))
        valid = dict(id=42, expired=False, workflow_run={'id':10},
                     name='compiler-10-1-x86_64-linux-'+'b'*40, digest='sha256:'+'a'*64)
        with patch.object(owner, 'service_artifact', return_value=valid), patch.object(owner, 'sha', return_value='a'*64), patch.object(owner.zipfile, 'ZipFile', side_effect=RuntimeError('validated prior-attempt ZIP')):
            with self.assertRaisesRegex(RuntimeError, 'validated prior-attempt ZIP'):
                owner.unpack_artifact(args, Path('never-created'))
        for changed in ('workflow_run', 'name'):
            value = copy.deepcopy(valid)
            value[changed] = {'id':9} if changed == 'workflow_run' else valid['name'].replace('b'*40, 'c'*40)
            with patch.object(owner, 'service_artifact', return_value=value), patch.object(owner.zipfile, 'ZipFile') as unzip:
                with self.assertRaises(ValueError):
                    owner.unpack_artifact(args, Path('never-created'))
                unzip.assert_not_called()

    def test_official_import_uses_stream_and_propagates_failure(self):
        with patch.object(owner, 'run') as execute:
            execute.return_value.stdout = '/nix/store/output\n'
            stream = object()
            self.assertEqual(owner.text_import(stream), ['/nix/store/output'])
            self.assertEqual(execute.call_args.args[0], ['nix-store', '--import'])
            self.assertIs(execute.call_args.kwargs['stdin'], stream)
            execute.side_effect = RuntimeError('import refused')
            with self.assertRaisesRegex(RuntimeError, 'import refused'):
                owner.text_import(stream)


class WorkflowOwnershipTests(unittest.TestCase):
    def test_named_consumers_require_same_platform_success_and_exact_artifact(self):
        workflow = (owner.ROOT/'.github/workflows/build-compiler.yml').read_text()
        for job, next_job, producer, system in (
            ('build-macos', 'license-check', 'compiler-darwin', 'aarch64-darwin'),
            ('build', None, 'compiler-linux', 'x86_64-linux')):
            section = workflow.split('  '+job+':\n', 1)[1]
            if next_job:
                section = section.split('  '+next_job+':\n', 1)[0]
            self.assertIn('if: always()', section)
            first_step = section.split('    steps:\n', 1)[1].split('      - name: Checkout', 1)[0]
            self.assertIn('needs.'+producer+'.result', first_step)
            self.assertIn('test "$PRODUCER_RESULT" = success', first_step)
            self.assertIn('test -n "$PRODUCER_ARTIFACT"', first_step)
            self.assertIn('needs.'+producer+'.outputs.artifact-id', section)
            self.assertIn('--artifact-id "$ARTIFACT_ID"', section)
            self.assertIn('--system '+system, section)
            self.assertIn('ref: ${{ github.sha }}', section)
            self.assertNotIn('workflow_run:', section)
        apple = workflow.split('  build-macos:\n', 1)[1].split('  license-check:', 1)[0]
        self.assertNotIn('nix build', apple)
        self.assertIn('check_compactc_target.py --consumer', apple)
        linux = workflow.split('  build:\n', 1)[1]
        for retained in ('nix build', './compiler/go', './srcMaps/test.sh', 'check_rejections.py',
                         'check_functional_effects.py', 'check_archive_consumer.py', 'cargo clippy',
                         'check_native_ci.sh', 'yarn test'):
            self.assertIn(retained, linux)
        native = (owner.ROOT/'tools/compact-rust-backend/check_native_ci.sh').read_text()
        self.assertIn('cargo test', native)

    def test_reusable_output_is_upload_service_id_not_a_manifest_claim(self):
        producer = (owner.ROOT/'.github/workflows/compiler-producer.yml').read_text()
        self.assertIn('value: ${{ jobs.compiler.outputs.artifact-id }}', producer)
        self.assertIn('artifact-id: ${{ steps.upload.outputs.artifact-id }}', producer)
        self.assertIn('id: upload', producer)
        self.assertNotIn('strategy:', producer)
        self.assertNotIn('secrets:', producer)


class CompilerFanoutTests(unittest.TestCase):
    def test_consumers_keep_exact_platform_producers_and_complete_gates(self):
        caller = (owner.ROOT/'.github/workflows/build-compiler.yml').read_text()
        self.assertEqual(caller.count('uses: ./.github/workflows/compiler-producer.yml'), 2)
        self.assertIn('    needs: compiler-darwin', caller)
        self.assertIn('    needs: [license-check, compiler-linux]', caller)
        for required in ('check_native_ci.sh', 'check_archive_consumer.py', 'check_rejections.py',
                         'cargo clippy --workspace', 'yarn test'):
            self.assertIn(required, caller)
        self.assertNotIn('continue-on-error', caller)
        self.assertIn('  push:\n    branches: [codex/rust-backend-ast]\n',
                      caller.split('permissions:', 1)[0])


if __name__ == '__main__':
    unittest.main()
