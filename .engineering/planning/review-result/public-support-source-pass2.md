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

This is the second and final source attack under adversary 0.8.0, covering the complete original eight-file unit from base `60279c34737863b59e9b46fa4fb07deaf1b010a3`. Unit: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims`. The executable brief expressly permits additions in this implementation file's existing private `#[cfg(test)] mod tests`. Only one 56-line test was appended; every prior test and production byte remains an exact prefix. No source repair, documentation, Taskfile, binding, planning or Git mutation was made. The complete actual change is `attack.diff`; the full base...subject change is `subject.diff`, with the exact two-file correction delta in `correction.diff`.

1. Focused cases, in execution order

The new test at `crates/edge/ess-xtask/src/support.rs:878`, `adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file`, was written before any producer ran. It drives the actual CLI using a specification-file path, observes the adjacent README body by default, observes the replacement body with `--front-page`, and requires refusal for a missing explicit front page. Exact argv and output maps remain in each assigned `tmp/ess-support-adversary-front-page-override-*` fixture. This is a behavioral selection case, not a prose phrase test. All seven earlier adversarial cases remain unchanged and executed again.

Three different intermediate results must not be conflated:

- `focus-front-page`: Cargo exit 101 before compilation or tests. Unsetting RUSTC_WRAPPER did not override ancestor Cargo configuration; sccache rejected the longer inside-worktree temporary socket path. Zero cases executed.
- `focus-front-page-no-wrapper`: one selected case failed after a direct compiler build. My first fixture placed its sentinel solely in the leading H1. `ess-gen/src/authored.rs:56–84` splits that title from the body, and `ess-cli/src/site.rs:119–122` consumes the body. This was my fixture-oracle error, not a support-claim defect. `initial-case-source.rs`, complete original stdout/stderr and `initial-case-actual-binaries.json` retain that exact execution.
- `focus-front-page-body`: after moving the sentinels into Markdown body text, the same selection assertions passed, one case executed. The focused group then ran eight cases, all passed, including all 120 mutations across the 20 actual support rows. No case was skipped or weakened; only the newly authored fixture was corrected to measure selection.

Initial launch refusal, verbatim stderr (`commands/focus-front-page/stderr`):
```text
error: process didn't exit successfully: `/usr/bin/sccache /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/rustc -vV` (exit status: 2)
--- stderr
sccache: error: path must be shorter than SUN_LEN

```

Initial fixture failure: complete test-runner portion of stdout, verbatim (earlier build-script chatter remains in the complete stdout file):
```text
running 1 test
retained fixture: /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/tmp/ess-support-adversary-front-page-override-1657468-1788759232782244253
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
retained fixture: /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/tmp/ess-support-adversary-front-page-override-1670514-1788759425260647860
specification-file path uses adjacent README by default, explicit front page replaces it, and an absent explicit front page refuses
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.59s

```

2. Full package and local gate

Baseline 36 comes from the retained first-pass actual package output and the correction's explicit reuse record: 31 unit + 5 layout tests, with unchanged Rust bytes. It was not rerun as a baseline before writing the new case. The first package run after the focused cases and the final formatted-source run both executed 32 unit + 5 layout tests and passed. The first formatter reported four wrapping differences in my new test; only those were changed, and its original exit 1/output remain retained.

