---
format: aep.planning-md/1
id: review-result:public-support-source-pass2
kind: review-result
status: active
title: Public support implementation source attack pass 2
relations:
- reviews: story:review-public-support-claims
revision: 1
---
unit: story:review-public-support-claims; b5908e77a7fa8eafe5f8e387db431c967634cb67 plus one added test
verdict: nothing found
cases: executed 36→37, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none intentional; initial sccache refusal was not syscall-traced
needs-coordinator: record the wrapper-provenance correction; complete integration, rendered-site, release and delivery gates
```text
 crates/edge/ess-xtask/src/support.rs | 56 ++++++++++++++++++++++++++++++++++++
 1 file changed, 56 insertions(+)
```

This is the second and final source attack under adversary 0.8.0, covering the complete original eight-file unit from base `60279c34737863b59e9b46fa4fb07deaf1b010a3`. Unit: `home-path:sha256:acdb5db22a43eff8a5b5418e2870cf8f7793501f1b7f8d02b69df19b86520bbe`. The executable brief expressly permits additions in this implementation file's existing private `#[cfg(test)] mod tests`. Only one 56-line test was appended; every prior test and production byte remains an exact prefix. No source repair, documentation, Taskfile, binding, planning or Git mutation was made. The complete actual change is `attack.diff`; the full base...subject change is `subject.diff`, with the exact two-file correction delta in `correction.diff`.

1. Focused cases, in execution order

The new test at `crates/edge/ess-xtask/src/support.rs:878`, `adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file`, was written before any producer ran. It drives the actual CLI using a specification-file path, observes the adjacent README body by default, observes the replacement body with `--front-page`, and requires refusal for a missing explicit front page. Exact argv and output maps remain in each assigned `tmp/ess-support-adversary-front-page-override-*` fixture. This is a behavioral selection case, not a prose phrase test. All seven earlier adversarial cases remain unchanged and executed again.

Three different intermediate results must not be conflated:

- `focus-front-page`: Cargo exit 101 before compilation or tests. Unsetting RUSTC_WRAPPER did not override ancestor Cargo configuration; sccache rejected the longer inside-worktree temporary socket path. Zero cases executed.
- `focus-front-page-no-wrapper`: one selected case failed after a direct compiler build. My first fixture placed its sentinel solely in the leading H1. `ess-gen/src/authored.rs:56–84` splits that title from the body, and `ess-cli/src/site.rs:119–122` consumes the body. This was my fixture-oracle error, not a support-claim defect. `initial-case-source.rs`, complete original stdout/stderr and `initial-case-actual-binaries.json` retain that exact execution.
- `focus-front-page-body`: after moving the sentinels into Markdown body text, the same selection assertions passed, one case executed. The focused group then ran eight cases, all passed, including all 120 mutations across the 20 actual support rows. No case was skipped or weakened; only the newly authored fixture was corrected to measure selection.

Initial launch refusal, verbatim stderr (`commands/focus-front-page/stderr`):
```text
error: process didn't exit successfully: `/usr/bin/sccache home-path:sha256:1d953090b08462c2397465deee81a91200b2d5c56b895f4525dbd7f34ceeee8d -vV` (exit status: 2)
--- stderr
sccache: error: path must be shorter than SUN_LEN

```

Initial fixture failure: complete test-runner portion of stdout, verbatim (earlier build-script chatter remains in the complete stdout file):
```text
running 1 test
retained fixture: home-path:sha256:6b8ff6893006be2bc180f63f8d2b065034182b6efb402c46384112c13c8b314b
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... FAILED

failures:

failures:
    support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 35.85s

```

Panic/runner stderr, verbatim (all preceding verbose compiler commands remain in the complete stderr file):
```text
thread 'support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file' (1657469) panicked at crates/edge/ess-xtask/src/support.rs:893:9:
assertion failed: default_index.contains("ADJACENT_README_DEFAULT")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

Corrected focused test, complete stdout:
```text

running 1 test
retained fixture: home-path:sha256:3ac7f53e3fa8bb459c28dd281099bb0475a06be6e0b1921e450eac683fcc5b22
specification-file path uses adjacent README by default, explicit front page replaces it, and an absent explicit front page refuses
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.59s

```

2. Full package and local gate

Baseline 36 comes from the retained first-pass actual package output and the correction's explicit reuse record: 31 unit + 5 layout tests, with unchanged Rust bytes. It was not rerun as a baseline before writing the new case. The first package run after the focused cases and the final formatted-source run both executed 32 unit + 5 layout tests and passed. The first formatter reported four wrapping differences in my new test; only those were changed, and its original exit 1/output remain retained.

Final full package output follows verbatim:

Command: `home-path:sha256:dabeb2bcd0c107e3ac4da1ee67f2574e325891593b2f2d3542dccc0803641474 test --locked -p ess-xtask`

stdout, verbatim:
```text

