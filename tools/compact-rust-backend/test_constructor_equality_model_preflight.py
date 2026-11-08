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

"""HOST admission controls of the actual preflight checker; no model authority."""
from copy import deepcopy
import ast
import inspect
import unittest
from unittest.mock import patch

import check_constructor_equality_proof_preflight as owner


class PreflightModelControls(unittest.TestCase):
    def cohort(self, count):
        specs = [dict(role="HOST-role-" + str(i)) for i in range(count)]
        rows = [dict(source=s['role'], circuit=entry, k=6) for s in specs
                for entry in ('read', 'advance', 'advance_other')]
        return dict(inventory=dict(circuits=rows), observed_models=count * 3, observed_k=[6]), specs

    def admit(self, models, specs, count, producer=None):
        observed = owner.check_model_geometry(models, count * 3)
        owner.check_model_membership(models, specs,
            dict(sources={s['role']: {} for s in specs} if producer is None else producer),
            count, observed)
        return observed

    def test_boundary_and_tuple_positive_cohorts_are_not_forced_to_k6(self):
        for count in (8, 16):
            with self.subTest(count=count):
                models, specs = self.cohort(count)
                self.assertIs(self.admit(models, specs, count), models['inventory']['circuits'])
                for i, row in enumerate(models['inventory']['circuits']):
                    row['k'] = i % 8 + 1
                models['observed_k'] = list(range(1, 9))
                self.admit(models, specs, count)

    def test_invalid_geometry_refuses_before_any_runner(self):
        for count in (8, 16):
            for k in (0, 9, -1, True, False, '6', 6.0, None):
                with self.subTest(count=count, k=k):
                    models, specs = self.cohort(count)
                    models['inventory']['circuits'][-1]['k'] = k
                    with self.assertRaisesRegex(ValueError, 'automatic tiny policy'):
                        self.admit(models, specs, count)

    def test_missing_and_extra_rows_refuse_before_membership(self):
        for count in (8, 16):
            for extra in (False, True):
                with self.subTest(count=count, extra=extra):
                    models, specs = self.cohort(count)
                    rows = models['inventory']['circuits']
                    rows.append(deepcopy(rows[0])) if extra else rows.pop()
                    with patch.object(owner, 'check_model_membership') as membership:
                        with self.assertRaisesRegex(ValueError, 'automatic tiny policy'):
                            self.admit(models, specs, count)
                    membership.assert_not_called()

    def test_closed_membership_counts_and_observed_k_set(self):
        for count in (8, 16):
            original, specs = self.cohort(count)
            for mutant in ('duplicate-row', 'unknown-role', 'unknown-entry',
                           'wrong-observed-count', 'wrong-observed-k', 'extra-spec',
                           'wrong-producer-role'):
                with self.subTest(count=count, mutant=mutant):
                    models, declarations = deepcopy(original), deepcopy(specs)
                    rows = models['inventory']['circuits']
                    producer = {s['role']: {} for s in specs}
                    if mutant == 'duplicate-row': rows[-1] = deepcopy(rows[0])
                    elif mutant == 'unknown-role': rows[-1]['source'] = 'HOST-unknown'
                    elif mutant == 'unknown-entry': rows[-1]['circuit'] = 'HOST-unknown'
                    elif mutant == 'wrong-observed-count': models['observed_models'] -= 1
                    elif mutant == 'wrong-observed-k': models['observed_k'] = [7]
                    elif mutant == 'extra-spec': declarations.append(dict(role='HOST-unknown'))
                    else:
                        producer.pop(declarations[0]['role'])
                        producer['HOST-unknown'] = {}
                    with self.assertRaisesRegex(ValueError, 'membership or geometry differs'):
                        self.admit(models, declarations, count, producer)

    def test_original_count_refusal_precedes_missing_producer_sources(self):
        for count in (8, 16):
            models, specs = self.cohort(count)
            observed = owner.check_model_geometry(models, count * 3)
            for wrong in ('observed-count', 'spec-count'):
                with self.subTest(count=count, wrong=wrong):
                    invalid = deepcopy(models)
                    declarations = deepcopy(specs)
                    if wrong == 'observed-count': invalid['observed_models'] -= 1
                    else: declarations.pop()
                    with self.assertRaisesRegex(ValueError, 'membership or geometry differs'):
                        owner.check_model_membership(invalid, declarations, {}, count, observed)
            with self.assertRaises(KeyError) as error:
                owner.check_model_membership(models, specs, {}, count, observed)
            self.assertEqual(error.exception.args, ('sources',))

    def test_main_keeps_checks_before_spec_load_and_maintained_runners(self):
        tree = ast.parse(inspect.getsource(owner.main))
        calls = [n for n in ast.walk(tree) if isinstance(n, ast.Call)]
        geometry = [n for n in calls if isinstance(n.func, ast.Name) and n.func.id == 'check_model_geometry']
        membership = [n for n in calls if isinstance(n.func, ast.Name) and n.func.id == 'check_model_membership']
        self.assertEqual(len(geometry), 1)
        self.assertEqual(len(membership), 1)
        g, m = geometry[0], membership[0]
        spec_loads = [n for n in ast.walk(tree) if isinstance(n, ast.Assign)
                      and any(isinstance(t, ast.Name) and t.id == 'specs' for t in n.targets)
                      and g.lineno < n.lineno < m.lineno]
        self.assertEqual(len(spec_loads), 1)
        runners = [n.lineno for n in calls if isinstance(n.func, ast.Name) and n.func.id == 'run']
        self.assertLess(m.lineno, min(runners))
        # Passing the whole producer preserves short-circuit evaluation inside
        # the predicate; passing producer['sources'] eagerly changes errors.
        self.assertIsInstance(m.args[2], ast.Name)
        self.assertEqual(m.args[2].id, 'producer')
        enclosing = [n for n in ast.walk(tree) if isinstance(n, ast.If)
                     and isinstance(n.test, ast.Name) and n.test.id == 'extended'
                     and g in list(ast.walk(n)) and m in list(ast.walk(n))]
        self.assertEqual(len(enclosing), 1)

    def test_all_preflight_modes_stay_exclusive(self):
        with self.assertRaisesRegex(ValueError, 'one finite cohort'):
            owner.main(boundaries=True, tuple_guards=True)


if __name__ == '__main__':
    unittest.main()
