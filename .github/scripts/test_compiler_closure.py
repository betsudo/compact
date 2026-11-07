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
import os
import tarfile
import textwrap
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

    def test_import_failure_streams_diagnostics_without_touching_a_store(self):
        script = (
            "import importlib.util, sys; "
            f"s=importlib.util.spec_from_file_location('handoff', {str(Path('.github/scripts/compiler_closure.py').resolve())!r}); "
            "m=importlib.util.module_from_spec(s); s.loader.exec_module(m); original=m.run; "
            "m.run=lambda argv, **kwargs: original([sys.executable, '-c', "
            "\"import sys; print('/nix/store/machine-output'); "
            "print('import-failure-diagnostic', file=sys.stderr); sys.exit(7)\"], **kwargs); "
            "m.text_import(sys.stdin)"
        )
        result = owner.subprocess.run([owner.sys.executable, '-c', script],
                                      capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('import-failure-diagnostic', result.stderr.splitlines())
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
    def test_wide_zero_api_upload_retains_generated_sources_on_its_runner(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        api = workflow.split('  generated-api:\n', 1)[1].split('  build:\n', 1)[0]
        self.assertIn('id: wide_zero_api\n', api)
        upload = api.split('      - name: Retain generated API diagnostics\n', 1)[1].split('      - name:', 1)[0]
        self.assertIn('if: always()\n', upload)
        paths = {line.strip() for line in upload.split('          path: |\n', 1)[1].splitlines()}
        for path in ('compact-wide-zero.json', 'compact-wide-zero/generated/',
                     'compact-wide-zero/conditional-generated/',
                     'compact-wide-zero/consumer/Cargo.lock', 'compact-wide-zero/typescript.json',
                     'compact-wide-zero/*.stdout', 'compact-wide-zero/*.stderr'):
            with self.subTest(path=path):
                self.assertIn('${{ runner.temp }}/' + path, paths)

    def test_wide_zero_proof_upload_uses_only_its_own_step_and_outputs(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        proof = workflow.split('  build:\n', 1)[1].split('  product-proofs:\n', 1)[0]
        self.assertIn('id: wide_zero_proof\n', proof)
        self.assertNotIn('steps.wide_zero_api.', proof)
        upload = proof.split('          name: wide-zero-failure-', 1)[0].rsplit('      - name:', 1)[1]
        condition = next(line.strip()[4:] for line in upload.splitlines() if line.strip().startswith('if: '))
        self.assertEqual(condition, "failure() && steps.wide_zero_proof.outcome == 'failure'")
        upload = proof.split('          name: wide-zero-failure-', 1)[1].split('      - name:', 1)[0]
        paths = {line.strip() for line in upload.split('          path: |\n', 1)[1].splitlines()}
        for path in ('receipt.json', 'logs/', 'generated/'):
            self.assertIn('${{ runner.temp }}/compact-wide-zero-proof/' + path, paths)
        self.assertNotIn('${{ runner.temp }}/compact-wide-zero.json', paths)
        self.assertFalse(any(path.startswith('${{ runner.temp }}/compact-wide-zero/') for path in paths))

    def test_named_consumers_require_same_platform_success_and_exact_artifact(self):
        workflow = (owner.ROOT/'.github/workflows/build-compiler.yml').read_text()
        for job, next_job, producer, system in (
            ('build-macos', 'license-check', 'compiler-darwin', 'aarch64-darwin'),
            ('constructor-vector', 'source-preflight', 'compiler-linux', 'x86_64-linux'),
            ('build', 'product-proofs', 'compiler-linux', 'x86_64-linux')):
            section = workflow.split('  '+job+':\n', 1)[1]
            if next_job:
                section = section.split('  '+next_job+':\n', 1)[0]
            if job in ('build', 'constructor-vector'):
                self.assertIn('if: ${{ !cancelled() }}', section)
            else:
                self.assertIn("if: ${{ !cancelled() && github.event_name == 'workflow_dispatch' && inputs.run_macos }}", section)
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
        compiler_tests = workflow.split('  compiler-tests:\n', 1)[1].split('  build:\n', 1)[0]
        self.assertIn('needs: compiler-linux', compiler_tests)
        for retained in ('./compiler/go', './srcMaps/test.sh', 'Check doc/compact-reference.mdx',
                         'Merge report as ZIP', 'name: Coverage Report'):
            self.assertIn(retained, compiler_tests)
        linux = workflow.split('  build:\n', 1)[1]
        api = workflow.split('  generated-api:\n', 1)[1].split('  build:\n', 1)[0]
        for retained in ('nix build', 'check_native_ci.sh'):
            self.assertIn(retained, linux)
        for retained in ('check_rejections.py', 'check_functional_effects.py',
                         'check_archive_consumer.py'):
            self.assertIn(retained, api)
        workspace = workflow.split('  workspace-rust:\n', 1)[1].split('  generated-api:\n', 1)[0]
        self.assertIn('cargo test --workspace --exclude compact --locked', workspace)
        self.assertIn('cargo clippy --workspace --exclude compact', workspace)
        self.assertNotIn('./compiler/go', linux)
        packages = workflow.split('  package-outputs:\n', 1)[1].split('  build:\n', 1)[0]
        self.assertIn('needs: compiler-linux', packages)
        self.assertIn('--artifact-id "$ARTIFACT_ID"', packages)
        self.assertIn('run: nix build\n', packages)
        self.assertIn('nix build .#compactc', linux)
        complete = workflow.split('  complete:\n', 1)[1]
        self.assertIn('name: Build and test compiler', complete)
        self.assertIn('compiler-tests', complete)
        self.assertIn('test "$COMPILER_TESTS_RESULT" = success', complete)
        self.assertIn('test "$PACKAGES_RESULT" = success', complete)
        self.assertIn('test "$GENERATED_API_RESULT" = success', complete)
        self.assertIn('test "$WORKSPACE_RUST_RESULT" = success', complete)
        e2e = workflow.split('  e2e-tests:\n', 1)[1].split('  complete:\n', 1)[0]
        self.assertIn('needs: compiler-linux', e2e)
        self.assertIn('nix build .#compactc', e2e)
        self.assertIn('nix develop .#compiler --command yarn test', e2e)
        self.assertIn('name: test-results-ctrf-summary', e2e)
        self.assertIn('name: event-file', e2e)
        self.assertNotIn('- name: Test E2E', workflow.split('  build:\n', 1)[1].split('  e2e-tests:\n', 1)[0])
        self.assertIn('test "$E2E_RESULT" = success', complete)
        native = (owner.ROOT/'tools/compact-rust-backend/check_native_ci.sh').read_text()
        self.assertIn('cargo test', native)
        self.assertIn('check_functional_effects.py --compiler "$COMPACTC"', api)
        self.assertIn('id: runtime-packages', api)
        self.assertIn("if: always() && steps.runtime-packages.outcome == 'success'", api)

    def test_macos_is_manual_opt_in_without_weakening_linux(self):
        from types import SimpleNamespace
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        dispatch = workflow.split('  workflow_dispatch:\n', 1)[1].split('  pull_request:', 1)[0]
        for setting in ('run_macos:', 'type: boolean', 'required: false', 'default: false'):
            self.assertIn(setting, dispatch)
        producer = workflow.split('  compiler-darwin:\n', 1)[1].split('  build-macos:', 1)[0]
        consumer = workflow.split('  build-macos:\n', 1)[1].split('  license-check:', 1)[0]
        conditions = [section.split('    if: ${{ ', 1)[1].split(' }}', 1)[0]
                      for section in (producer, consumer)]
        # Evaluate the literal configured job conditions over event/input/status
        # combinations. Explicit !cancelled() keeps an opted-in failed producer
        # from silently skipping its consumer's material-admission guard.
        for event in ('pull_request', 'schedule', 'workflow_dispatch'):
            for selected in (False, True):
                for cancelled in (False, True):
                    context = dict(github=SimpleNamespace(event_name=event),
                                   inputs=SimpleNamespace(run_macos=selected),
                                   cancelled=lambda: cancelled)
                    actual = [eval(condition.replace('&&', ' and ').replace('!', 'not '),
                                   {'__builtins__': {}}, context) for condition in conditions]
                    requested = event == 'workflow_dispatch' and selected
                    self.assertEqual(actual, [requested, requested and not cancelled])
        linux_producer = workflow.split('  compiler-linux:\n', 1)[1].split('  compiler-darwin:', 1)[0]
        linux = workflow.split('  build:\n', 1)[1]
        self.assertNotIn('if:', linux_producer)
        self.assertIn('needs: [license-check, source-preflight, compiler-linux, fixture-freshness]', linux)
        self.assertNotIn('run_macos', linux)
        self.assertNotIn('compiler-darwin', linux)
        self.assertNotIn('continue-on-error', workflow)

    def test_opted_in_macos_refuses_missing_or_failed_material(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        consumer = workflow.split('  build-macos:\n', 1)[1].split('  license-check:', 1)[0]
        step = consumer.split('      - name: Require successful compiler producer\n', 1)[1].split('      - name:', 1)[0]
        shell = textwrap.dedent(step.split('        run: |\n', 1)[1])
        for result, artifact, attempt, success in (
                ('success', '42', '1', True), ('failure', '42', '1', False),
                ('abandoned', '', '', False), ('skipped', '', '', False),
                ('success', '', '1', False), ('success', '42', '', False)):
            env = dict(os.environ, PRODUCER_RESULT=result, PRODUCER_ARTIFACT=artifact,
                       PRODUCER_ATTEMPT=attempt)
            run = owner.subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', shell],
                                       env=env, capture_output=True, timeout=10)
            self.assertEqual(run.returncode == 0, success, (result, artifact, attempt))

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
        self.assertIn('    needs: [license-check, source-preflight, compiler-linux, fixture-freshness]', caller)
        for required in ('check_native_ci.sh', 'check_archive_consumer.py', 'check_rejections.py',
                         'cargo clippy --workspace', 'yarn test'):
            self.assertIn(required, caller)
        self.assertNotIn('continue-on-error', caller)
        self.assertIn('  pull_request:', caller.split('permissions:', 1)[0])
        self.assertNotIn('  push:', caller.split('permissions:', 1)[0])


class NestedProductWiringTests(unittest.TestCase):
    def test_component_vector_gate_requires_all_fourteen_proofs_and_parameters(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        linux = workflow.split('  component-vector-proof:\n', 1)[1].split('  outer-vector-proof:\n', 1)[0]
        before, step = linux.split('      - name: Qualify component-product Vector APIs and fourteen selected proofs\n', 1)
        step = step.split('      - name: Retain failed component-product Vector diagnostics', 1)[0]
        material = before.split('      - name: Prepare verified built-in proof material\n', 1)[1].split('      - name:', 1)[0]
        for k in (5, 6, 7, 8):
            self.assertIn('--parameter-k ' + str(k), material)
        self.assertIn('record_compiler_recipe.py --compiler "$COMPACTC" --output "$RUNNER_TEMP/compact-component-product-vector-recipe"', step)
        self.assertIn('check_component_product_vector_proof.py --compiler-receipt "$RUNNER_TEMP/compact-component-product-vector-recipe/receipt.json"', step)
        self.assertIn('--ts-runtime "$TS_RUNTIME" --zkir "$ZKIR"', step)
        self.assertIn('shutil.which("zkir")', step)
        self.assertLess(before.index('Import authenticated same-run compiler closure'), before.index('Prepare verified built-in proof material'))
        for weakened in ('--prepare-only', 'continue-on-error', 'if:', '/lib/zkir'):
            self.assertNotIn(weakened, step)
        self.assertIn("steps.component_product_vector.outcome == 'failure'", linux)
        nested = workflow.split('  product-proofs:\n', 1)[1].split('  component-vector-proof:\n', 1)[0]
        self.assertNotIn('check_component_product_vector_proof.py', nested)
        self.assertEqual(workflow.count('check_component_product_vector_proof.py'), 1)
        self.assertIn('COMPONENT_VECTOR_RESULT: ${{ needs.component-vector-proof.result }}', workflow)
        self.assertIn('test "$COMPONENT_VECTOR_RESULT" = success', workflow)
        for role in ('recipe', 'proof'):
            self.assertGreaterEqual(linux.count('${{ runner.temp }}/compact-component-product-vector-' + role + '/'), 2)

    def test_outer_vector_gate_keeps_full_current_source_and_boundary_execution(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        linux = workflow.split('  outer-vector-proof:\n', 1)[1].split('  native-backend:\n', 1)[0]
        main = workflow.split('  build:\n', 1)[1].split('  product-proofs:\n', 1)[0]
        before, step = linux.split('      - name: Qualify outer-Vector product APIs and fifteen selected proofs\n', 1)
        step = step.split('      - name: Retain outer-Vector product proof diagnostics', 1)[0]
        self.assertIn('Import authenticated same-run compiler closure', before)
        self.assertIn('Prepare verified built-in proof material', before)
        material = before.split('      - name: Prepare verified built-in proof material\n', 1)[1].split('      - name:', 1)[0]
        for k in (5, 7, 8):
            self.assertIn('--parameter-k ' + str(k), material)
        self.assertLess(before.index('Import authenticated same-run compiler closure'), before.index('Prepare verified built-in proof material'))
        self.assertIn('shutil.which("zkir")', step)
        self.assertIn('--ts-runtime "$TS_RUNTIME" --zkir "$ZKIR"', step)
        self.assertEqual(linux.count('check_outer_vector_product_proof.py'), 1)
        self.assertNotIn('check_outer_vector_product_semantics.py', linux)
        # The combined proof owner must actually call the complete native gate.
        proof_owner = (owner.ROOT / 'tools/compact-rust-backend/check_outer_vector_product_proof.py').read_text()
        self.assertIn('run_logged([sys.executable, str(Path(main_gate.__file__)),', proof_owner)
        self.assertIn('record_compiler_recipe.py --compiler "$COMPACTC"', step)
        self.assertIn('check_outer_vector_product_proof.py --compiler-receipt "$RUNNER_TEMP/compact-outer-vector-product-recipe/receipt.json"', step)
        self.assertIn('--ts-runtime "$TS_RUNTIME"', step)
        for weakened in ('continue-on-error', 'if:', '--prepare-only', '/lib/zkir'):
            self.assertNotIn(weakened, step)
        upload = linux.split('      - name: Retain outer-Vector product proof diagnostics\n', 1)[1].split('      - name:', 1)[0]
        condition = next(line.strip()[4:] for line in upload.splitlines() if line.strip().startswith('if: '))
        self.assertEqual(condition, "always() && (steps.outer_vector_product_proof.outcome == 'success' || steps.outer_vector_product_proof.outcome == 'failure')")
        # The explicit always() removes GitHub's implicit success() dependency:
        # a successful gate must upload even when a subsequent step fails.
        for outcome, later_failed, expected in [('success', False, True), ('success', True, True), ('failure', True, True), ('skipped', True, False)]:
            with self.subTest(gate=outcome, later_failed=later_failed):
                expression = condition.replace('always()', 'True').replace('steps.outer_vector_product_proof.outcome', repr(outcome)).replace('&&', 'and').replace('||', 'or')
                self.assertEqual(eval(expression, {"__builtins__": {}}, {}), expected)
        self.assertNotIn('steps.runtime-packages', upload)
        self.assertNotIn('Qualify isolated witnessed functional SDK', linux)
        self.assertIn('Qualify isolated witnessed functional SDK', main)
        self.assertIn('product-proofs, component-vector-proof, outer-vector-proof, native-scenarios, native-backend, e2e-tests, build, constructor-vector]', workflow)
        for role in ('recipe', 'proof'):
            self.assertEqual(linux.splitlines().count('            ${{ runner.temp }}/compact-outer-vector-product-' + role + '/'), 1)
        self.assertNotIn('Qualify component-product Vector APIs and fourteen selected proofs', main)
        product = workflow.split('  component-vector-proof:\n', 1)[1].split('  outer-vector-proof:\n', 1)[0]
        self.assertIn('Qualify component-product Vector APIs and fourteen selected proofs', product)
        self.assertIn('Qualify constant vector projection proofs', main)

    def test_outer_vector_literal_shell_forwards_tools_and_stops_on_failures(self):
        import os
        import subprocess
        import sys
        import textwrap
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        step = workflow.split('      - name: Qualify outer-Vector product APIs and fifteen selected proofs\n', 1)[1].split('      - name:', 1)[0]
        shell = textwrap.dedent(step.split('        run: |\n', 1)[1])
        # Execute the configured shell with observable command boundaries. This
        # is argument/failure propagation coverage, not a mock proof verdict.
        with tempfile.TemporaryDirectory(prefix='outer vector shell ') as directory:
            root = Path(directory)
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            wrapper = '''import json, os, pathlib, sys
args = sys.argv[1:]
kind = pathlib.Path(sys.argv[0]).name
stage = 'recipe' if kind == 'python3' else ('runtime' if args[0] == 'build' else ('zkir' if args[-2] == '-c' else 'gate'))
with open(os.environ['TEST_LOG'], 'a') as log:
    log.write(json.dumps(dict(stage=stage, args=args, ld=os.environ.get('LD_LIBRARY_PATH'))) + "\\n")
if os.environ.get('TEST_FAIL') == stage:
    raise SystemExit({'recipe':71,'runtime':72,'zkir':73,'gate':74}[stage])
if stage == 'runtime': print('/mock/runtime-package')
if stage == 'zkir': print('/mock/bin/zkir')
'''
            for name in ('python3', 'nix'):
                path = bin_dir / name
                path.write_text('#!' + sys.executable + '\n' + wrapper)
                path.chmod(0o755)
            for failure, expected in [('', 0), ('recipe', 71), ('zkir', 73), ('runtime', 72), ('gate', 74)]:
                with self.subTest(failure=failure):
                    log = root / ('log-' + (failure or 'success'))
                    env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'],
                               TEST_LOG=str(log), TEST_FAIL=failure, COMPACTC='/mock/compiler with spaces/compactc',
                               RUNNER_TEMP=str(root / 'runner with spaces'), LD_LIBRARY_PATH='/untrusted/setup-python/lib')
                    result = subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', shell], env=env, capture_output=True, text=True, timeout=30)
                    self.assertEqual(result.returncode, expected, result.stderr)
                    rows = [json.loads(line) for line in log.read_text().splitlines()]
                    stages = ['recipe', 'zkir', 'runtime', 'gate']
                    self.assertEqual([r['stage'] for r in rows], stages if not failure else stages[:stages.index(failure) + 1])
                    self.assertEqual(rows[0]['args'], ['tools/compact-rust-backend/record_compiler_recipe.py', '--compiler', env['COMPACTC'], '--output', env['RUNNER_TEMP'] + '/compact-outer-vector-product-recipe'])
                    for row in rows:
                        if row['stage'] in ('zkir', 'gate'):
                            self.assertIsNone(row['ld'])
                    if not failure:
                        self.assertEqual(rows[-1]['args'], ['develop', '.#compiler', '--command', 'python3', '-X', 'faulthandler', '-u', 'tools/compact-rust-backend/check_outer_vector_product_proof.py', '--compiler-receipt', env['RUNNER_TEMP'] + '/compact-outer-vector-product-recipe/receipt.json', '--ts-runtime', '/mock/runtime-package/lib/node_modules/@midnight-ntwrk/compact-runtime', '--zkir', '/mock/bin/zkir', '--output', env['RUNNER_TEMP'] + '/compact-outer-vector-product-proof'])

    def test_constructor_vector_literal_shell_forwards_tools_and_stops_on_failures(self):
        import os
        import subprocess
        import sys
        import textwrap
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        step = workflow.split('      - name: Qualify constructor Vector APIs and seventeen selected proof cases\n', 1)[1].split('      - name:', 1)[0]
        shell = textwrap.dedent(step.split('        run: |\n', 1)[1])
        # Execute the configured shell with observable command boundaries. This
        # is argument/failure propagation coverage, not a mock proof verdict.
        with tempfile.TemporaryDirectory(prefix='constructor vector shell ') as directory:
            root = Path(directory)
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            wrapper = '''import json, os, pathlib, sys
args = sys.argv[1:]
kind = pathlib.Path(sys.argv[0]).name
stage = 'recipe' if kind == 'python3' else ('runtime' if args[0] == 'build' else ('zkir' if args[-2] == '-c' else 'gate'))
with open(os.environ['TEST_LOG'], 'a') as log:
    log.write(json.dumps(dict(stage=stage, args=args, ld=os.environ.get('LD_LIBRARY_PATH'), target=os.environ.get('CARGO_TARGET_DIR'))) + "\\n")
if os.environ.get('TEST_FAIL') == stage:
    raise SystemExit({'recipe':71,'runtime':72,'zkir':73,'gate':74}[stage])
if stage == 'runtime': print('/mock/runtime-package')
if stage == 'zkir': print('/mock/bin/zkir')
'''
            for name in ('python3', 'nix'):
                path = bin_dir / name
                path.write_text('#!' + sys.executable + '\n' + wrapper)
                path.chmod(0o755)
            for failure, expected in [('', 0), ('recipe', 71), ('zkir', 73), ('runtime', 72), ('gate', 74)]:
                with self.subTest(failure=failure):
                    log = root / ('log-' + (failure or 'success'))
                    env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'],
                               TEST_LOG=str(log), TEST_FAIL=failure, COMPACTC='/mock/compiler with spaces/compactc',
                               RUNNER_TEMP=str(root / 'runner with spaces'), CARGO_TARGET_DIR='/mock/existing-shared-target', LD_LIBRARY_PATH='/untrusted/setup-python/lib')
                    result = subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', shell], env=env, capture_output=True, text=True, timeout=30)
                    self.assertEqual(result.returncode, expected, result.stderr)
                    rows = [json.loads(line) for line in log.read_text().splitlines()]
                    stages = ['recipe', 'zkir', 'runtime', 'gate']
                    self.assertEqual([r['stage'] for r in rows], stages if not failure else stages[:stages.index(failure) + 1])
                    self.assertEqual(rows[0]['args'], ['tools/compact-rust-backend/record_compiler_recipe.py', '--compiler', env['COMPACTC'], '--output', env['RUNNER_TEMP'] + '/compact-constructor-vector-recipe'])
                    for row in rows:
                        if row['stage'] in ('zkir', 'gate'):
                            self.assertIsNone(row['ld'])
                    for row in rows:
                        self.assertEqual(row['target'], env['RUNNER_TEMP'] + '/compact-constructor-vector-target' if row['stage']=='gate' else env['CARGO_TARGET_DIR'])
                    if not failure:
                        self.assertEqual(rows[-1]['args'], ['develop', '.#compiler', '--command', 'python3', '-X', 'faulthandler', '-u', 'tools/compact-rust-backend/check_constructor_vector_proof.py', '--compiler-receipt', env['RUNNER_TEMP'] + '/compact-constructor-vector-recipe/receipt.json', '--ts-runtime', '/mock/runtime-package/lib/node_modules/@midnight-ntwrk/compact-runtime', '--zkir', '/mock/bin/zkir', '--output', env['RUNNER_TEMP'] + '/compact-constructor-vector-proof'])

    def test_constructor_gate_requires_material_full_execution_and_diagnostics(self):
        workflow=(owner.ROOT/'.github/workflows/build-compiler.yml').read_text()
        linux=workflow.split('  constructor-vector:\n',1)[1].split('  source-preflight:\n',1)[0]
        before,step=linux.split('      - name: Qualify constructor Vector APIs and seventeen selected proof cases\n',1)
        step=step.split('      - name:',1)[0]
        self.assertIn('Prepare verified built-in proof material',before)
        self.assertIn('Import authenticated same-run compiler closure',before)
        self.assertIn('Resolve locked ledger proof fixtures',before)
        self.assertIn('id: constructor_vector_proof',step)
        self.assertIn('check_constructor_vector_proof.py --compiler-receipt',step)
        self.assertIn('CARGO_TARGET_DIR="$RUNNER_TEMP/compact-constructor-vector-target" env -u LD_LIBRARY_PATH',step)
        for weakened in ('if:','continue-on-error','--prepare-only','/lib/zkir'):
            self.assertNotIn(weakened,step)
        upload=linux.split('      - name: Retain constructor Vector proof diagnostics\n',1)[1].split('      - name:',1)[0]
        condition=next(line.strip()[4:] for line in upload.splitlines() if line.strip().startswith('if: '))
        self.assertEqual(condition,"always() && (steps.constructor_vector_proof.outcome == 'success' || steps.constructor_vector_proof.outcome == 'failure')")
        for outcome,expected in [('success',True),('failure',True),('skipped',False),('cancelled',False)]:
            with self.subTest(outcome=outcome):
                expression=condition.replace('always()','True').replace('steps.constructor_vector_proof.outcome',repr(outcome)).replace('&&','and').replace('||','or')
                self.assertEqual(eval(expression,{"__builtins__":{}},{}),expected)
        self.assertIn('${{ runner.temp }}/compact-constructor-vector-recipe/',upload)
        self.assertIn('${{ runner.temp }}/compact-constructor-vector-proof/',upload)
        self.assertIn('!${{ runner.temp }}/compact-constructor-vector-proof/**/cargo-target/**',upload)
        legacy=workflow.split('  build:\n',1)[1].split('  product-proofs:\n',1)[0]
        self.assertNotIn('constructor_vector_proof', legacy)
        self.assertIn('Qualify isolated witnessed functional SDK',legacy)
        self.assertIn('Qualify component-product Vector APIs and fourteen selected proofs',workflow)
        self.assertIn('Qualify wide-zero multiplication proofs',legacy)

    def test_constructor_equality_literal_shell_forwards_tools_and_stops_on_failures(self):
        import os
        import subprocess
        import sys
        import textwrap
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        step = workflow.split('      - name: Qualify constructor equality APIs and sixteen selected proof cases\n', 1)[1].split('      - name:', 1)[0]
        shell = textwrap.dedent(step.split('        run: |\n', 1)[1])
        # Execute the configured shell with observable command boundaries. This
        # is argument/failure propagation coverage, not a mock proof verdict.
        with tempfile.TemporaryDirectory(prefix='constructor equality shell ') as directory:
            root = Path(directory)
            bin_dir = root / 'bin'
            bin_dir.mkdir()
            wrapper = '''import json, os, pathlib, sys
args = sys.argv[1:]
kind = pathlib.Path(sys.argv[0]).name
stage = 'recipe' if kind == 'python3' else ('runtime' if args[0] == 'build' else ('zkir' if args[-2] == '-c' else 'gate'))
with open(os.environ['TEST_LOG'], 'a') as log:
    log.write(json.dumps(dict(stage=stage, args=args, ld=os.environ.get('LD_LIBRARY_PATH'), target=os.environ.get('CARGO_TARGET_DIR'))) + "\\n")
if os.environ.get('TEST_FAIL') == stage:
    raise SystemExit({'recipe':71,'runtime':72,'zkir':73,'gate':74}[stage])
if stage == 'runtime': print('/mock/runtime-package')
if stage == 'zkir': print('/mock/bin/zkir')
'''
            for name in ('python3', 'nix'):
                path = bin_dir / name
                path.write_text('#!' + sys.executable + '\n' + wrapper)
                path.chmod(0o755)
            for failure, expected in [('', 0), ('recipe', 71), ('zkir', 73), ('runtime', 72), ('gate', 74)]:
                with self.subTest(failure=failure):
                    log = root / ('log-' + (failure or 'success'))
                    env = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ['PATH'],
                               TEST_LOG=str(log), TEST_FAIL=failure, COMPACTC='/mock/compiler with spaces/compactc',
                               RUNNER_TEMP=str(root / 'runner with spaces'), CARGO_TARGET_DIR='/mock/existing-shared-target', LD_LIBRARY_PATH='/untrusted/setup-python/lib')
                    result = subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', shell], env=env, capture_output=True, text=True, timeout=30)
                    self.assertEqual(result.returncode, expected, result.stderr)
                    rows = [json.loads(line) for line in log.read_text().splitlines()]
                    stages = ['recipe', 'zkir', 'runtime', 'gate']
                    self.assertEqual([r['stage'] for r in rows], stages if not failure else stages[:stages.index(failure) + 1])
                    self.assertEqual(rows[0]['args'], ['tools/compact-rust-backend/record_compiler_recipe.py', '--compiler', env['COMPACTC'], '--output', env['RUNNER_TEMP'] + '/compact-constructor-equality-recipe'])
                    for row in rows:
                        if row['stage'] in ('zkir', 'gate'):
                            self.assertIsNone(row['ld'])
                    for row in rows:
                        self.assertEqual(row['target'], env['RUNNER_TEMP'] + '/compact-constructor-equality-target' if row['stage']=='gate' else env['CARGO_TARGET_DIR'])
                    if not failure:
                        self.assertEqual(rows[-1]['args'], ['develop', '.#compiler', '--command', 'python3', '-X', 'faulthandler', '-u', 'tools/compact-rust-backend/check_constructor_equality_proof.py', '--compiler-receipt', env['RUNNER_TEMP'] + '/compact-constructor-equality-recipe/receipt.json', '--ts-runtime', '/mock/runtime-package/lib/node_modules/@midnight-ntwrk/compact-runtime', '--zkir', '/mock/bin/zkir', '--output', env['RUNNER_TEMP'] + '/compact-constructor-equality-proof'])

    def test_constructor_equality_gate_requires_material_full_execution_and_diagnostics(self):
        workflow=(owner.ROOT/'.github/workflows/build-compiler.yml').read_text()
        linux=workflow.split('  build:\n',1)[1]
        before,step=linux.split('      - name: Qualify constructor equality APIs and sixteen selected proof cases\n',1)
        step=step.split('      - name:',1)[0]
        self.assertIn('Prepare verified built-in proof material',before)
        self.assertIn('Import authenticated same-run compiler closure',before)
        self.assertIn('Resolve locked ledger proof fixtures',before)
        self.assertEqual(linux.count('      - name: Qualify constructor equality APIs and sixteen selected proof cases'),1)
        self.assertNotIn('id: constructor_vector_proof',linux)
        required=workflow.split('  complete:\n',1)[1]
        self.assertIn('needs.constructor-vector.result',required)
        self.assertIn('needs.build.result',required)
        self.assertIn('id: constructor_equality_proof',step)
        self.assertIn('check_constructor_equality_proof.py --compiler-receipt',step)
        self.assertIn('CARGO_TARGET_DIR="$RUNNER_TEMP/compact-constructor-equality-target" env -u LD_LIBRARY_PATH',step)
        for weakened in ('if:','continue-on-error','--prepare-only','/lib/zkir'):
            self.assertNotIn(weakened,step)
        upload=linux.split('      - name: Retain constructor equality proof diagnostics\n',1)[1].split('      - name:',1)[0]
        condition=next(line.strip()[4:] for line in upload.splitlines() if line.strip().startswith('if: '))
        self.assertEqual(condition,"always() && (steps.constructor_equality_proof.outcome == 'success' || steps.constructor_equality_proof.outcome == 'failure')")
        for outcome,expected in [('success',True),('failure',True),('skipped',False),('cancelled',False)]:
            with self.subTest(outcome=outcome):
                expression=condition.replace('always()','True').replace('steps.constructor_equality_proof.outcome',repr(outcome)).replace('&&','and').replace('||','or')
                self.assertEqual(eval(expression,{"__builtins__":{}},{}),expected)
        self.assertIn('${{ runner.temp }}/compact-constructor-equality-recipe/',upload)
        self.assertIn('${{ runner.temp }}/compact-constructor-equality-proof/',upload)
        self.assertIn('!${{ runner.temp }}/compact-constructor-equality-proof/**/cargo-target/**',upload)
        self.assertLess(linux.index('Retain constructor equality proof diagnostics'),linux.index('Qualify isolated witnessed functional SDK'))
        self.assertIn('Qualify component-product Vector APIs and fourteen selected proofs',workflow)
        self.assertIn('Qualify wide-zero multiplication proofs',linux)

    def test_source_map_gate_is_required_and_retains_publication_evidence(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        api = workflow.split('  generated-api:\n', 1)[1].split('  build:\n', 1)[0]
        before, step = api.split('      - name: Check complete Counter source-map publication\n', 1)
        step = step.split('      - name: Package source-map publication diagnostics', 1)[0]
        self.assertIn('Import authenticated same-run compiler closure', before)
        self.assertIn('record_compiler_recipe.py --compiler "$COMPACTC"', step)
        self.assertIn('check_source_map_publication.py --compiler-receipt "$RUNNER_TEMP/compact-source-map-recipe/receipt.json"', step)
        for weakened in ('continue-on-error', 'if:', '--prepare-only', '/lib/zkir'):
            self.assertNotIn(weakened, step)
        upload = api.split('      - name: Retain source-map publication diagnostics\n', 1)[1].split('      - name:', 1)[0]
        self.assertIn("if: always() && (steps.source_map_publication.outcome == 'success' || steps.source_map_publication.outcome == 'failure')", upload)
        self.assertNotIn('steps.runtime-packages', upload)
        self.assertIn('test "$GENERATED_API_RESULT" = success', workflow)
        self.assertIn('path: ${{ runner.temp }}/compact-source-map-diagnostics.tar.gz', upload)
        self.assertNotIn('compact-source-map-publication/', upload)
        self.assertIn('Qualify component-product Vector APIs and fourteen selected proofs', workflow)

    def test_required_compiler_status_fails_closed_for_every_dependency_outcome(self):
        import itertools
        import re
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        required = workflow.split('  complete:\n', 1)[1]
        self.assertIn('name: Build and test compiler', required)
        self.assertEqual(workflow.count('name: Build and test compiler\n'), 1)
        self.assertIn('constructor-vector]', required)
        self.assertIn('CONSTRUCTOR_RESULT: ${{ needs.constructor-vector.result }}', required)
        self.assertIn('if: ${{ always() }}', required)
        self.assertNotIn('continue-on-error', workflow)
        shell = textwrap.dedent(required.split('        run: |\n', 1)[1])
        all_names = re.findall(r'test "\$([A-Z0-9_]+)" = success', shell)
        self.assertEqual(len(all_names), len(set(all_names)))
        self.assertIn('CONSTRUCTOR_RESULT', all_names)
        names = ('LICENSE_RESULT', 'PRODUCER_RESULT', 'NATIVE_RESULT', 'CONSTRUCTOR_RESULT')
        baseline = dict.fromkeys(all_names, 'success')
        # Preserve the original four-owner combination coverage and require
        # the merged graph's other owners to succeed at the same time.
        for outcomes in itertools.product(('success', 'failure', 'skipped', 'cancelled', ''), repeat=4):
            with self.subTest(outcomes=outcomes):
                env = dict(os.environ, **(baseline | dict(zip(names, outcomes))))
                run = owner.subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', shell],
                                           env=env, capture_output=True, timeout=10)
                self.assertEqual(run.returncode == 0, all(x == 'success' for x in outcomes))

    def test_constructor_is_an_independent_required_same_producer_consumer(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        constructor = workflow.split('  constructor-vector:\n', 1)[1].split('  source-preflight:\n', 1)[0]
        legacy = workflow.split('  build:\n', 1)[1].split('  product-proofs:\n', 1)[0]
        for consumer in (constructor, legacy):
            self.assertIn('compiler-linux', consumer.split('    steps:',1)[0])
            self.assertIn('if: ${{ !cancelled() }}', consumer)
            self.assertIn('needs.compiler-linux.outputs.artifact-id', consumer)
            self.assertIn('needs.compiler-linux.outputs.producer-attempt', consumer)
            self.assertIn('--source-sha "$SOURCE_SHA" --system x86_64-linux', consumer)
            self.assertIn('--run-id "$GITHUB_RUN_ID" --run-attempt "$GITHUB_RUN_ATTEMPT"', consumer)
            self.assertIn('ref: ${{ github.sha }}', consumer)
        self.assertIn('needs: [license-check, compiler-linux]', constructor)
        self.assertIn('timeout-minutes: 180', constructor)
        self.assertIn("skip_cache: 'true'", constructor)
        self.assertNotIn('secrets.', constructor)
        self.assertNotIn('CARGO_TARGET_DIR: ${{ runner.', constructor)
        self.assertIn('Select external Cargo bootstrap target', constructor)
        self.assertNotIn('Qualify outer-Vector product APIs', constructor)
        self.assertIn('Check and test native Rust backend', workflow.split('  native-backend:\n',1)[1].split('  e2e-tests:\n',1)[0])
        self.assertIn('Rehearse Rust runtime packages', workflow.split('  generated-api:\n',1)[1].split('  build:\n',1)[0])
        self.assertIn('Test E2E', workflow.split('  e2e-tests:\n',1)[1])

    def test_constructor_bootstrap_target_uses_runtime_environment(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        constructor = workflow.split('  constructor-vector:\n', 1)[1].split('  source-preflight:\n', 1)[0]
        start = constructor.index('      - name: Select external Cargo bootstrap target\n')
        self.assertLess(start, constructor.index('      - name: Resolve locked ledger proof fixtures'))
        step = constructor[start:].split('      - name:', 2)[1]
        shell = textwrap.dedent(step.split('        run: |\n', 1)[1])
        with tempfile.TemporaryDirectory(prefix='runner temp $quote with spaces ') as directory:
            root = Path(directory)
            exported = root / 'github-env'
            run = owner.subprocess.run(['bash', '-e', '-o', 'pipefail', '-c', shell],
                                       env=dict(os.environ, RUNNER_TEMP=str(root), GITHUB_ENV=str(exported)),
                                       capture_output=True, timeout=10)
            self.assertEqual(run.returncode, 0, run.stderr)
            self.assertEqual(exported.read_text(),
                             'CARGO_TARGET_DIR=' + str(root / 'constructor-vector-bootstrap-target') + '\n')

    def test_literal_source_map_packaging_preserves_names_bytes_and_partial_failures(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        step = workflow.split('      - name: Package source-map publication diagnostics\n', 1)[1].split('      - name:', 1)[0]
        self.assertIn("if: always() && (steps.source_map_publication.outcome == 'success' || steps.source_map_publication.outcome == 'failure')", step)
        self.assertNotIn('continue-on-error', step)
        script = textwrap.dedent(step.split('        run: |\n', 1)[1])
        # Execute the literal workflow shell on physical adversarial paths. The
        # archive transports diagnostics; it does not normalize test inputs.
        with tempfile.TemporaryDirectory(prefix='archive quote" β ') as temporary:
            root = Path(temporary)
            env = dict(os.environ, RUNNER_TEMP=str(root))
            archive = root / 'compact-source-map-diagnostics.tar.gz'
            result = owner.subprocess.run(['bash', '-e', '-c', script], env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(archive.exists())
            recipe = root / 'compact-source-map-recipe/receipt.json'
            recipe.parent.mkdir()
            recipe.write_bytes(b'{"partial":true}\n')
            result = owner.subprocess.run(['bash', '-e', '-c', script], env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            with tarfile.open(archive) as retained:
                self.assertEqual(retained.extractfile(recipe.relative_to(root).as_posix()).read(), recipe.read_bytes())
            publication = root / 'compact-source-map-publication'
            publication.mkdir()
            names = ['combined-literal"quote β space-relative-absolute.stderr', 'line\nbreak:*.stdout', 'maps/quote" β /main.js.map', 'nested/cargo-target/retain.txt']
            expected = {recipe.relative_to(root).as_posix(): recipe.read_bytes()}
            for name in names:
                path = publication / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b'raw\r\n\xff\x00')
                expected[path.relative_to(root).as_posix()] = path.read_bytes()
            excluded = publication / 'cargo-target/debug/build-output'
            excluded.parent.mkdir(parents=True)
            excluded.write_bytes(b'reproducible build material')
            result = owner.subprocess.run(['bash', '-e', '-c', script], env=env, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            with tarfile.open(archive) as retained:
                files = {item.name: retained.extractfile(item).read() for item in retained if item.isfile()}
            self.assertEqual(files, expected)
            # A packaging error remains a real CI failure.
            mock_bin = root / 'mock-bin'
            mock_bin.mkdir()
            tar = mock_bin / 'tar'
            tar.write_text('#!/bin/sh\nexit 7\n')
            tar.chmod(0o755)
            result = owner.subprocess.run(['bash', '-e', '-c', script], env=dict(env, PATH=str(mock_bin) + os.pathsep + env['PATH']), capture_output=True)
            self.assertEqual(result.returncode, 7)

    def test_nested_component_gate_keeps_required_material_and_fresh_proof_execution(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        linux = workflow.split('  build:\n', 1)[1]
        before, step = linux.split('      - name: Qualify nested component product APIs and four selected proofs\n', 1)
        step = step.split('      - name: Retain failed nested component product diagnostics', 1)[0]
        material = before.split('      - name: Prepare verified built-in proof material\n', 1)[1].split('      - name:', 1)[0]
        for k in (6, 7, 8):
            self.assertIn('--parameter-k ' + str(k), material)
        self.assertIn('record_compiler_recipe.py --compiler "$COMPACTC" --output "$RUNNER_TEMP/compact-nested-component-products-recipe"', step)
        self.assertIn('check_nested_component_products_proof.py --compiler-receipt "$RUNNER_TEMP/compact-nested-component-products-recipe/receipt.json"', step)
        self.assertIn('--ts-runtime "$TS_RUNTIME" --zkir "$ZKIR"', step)
        self.assertIn('shutil.which("zkir")', step)
        for weakened in ('--prepare-only', 'continue-on-error', 'if:', '/lib/zkir'):
            self.assertNotIn(weakened, step)
        self.assertIn("steps.nested_component_products.outcome == 'failure'", linux)
        self.assertIn('${{ runner.temp }}/compact-nested-component-products-recipe/', linux)
        self.assertIn('${{ runner.temp }}/compact-nested-component-products-proof/', linux)

    def test_full_linux_gate_uses_current_imported_compiler_and_complete_proofs(self):
        workflow = (owner.ROOT / '.github/workflows/build-compiler.yml').read_text()
        linux = workflow.split('  product-proofs:\n', 1)[1].split('  outer-vector-proof:\n', 1)[0]
        before, step = linux.split('      - name: Qualify complete nested-product APIs and selected proofs\n', 1)
        step = step.split('      - name: Retain failed nested-product', 1)[0]
        self.assertIn('--prepare-proof-material --parameter-k 6 --parameter-k 8', before)
        self.assertIn('record_compiler_recipe.py --compiler "$COMPACTC"', step)
        self.assertIn('check_vector_component_products_proof.py --compiler-receipt', step)
        self.assertIn('ZKIR="$(env -u LD_LIBRARY_PATH nix develop .#compiler --command python3', step)
        self.assertIn('shutil.which("zkir")', step)
        self.assertIn('--zkir "$ZKIR"', step)
        self.assertNotIn('/lib/zkir', step)
        self.assertNotIn('if:', step)
        self.assertNotIn('--prepare-only', step)
        self.assertNotIn('continue-on-error', step)
        self.assertIn('steps.vector_component_products.outcome == \'failure\'', linux)
        self.assertIn('vector-component-products-failure-${{ github.sha }}-attempt-${{ github.run_attempt }}', linux)
        self.assertGreaterEqual(linux.count('${{ runner.temp }}/compact-vector-component-products-proof/'), 2)
        license_job = workflow.split('  license-check:\n', 1)[1].split('  build:\n', 1)[0]
        self.assertIn('python3 -m unittest discover -s scripts -p test_header_regressions.py', license_job)
        self.assertLess(license_job.index('Test exact header-regression accounting'),
                        license_job.index('Reject new header violations on pull requests'))


if __name__ == '__main__':
    unittest.main()
