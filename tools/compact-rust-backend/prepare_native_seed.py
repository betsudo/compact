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

"""Plan one explicit trusted pilot seed only after genuine current native gates."""
import argparse
import os
from pathlib import Path
import re

import forge_ci
import qualify_linux_forge as finite

TRUSTED_REF = 'refs/heads/codex/issue-679-trusted-native-seed'
SNAPSHOT_NAME = 'forge-linux-supplied-snapshot'
LIMIT = 268435456
require = forge_ci.require


def authorize(event, ref, mode, publish, approval, restore):
    require(not publish or (event == 'workflow_dispatch' and ref == TRUSTED_REF and mode == 'linux-owner'),
            'seed publication is limited to the fixed explicit pilot ref/mode')
    require(not publish or not restore, 'seed publication cannot accompany a supplied restore tuple')
    require(not publish or bool(re.fullmatch('[A-Za-z0-9][A-Za-z0-9 .:_-]{0,159}', approval)),
            'explicit owner single-writer window approval label required')
    require(publish or not approval, 'window approval requires explicit publication')


def guard(plan):
    """Read-only current gates and keyed source/tool/parameter validation."""
    require(forge_ci.load(plan/'run.json').get('passed') is True
            and forge_ci.load(plan/'run.json').get('mode') == 'native-script', 'genuine native gate incomplete')
    for name in ('native-script', 'uncached'):
        require(forge_ci.load(plan/(name+'.command.json'))['exit_code'] == 0, name+' gate failed')
    finite.checked_lap(forge_ci.load(plan/'uncached-report.json'), None)
    invocation = forge_ci.load(plan/'invocation.json')
    key = forge_ci.load(plan/'key.json')
    require(bool(invocation.get('native')), 'native source-bound plan required')
    forge_ci.validate_invocation(invocation, key)
    forge_ci.check_tools(invocation)
    return invocation, key


def prepare(plan, snapshot, approval, env):
    authorize(env.get('GITHUB_EVENT_NAME'), env.get('GITHUB_REF'), 'linux-owner', True, approval, False)
    expected = Path(env['RUNNER_TEMP']) / SNAPSHOT_NAME
    require(snapshot.is_absolute() and snapshot == expected and not snapshot.exists() and not snapshot.is_symlink(),
            'fresh fixed restore-compatible seed snapshot path required')
    invocation, key = guard(plan)
    value = forge_ci.helper(invocation, plan, 'seed-prepare', 'prepare-save',
        '--key', plan/'key.json', '--cache-dir', plan/'cache', '--baseline', plan/'baseline.json',
        '--snapshot', snapshot, '--max-bytes', LIMIT, '--trusted-ref', TRUSTED_REF, '--tests-passed')
    if value['publish']:
        require(value['key'] == key['primary'] and value['snapshot']['path'] == str(snapshot)
                and value['snapshot']['partition'] == key['partition']
                and re.fullmatch('[0-9a-f]{64}', value['snapshot']['snapshot_id'])
                and 0 < value['snapshot']['bytes'] <= LIMIT, 'official seed identity/path exceeds plan')
    guard(plan)
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path)
    parser.add_argument('--guard-plan', type=Path)
    parser.add_argument('--snapshot', type=Path)
    parser.add_argument('--approval')
    parser.add_argument('--prepared', type=Path)
    parser.add_argument('--window', type=Path)
    args = parser.parse_args()
    if args.guard_plan is not None:
        require(all(value is None for value in (args.plan,args.snapshot,args.approval,args.prepared,args.window)),
                'guard mode accepts only its native plan')
        guard(args.guard_plan)
        return
    require(all(value is not None for value in (args.plan,args.snapshot,args.approval,args.prepared,args.window)),
            'publication preparation requires all arguments')
    value = prepare(args.plan, args.snapshot, args.approval, os.environ)
    forge_ci.write(args.prepared, value)
    forge_ci.write(args.window, dict(repository=os.environ['GITHUB_REPOSITORY'], run_id=os.environ['GITHUB_RUN_ID'],
        trusted_ref=TRUSTED_REF, approval=args.approval))


if __name__ == '__main__':
    main()
