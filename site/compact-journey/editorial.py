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

"""Editorial scope shared by the frozen catalogue and its refresh tools."""
import re

EXPERIMENT = re.compile(r'\b(?:zkir[-_ ]?)?forge\b', re.IGNORECASE)
EXPERIMENT_NUMBERS = {642, 643, 657, 679}
EXPERIMENT_REFERENCE = re.compile(r'(?:#|/(?:pull|issues)/)(?:642|643|657|679)\b')

# Experiment-only commits whose subjects do not name the tool.
EXPERIMENT_COMMITS = {
    '907458e7388925a46de9a9c7e7120aa6d959f3a6',
    'be2790ce724dd5d3a371d520e2fe64c2c265d7da',
    'ebdac5363a6e69801f61229cd25fe1230e15d3bf',
    '278f7c3ddb6d214fbd01ca7559904824cf865b0f',
    '390343b63d43fd39927a9be081cdb2a37cce8ffb',
    '922974b9dc34d217d06ce2762277a4d994363e14',
    'ccb1608d522907278a328a902686c5cb43c7e0f3',
    'f83dfad71b767bd3b76e666093333c00516cf96d',
    'fa9a7aebf90e22befbfb3157e70f4183795de478',
}


def include_commit(sha, subject):
    return sha not in EXPERIMENT_COMMITS and not EXPERIMENT.search(subject)


def include_metadata(row):
    return row['number'] not in EXPERIMENT_NUMBERS and not EXPERIMENT.search(row['title'])


def select_message(message, omitted_commits):
    """Return excerpts as written, omitting paragraphs about the experiment."""
    paragraphs = re.split(r'\n[ \t]*\n', message)
    return '\n\n'.join(
        paragraph for paragraph in paragraphs
        if not EXPERIMENT.search(paragraph)
        and not EXPERIMENT_REFERENCE.search(paragraph)
        and not any(sha in omitted_commits for sha in re.findall(r'\b[0-9a-f]{40}\b', paragraph))
    ).strip()


def select_snapshot(name, text):
    if name == 'ledger9-adoption.md':
        text = text.replace(
            'The Ledger 8 branch, historical fixture and proof evidence, and the\n'
            'deferred Forge branch were preserved.',
            'The Ledger 8 branch and historical fixture and proof evidence were preserved.',
        )
    return text