Final full package output follows verbatim:

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask`

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
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask)
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

Cargo walks ancestor `.cargo/config.toml` files even with an isolated CARGO_HOME. `/home/timo/.cargo/config.toml` configures `/usr/bin/sccache` and the lld link flag. The supplied environment only unset RUSTC_WRAPPER. With root's explicit authorization, the retry uses `RUSTC_WRAPPER=""` and `RUSTC_WORKSPACE_WRAPPER=""`; the original environment/attempt remain unchanged. Both CARGO_BUILD wrapper counterparts and RUSTFLAGS/CARGO_ENCODED_RUSTFLAGS were additionally recorded as absent by the final runner. `effective-environment.json`, `ancestor-cargo-config.toml` and `wrapper-correction.json` retain this exact boundary.

`direct-compiler-readback.json` accounts for all 67 direct pinned rustc command lines in the verbose retry, with no sccache command prefix. The earlier pass used the same unset-wrapper recipe and ancestor path, so its wrapper-free assertion was not established by its nonverbose records. Its historical per-process wrapper use and wrapper-origin outside writes cannot be reconstructed from those records alone. This report corrects that unsupported assertion; it does not alter the first-pass evidence or convert uncertainty into a claim of observed cache use. The fresh final source/package checks use the explicit empty overrides.

The initial pass-2 sccache refusal was not syscall-traced. This agent intentionally wrote no path outside the assigned unit; I cannot claim a complete outside-write history for that failed wrapper attempt. All intentional scratch, fixture, build and check writes are inside the unit. No successful wrapper-backed test execution is used for the final result. Root has separately corrected the integration launch policy and historical provenance wording.

5. Exact input and read accounting

The full charter, AGENTS, complete story/binding, original implementor handoff, first-pass report and correction handoff remain the authorities. This brief permits carrying previously inspected stable source forward by verified identity. All original eight-file changes and earlier source excerpts were carried forward that way; the complete correction and new test were read directly. `subject.diff`, `correction.diff`, `original-source.json` and `final-source.json` distinguish exact Git subject, final tests and historical sources.

`final-inspected-inputs.json` contains 88 exact input records, including full paths, SHA256, bytes and detailed read extents/selected line numbers. `tracked-inputs.json` hashes the complete 1,142-file tracked source/fixture closure (38,271,235 bytes); that whole-payload identity census does not assert semantic review of every unchanged repository line. The 549 original implementation/pass1/correction evidence entries retained identical complete native names, payloads and non-atime identity; all 73 earlier external TMP entries were also verified unchanged. Prior source/receipt excerpts are available in the immutable first-pass files and are not duplicated here.

The full frozen Rust 1.98.1 sysroot was reverified: 378 entries, every payload/mode/name, using the new assigned manifest. Each producer rechecked the pinned Cargo/rustc/rustdoc/formatter/Clippy hashes. No floating stable tool was executed. The current test executables differ from the prior fingerprints; four final executed CLI/package files, totaling 133,846,240 bytes, are independently copied under `final-binaries/` and hashed in `final-binaries.json`. Baseline copies were reused only after their complete current hashes matched the immutable first-pass copies. Exact initial failed-case source/binaries and full verbose output remain separate.

Exact input table follows. “Retained read” means the earlier detailed read extent was reverified by whole-payload identity, as permitted by this dispatch; exact original line selections and full metadata/prose read extents remain in `final-inspected-inputs.json`.

| Absolute path | Bytes | SHA256 | Extent |
|---|---:|---|---|
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/CHANGELOG.md | 63660 | `04c7a77afd7539d0333f4e1f5282aceb224d49abb2dd97c1a1d4e4f77d35d211` | Retained read; exact identity verified; lines 3–14,148–196 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/Cargo.lock | 33730 | `8ca4848311f5c82eef7e170f3b8562fe0132b874b9b6f630f2bd28e7d7fc5842` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/Cargo.toml | 3785 | `f806132b62b38f010175cfdcb8fbe627f25bf0d3aec08a923959224a66718c10` | Retained read; exact identity verified; lines 39–46 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/Taskfile.yml | 7315 | `3245661ae1bc707e5657abd4d7af13e1cb1e09806a95d684c5b315d7d4047e2c` | Retained read; exact identity verified; lines 38–46,95–131,144–160 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/b10x.docs.yaml | 2340 | `af67e8dbbd482583c9c72afc2cf7800f134dcda7ac6cd129381e21aaace0df49` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/coverage.rs | 8137 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` | Retained read; exact identity verified; lines 203–205 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/main.rs | 139710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | Retained read; exact identity verified; lines 140–157,379–384,449–450,453–461,501–520,576–580,584–601,615–638,640–649,1275–1279,1481–1486,1634–1745,2355–2379,2518–2522,2915–2968,2976–3058,3065–3075,3078–3091,3094–3099,3199–3207 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/schema.rs | 10135 | `345f81fc51f463bf2e68e1f28ea2a90c48e5cc4ef4d45391b3e8476f8838a8c2` | Retained read; exact identity verified; lines 15–36,73–83 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/site.rs | 11531 | `1e179ed7f2684bfd3c51e7c231a27371cc21e0d294c0a05aadd23b29489843bd` | Retained read; exact identity verified; lines 67–71,152–155,310–350 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/authored_site.rs | 8138 | `542c3fcecae0bd29ab58de89e947da89fc96a3258dc7b31a80b31860b70abec4` | Retained read; exact identity verified; lines 85–111 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/count_reports.rs | 6049 | `518bd521c0d361f080f552fdf1617b9ee3e7ba32a4fbc90077af7e98ae8a7a0b` | Retained read; exact identity verified; lines 5–34,62–117 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/coverage_browser.rs | 17051 | `27c79ffc2f5b74d9696fa0ffb1c245d0b3e46c9d2add1d613b7804e3bd62d7f9` | Retained read; exact identity verified; lines 9,211,311 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/coverage_cli.rs | 16006 | `6602c0ec47f090bbac6a0566fd9ded620f9d35494b3d814be1712982a5f7f424` | Retained read; exact identity verified; lines 246–299 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/Cargo.toml | 373 | `4f06e40288fd75aae6294ab55ef016d3d541edce2198b4f396ecaa0e7d180357` | Retained read; exact identity verified; lines 10–16 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/src/main.rs | 38863 | `c583a1c3487f6ad9f726704c1d028c463ae10f5deaef39bbb863405e915b6b2c` | Retained read; exact identity verified; lines 105–144,466–535 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/src/support.rs | 39781 | `57a1521f317fb1a22e7a204ef60cb295be176fc15a2fd73db3e04a2ee30554e3` | Complete 876-line pre-addition module unchanged; added case/diff read in full |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-deployment/src/runtime.rs | 33627 | `b119a25325ba7e01bb533febbd6a59f4be26e122d72bfcdfc9fd1f8f4bc12138` | Retained read; exact identity verified; lines 482–531,622–685 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-deployment/tests/deployment.rs | 43648 | `189c6584925154335c7c2c7b1777f79222c5e2391202ab115ec2d8ef8f4cbbc3` | Retained read; exact identity verified; lines 313–334,447–458 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/asyncapi.rs | 38193 | `30fc11197be98c2faefee61fe01b17d12d640410167fc59c0823b62dec356034` | Retained read; exact identity verified; lines 154,187–190 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/docs.rs | 117964 | `1084bb9132beb7e7301e1896389039d18b0632fa180a5ed277734dd33a4debe6` | Retained read; exact identity verified; lines 69–105 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/document.rs | 16611 | `bf3c6a1ec618a2e7269a4e1eef51e03d341608d427981b16dac38929a92a0d6c` | Retained read; exact identity verified; lines 54–55 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/html.rs | 41762 | `966e43244a9910acba981def0173681a881c9231538ef4d90ea899e819943aa3` | Retained read; exact identity verified; lines 51–58,625–642 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/lib.rs | 3657 | `eb1b0e45d1c5ebb81dfb5e44b83e1417772d6935604fe1154d8323918c3a057e` | Retained read; exact identity verified; lines 46–59 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/openapi.rs | 58326 | `5843677bd6366f299fa3801cfb6859b4f613299519bd8e172110c830bf455262` | Retained read; exact identity verified; lines 210,239–254 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/schema.rs | 10462 | `ab84031634a24d71e24643f321d7902230a4213b5964f0c7577e2903f6068f49` | Retained read; exact identity verified; lines 66,103–144 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-openapi/src/accounting.rs | 11074 | `65ada37ec0bd9bc917bf5a89b20c60742a0045b172447a99358c5324fe05b7b9` | Retained read; exact identity verified; lines 9–14,140–150,224–253 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-openapi/src/lib.rs | 52059 | `52105880e6cb92a786a81053de8a7a82ad042e5cec2a6b32611da0cfac5ec9dd` | Retained read; exact identity verified; lines 495–508,1232–1240 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-openapi/tests/accounting.rs | 11063 | `a885f1b32597fdb5323c7ee2d1fe754edb496de8c17124d5546b22ce0db7d168` | Retained read; exact identity verified; lines 15–60,77–88 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/src/clap/mod.rs | 6391 | `b29f950bf6e7acc80d6912469a179da316fffd6e3ff488b24b887f711e8440d5` | Retained read; exact identity verified; lines 23–32,62–65,101–105 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/src/clap/tree.rs | 18456 | `028b76d8d982388d1360e5cc886e010994623e4c369adf1303878e178918296c` | Retained read; exact identity verified; lines 453 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/src/lib.rs | 14718 | `d4e0b1da54d0a1c798c931c6251555805a209feeb75f9c03dccbd0fa760f2e7b` | Retained read; exact identity verified; lines 80–115 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/tests/clap.rs | 9950 | `c28481cf6d6ea2782b87145facac138c9d3b9502c315c574b27f6a567aca3c7f` | Retained read; exact identity verified; lines 249–279 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/tests/feasibility.rs | 44608 | `2d3356a2ba97f0492b4bfa55144cdee4719b80eb0ca7b904489c0f4dbcc0649e` | Retained read; exact identity verified; lines 50–84 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/ess-kubernetes/src/lib.rs | 7832 | `e4ad23c6773b62cf59b56497b5620cb0b83abde5bbaaef2a4462f7f061fb4fc4` | Retained read; exact identity verified; lines 54–112 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/infra-project/src/lib.rs | 3875 | `9adacde58ea017936b1501279fefbc54fa3e808a543eaed5985c2a54cc504b3d` | Retained read; exact identity verified; lines 8–35 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/infra-project/src/project.rs | 66265 | `26324788584cf0ef89346f51cc936fa1b1901dbbf41444108aba4a81a926524b` | Retained read; exact identity verified; lines 552–574 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/infra-project/tests/projection.rs | 24637 | `3abdb915961028a6dc532571d2cf3a1361404e2f128ec9cb37ff2d6ed5a72214` | Retained read; exact identity verified; lines 113–179,438–478 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/verify/ess-conformance/src/counts.rs | 17936 | `291d118f03e11fbc301db50d3f5c5a0481d7d550201255865fd43ae96fbdb76d` | Retained read; exact identity verified; lines 12–15,319–345 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/verify/ess-conformance/src/coverage.rs | 33620 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` | Retained read; exact identity verified; lines 358–366 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/verify/ess-conformance/src/web_replay.rs | 8656 | `30a6a29bf8aabd3d62f6ba285dd4f4921ae6e7f86af77538ea4d27fb37cf0ba3` | Retained read; exact identity verified; lines 1,120–129 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/concepts/ess.md | 18021 | `c8a711875e7da7a57919966e75e3738125ef623deffbade32289675ccce0e2c6` | full correction diff and paragraph :156–165; complete original eight-file diff carried forward by verified identity |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/guides/synthesize.md | 8742 | `2ceb397d668905b076e18e19e0e0356d566d195ae66a85281adf14187f6fbf95` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/guides/verify-conformance.md | 9713 | `035e4897f23f69e7e1ba236c807c000405d88edfe2a2a999135ac629746db52b` | Retained read; exact identity verified; lines 145–150 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/reference/cli.md | 16656 | `14d2339613e34e6087edbc080f0551406617a7688ed83ab32af18c348eb93f46` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/reference/formats.md | 38918 | `3301a66f41e6e3c6fcc0997dbc5889ddfaad9ae68b1307bcbedea16194e2b24e` | Retained read; exact identity verified; lines 8–15,39–79,163–176 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/status/where-this-stands.md | 10520 | `197040308ec97e642410f468fc3a85cf4b6d29155a1740aef46e1669b3e90edc` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/AGENTS.md | 7816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/.engineering/planning/story/review-public-support-claims.md | 6754 | `882faaf36f46d29c89653e7fdf7642daad0a91e0f79559b836d7011923bc6b0f` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/docs/design/review-public-support-claims.md | 19869 | `79840752b56b0a0995866240040411a7467ea7087b4caa83ba08714cd74defa7` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/brief.md | 4595 | `6854212c33786f1c86251ec8edab83415ee07025eefbad90c052a3d6236e8c64` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/environment.json | 3509 | `96719b0d9e049313032a7e6c735e15ff1c6484f62124e5250c651bf552b3f1eb` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/toolchain-manifest.json | 87168 | `387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/brief.md | 6722 | `ae393081a4e44209cf266a57a38e117b11d6e6dab1eb25cfa7ce0aec99a9dd91` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/report.md | 41716 | `20f61ec77772038624a76beb43ebaa8289d95a9584cabe5c0100c60df608ed00` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/seal.json | 3348 | `bc5643d58336211d1027d0ba7b98d97b4d1b11f68315776521b818272d63ee86` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/matrix-accounting.json | 19043 | `23b3450692eac66f29469b8dce5c7c49ec19cfcdda34514bbbe2a5ce4497b244` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/readback-v2/link-readback.json | 7199 | `4842165c013f53307204fd26fe71e88107c6593a187f33c89a6589ebce121852` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/readback-v2/release-local-readback.json | 558 | `614f81b0dbfb9831824ec67d25c84cacf4d1f50d5ea0d503c377a9d37461b482` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/readback-v2/evidence-owner-hashes.json | 13819 | `1fc9df1bf28e24c19a3372796fe2c400e57812237c6695a27a00e3c13cbe9f9e` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/tests/layout.rs | 13810 | `ea030972952a17c01f410a527ef4dd59f92766f627517c38096dc24baafdd328` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/.cargo/config.toml | 53 | `f151f199a14cb96356203380ebe3a999ac897365b507eddbd2f17d172a6c1ca2` | Retained read; exact identity verified |
| /home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md | 19212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/unit-handoff/verified.json | 8393 | `89ece113f6a00f75e05255e3f81f9ad258b80d64e4ed91efc1d5b44de779c53c` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/latest-release-readback.json | 714 | `3ba3aac290074895cb94d45a87837941beb5d6a34eec596f0bb61c3f534dd5e8` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/latest-release.json | 18435 | `2ce2479a525e9ffbffb5778333109e16995d1a323df7fabc89c7365435f449a7` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/CHANGELOG.md | 62196 | `4ee501ce2b077d0a05febf8e53eff29302eecdae703b8be8973c9f9318d7d8fe` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/crates/edge/ess-cli/src/coverage.rs.stderr | 120 | `f7fc44a9abf1420095647c5f77b7dcb8259ef11a6646fb0a23d2c2fba76a2bcb` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/crates/edge/ess-cli/src/main.rs | 140226 | `f468192fd7aa10ca4b84a9615ef52e086ca5a649b0c355011209c3d1c8937cc6` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/crates/verify/ess-conformance/src/coverage.rs.stderr | 130 | `a08b9661603d15723f6f4d293941459399f9b164e29dbf044a629618d27f0f89` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source-readback.json | 970 | `ddc13436aaa9f9a0549dc48288f3053e0384ea6dca56f92d40bf1ca9dddc764f` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-tag-remote.json | 257 | `d9b9d7ddb341d35b89eb71c53c251f5b47d3f3a11a0dba94954234c18e1b4382` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-tag-remote.stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-tag-remote.stdout | 119 | `6885346b1d437701244c3e3a3443ad34fdb7aaca04126a62723ed7891670e7db` | Retained read; exact identity verified |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/brief.md | 5179 | `3bf36c63caff4e2e3043b2d7c6956ff541f92ebd86d6744d90b2b43555d970aa` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/environment.json | 3657 | `048518fb65a86f6767352e146a36ab578dc66075ec429b75412d64118f4b415d` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/toolchain-manifest.json | 87168 | `387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/correction-1/report.md | 3632 | `01622e931b02d0f733a3d3fa3da09e40443181105cf80e0360d956cc82b852d8` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/correction-1/seal.json | 1450 | `4d6700d474f859e19d32283a1fec045be169472dd4512cd142f8b3bb8b195985` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/correction-1/final-source.json | 2136 | `d5abae38089279efcb84812a138da3a93d27ca09b3bd9d233f4e7af2059ec1dd` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/correction-1/reused-evidence.json | 7419 | `6ae9980ff710f6dc0ba585576a9768974effff3f62de18bf1ea6e5da301414a9` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/correction-1/owner-readback.json | 2308 | `bd1c245393acc1b5c777369559012a890f7cacf1b8603d887719db5a244ab501` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/correction-1-readback/reviewed.json | 628 | `185c486b0106756de6f1e4c039f6daf7c203f2ba6ceaf34cc82378fdd2ab5372` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/correction-1-readback/commit/commit.json | 1055 | `1e656e717e3ee0854d1bcfb3cfc3f0d8f94fd8c184c7b6a744bc99a7e00c1475` | complete text/structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/authored.rs | 16806 | `2e329eadd0d76320f2f32e0cb41b91363320d6eff73e6f08ea4eb4b96f8cdabf` | whole Git payload identity; source :56–84 and :339–345 inspected for initial fixture-oracle error |
| /home/timo/.cargo/config.toml | 3163 | `e41aa5953bfbdb6337fe0a2642bd87723b6bd581504797160af2f2aaef444e29` | complete config text and mtime inspected; ancestor Cargo config explains unset-wrapper insufficiency |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/effective-environment.json | 3690 | `141941c8452a239c47cebd2b81034dea5c0c8422ebc94d414b1efed69576f3a8` | complete structured effective environment read; two explicitly empty wrapper variables |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/wrapper-correction.json | 839 | `b622996d5336674f9b4da104969c86fc7dcd70ab9885624b2ed293f196938c50` | complete launch-refusal and historical limitation record |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/direct-compiler-readback.json | 543 | `33cc7bee06d39087c468643231338f5ddde74bce1be2626c22d1394e9690b454` | whole verbose output machine-inspected; all 67 direct rustc command lines checked and named |

6. Resource and quiescent handoff

The minimum recorded producer free space was 11115646976 bytes, above the 8,589,934,592-byte floor. `resource-growth.json` reports target payload growth separately from filesystem free space; the final check saw 1,819,769,451 target bytes, +627,007,559 against the first-pass target-only census. That includes new compiler fingerprints, fixtures and evidence; it is not all attributed to one producer. Later final input/binary-copy preparation measured 12,341,284,864 free before and 12,188,295,168 after. The increase in filesystem free space was an observation, not a deletion performed by this agent.

Assigned scratch: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2`. Assigned TMP: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-2/tmp`. Outside scratch/TMP, the only intentional writes were the new test block in `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/src/support.rs`, this unit's `target/debug` build products and `target/ess-support-check` probe reports. There are no intentionally written outside-worktree paths. The untraced initial wrapper refusal limitation is stated above. The unchanged package test creates/removes its own disposable sync fixture inside the new assigned TMP; new behavior fixtures have no cleanup and remain retained. I issued no cleanup operation.

`native-census.json` inventories the entire unit target, including the new inside-scratch TMP, hidden files, empty directories, raw native path bytes, full lstat identity, regular payload hashes and literal symlink targets without traversal. It is a retained-state inventory, not a syscall history. Only the census and seal themselves are excluded to avoid self-reference; final creation changes their parent-directory metadata after a second complete stable name/identity check. Special files, if any, are explicitly metadata-only.

All owned producers have exited. The final seal binds this exact report, source/input/binary manifests, full outputs/receipts, intermediate failures and the complete census. After sealing, this agent relinquishes all source/target/scratch/TMP writes. No Git/store/lifecycle command, network operation, real executor, integration gate or publication was performed by this pass. Root owns recording and the eleven-lane/full delivery gate; this report claims neither approval nor verifier independence.

```findings
[]
```
