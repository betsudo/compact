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

"""Scoped test-launcher, matching-profile and materialization boundaries."""
import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
import unittest
from unittest.mock import patch

import forge_circuits as cache
import test_forge_cache as fixtures


class ForgeHarnessTests(unittest.TestCase):
    setUp = fixtures.ForgeCacheBoundaryTests.setUp
    validate = fixtures.ForgeCacheBoundaryTests.validate

    def test_unset_mode_keeps_ordinary_tools_without_forge_or_shim(self):
        env = {'PATH': '/ordinary', 'COMPACT_ZKIR_FORGE_BIN': '/absent/forge'}
        directory = self.root / 'unused-harness'
        with patch.object(cache, 'zkir_executable', side_effect=AssertionError('tool lookup')):
            configured = cache.configure_environment(env, directory)
        self.assertEqual(configured, env)
        self.assertFalse(directory.exists())
        argv = ['/ordinary/zkir', 'compile', 'current.zkir', 'prover', 'verifier']
        self.assertEqual(cache.command(argv, configured), argv)
        self.assertEqual(cache.provenance(configured)['mode'], 'uncached')
        self.assertTrue(cache.provenance(configured)['legacy_cli_keygen_on_generation_requests'])

    def test_wrapper_is_explicit_opt_in_and_uncached_remains_available(self):
        for flags, expected in (([], 'forge'), (['--uncached'], 'uncached')):
            with self.subTest(mode=expected), patch.dict(os.environ, {}, clear=True), \
                    patch.object(sys, 'argv', ['forge_circuits.py', *flags, '--', 'test-command']), \
                    patch.object(cache, 'configure_environment', side_effect=lambda env, directory: env) as configure, \
                    patch.object(cache.subprocess, 'call', return_value=7) as call:
                self.assertEqual(cache.main(), 7)
                self.assertEqual(configure.call_args.args[0]['COMPACT_ZKIR_MODE'], expected)
                self.assertEqual(call.call_args.args[0], ['test-command'])
                self.assertEqual(call.call_args.kwargs['env']['COMPACT_ZKIR_MODE'], expected)

    def test_new_current_circuit_is_not_limited_to_old_two_fixtures(self):
        self.source.write_bytes(b'new current circuit')
        (self.base / 'source').write_bytes(self.source.read_bytes())
        pin = cache.digest(self.source)
        self.value['export']['files'][0].update(sha256=pin, bytes=self.source.stat().st_size)
        self.manifest['source'].update(digest=pin, size=self.source.stat().st_size)
        (self.base / 'manifest.json').write_text(json.dumps(self.manifest))
        paths, _ = self.validate()
        self.assertEqual(paths['source'].read_bytes(), b'new current circuit')

    def test_exact_shared_git_and_registry_consumer_graph_with_optional_parser(self):
        lock = self.root / 'Cargo.lock'
        rows = [(name, version, cache.SOURCE if name in cache.GIT_PACKAGES else cache.REGISTRY)
                for name, version in cache.CONSUMER.items() if name != 'midnight-zkir']
        text = ''.join(f'[[package]]\nname="{n}"\nversion="{v}"\nsource="{s}"\n' for n, v, s in rows)
        lock.write_text(text)
        self.consumer_validator(lock)
        for wrong in (text.replace('8.1.3', '8.0.3'), text.replace(cache.SOURCE, cache.REGISTRY),
                      text.replace('version="0.7.1"', 'version="0.7.0"')):
            lock.write_text(wrong)
            with self.assertRaises(cache.CompatibilityError):
                self.consumer_validator(lock)

    def test_manifest_identity_and_duplicate_json_refuse(self):
        self.manifest['bundle_id'] = 'different'
        (self.base / 'manifest.json').write_text(json.dumps(self.manifest))
        with self.assertRaises(cache.CompatibilityError):
            self.validate()
        with self.assertRaises(cache.CompatibilityError):
            cache.read_json('{"profile":1,"profile":2}')

    def test_export_symlink_refuses_before_consumption(self):
        path = self.base / 'prover'
        path.unlink()
        path.symlink_to(self.base / 'verifier')
        with self.assertRaises(cache.CompatibilityError):
            self.validate()

    def test_context_restores_environment_and_routes_snapshot_compiles_only(self):
        actual = self.root / 'actual-zkir'
        actual.write_text('tool')
        env = cache.configure_environment({'PATH': '/unchanged', 'MIDNIGHT_ZKIR_FORGE_DIR': '/cache', 'COMPACT_ZKIR_MODE': 'forge'},
                                          self.root / 'harness', real_zkir=actual)
        self.assertEqual(env['COMPACT_ZKIR_REAL'], str(actual))
        self.assertEqual(env['MIDNIGHT_ZKIR_FORGE_DIR'], '/cache')
        self.assertEqual(cache.command(['/snapshot/zkir', 'compile', 'a', 'b', 'c'], env)[0], env['COMPACT_ZKIR_SHIM'])
        self.assertEqual(cache.command(['zkir-v3', 'compile', 'a', 'b', 'c'], env)[0], 'zkir-v3')
        self.assertEqual(cache.command(['/snapshot/zkir', '--version'], env)[0], '/snapshot/zkir')
        before = dict(os.environ)
        with patch.object(cache, 'configure_environment', return_value=dict(before, COMPACT_TEST_SENTINEL='scope')):
            with self.assertRaisesRegex(RuntimeError, 'exit'):
                with cache.test_environment(self.root / 'unused'):
                    self.assertEqual(os.environ['COMPACT_TEST_SENTINEL'], 'scope')
                    raise RuntimeError('exit')
        self.assertEqual(dict(os.environ), before)

    def test_uncached_mode_leaves_original_keygen_argv_unchanged(self):
        env = {'COMPACT_ZKIR_MODE': 'uncached', 'PATH': '/unchanged'}
        self.assertEqual(cache.configure_environment(env, self.root / 'not-created'), env)
        self.assertFalse((self.root / 'not-created').exists())
        argv = ['/snapshot/zkir', 'compile', 'a', 'b', 'c']
        self.assertEqual(cache.command(argv, env), argv)
        with self.assertRaises(cache.CompatibilityError):
            cache.configure_environment({'COMPACT_ZKIR_MODE': 'fallback'}, self.root / 'bad')

    def test_nested_harness_selects_actual_tool_not_outer_shim(self):
        env = {'COMPACT_ZKIR_REAL': '/real/zkir', 'PATH': '/outer/shim'}
        with patch.object(cache.shutil, 'which') as which:
            self.assertEqual(cache.zkir_executable(env), '/real/zkir')
        which.assert_not_called()

    def test_generated_launcher_really_forwards_non_keygen_cli(self):
        actual = self.root / 'actual-zkir'
        actual.write_text(f'#!{sys.executable}\nimport sys\nprint("actual " + " ".join(sys.argv[1:]))\n')
        actual.chmod(0o700)
        env = cache.configure_environment(dict(os.environ, COMPACT_ZKIR_MODE='forge'), self.root / 'harness', real_zkir=actual)
        result = subprocess.run([env['COMPACT_ZKIR_SHIM'], '--version'], env=env, check=True, capture_output=True, text=True)
        self.assertEqual(result.stdout.strip(), 'actual --version')

    def test_actual_bundle_launcher_preserves_scoped_shim_and_uncached_bundle_priority(self):
        bundle = self.root / 'bundle with spaces'
        binary, library = bundle / 'bin', bundle / 'lib'
        binary.mkdir(parents=True)
        library.mkdir()
        launcher = binary / 'compactc'
        shutil.copyfile(Path(cache.__file__).with_name('compactc-launcher.sh'), launcher)
        launcher.chmod(0o700)
        frontend = binary / 'compactc.bin'
        frontend.write_text('#!/usr/bin/env bash\nexec zkir "$@"\n')
        frontend.chmod(0o700)
        bundled = library / 'zkir'
        bundled.write_text('#!/usr/bin/env bash\nprintf "bundled %s\\n" "$*"\n')
        bundled.chmod(0o700)
        actual = self.root / 'actual-zkir'
        actual.write_text('#!/usr/bin/env bash\nprintf "scoped %s\\n" "$*"\n')
        actual.chmod(0o700)
        env = cache.configure_environment(dict(os.environ, COMPACT_ZKIR_MODE='forge'), self.root / 'harness', real_zkir=actual)
        cached = subprocess.run([str(launcher), '--version'], env=env, capture_output=True, text=True, check=True)
        self.assertEqual(cached.stdout.strip(), 'scoped --version')
        uncached = subprocess.run([str(launcher), '--version'], env=dict(env, COMPACT_ZKIR_MODE='uncached'),
                                  capture_output=True, text=True, check=True)
        self.assertEqual(uncached.stdout.strip(), 'bundled --version')
        refused = subprocess.run([str(launcher), '--version'], env=dict(env, COMPACT_ZKIR_SHIM='relative/zkir'),
                                 capture_output=True, text=True)
        self.assertEqual(refused.returncode, 2)
        self.assertIn('absolute zkir executable', refused.stderr)

    def test_many_uses_current_json_once_and_refuses_binary_only(self):
        sources = self.root / 'circuits'
        sources.mkdir()
        for name in ('b', 'a'):
            (sources / f'{name}.zkir').write_text('current')
        (sources / 'a.bzkir').write_text('older')
        with patch.dict(os.environ, COMPACT_ZKIR_REAL='/actual/zkir', COMPACT_ZKIR_MODE='forge'), patch.object(cache, 'compile_circuit') as compile:
            self.assertEqual(cache._shim(['compile-many', str(sources), str(self.root / 'keys')]), 0)
            self.assertEqual([row.args[0].name for row in compile.call_args_list], ['a.zkir', 'b.zkir'])
            (sources / 'c.bzkir').write_text('stale')
            with self.assertRaises(cache.CompatibilityError):
                cache._shim(['compile-many', str(sources), str(self.root / 'keys')])

    def test_empty_many_keeps_original_directory_semantics_without_preparation(self):
        sources = self.root / 'empty'
        sources.mkdir()
        with patch.dict(os.environ, COMPACT_ZKIR_REAL='/actual/zkir', COMPACT_ZKIR_MODE='forge'), patch.object(cache, 'compile_circuit') as compile:
            cache._shim(['compile-many', str(sources), str(self.root / 'keys')])
        self.assertTrue((self.root / 'keys').is_dir())
        compile.assert_not_called()

    def material(self):
        paths, _ = self.validate()
        return paths, {'persistent_export': str(self.base), 'artifacts': self.value['export']['files'],
                       'cache': {'hit': True}, 'k': 6, 'source_sha256': cache.digest(self.source)}

    def test_fresh_model_and_serialization_then_private_copies_preserve_exports(self):
        paths, receipt = self.material()
        def actual_cli(argv, **kwargs):
            if argv[1] == '--version':
                return subprocess.CompletedProcess(argv, 0, 'zkir 2.1.1\n', '')
            self.assertEqual(argv[1], 'mock-compile')
            self.source.with_suffix('.bzkir').write_bytes(paths['binary_ir'].read_bytes())
            return subprocess.CompletedProcess(argv, 0, '', 'Mock compiling circuit (k=6, rows=19)\n')
        prover, verifier = self.root / 'keys/circuit.prover', self.root / 'keys/circuit.verifier'
        with patch.object(cache, 'prepare', return_value=(paths, receipt)), patch.object(cache.subprocess, 'run', side_effect=actual_cli):
            result = cache.compile_circuit(self.source, prover, verifier, '/actual/zkir', {})
        self.assertEqual(result['rows'], 19)
        self.assertIn('not keygen', result['model_provider'])
        self.assertEqual(prover.read_bytes(), paths['prover'].read_bytes())
        prover.write_bytes(b'private corruption')
        self.assertEqual(cache.digest(paths['prover']), self.q['prover'])
        paths['prover'].write_bytes(b'corrupt export')
        with self.assertRaises(cache.CompatibilityError):
            cache._copy(paths['prover'], prover, receipt, {})

    def test_bad_serialization_or_destination_alias_refuses_without_keys(self):
        paths, receipt = self.material()
        self.source.with_suffix('.bzkir').write_bytes(b'wrong serialization')
        results = [subprocess.CompletedProcess([], 0, 'zkir 2.1.1', ''),
                   subprocess.CompletedProcess([], 0, '', '(k=6, rows=19)')]
        prover = self.root / 'keys/circuit.prover'
        with patch.object(cache, 'prepare', return_value=(paths, receipt)), patch.object(cache.subprocess, 'run', side_effect=results):
            with self.assertRaises(cache.CompatibilityError):
                cache.compile_circuit(self.source, prover, self.root/'keys/circuit.verifier', '/actual/zkir', {})
        self.assertFalse(prover.exists())
        with self.assertRaises(cache.CompatibilityError):
            cache.compile_circuit(self.source, self.source, self.root/'verifier', '/actual/zkir', {})


