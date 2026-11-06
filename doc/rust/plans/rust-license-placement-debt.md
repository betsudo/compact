# Existing Rust license placement debt

Snapshot: PR #705 revision `ba0758c331425cd4b333e9200b9449360f5cad41` (6 November 2026).
This is an inventory, not an exemption list or a full-tree compliance claim.
The 67 tracked files below pass the original full-template search but lack a
leading full license. Their bytes remain unchanged by this repair.

## Policy and repair

The PR placement gate checks new or modified tracked Rust sources. These existing
files do not fail while unchanged. **Editing one requires a leading full license
in the same PR.** Running `python3 add_headers.py` alone will not repair their
placement: the unchanged shared tool finds the existing template anywhere.

For an ordinary footer, mechanically move the existing complete comment block
before module docs, attributes and code; retain its copyright year and text.
For the crate-attribute case, move the existing comment before the attribute.
Do not change executable code or the generated-source owner as part of that move.
The retained-conformance generator is different: its `GENERATED_HEADER` literal
is program data and must stay intact. Do not modify this historical fixture just
to clear debt; any intentional future source revision needs its own file license
and must satisfy its historical source/proof-binding owner separately.

Run the regression gate against the actual frozen PR base after staging new
files (untracked files are outside the placement scan):

```sh
python3 -B scripts/check_header_regressions.py --base BASE_COMMIT_SHA \
  --report /tmp/compact-header-report-UNIQUE.json
```

A gate failure accompanied by a passing shared `--validate` is expected for
misplaced templates. `rust_placement_violations` in the JSON identifies those
paths. Source hashes below identify this historical snapshot; they grant no
permission to bypass the gate and are not an automatically maintained allowlist.

## Inventory

