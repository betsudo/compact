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


"""Pre-render code colors while preserving every source character."""
from html import escape, unescape
import json
from pathlib import Path
import re

from pygments.lexer import RegexLexer, inherit
from pygments.lexers import RustLexer, TypeScriptLexer
from pygments.token import Comment, Keyword, Name, Number, Operator, Punctuation, String, Text

REPO = Path(__file__).resolve().parents[2]
GRAMMAR = json.loads((
    REPO / 'editor-support/vsc/compact/syntaxes/compact.tmLanguage.json'
).read_text())['repository']


class CompactLexer(RegexLexer):
    """Use the repository's Compact editor keyword and built-in type rules."""

    tokens = {
        'root': [
            (r'\s+', Text.Whitespace),
            (r'//[^\n]*', Comment.Single),
            (r'/\*', Comment.Multiline, 'comment'),
            (r'"(?:\\.|[^"\\])*"', String.Double),
            *[(rule['match'], Keyword) for rule in GRAMMAR['keywords']['patterns']],
            *[(rule['match'], Keyword.Type) for rule in GRAMMAR['support']['patterns']],
            (r'\b0[xX][0-9a-fA-F]+\b', Number.Hex),
            (r'\b0[bB][01]+\b', Number.Bin),
            (r'\b0[oO][0-7]+\b', Number.Oct),
            (r'\b[0-9]+(?:_[0-9]+)*\b', Number.Integer),
            (r'[A-Za-z_][A-Za-z0-9_$]*(?=\s*\()', Name.Function),
            (r'\b[A-Z][A-Za-z0-9_$]*', Name.Class),
            (r'[A-Za-z_][A-Za-z0-9_$]*', Name),
            (r'[+\-*/%!=<>|&^~?:]+', Operator),
            (r'[{}()\[\],.;]', Punctuation),
            (r'.', Text),
        ],
        'comment': [
            (r'[^*/]+', Comment.Multiline),
            (r'/\*', Comment.Multiline, '#push'),
            (r'\*/', Comment.Multiline, '#pop'),
            (r'[*/]', Comment.Multiline),
        ],
    }


class TypeScriptExampleLexer(TypeScriptLexer):
    """Keep separated bigint literals together in the pinned lexer."""

    tokens = {'root': [(r'\b[0-9](?:_?[0-9])*n\b', Number.Integer), inherit]}


LEXERS = {
    'compact': ('Compact', CompactLexer()),
    'rust': ('Rust', RustLexer()),
    'typescript': ('TypeScript', TypeScriptExampleLexer()),
}
COLORS = (
    (Comment, 'comment'),
    (Keyword.Type, 'type'),
    (Keyword, 'keyword'),
    (String, 'string'),
    (Number, 'number'),
    (Name.Class, 'type'),
    (Name.Builtin, 'type'),
    (Name.Namespace, 'type'),
    (Name.Function, 'function'),
    (Name.Decorator, 'attribute'),
    (Name.Attribute, 'attribute'),
    (Operator, 'operator'),
)
FENCE = re.compile(
    r'<pre><code class="language-(compact|rust|typescript)">(.*?)</code></pre>',
    re.DOTALL,
)


def highlight_code_blocks(markup):
    def render(match):
        language, escaped = match.groups()
        label, lexer = LEXERS[language]
        source = unescape(escaped)
        tokens = list(lexer.get_tokens_unprocessed(source))
        if ''.join(value for _, _, value in tokens) != source:
            raise ValueError(f'{label} lexer changed the code example')
        fragments = []
        for _, token, value in tokens:
            color = next((color for group, color in COLORS if token in group), None)
            value = escape(value, quote=False)
            fragments.append(f'<span class="syn-{color}">{value}</span>' if color else value)
        return (
            f'<pre data-language="{label}"><code class="language-{language}">'
            + ''.join(fragments) + '</code></pre>'
        )
    return FENCE.sub(render, markup)