running 32 tests
test support::tests::a_complete_source_block_is_accepted_without_owning_release_prose ... ok
test support::tests::command_inventory_requires_a_complete_nonempty_unique_section ... ok
test support::tests::cargo_version_changes_invalidate_only_the_source_block ... ok
test support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent ... ok
test support::tests::every_material_row_change_is_refused_with_its_location ... ok
test support::tests::html_requires_the_actual_output_root_and_both_nonempty_local_assets ... ok
test support::tests::missing_duplicated_or_reordered_block_markers_refuse ... ok
test support::tests::help_inventory_reads_both_clap_layouts_without_swallowing_neighbor_options ... ok
test support::tests::emitted_version_markers_are_parsed_and_must_agree_across_the_projection ... ok
test tests::a_named_but_untagged_version_is_not_an_incomplete_release ... ok
test tests::a_version_tag_whose_commit_never_reached_main_is_refused ... ok
test tests::a_version_tag_with_no_release_behind_it_is_refused ... ok
test tests::an_empty_release_is_refused_by_name ... ok
test tests::an_undated_release_is_refused_by_name ... ok
test tests::exclusions_cover_only_the_named_subtree ... ok
test tests::generated_paths_must_stay_below_the_projection_root ... ok
test support::tests::support_check_is_an_available_maintenance_command ... ok
test tests::only_bare_version_tags_are_release_tags ... ok
test tests::published_release_tags_come_from_the_json_report ... ok
test tests::release_notes_stop_before_the_next_release ... ok
test tests::release_publication_is_evaluated_after_every_dependency_finishes ... ok
test tests::the_release_record_is_checked_after_every_release_run ... ok
test tests::the_generated_index_includes_static_site_source ... ok
test tests::workspace_version_comes_only_from_the_workspace_package_table ... ok
test tests::sync_checks_and_reconciles_in_both_directions ... ok
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values ... ok
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text ... ok
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation ... ok
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags ... ok
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... ok
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots ... ok
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s


running 5 tests
test the_path_scan_reads_an_area_qualified_path ... ok
test every_workspace_crate_lives_under_an_area_directory ... ok
test the_path_scan_excludes_by_root_relative_path_and_reads_published_website_source ... ok
test the_path_scan_finds_at_least_one_path_in_this_repository ... ok
test every_literal_path_naming_a_workspace_crate_exists ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

```

stderr, verbatim:
```text
   Compiling ess-xtask v0.20.0 (home-path:sha256:90f5735c096ee7c4d65afd655ee3d8601ab5544ea3046dff69ad23dd597a2358)
    Finished `test` profile [unoptimized] target(s) in 0.90s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-8ee12e5f8af6c10d)
     Running tests/layout.rs (target/debug/deps/layout-ab47f6be39bab59a)