| Path | Placement | SHA-256 |
| --- | --- | --- |
| `tests-rust-backend/asset-registry-oracle/tests/asset_registry_oracle.rs` | after crate attribute (line 3) | `b3312cfa89277b3ce045f6682479edac6bc530757d586cf29e3e6cdf086f152d` |
| `tools/compact-rust-backend/src/pure_semantics/coercion/tests.rs` | footer (line 425) | `d2891f77f2788a243609a8e15b78baeef548255857e1d9be8cf0526f1e64d330` |
| `tools/compact-rust-backend/src/recorded/effect_plan/tests/component_product_vectors.rs` | footer (line 155) | `2dd1813fbc528c029c42896514beef52c6aafce8141a3736e2fc126b0cfe4f05` |
| `tools/compact-rust-backend/src/recorded/effect_plan/tests/outer_vector_products.rs` | footer (line 278) | `238ac80ea3e47c1bd7d11e116b0223634c081b0ea07ff262bdf7540f440e78c8` |
| `tools/compact-rust-backend/src/recorded/effect_plan/validate/domain/coercion_tests.rs` | footer (line 273) | `bd1859977313887a495aa1cf88ea2a58dce22aef2567f502ca42294fd7bd6644` |
| `tools/compact-rust-backend/src/stateful/expression/tests.rs` | footer (line 829) | `cc4255241de9be0d5a5787778d00f1488f746eef902cc225d5d4d8c7f77a620c` |
| `tools/compact-rust-backend/tests/aggregate_literal_projection_widening/consumer.rs` | footer (line 191) | `b0acb93c072eb6eb043c329b66268caa615a095c2b21cd249f30d2d714e30d6d` |
| `tools/compact-rust-backend/tests/aggregate_literal_projection_widening/field_result.rs` | footer (line 10) | `a1fa91ed22ec79afd11f710744ae4ef99138bba25e81bfd972b24af7f966b230` |
| `tools/compact-rust-backend/tests/aggregate_literal_projection_widening/program_reuse.rs` | footer (line 17) | `8cfe0d80a71a4d6289d251c4c299e6f18180e30df572c25b4c439cfe88cef664` |
| `tools/compact-rust-backend/tests/aggregate_literal_projection_widening/wrong_element.rs` | footer (line 10) | `bc6f437e295c1e425c5e3a7c872a4b37a06f2a3ecff295326e352ed9ec974a9d` |
| `tools/compact-rust-backend/tests/aggregate_literal_projection_widening/wrong_length.rs` | footer (line 10) | `d24333abf32d631830a83038028d9e226db899ece95c7cc6a87a1e48212171fe` |
| `tools/compact-rust-backend/tests/aggregate_literal_projection_widening/wrong_result_max.rs` | footer (line 10) | `f3be5d7c589e82c890113291a59158cb7a8adfc38de1639c61c405244f3561af` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/consumer.rs` | footer (line 545) | `bc03de8d542def63ab953308fd937e679f4f4b0a0998cfcf1a01ff3f20a99f63` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/context_reuse.rs` | footer (line 22) | `532d14c2f7bf52c791fa5b220d1c1550b6dccf9a4b951e560f5406758a992c8b` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/field_result.rs` | footer (line 20) | `7b261326f4384d7250c49b9f43a31d5c153515c89c66c05a8047702c0a7a8121` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/program_reuse.rs` | footer (line 22) | `ae73d2507c5bacd8ac6c783c25a1a11cb0d1fd558c440ba3ea26a87de19a12d2` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/wrong_element.rs` | footer (line 21) | `56a11a20e342a965d41a4c6f70550603c5c59c34152fd0ab3d907bc377748968` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/wrong_length.rs` | footer (line 20) | `2b9157fc0999676c25b377930b7e6b13752f8c6f69383b2e90b6b6c6f9e8e6e1` |
| `tools/compact-rust-backend/tests/direct_stateful_vector_literal_returns/wrong_result_max.rs` | footer (line 20) | `56bef1d4444126cd4435aea39c22e5948c4050aeb7c412e6c89240fdfb636883` |
| `tools/compact-rust-backend/tests/fold_vector_result_widening/consumer.rs` | footer (line 224) | `3195745df8d1b3af460b1511ed1efa1ac8bc88e363bda7b12fae082d854cf629` |
| `tools/compact-rust-backend/tests/fold_vector_result_widening/program_reuse.rs` | footer (line 17) | `99ce330ca858f1ba795fa5fd155b99d53ed3ec0c635dfb9eb219de677c8b86c6` |
| `tools/compact-rust-backend/tests/fold_vector_result_widening/wrong_element.rs` | footer (line 11) | `bcbca110001afff95e3900d7627fe3f63eddecd6ae960cf0cd68a076abf7781e` |
| `tools/compact-rust-backend/tests/fold_vector_result_widening/wrong_length.rs` | footer (line 11) | `23f747a8e047afb5cc6d94de35613f119ef24d272af5ade9e8e37f30a6a7da01` |
| `tools/compact-rust-backend/tests/fold_vector_result_widening/wrong_result_max.rs` | footer (line 10) | `bfc5c6ef80e09b308e1817aaddec276264f86fba3e365d5f1f4fbe0351f6ac54` |
| `tools/compact-rust-backend/tests/retained-conformance-6e37443b/tools/compact-rust-backend/src/lib.rs` | embedded generator string (line 24) | `0c76c00acb1a9097016782bbcf8a6268a054acfca150e6fd425f931db14db678` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/consumer.rs` | footer (line 420) | `0309a13a405d69ab88aa9c0e8a2c8123fb96189d18ae2512c1aadf14b160acf5` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/context_reuse.rs` | footer (line 22) | `2a0f1d5d68ee93e746d648783f0de7545093f693d2ebe022605c1757056e47be` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/field_result.rs` | footer (line 20) | `ea45419616f764111330ee91f3dff007a0dcdc4856e88aa4a74784b38aaf5062` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/program_reuse.rs` | footer (line 22) | `8e0eb62bf836e6a74b1adb239f1da04c3f8acaeabf3f5525bef7b84c452b417d` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/wrong_element.rs` | footer (line 21) | `6813372f4ae0798cc1d54c5712f86888f19f7ecc009ea45c543fea2675cd46b7` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/wrong_input_bound.rs` | footer (line 21) | `9e7fd83861fedc96af795b00eebbe3a8e32124c132db4202a2166adc49480d70` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/wrong_input_length.rs` | footer (line 21) | `fafd67940de4ec8a1b1dab389c32260cd717011cee72864b0bd4c31f3cb6fbe5` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/wrong_length.rs` | footer (line 20) | `725219d64f88db451723badff6be39d878ac072f23412db9eb3d1ec17e8129ea` |
| `tools/compact-rust-backend/tests/stateful_local_vector_widening/wrong_result_max.rs` | footer (line 20) | `6db15da215a7619c626771e5871267ea0b60432dc6475eb98a4f04afae80566c` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/consumer.rs` | footer (line 501) | `8a1fc98ac781991457eefac1fb35d1ad0dd041f87f88355ad541b9e51f4192ab` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/context_reuse.rs` | footer (line 13) | `7937d43df823e8d44156974ddff57e3a185c81214531c190aa96f3fb70c78ea5` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/field_result.rs` | footer (line 11) | `961e97c3bd09bd08ca3fc557d6d57ee332cd9a07e0483d3fe4e49d3cfa9ef855` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/program_reuse.rs` | footer (line 13) | `59138cf80435ebdb879adf7a07745c3984e9e11b0001949d825771e92f5e28d3` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_element.rs` | footer (line 11) | `c5310fa5b80648e4e24574f5826675cb2924d359afe5a814d8ba28c2005e158d` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_input_arity.rs` | footer (line 12) | `efa186d1e32ad02959c9a6d6af27deb1424ef1514d96c986fc7ae589d185520d` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_input_bound.rs` | footer (line 12) | `98267fd2a9fecef1eef3f99b7019036a705cad0fa91eb9505990e10e03816227` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_input_length.rs` | footer (line 12) | `ce0cc52cf0416e38bac5cb2e34c0d8d7e6446d1635fd75e8f0ef9c1f16b348d0` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_length.rs` | footer (line 11) | `81b9844edd8f13d054efe22f06db125ddced6fcfa66a0be6e2588c7c4b785e30` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_result_arity.rs` | footer (line 11) | `e768c4d78e6ec2a54dd90b6bfd8fe19db0d59ab4b498bcba7eaf5c5f4b4084c6` |
| `tools/compact-rust-backend/tests/stateful_product_vector_widening/wrong_result_max.rs` | footer (line 11) | `d856e342a317c211fa76251a0144a04186268d2186ebac364fc7a305f730a4c5` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/consumer.rs` | footer (line 434) | `b40c27627a09decc0bfca931f8ec9a6f3bca68038eaf59b007f5e9a9e6a8f232` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/context_reuse.rs` | footer (line 22) | `bcc9143bedb4322f16926efc1a02edfdf9a8fef19bb844695924d29501a4e4fc` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/field_result.rs` | footer (line 20) | `819763779eee7dc6df1c5007c588f32af920c6426af38ed41789d5874580924c` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/program_reuse.rs` | footer (line 22) | `0c5d698f7a883ae2d0331f0465ca3860d027d4ba7e889ec07a76c476b3cf935b` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/wrong_element.rs` | footer (line 21) | `1be1d7c42eadd843ace4036220271d29da192ffca0b33dcfd24c4bde416c8d27` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/wrong_input_bound.rs` | footer (line 21) | `3b7003b0acf8b5b64176277e2c0ae257a2fecc2663912d582a616941593a71ad` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/wrong_input_length.rs` | footer (line 21) | `195bde2acaf1ccfa0f4907292c9775a2ba6b819f7fa968b5bc90164a4d2c8ae0` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/wrong_length.rs` | footer (line 20) | `7c6916df3496eda03a4a6ab86f2c76d9c14dc87f63e325abc995ae09b7cf7ccb` |
| `tools/compact-rust-backend/tests/stateful_vector_value_widening/wrong_result_max.rs` | footer (line 20) | `3db4d16aa76950b656a1c4583b6af3ed5ca648cf94c20b125b5304591b655bc3` |
| `tools/compact-rust-backend/tests/struct_member_literal_widening/consumer.rs` | footer (line 242) | `df5e14b8c20be0a8ad8529794faeb2d30164edf7309a9bce2e1187713b5a1cc6` |
| `tools/compact-rust-backend/tests/struct_member_literal_widening/field_result.rs` | footer (line 11) | `0fb682504d9bc487a321379619b0f117c4a7c288edbbccbb824c4c1cb5c750fb` |
| `tools/compact-rust-backend/tests/struct_member_literal_widening/program_reuse.rs` | footer (line 17) | `95f4c3bae25dd3b0b31fd87d1c191c184a8c5c5527c303149ebd387aa1ec2aa5` |
| `tools/compact-rust-backend/tests/struct_member_literal_widening/wrong_element.rs` | footer (line 8) | `08ab16870533779aabfa13cafa022cdac69cfb2bbd66cd563d940de3607dca05` |
| `tools/compact-rust-backend/tests/struct_member_literal_widening/wrong_length.rs` | footer (line 11) | `54bc48120ca996396d279f8508c81ebce8436035786e403c1cf93ff449938662` |
| `tools/compact-rust-backend/tests/struct_member_literal_widening/wrong_result_max.rs` | footer (line 11) | `54134e4a10a797e578c04ac03b72404012d8ae75fa6f3578625a99f22955f1b8` |
| `tools/compact-rust-backend/tests/vector_alias_source_widening/consumer.rs` | footer (line 153) | `6b6f05f281230239ad89a6d4724909ec2ca17f801357e7b49c476dde43752e38` |
| `tools/compact-rust-backend/tests/vector_alias_source_widening/program_reuse.rs` | footer (line 17) | `8cfe0d80a71a4d6289d251c4c299e6f18180e30df572c25b4c439cfe88cef664` |
| `tools/compact-rust-backend/tests/vector_alias_source_widening/wrong_element.rs` | footer (line 10) | `701917225655d0b8e17496638ccc660d8650b73da4f96c5d7d9afdefd933e905` |
| `tools/compact-rust-backend/tests/vector_alias_source_widening/wrong_length.rs` | footer (line 10) | `89418621ee8ede5693cfe0bb37f66b2f1fd9622461315284ee636357fd8e6238` |
| `tools/compact-rust-backend/tests/vector_alias_source_widening/wrong_result_max.rs` | footer (line 10) | `ac9fda1cb801e475d5cace0645e8097da929ebb9cd7ffb647c1fe2d31a97878c` |
| `tools/compact-rust-proof-smoke/vector-product-elements-helper-sdk/build.rs` | footer (line 29) | `b0366abe35a27cd722ebff57c6c00213b138294dbc696a0c79a6f539822403e2` |
| `tools/compact-rust-proof-smoke/vector-product-elements-helper-sdk/lib.rs` | footer (line 13) | `74316306bd4a9619a0523dc3bb666322f0e9745f9e606e88337545311b988914` |
