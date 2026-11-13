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


"""Protect source text and language boundaries in highlighted examples."""
from html import escape
from html.parser import HTMLParser
import unittest

from highlighting import highlight_code_blocks


class CodeText(HTMLParser):
    def __init__(self, markup):
        super().__init__()
        self.text = []
        self.classes = []
        self.tags = []
        self.feed(markup)

    def handle_starttag(self, tag, attrs):
        self.tags.append(tag)
        self.classes.append(dict(attrs).get('class'))

    def handle_data(self, data):
        self.text.append(data)


class HighlightingTests(unittest.TestCase):
    def render(self, language, source):
        markup = f'<pre><code class="language-{language}">{escape(source)}</code></pre>'
        return highlight_code_blocks(markup)

    def test_supported_examples_preserve_exact_text(self):
        examples = {
            'compact': (
                '// circuit words inside a comment\n'
                'export circuit run(x: Uint<16>): [] {\n'
                '  assert(disclose(x) == 7, "<script>& \\"return\\"");\n'
                '  const observed = private$secret_key();\n'
                '}\n'
            ),
            'rust': (
                "fn read<'a>(value: &'a str) -> &'a str {\n"
                '    let literal = r#"<script>& \\""#;\n'
                '    value // return 7 stays a comment\n'
                '}\n'
            ),
            'typescript': (
                'const amount: bigint = 7n;\n'
                'const label = `value ${amount}`;\n'
                'const safe = "<script>& return";\n'
                '// const words inside a comment\n'
            ),
        }
        for language, source in examples.items():
            with self.subTest(language=language):
                result = CodeText(self.render(language, source))
                self.assertEqual(''.join(result.text), source)
                self.assertTrue(set(result.tags) <= {'pre', 'code', 'span'})
                self.assertIn('syn-keyword', result.classes)
                self.assertIn('syn-comment', result.classes)
                self.assertIn('syn-string', result.classes)

    def test_whitespace_and_missing_final_newline(self):
        for source in ('', ' \n\t', 'const x = 7;\r\n', 'const x = 7;'):
            result = CodeText(self.render('typescript', source))
            self.assertEqual(''.join(result.text), source)

    def test_compact_comment_and_string_boundaries(self):
        for source, token in [
            ('"return circuit 7"', 'syn-string'),
            ('/* export circuit */', 'syn-comment'),
            ('// Uint<16> "a"\n', 'syn-comment'),
        ]:
            result = CodeText(self.render('compact', source))
            self.assertIn(token, result.classes)
            self.assertNotIn('syn-keyword', result.classes)
            self.assertNotIn('syn-type', result.classes)

    def test_compact_domain_types_and_private_witness_name(self):
        result = CodeText(self.render(
            'compact', 'witness private$key(): Bytes<32>;\n'
        ))
        self.assertIn('syn-type', result.classes)
        self.assertIn('syn-function', result.classes)
        self.assertEqual(''.join(result.text), 'witness private$key(): Bytes<32>;\n')

    def test_typescript_bigint_numeric_separator(self):
        result = self.render('typescript', 'const value = 65_536n;')
        self.assertIn('<span class="syn-number">65_536n</span>', result)
        self.assertEqual(''.join(CodeText(result).text), 'const value = 65_536n;')

    def test_plain_plans_and_inline_code_are_untouched(self):
        markup = (
            '<p><code>return &lt;Field&gt;</code></p>'
            '<pre><code class="language-text">If(read, write)</code></pre>'
        )
        self.assertEqual(highlight_code_blocks(markup), markup)


if __name__ == '__main__':
    unittest.main()