```

Direct exit 0.

| Producer receipt directory | Direct exit | Executed test summary | Free before | Free after |
|---|---:|---|---:|---:|
| commands/focus-front-page | 101 | no tests selected by this command | 11920470016 | 11920461824 |
| commands/focus-front-page-no-wrapper | 101 | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 31 filtered out; finished in 35.85s | 11917246464 | 11489202176 |
| commands/focus-front-page-body | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.59s | 11365523456 | 11358007296 |
| commands/focused-eight | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 2.21s | 11354226688 | 11335286784 |
| commands/package | 0 | test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.24s; test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s | 11333988352 | 11296927744 |
| commands/format-check | 1 | no tests selected by this command | 11296915456 | 11296903168 |
| commands/final-package | 0 | test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s; test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s | 11249033216 | 11198717952 |
| commands/final-format | 0 | no tests selected by this command | 11198631936 | 11197648896 |
| commands/clippy | 0 | no tests selected by this command | 11194753024 | 11143528448 |
| commands/support-check | 0 | no tests selected by this command | 11140349952 | 11123593216 |
| commands/shared-projections | 0 | no tests selected by this command | 11120119808 | 11115646976 |

Final formatting, strict `cargo clippy --locked -p ess-xtask --all-targets -- -D warnings`, actual `cargo xtask support --check`, and shared `cargo xtask generate --check` all exited 0. All raw outputs are retained in adjacent `stdout`/`stderr` files; receipts bind exact argv, cwd, controlled environment, source/HEAD, direct status, duration, disk measurements and resulting binaries. The final package stderr identifies the actual new test executables `ess_xtask-8ee12e5f8af6c10d` and `layout-ab47f6be39bab59a`. The first reused runner still inventoried prior fixed basenames; `initial-case-actual-binaries.json` explicitly supplies the correct initial executable identities from verbose output. Later runners derive test paths from actual Cargo output. No stale binary is counted as an executed test.

3. Source result and complete finite boundary

Nothing found. The first pass's sole source finding is resolved: the corrected paragraph states the adjacent README default, explicit override, and explicit selection of additional authored pages/downloads. The actual default/override/refusal observations agree with `site.rs:319–348`; CLI documentation at `main.rs:149–152` and explicit-site dispatch at :2370–2372 establish the route. Authored options require `--kind site` at :2344–2351. Existing positive default and nonselection controls also pass. Ordinary prose was reviewed against source and retained links; no phrase-matching test was fabricated.

The complete accepted 27 material rows and 21 validation obligations remain in scope, with all eight original changes rechecked through their exact identities and correction diff:

| Accepted material rows | Scope challenged and result |
|---|---|
| 1–3 | Actual source version; source-version mutation; dated retained release JSON/readback and asset-list limits. Offline source comparison remains independent of release metadata. |
| 4–9 | Actual default generator inventory, Markdown/Mermaid, explicit/combined HTML roots/assets, docs-ir JSON path/marker, and Schema/OpenAPI/AsyncAPI markers. All focused, support and shared-projection observations passed. |
| 10–15 | Actual import/project command inventories and unchanged OpenAPI accounting/refusal, Kubernetes credential/obligation, BuildKit and Helm owners. Public qualifiers retain bounded support and do not turn availability into semantic, apply or live-resource proof. |
| 16–18 | Actual four-target CLI metadata, loss-of-metadata refusal, unchanged Clap ArgMatches/handler/dependency boundaries and full-target Binary64 refusal owners. |
| 19–24 | Actual target/default/report/suite markers; unchanged qualification and strict-mode owners; legacy inconclusive coverage, current-source suite/5, replay and publisher-authentication limits remain distinct. Retained release-source absence receipts are existence results, not failed tests. |
| 25–27 | Runtime checks remain scoped to supplied identities/components/replicas/storage; executors remain explicit credential edges using supplied state; schema command inventory is current-source availability. |

All 60 cell changes, 20 removed rows, 20 duplicates, 19 neighboring row swaps and one extra row refused in the fresh focused execution. Existing checks also covered marker absence/duplication, malformed marker types and inconsistent emitted versions. Actual CLI refusal could not become a successful observation; lost target metadata could not borrow neighbor values. All output observations used this unit's actual Cargo/CLI route, with no Rust-token support search or second generator dispatcher.

Validation rows 1, 2, 4–9 were exercised through the focused cases and actual support/shared-projection commands. Row 3 retains authored-site support tests and adds measured default/override boundaries. Semantic/refusal owners for rows 10–17 remain byte-identical and were previously source-read; this package lane does not claim fresh execution of those broader packages or browser tests. Rows 18–20 (rendered site, complete integration, fresh remote release and Website/Atlas delivery) remain root-owned. Row 21 retains the unchanged public allowlist and exact implementation scope. No third source attack, new story, new registry, dependency or format was created.

4. Launch provenance correction

Cargo walks ancestor `.cargo/config.toml` files even with an isolated CARGO_HOME. `home-path:sha256:acd2952e2d81c785408e0d83690ab14a7df5bd58ff66a00731881d76f8fe98af` configures `/usr/bin/sccache` and the lld link flag. The supplied environment only unset RUSTC_WRAPPER. With root's explicit authorization, the retry uses `RUSTC_WRAPPER=""` and `RUSTC_WORKSPACE_WRAPPER=""`; the original environment/attempt remain unchanged. Both CARGO_BUILD wrapper counterparts and RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS were additionally recorded as absent by the final runner. `effective-environment.json`, `ancestor-cargo-config.toml` and `wrapper-correction.json` retain this exact boundary.

`direct-compiler-readback.json` accounts for all 67 direct pinned rustc command lines in the verbose retry, with no sccache command prefix. The earlier pass used the same unset-wrapper recipe and ancestor path, so its wrapper-free assertion was not established by its nonverbose records. Its historical per-process wrapper use and wrapper-origin outside writes cannot be reconstructed from those records alone. This report corrects that unsupported assertion; it does not alter the first-pass evidence or convert uncertainty into a claim of observed cache use. The fresh final source/package checks use the explicit empty overrides.

The initial pass-2 sccache refusal was not syscall-traced. This agent intentionally wrote no path outside the assigned unit; I cannot claim a complete outside-write history for that failed wrapper attempt. All intentional scratch, fixture, build and check writes are inside the unit. No successful wrapper-backed test execution is used for the final result. Root has separately corrected the integration launch policy and historical provenance wording.

5. Exact input and read accounting

The full charter, AGENTS, complete story/binding, original implementor handoff, first-pass report and correction handoff remain the authorities. This brief permits carrying previously inspected stable source forward by verified identity. All original eight-file changes and earlier source excerpts were carried forward that way; the complete correction and new test were read directly. `subject.diff`, `correction.diff`, `original-source.json` and `final-source.json` distinguish exact Git subject, final tests and historical sources.

`final-inspected-inputs.json` contains 88 exact input records, including full paths, SHA256, bytes and detailed read extents/selected line numbers. `tracked-inputs.json` hashes the complete 1,142-file tracked source/fixture closure (38,271,235 bytes); that whole-payload identity census does not assert semantic review of every unchanged repository line. The 549 original implementation/pass1/correction evidence entries retained identical complete native names, payloads and non-atime identity; all 73 earlier external TMP entries were also verified unchanged. Prior source/receipt excerpts are available in the immutable first-pass files and are not duplicated here.

The full frozen Rust 1.98.1 sysroot was reverified: 378 entries, every payload/mode/name, using the new assigned manifest. Each producer rechecked the pinned Cargo/rustc/rustdoc/formatter/Clippy hashes. No floating stable tool was executed. The current test executables differ from the prior fingerprints; four final executed CLI/package files, totaling 133,846,240 bytes, are independently copied under `final-binaries/` and hashed in `final-binaries.json`. Baseline copies were reused only after their complete current hashes matched the immutable first-pass copies. Exact initial failed-case source/binaries and full verbose output remain separate.

Exact input table follows. “Retained read” means the earlier detailed read extent was reverified by whole-payload identity, as permitted by this dispatch; exact original line selections and full metadata/prose read extents remain in `final-inspected-inputs.json`.

| Absolute path | Bytes | SHA256 | Extent |
|---|---:|---|---|
| home-path:sha256:4a67fc9f38ed314baad6a20a035f1e87db09e682f62776c2660e824f7b1ecd56 | 63660 | `04c7a77afd7539d0333f4e1f5282aceb224d49abb2dd97c1a1d4e4f77d35d211` | Retained read; exact identity verified; lines 3–14,148–196 |
| home-path:sha256:6d457353280d5cb46071dcdfc37ea92a6df4858447c5339e6d12099a9e258210 | 33730 | `8ca4848311f5c82eef7e170f3b8562fe0132b874b9b6f630f2bd28e7d7fc5842` | Retained read; exact identity verified |
| home-path:sha256:4cbb18188e19b3409d257ec7819c40299782a82610b73785d5397f03ac555882 | 3785 | `f806132b62b38f010175cfdcb8fbe627f25bf0d3aec08a923959224a66718c10` | Retained read; exact identity verified; lines 39–46 |
| home-path:sha256:a683efcb3cc39e942cec1a909996f2291f36d04add8360359cd094023373faee | 7315 | `3245661ae1bc707e5657abd4d7af13e1cb1e09806a95d684c5b315d7d4047e2c` | Retained read; exact identity verified; lines 38–46,95–131,144–160 |
| home-path:sha256:285fc4f9d2210b67a81c40c5959ea6bf5504e10ddbb1be92c5d50a589b431977 | 2340 | `af67e8dbbd482583c9c72afc2cf7800f134dcda7ac6cd129381e21aaace0df49` | Retained read; exact identity verified |
| home-path:sha256:33425ebbd484344767a03e83262a2f89c387121fe158ae405828ff3abdd1cb3e | 8137 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` | Retained read; exact identity verified; lines 203–205 |
| home-path:sha256:41a95f1d0e8c455fc3ca565ed21b6be9b42ee80a3ff2963ffb691b885baa69bd | 139710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | Retained read; exact identity verified; lines 140–157,379–384,449–450,453–461,501–520,576–580,584–601,615–638,640–649,1275–1279,1481–1486,1634–1745,2355–2379,2518–2522,2915–2968,2976–3058,3065–3075,3078–3091,3094–3099,3199–3207 |
| home-path:sha256:2eb82a0d86d6aae7f12ac0087d7ce12b00503814e2fe3d6c1132c8090628e7d7 | 10135 | `345f81fc51f463bf2e68e1f28ea2a90c48e5cc4ef4d45391b3e8476f8838a8c2` | Retained read; exact identity verified; lines 15–36,73–83 |
| home-path:sha256:ebe1a35b8f5cc46a5accb538715e95db4d74a4a591be0e37fc9d162b3d53d4b8 | 11531 | `1e179ed7f2684bfd3c51e7c231a27371cc21e0d294c0a05aadd23b29489843bd` | Retained read; exact identity verified; lines 67–71,152–155,310–350 |
| home-path:sha256:7f2f74e6ec80dac6eeba2bc74a6144dc04f9c9975badcd77b81e353624c0999a | 8138 | `542c3fcecae0bd29ab58de89e947da89fc96a3258dc7b31a80b31860b70abec4` | Retained read; exact identity verified; lines 85–111 |
| home-path:sha256:12dc0a243d3d1b7cd14b9adb3e51bd8233281ab675c3ae23eb200ea5e5a47899 | 6049 | `518bd521c0d361f080f552fdf1617b9ee3e7ba32a4fbc90077af7e98ae8a7a0b` | Retained read; exact identity verified; lines 5–34,62–117 |
| home-path:sha256:ba49195d715de33be242c3e79650ac833be8d4e62514ba688c31662d226012ac | 17051 | `27c79ffc2f5b74d9696fa0ffb1c245d0b3e46c9d2add1d613b7804e3bd62d7f9` | Retained read; exact identity verified; lines 9,211,311 |
| home-path:sha256:f602feaf850c17e8b75608ff44d47c014cebe53b0f544bf181ac375334f042d3 | 16006 | `6602c0ec47f090bbac6a0566fd9ded620f9d35494b3d814be1712982a5f7f424` | Retained read; exact identity verified; lines 246–299 |
| home-path:sha256:12d39830f4b8e3766fed9f01dafab6c92b674433f2491f17bf17803e7c29bb0a | 373 | `4f06e40288fd75aae6294ab55ef016d3d541edce2198b4f396ecaa0e7d180357` | Retained read; exact identity verified; lines 10–16 |
| home-path:sha256:55e04fe65df87063a6040390e2751c82168abc9f1c89652171aec772ff8d4506 | 38863 | `c583a1c3487f6ad9f726704c1d028c463ae10f5deaef39bbb863405e915b6b2c` | Retained read; exact identity verified; lines 105–144,466–535 |
| home-path:sha256:f25dd2680558fa693af2b786cc17d13ebaecb02a346ca7b44bcf0189d2973dce | 39781 | `57a1521f317fb1a22e7a204ef60cb295be176fc15a2fd73db3e04a2ee30554e3` | Complete 876-line pre-addition module unchanged; added case/diff read in full |
| home-path:sha256:c22c2a1c5618d1f6826a2e326b9946c85ff279f9e345877a9d69f958fcae2ee5 | 33627 | `b119a25325ba7e01bb533febbd6a59f4be26e122d72bfcdfc9fd1f8f4bc12138` | Retained read; exact identity verified; lines 482–531,622–685 |
| home-path:sha256:812f2a3e5cea93edd9dd1c805dd4cdf3b2f248e541a18a4616c0b87b3e1abfd5 | 43648 | `189c6584925154335c7c2c7b1777f79222c5e2391202ab115ec2d8ef8f4cbbc3` | Retained read; exact identity verified; lines 313–334,447–458 |
| home-path:sha256:addc79ffacb845e38d982f699a9f7e5c221620957a65b8e59bf893061cbec233 | 38193 | `30fc11197be98c2faefee61fe01b17d12d640410167fc59c0823b62dec356034` | Retained read; exact identity verified; lines 154,187–190 |
| home-path:sha256:59dc3822928f289574c55f863c3e5592667774f5d48c5fe16731b744cb1ec6e6 | 117964 | `1084bb9132beb7e7301e1896389039d18b0632fa180a5ed277734dd33a4debe6` | Retained read; exact identity verified; lines 69–105 |
| home-path:sha256:c71b55a41bd4e7fc4b1edf22fc65ef50cdbe738127cccba8f4ef5971c3ef93ce | 16611 | `bf3c6a1ec618a2e7269a4e1eef51e03d341608d427981b16dac38929a92a0d6c` | Retained read; exact identity verified; lines 54–55 |
| home-path:sha256:9816397f164058ea22fd412ba977cf7ee954fa4d77c4d7ded867ce3d9bb3ab07 | 41762 | `966e43244a9910acba981def0173681a881c9231538ef4d90ea899e819943aa3` | Retained read; exact identity verified; lines 51–58,625–642 |
| home-path:sha256:a4dcb2b4661c8d5e3ce0f50b7d5fd897271823433759e11e1dbc294c53fb06b0 | 3657 | `eb1b0e45d1c5ebb81dfb5e44b83e1417772d6935604fe1154d8323918c3a057e` | Retained read; exact identity verified; lines 46–59 |
| home-path:sha256:efe5c73bc2076c289c9de5b8cef3216c008c608b5c1219e09e0a83eeaa5e91a3 | 58326 | `5843677bd6366f299fa3801cfb6859b4f613299519bd8e172110c830bf455262` | Retained read; exact identity verified; lines 210,239–254 |
| home-path:sha256:5340953980d41b74744bff1349399a1cdee40d8281d29a06df1662aece5e1e42 | 10462 | `ab84031634a24d71e24643f321d7902230a4213b5964f0c7577e2903f6068f49` | Retained read; exact identity verified; lines 66,103–144 |
| home-path:sha256:f03763035c00c10c79b04fad5f2b415c71d7c3a9747bdb0a343c370204360e88 | 11074 | `65ada37ec0bd9bc917bf5a89b20c60742a0045b172447a99358c5324fe05b7b9` | Retained read; exact identity verified; lines 9–14,140–150,224–253 |
| home-path:sha256:7d9c99d07f57f35f3c7e8ba3d3a754ede97d7c86027d25e5b75e46ab476d8167 | 52059 | `52105880e6cb92a786a81053de8a7a82ad042e5cec2a6b32611da0cfac5ec9dd` | Retained read; exact identity verified; lines 495–508,1232–1240 |
| home-path:sha256:3745c741b80380c13a072b958dd120e069d8f8cb90b4cffc51139ed1b94ad108 | 11063 | `a885f1b32597fdb5323c7ee2d1fe754edb496de8c17124d5546b22ce0db7d168` | Retained read; exact identity verified; lines 15–60,77–88 |
| home-path:sha256:c1430320c41b7d4a3ae2360e1b945e5f70808b5832249f868a617d1c9b1f9d27 | 6391 | `b29f950bf6e7acc80d6912469a179da316fffd6e3ff488b24b887f711e8440d5` | Retained read; exact identity verified; lines 23–32,62–65,101–105 |
| home-path:sha256:80c2ad969994005ebe26f504c6cd2a21fcf5d8d579edee828b700c5ce86e7bbb | 18456 | `028b76d8d982388d1360e5cc886e010994623e4c369adf1303878e178918296c` | Retained read; exact identity verified; lines 453 |
| home-path:sha256:de13ea99c1d86f1ac7137661c831665c2b23fc16d778def9280ed5a09ec5a198 | 14718 | `d4e0b1da54d0a1c798c931c6251555805a209feeb75f9c03dccbd0fa760f2e7b` | Retained read; exact identity verified; lines 80–115 |
| home-path:sha256:0d843c954f20fca35df63323745ee00e38f8b74c86dbb026b122614aca3f2885 | 9950 | `c28481cf6d6ea2782b87145facac138c9d3b9502c315c574b27f6a567aca3c7f` | Retained read; exact identity verified; lines 249–279 |
| home-path:sha256:95e68e79e7fce3cea0e0ddb43c5958f5acda0fc9a84c2c4b1df6857e652fa89d | 44608 | `2d3356a2ba97f0492b4bfa55144cdee4719b80eb0ca7b904489c0f4dbcc0649e` | Retained read; exact identity verified; lines 50–84 |
| home-path:sha256:b9de196dfa6610578fb532097fb42ee81f1d4784cd04ac38b76cb1543b36a04c | 7832 | `e4ad23c6773b62cf59b56497b5620cb0b83abde5bbaaef2a4462f7f061fb4fc4` | Retained read; exact identity verified; lines 54–112 |
| home-path:sha256:3b977705f60388cb85380e20f0671724ccc0bdd3f8480292da9b7ecff5a64f15 | 3875 | `9adacde58ea017936b1501279fefbc54fa3e808a543eaed5985c2a54cc504b3d` | Retained read; exact identity verified; lines 8–35 |
| home-path:sha256:20ad71512ce97d952ff02cebfb6265ca50af551ca0fb5fa15942bc14ebd56670 | 66265 | `26324788584cf0ef89346f51cc936fa1b1901dbbf41444108aba4a81a926524b` | Retained read; exact identity verified; lines 552–574 |
| home-path:sha256:b0d352731352ea2181429a16027f0f9bd269e895b83a9ff31a423d57fab05cf7 | 24637 | `3abdb915961028a6dc532571d2cf3a1361404e2f128ec9cb37ff2d6ed5a72214` | Retained read; exact identity verified; lines 113–179,438–478 |
| home-path:sha256:73ab424636e61011c75ad78269615e144b3dc9b84a8199cf22bac083215b3da1 | 17936 | `291d118f03e11fbc301db50d3f5c5a0481d7d550201255865fd43ae96fbdb76d` | Retained read; exact identity verified; lines 12–15,319–345 |
| home-path:sha256:1296960875fc3a6b1a9775c5ed60b4785e52a16c0b474c00ecf90f3f41d88453 | 33620 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` | Retained read; exact identity verified; lines 358–366 |
| home-path:sha256:972c72928794a436df8eb8785e4e28747ac91c30b2d568d26c923c0a85fde56d | 8656 | `30a6a29bf8aabd3d62f6ba285dd4f4921ae6e7f86af77538ea4d27fb37cf0ba3` | Retained read; exact identity verified; lines 1,120–129 |
| home-path:sha256:70bf93b147c29fc4c54db14d8cc35b931e5c517d64c15edec8c59cccdf28b7b6 | 18021 | `c8a711875e7da7a57919966e75e3738125ef623deffbade32289675ccce0e2c6` | full correction diff and paragraph :156–165; complete original eight-file diff carried forward by verified identity |
| home-path:sha256:523b687a018b9aebdf34d006d92d37b88a174f32ccfdc702a5085612258f7703 | 8742 | `2ceb397d668905b076e18e19e0e0356d566d195ae66a85281adf14187f6fbf95` | Retained read; exact identity verified |
| home-path:sha256:736e1fa6c489e0d8dc1f7cbf9a9b6b4c28db5275925dab35d8671a62723fad1c | 9713 | `035e4897f23f69e7e1ba236c807c000405d88edfe2a2a999135ac629746db52b` | Retained read; exact identity verified; lines 145–150 |
| home-path:sha256:3cfd3915e0eff13be1325b3281b17fab9921cb916c6788023e69fea628e4853e | 16656 | `14d2339613e34e6087edbc080f0551406617a7688ed83ab32af18c348eb93f46` | Retained read; exact identity verified |
| home-path:sha256:4121cb065375b80886a45517df1c25b64ce683262b3cd1d8eb819bc0f1802736 | 38918 | `3301a66f41e6e3c6fcc0997dbc5889ddfaad9ae68b1307bcbedea16194e2b24e` | Retained read; exact identity verified; lines 8–15,39–79,163–176 |
| home-path:sha256:6288fb8aa5521e8b8a88b9bd929683a4eb79f85d182a41ece6f61aea1b854cf5 | 10520 | `197040308ec97e642410f468fc3a85cf4b6d29155a1740aef46e1669b3e90edc` | Retained read; exact identity verified |
| home-path:sha256:c96122e5dc9ddc3a492135cc66dbce946403db82c64d90e090e819af55427895 | 7816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | Retained read; exact identity verified |
| home-path:sha256:20d61068eea651b76789965ddb1eb60613e5d12922124a8f34e07e0a26bccf71 | 6754 | `882faaf36f46d29c89653e7fdf7642daad0a91e0f79559b836d7011923bc6b0f` | Retained read; exact identity verified |
| home-path:sha256:b4d56d0af490a2004b419bfcbedbeb4e090e1b70ffa85c159c15d346c02bd2b4 | 19869 | `79840752b56b0a0995866240040411a7467ea7087b4caa83ba08714cd74defa7` | Retained read; exact identity verified |
| home-path:sha256:45199365ad5c226913764f27feffa6aeab1c482cf57d2f80bfd7de710de2472c | 4595 | `6854212c33786f1c86251ec8edab83415ee07025eefbad90c052a3d6236e8c64` | Retained read; exact identity verified |
| home-path:sha256:48db003184005cbafa375708d51d54880db9445da4c7f4e9dd7f9731a173fbf5 | 3509 | `96719b0d9e049313032a7e6c735e15ff1c6484f62124e5250c651bf552b3f1eb` | Retained read; exact identity verified |
| home-path:sha256:1abe18c8e77f30302d9de72e8be5050c36ced983218aa1a0cd3f176ca1047164 | 87168 | `387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d` | Retained read; exact identity verified |
| home-path:sha256:cb84554fff1a3245666cebf25654645235ea7fbfd78416e8a3fcdb06a9fd30eb | 6722 | `ae393081a4e44209cf266a57a38e117b11d6e6dab1eb25cfa7ce0aec99a9dd91` | Retained read; exact identity verified |
| home-path:sha256:4914d8772ebbdec4b567c85b388092f966ca822d5b0d845922740c342029c006 | 41716 | `20f61ec77772038624a76beb43ebaa8289d95a9584cabe5c0100c60df608ed00` | Retained read; exact identity verified |
| home-path:sha256:0927e45bd8ddd6a53fb5f6254f234731fd1b02f54a92c19bafec17e18b142b6b | 3348 | `bc5643d58336211d1027d0ba7b98d97b4d1b11f68315776521b818272d63ee86` | Retained read; exact identity verified |
| home-path:sha256:c87e7bb54c7127c749db06b1cc2a9d280682559482c642a03f03a5d2d9994b5e | 19043 | `23b3450692eac66f29469b8dce5c7c49ec19cfcdda34514bbbe2a5ce4497b244` | Retained read; exact identity verified |
| home-path:sha256:ba248afd09565e94fdf66fd3e537f8728a4ef5e973147adfe6fa764ab6418556 | 7199 | `4842165c013f53307204fd26fe71e88107c6593a187f33c89a6589ebce121852` | Retained read; exact identity verified |
| home-path:sha256:53a4ff41c2197f97b4d711c3f4d22aaa5746ea64ae8a490722126a836ed841b3 | 558 | `614f81b0dbfb9831824ec67d25c84cacf4d1f50d5ea0d503c377a9d37461b482` | Retained read; exact identity verified |
| home-path:sha256:a47d4ff87cb9d4c77e4434bce5c3a7d611830a384cc18f4dd42040a4ad5f86ee | 13819 | `1fc9df1bf28e24c19a3372796fe2c400e57812237c6695a27a00e3c13cbe9f9e` | Retained read; exact identity verified |
| home-path:sha256:c9946c51ebe61cd4d30bc11d517c4430d13c1e67f6fb7ad1df6a3bcd889b5b90 | 13810 | `ea030972952a17c01f410a527ef4dd59f92766f627517c38096dc24baafdd328` | Retained read; exact identity verified |
| home-path:sha256:f79bfd0ca4f0aa2993a1417af285fb2caed942acdfcd01fcc5276f884f9dfe8d | 53 | `f151f199a14cb96356203380ebe3a999ac897365b507eddbd2f17d172a6c1ca2` | Retained read; exact identity verified |
| home-path:sha256:e82505babddc671cebf5b7f4c841d234b8ece153cba47dc082beaaf48b4a8210 | 19212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | Retained read; exact identity verified |
| home-path:sha256:71b24e9d3bf9d8fcd3188f17c1248126a63a5c09dea34b14de38266ea78cb68a | 8393 | `89ece113f6a00f75e05255e3f81f9ad258b80d64e4ed91efc1d5b44de779c53c` | Retained read; exact identity verified |
| home-path:sha256:2dd8371178f08a655a77bd8367f948945a419e9b65da56284e7b52c1c9b64f4e | 714 | `3ba3aac290074895cb94d45a87837941beb5d6a34eec596f0bb61c3f534dd5e8` | Retained read; exact identity verified |
| home-path:sha256:c23643b1a106a3b9a23fe5790a669f48d09f1a2c1b9d310ec1be279b10696725 | 18435 | `2ce2479a525e9ffbffb5778333109e16995d1a323df7fabc89c7365435f449a7` | Retained read; exact identity verified |
| home-path:sha256:a8b649d21e14a939f67bf05383606eeb65753fd6f68990691919cd1fa12052c5 | 62196 | `4ee501ce2b077d0a05febf8e53eff29302eecdae703b8be8973c9f9318d7d8fe` | Retained read; exact identity verified |
| home-path:sha256:a645852fe18e252d96a83fbbd1946b0ebaead51f07c502fa1d067ed4dbf98c98 | 120 | `f7fc44a9abf1420095647c5f77b7dcb8259ef11a6646fb0a23d2c2fba76a2bcb` | Retained read; exact identity verified |
| home-path:sha256:ecd17951e4b48a734eba11dc48a3016f270a4eb93ae16e00f68c4b8add607726 | 140226 | `f468192fd7aa10ca4b84a9615ef52e086ca5a649b0c355011209c3d1c8937cc6` | Retained read; exact identity verified |
| home-path:sha256:9d38d59b6004cad5053faf96e9d211db429efa9abef8142404270139ed662570 | 130 | `a08b9661603d15723f6f4d293941459399f9b164e29dbf044a629618d27f0f89` | Retained read; exact identity verified |
| home-path:sha256:036f2bd9c775d5c7ae3bc4684ddd3b9f3ad6270da6464995e7aa0380d8dbd437 | 970 | `ddc13436aaa9f9a0549dc48288f3053e0384ea6dca56f92d40bf1ca9dddc764f` | Retained read; exact identity verified |
| home-path:sha256:6283b80942d722ac4db83e3f15f89b1aaa7b786f62087f0f4f8bd8d82fc43448 | 257 | `d9b9d7ddb341d35b89eb71c53c251f5b47d3f3a11a0dba94954234c18e1b4382` | Retained read; exact identity verified |
| home-path:sha256:e9728a8b47d368849602a86306bf4febfc7c31a01704934c53a1d189640edd02 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Retained read; exact identity verified |
| home-path:sha256:cb042f84bc2b8a1097e344c30085198a8089e0253f1dc392aec108cfb936f6c1 | 119 | `6885346b1d437701244c3e3a3443ad34fdb7aaca04126a62723ed7891670e7db` | Retained read; exact identity verified |
| home-path:sha256:38acfe0fb73732858f57221b043c3151c87461e1f859dacf38a9682a436e8c82 | 5179 | `3bf36c63caff4e2e3043b2d7c6956ff541f92ebd86d6744d90b2b43555d970aa` | complete text/structured metadata read |
| home-path:sha256:ba1f6a1addc6377cea6578ce4301918753a3820e20c415d4e11a2c95952ef34e | 3657 | `048518fb65a86f6767352e146a36ab578dc66075ec429b75412d64118f4b415d` | complete text/structured metadata read |
| home-path:sha256:01b28fd1be2b83f186e29d79849fcbfe76774fe75ff679ab2b882731b1238af2 | 87168 | `387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d` | complete text/structured metadata read |
| home-path:sha256:7699cbca245666cadf0efc2f083cef1be7c5cf34b4bea2daf4012c49d4615d7d | 3632 | `01622e931b02d0f733a3d3fa3da09e40443181105cf80e0360d956cc82b852d8` | complete text/structured metadata read |
| home-path:sha256:b4c757dbbc2fcd24b0f63125f5a73def9279d5d9e9aad976dbb0d237e0129a65 | 1450 | `4d6700d474f859e19d32283a1fec045be169472dd4512cd142f8b3bb8b195985` | complete text/structured metadata read |
| home-path:sha256:fe85c38ffc975817e953002c280007cd927f9e263b907926c5bc449d96174b5f | 2136 | `d5abae38089279efcb84812a138da3a93d27ca09b3bd9d233f4e7af2059ec1dd` | complete text/structured metadata read |
| home-path:sha256:0f92665b87c1078ad369446161682c4bac8d44e356505bde77a6d19437b34e42 | 7419 | `6ae9980ff710f6dc0ba585576a9768974effff3f62de18bf1ea6e5da301414a9` | complete text/structured metadata read |
| home-path:sha256:3f4d78cd35b7f7f55ec74b89c1a38d65bfe3c1caac0bcfa57c233f7c647bb183 | 2308 | `bd1c245393acc1b5c777369559012a890f7cacf1b8603d887719db5a244ab501` | complete text/structured metadata read |
| home-path:sha256:7fe0ee1b20bb2d8016b95a7c452e56c230d2855ed4310e3fe5cf1a08fdbb96b5 | 628 | `185c486b0106756de6f1e4c039f6daf7c203f2ba6ceaf34cc82378fdd2ab5372` | complete text/structured metadata read |
| home-path:sha256:a5ab1a9bf44bf38d22139de4c0c94982e7d50b6fb8a35761abb370e1daf07bc1 | 1055 | `1e656e717e3ee0854d1bcfb3cfc3f0d8f94fd8c184c7b6a744bc99a7e00c1475` | complete text/structured metadata read |
| home-path:sha256:6860dfd0daa7b6a7023faca19e0f361e7af1233fc4df127b39403bcc186d4ec8 | 16806 | `2e329eadd0d76320f2f32e0cb41b91363320d6eff73e6f08ea4eb4b96f8cdabf` | whole Git payload identity; source :56–84 and :339–345 inspected for initial fixture-oracle error |
| home-path:sha256:acd2952e2d81c785408e0d83690ab14a7df5bd58ff66a00731881d76f8fe98af | 3163 | `e41aa5953bfbdb6337fe0a2642bd87723b6bd581504797160af2f2aaef444e29` | complete config text and mtime inspected; ancestor Cargo config explains unset-wrapper insufficiency |
| home-path:sha256:75d2c36ecb7596a932a1a7b8df6d886cb04b70090d833884fe8c04bff8595756 | 3690 | `141941c8452a239c47cebd2b81034dea5c0c8422ebc94d414b1efed69576f3a8` | complete structured effective environment read; two explicitly empty wrapper variables |
| home-path:sha256:3d29d1c0925fd822252cc04f41a5a96f457f35e7f03c535c34aa4f7e8f750e4e | 839 | `b622996d5336674f9b4da104969c86fc7dcd70ab9885624b2ed293f196938c50` | complete launch-refusal and historical limitation record |
| home-path:sha256:d591863c6ed2d9f623570ff859631c151a0ac2e915206f19970162872bb3bb8d | 543 | `33cc7bee06d39087c468643231338f5ddde74bce1be2626c22d1394e9690b454` | whole verbose output machine-inspected; all 67 direct rustc command lines checked and named |

6. Resource and quiescent handoff

The minimum recorded producer free space was 11115646976 bytes, above the 8,589,934,592-byte floor. `resource-growth.json` reports target payload growth separately from filesystem free space; the final check saw 1,819,769,451 target bytes, +627,007,559 against the first-pass target-only census. That includes new compiler fingerprints, fixtures and evidence; it is not all attributed to one producer. Later final input/binary-copy preparation measured 12,341,284,864 free before and 12,188,295,168 after. The increase in filesystem free space was an observation, not a deletion performed by this agent.

Assigned scratch: `home-path:sha256:ae987a2b672d28745b8f6a5d3ec716e25f979a57233bb7a249206bf3cbc0319b`. Assigned TMP: `home-path:sha256:488e6933097181b0c30e4c4d186ebd849f11faa8b49a0db09322e6c8ce3c0e08`. Outside scratch/TMP, the only intentional writes were the new test block in `home-path:sha256:f25dd2680558fa693af2b786cc17d13ebaecb02a346ca7b44bcf0189d2973dce`, this unit's `target/debug` build products and `target/ess-support-check` probe reports. There are no intentionally written outside-worktree paths. The untraced initial wrapper refusal limitation is stated above. The unchanged package test creates/removes its own disposable sync fixture inside the new assigned TMP; new behavior fixtures have no cleanup and remain retained. I issued no cleanup operation.

`native-census.json` inventories the entire unit target, including the new inside-scratch TMP, hidden files, empty directories, raw native path bytes, full lstat identity, regular payload hashes and literal symlink targets without traversal. It is a retained-state inventory, not a syscall history. Only the census and seal themselves are excluded to avoid self-reference; final creation changes their parent-directory metadata after a second complete stable name/identity check. Special files, if any, are explicitly metadata-only.

All owned producers have exited. The final seal binds this exact report, source/input/binary manifests, full outputs/receipts, intermediate failures and the complete census. After sealing, this agent relinquishes all source/target/scratch/TMP writes. No Git/store/lifecycle command, network operation, real executor, integration gate or publication was performed by this pass. Root owns recording and the eleven-lane/full delivery gate; this report claims neither approval nor verifier independence.

```findings
[]
```