class PublicCompilerCacheArgumentsTests(unittest.TestCase):
    def test_public_mode_is_exclusive_and_has_no_skip_zk_or_second_preparation(self):
        import check_forge_circuit_cache as smoke
        base=["--compiler", "/compiler", "--proof-runner", "/proof-runner",
              "--params-dir", "/params", "--report", "/report"]
        ordinary=smoke.parse_args(base)
        qualification={"compact_source":"examples/tiny.compact"}
        self.assertIn("--skip-zk",smoke.compiler_command(ordinary,qualification,Path("/output")))
        public=smoke.parse_args([*base,"--compiler-cache-policy","/policy","--cache-dir","/cache"])
        command=smoke.compiler_command(public,qualification,Path("/output"))
        self.assertNotIn("--skip-zk",command)
        self.assertEqual(command[command.index("--zkir-cache")+1],"local")
        self.assertEqual(command[command.index("--zkir-cache-policy")+1],Path("/policy"))
        self.assertEqual(command[command.index("--zkir-cache-dir")+1],Path("/cache"))
        with self.assertRaises(SystemExit):
            smoke.parse_args([*base,"--compiler-cache-policy","/policy","--uncached-zkir","/zkir"])


class PublicCompilerAcquisitionTests(unittest.TestCase):
    def test_all_actual_routes_and_exact_circuit_names(self):
        import check_forge_circuit_cache as smoke
        for diagnostic, expected in [
            ("ZKIR local cache hit: actual; fresh k=6 model checked", "forge-hit"),
            ("ZKIR local cache miss: actual; fresh k=9 model checked", "forge-miss"),
            ("ZKIR ordinary key generation: actual; automatic tiny-circuit bypass", "auto-bypass"),
            ("ZKIR ordinary key generation: actual; explicit unavailable policy", "unavailable-keygen"),
        ]:
            with self.subTest(expected=expected):
                other = "ZKIR local cache hit: another; fresh k=6 model checked"
                self.assertEqual(smoke.acquisition_path(other + "\n" + diagnostic, "actual"), expected)
        self.assertEqual(smoke.acquisition_path("ZKIR local cache hit: a.b; fresh k=8 model checked", "a.b"), "forge-hit")

    def test_missing_malformed_or_ambiguous_decisions_refuse(self):
        import check_forge_circuit_cache as smoke
        valid = "ZKIR local cache hit: actual; fresh k=6 model checked"
        for text in ["", "ZKIR local cache hit: another; fresh k=6 model checked",
                     valid + " trailing", valid + "\n" + valid,
                     valid + "\nZKIR ordinary key generation: actual; automatic tiny-circuit bypass",
                     "ZKIR local cache hit: axb; fresh k=8 model checked"]:
            with self.subTest(text=text), self.assertRaises(cache.CompatibilityError):
                smoke.acquisition_path(text, "a.b" if "axb" in text else "actual")

    def test_stats_provider_identity_before_any_subprocess(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as private:
            root = Path(private)
            actual, other = root / "actual", root / "other"
            actual.write_bytes(b"provider")
            other.write_bytes(b"different provider")
            policy = root / "policy.json"
            policy.write_text(json.dumps({"forge": {"path": str(actual)}}))
            alias = root / "alias"
            alias.symlink_to(actual)
            with patch.object(smoke.subprocess, "run", side_effect=AssertionError("provider must not execute")):
                smoke.validate_stats_provider(alias, policy)
                with self.assertRaises(cache.CompatibilityError):
                    smoke.validate_stats_provider(other, policy)


class ExplicitFallbackQualificationTests(unittest.TestCase):
    def test_both_targets_and_exclusive_fallback_arguments(self):
        import check_forge_circuit_cache as smoke
        base = ["--compiler", "/compiler", "--proof-runner", "/proof", "--params-dir", "/params", "--report", "/report"]
        args = smoke.parse_args([*base, "--targets", "both", "--compiler-fallback-policy", "/policy"])
        command = smoke.compiler_command(args, {"compact_source": "examples/tiny.compact"}, Path("/output"))
        self.assertEqual([command[i + 1] for i, x in enumerate(command) if x == "--target"], ["ts", "rust"])
        self.assertNotIn("--skip-zk", command)
        self.assertEqual(command[command.index("--zkir-cache-policy") + 1], Path("/policy"))
        with self.assertRaises(SystemExit):
            smoke.parse_args([*base, "--compiler-fallback-policy", "/policy", "--compiler-cache-policy", "/other"])
        with self.assertRaises(SystemExit):
            smoke.parse_args([*base, "--compiler-fallback-policy", "/policy", "--uncached-zkir", "/zkir"])

    def test_missing_provider_and_invalid_policy_controls(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as private:
            root = Path(private)
            policy = root / "policy.json"
            absent = root / "missing-provider"
            valid = dict(schema=1, selection="force", unavailable="keygen", forge={"path": str(absent)})
            policy.write_text(json.dumps(valid))
            with patch.object(smoke.subprocess, "run", side_effect=AssertionError("no provider process")):
                self.assertEqual(smoke.validate_fallback_policy(policy), absent)
                existing = root / "existing"
                existing.write_bytes(b"provider")
                symlink = root / "dangling"
                symlink.symlink_to(absent)
                for change in ({"selection": "auto"}, {"unavailable": "refuse"},
                               {"forge": {}}, {"forge": {"path": "relative"}},
                               {"forge": {"path": str(existing)}}, {"forge": {"path": str(symlink)}},
                               {"forge": {"path": ""}}, {"schema": 2}):
                    with self.subTest(change=change):
                        policy.write_text(json.dumps({**valid, **change}))
                        with self.assertRaises(cache.CompatibilityError):
                            smoke.validate_fallback_policy(policy)
                policy.write_text('{"schema":1,"schema":1}')
                with self.assertRaises(ValueError):
                    smoke.validate_fallback_policy(policy)


class JsCounterQualificationTests(unittest.TestCase):
    def test_node_overrides_refuse_before_lookup_and_node_path_is_absolute(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        for key in ("NODE_OPTIONS", "NODE_PATH", "NODE_V8_COVERAGE"):
            with self.subTest(key=key), patch.object(smoke.shutil, "which", side_effect=AssertionError("must refuse before lookup")):
                with self.assertRaisesRegex(cache.CompatibilityError, "NODE_"):
                    smoke.js_node({key: "inherited override"})
        with tempfile.TemporaryDirectory() as private:
            node = Path(private) / "node"
            node.write_bytes(b"pinned executable")
            with patch.object(smoke.shutil, "which", return_value=str(node)):
                self.assertEqual(smoke.js_node({"PATH": private}), node.resolve())

    def test_swapped_preimage_requires_specific_failure(self):
        import check_forge_circuit_cache as smoke
        expected = "JS official preimage differs from replayed call proof inputs"
        smoke.check_js_preimage_negative(subprocess.CompletedProcess([], 1, "", expected))
        for code, error in ((0, expected), (2, expected), (1, "unrelated failure")):
            with self.subTest(code=code, error=error), self.assertRaises(cache.CompatibilityError):
                smoke.check_js_preimage_negative(subprocess.CompletedProcess([], code, "", error))

    def test_original_producer_inventory_refuses_mutated_and_missing_components(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as private:
            component = Path(private) / "implementation"
            component.write_bytes(b"actual producer bytes")
            actual = smoke.digest(component)
            execution = {"producer_files_sha256": {str(component): actual}}
            for field in ("runtime_entry", "onchain_entry", "onchain_wasm", "node_executable"):
                execution[field] = str(component)
                execution[field + "_sha256"] = actual
            self.assertEqual(smoke.validate_js_producer_files(execution), {str(component): actual})
            component.write_bytes(b"changed producer bytes")
            with self.assertRaisesRegex(cache.CompatibilityError, "changed"):
                smoke.validate_js_producer_files(execution)
            component.write_bytes(b"actual producer bytes")
            execution["node_executable"] = str(Path(private) / "unreported-node")
            with self.assertRaisesRegex(cache.CompatibilityError, "missing"):
                smoke.validate_js_producer_files(execution)
            for inventory in ({}, {"relative": actual}, {str(component): "not a digest"}):
                with self.subTest(inventory=inventory), self.assertRaises(cache.CompatibilityError):
                    smoke.validate_js_producer_files({**execution, "producer_files_sha256": inventory})

    def test_js_requires_both_targets(self):
        import check_forge_circuit_cache as smoke
        base = ["--compiler", "/compiler", "--proof-runner", "/proof", "--params-dir", "/params", "--report", "/report"]
        with self.assertRaises(SystemExit):
            smoke.parse_args([*base, "--js-runtime-root", "/runtime"])
        self.assertEqual(smoke.parse_args([*base, "--targets", "both", "--js-runtime-root", "/runtime"]).js_runtime_root, Path("/runtime"))
        self.assertIsNone(smoke.parse_args(base).js_runtime_root)

    def test_actual_package_identity_and_absolute_path(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as private:
            package = Path(private) / "package.json"
            package.write_text(json.dumps(dict(name="@midnight-ntwrk/compact-runtime", version="test-explicit")))
            self.assertEqual(smoke.validate_js_runtime(Path(private)), Path(private).resolve())
            for value in (dict(name="other", version="1"), dict(name="@midnight-ntwrk/compact-runtime")):
                package.write_text(json.dumps(value))
                with self.assertRaises(cache.CompatibilityError):
                    smoke.validate_js_runtime(Path(private))
            with self.assertRaises(cache.CompatibilityError):
                smoke.validate_js_runtime(Path("relative"))

    def test_both_exact_proof_markers_and_negative_attribution(self):
        import check_forge_circuit_cache as smoke
        passed = subprocess.CompletedProcess([], 0, "\n".join(smoke.JS_MARKERS), "")
        smoke.check_js_success(passed)
        for text in (smoke.JS_MARKERS[0], "\n".join((*smoke.JS_MARKERS, smoke.JS_MARKERS[0]))):
            with self.assertRaises(cache.CompatibilityError):
                smoke.check_js_success(subprocess.CompletedProcess([], 0, text, ""))
        message = "expected header tag 'midnight:verifier-key[v6]:'"
        smoke.check_js_negative(subprocess.CompletedProcess([], 1, "", message))
        for result in (subprocess.CompletedProcess([], 2, "", message), subprocess.CompletedProcess([], 1, "", "unrelated crash"), subprocess.CompletedProcess([], 0, "", message)):
            with self.assertRaises(cache.CompatibilityError):
                smoke.check_js_negative(result)


class LargeWorkloadTests(unittest.TestCase):
    def test_explicit_large_public_uncached_and_default_compatibility(self):
        import check_forge_circuit_cache as smoke
        base = ["--compiler", "/compiler", "--proof-runner", "/proof", "--params-dir", "/params", "--report", "/report"]
        default = smoke.parse_args(base)
        self.assertEqual(default.workload_set, "smoke")
        self.assertIn("--skip-zk", smoke.compiler_command(default, {"compact_source": "source"}, Path("/out")))
        args = smoke.parse_args([*base, "--workload-set", "ledger8-large", "--compiler-uncached"])
        command = smoke.compiler_command(args, {"compact_source": "source"}, Path("/out"))
        self.assertNotIn("--skip-zk", command)
        self.assertNotIn("--zkir-cache", command)
        for extra in (["--compiler-uncached"], ["--workload-set", "ledger8-large"],
                      ["--workload-set", "ledger8-large", "--compiler-uncached", "--compiler-cache-policy", "/policy"]):
            with self.assertRaises(SystemExit):
                smoke.parse_args([*base, *extra])

    def test_compile_failure_receipt_saved_before_acquisition(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / "failed.json"
            report = dict(passed=False, cases=[dict(circuit="schnorr", compile=dict(exit_code=7, stderr="original compile failure"))])
            smoke.write_report(destination, report, large=True, expected_acquisitions=9)
            saved = json.loads(destination.read_text())
            self.assertEqual(saved["cases"][0]["compile"]["stderr"], "original compile failure")
            self.assertFalse(saved["acquisition_accounting_complete"])
            self.assertEqual(sum(saved["acquisition_path_counts"].values()), 0)
            with self.assertRaises(cache.CompatibilityError):
                smoke.write_report(Path(temporary) / "false-pass.json", {**report, "passed": True}, large=True, expected_acquisitions=9)

    def test_nine_actual_routes_flatten_without_smoke_regression(self):
        import check_forge_circuit_cache as smoke
        rows = [dict(acquisition_paths={"verify": "forge-hit", "accept": "forge-hit"}),
                dict(acquisition_paths={str(i): "forge-hit" for i in range(7)})]
        self.assertEqual(smoke.acquisition_counts(rows, True)["forge-hit"], 9)
        self.assertEqual(smoke.acquisition_counts([dict(acquisition_path="forge-miss")])["forge-miss"], 1)
        with self.assertRaises(cache.CompatibilityError):
            smoke.acquisition_counts([dict(acquisition_paths={"one": "invented"})], True)

    def test_parameter_authority_and_bytes_frozen_before_compile(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            parameter, authority = root / "bls_midnight_2p8", root / "pins.json"
            parameter.write_bytes(b"official bytes")
            expected = cache.digest(parameter)
            authority.write_text(json.dumps(dict(parameters={"8": expected})))
            premises = smoke.large_parameter_premises(authority, root)
            self.assertEqual(premises[str(authority)], cache.digest(authority))
            self.assertEqual(premises[str(parameter)], expected)
            parameter.write_bytes(b"changed")
            with self.assertRaises(cache.CompatibilityError):
                smoke.large_parameter_premises(authority, root)
            parameter.write_bytes(b"official bytes")
            authority.write_text(json.dumps(dict(parameters={"8": "a" * 64})))
            with self.assertRaises(cache.CompatibilityError):
                smoke.large_parameter_premises(authority, root)
            authority.write_text(json.dumps(dict(parameters={"../bad": expected})))
            with self.assertRaises(cache.CompatibilityError):
                smoke.large_parameter_premises(authority, root)

    def test_publication_manifest_and_contract_identity_retained(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "compiler").mkdir()
            (root / "contract").mkdir()
            manifest, source = root / "compiler/contract-manifest.json", root / "contract/lib.rs"
            manifest.write_bytes(b"actual manifest bytes")
            source.write_bytes(b"actual generated source")
            before = smoke.large_publication_identity(root)
            self.assertEqual(before["manifest_sha256"], cache.digest(manifest))
            self.assertEqual(before["generated_contract_sha256"], {"contract/lib.rs": cache.digest(source)})
            manifest.write_bytes(b"changed manifest")
            self.assertNotEqual(smoke.large_publication_identity(root), before)
            manifest.write_bytes(b"actual manifest bytes")
            source.write_bytes(b"changed source")
            self.assertNotEqual(smoke.large_publication_identity(root), before)

    def test_complete_multicircuit_artifact_accounting(self):
        import check_forge_circuit_cache as smoke
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "zkir").mkdir()
            (root / "keys").mkdir()
            for name in ("first", "second"):
                for tree, suffix in (("zkir", ".zkir"), ("zkir", ".bzkir"), ("keys", ".prover"), ("keys", ".verifier")):
                    (root / tree / (name + suffix)).write_bytes((name + suffix).encode())
            hashes = smoke.large_artifacts(root, ["first", "second"])
            self.assertEqual(set(hashes), {"first", "second"})
            self.assertEqual(set(hashes["first"]), {"source", "binary_ir", "prover", "verifier"})
            with self.assertRaises(cache.CompatibilityError):
                smoke.large_artifacts(root, ["first"])
            (root / "keys/second.verifier").unlink()
            with self.assertRaises(OSError):
                smoke.large_artifacts(root, ["first", "second"])

    def test_all_proof_markers_once_and_exact_negative(self):
        import check_forge_circuit_cache as smoke
        smoke.exact_markers("first\nsecond\n", ["first", "second"])
        for output in ("first\n", "first\nsecond\nsecond\n"):
            with self.assertRaises(cache.CompatibilityError):
                smoke.exact_markers(output, ["first", "second"])
        smoke.check_js_negative(subprocess.CompletedProcess([], 1, "", "expected header tag 'midnight:verifier-key[v6]:'"))
        with self.assertRaises(cache.CompatibilityError):
            smoke.check_js_negative(subprocess.CompletedProcess([], 1, "", "unrelated failure"))


if __name__ == '__main__':
    unittest.main()
