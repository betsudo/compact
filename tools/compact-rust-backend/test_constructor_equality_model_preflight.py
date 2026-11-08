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
            {s['role']: {} for s in specs} if producer is None else producer,
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
                    with patch.object(owner.process_owner, 'bounded_command') as process, patch.object(owner.sdk, 'bounded_command') as sdk:
                        with self.assertRaisesRegex(ValueError, 'automatic tiny policy'):
                            self.admit(models, specs, count)
                    process.assert_not_called()
                    sdk.assert_not_called()

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
                    with patch.object(owner.process_owner, 'bounded_command') as process:
                        with self.assertRaisesRegex(ValueError, 'membership or geometry differs'):
                            self.admit(models, declarations, count, producer)
                    process.assert_not_called()

    def test_all_preflight_modes_stay_exclusive(self):
        with self.assertRaisesRegex(ValueError, 'one finite cohort'):
            owner.main(boundaries=True, tuple_guards=True)


if __name__ == '__main__':
    unittest.main()
