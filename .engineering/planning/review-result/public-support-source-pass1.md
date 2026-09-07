---
format: aep.planning-md/1
id: review-result:public-support-source-pass1
kind: review-result
status: active
title: Public support implementation source attack pass 1
relations:
- reviews: story:review-public-support-claims
revision: 1
---
unit: story:review-public-support-claims; subject 254db232b785ba3ed6166e9cd5f23e2ad7fe9679 plus seven added tests
verdict: NEEDS-CHANGE
cases: executed 29→36, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: one assigned temporary root; 73 retained native paths and 16 known transient test paths, listed below
needs-coordinator: route the introduced prose correction; retain the full integration, rendered-site, release and publication gates
```text
 crates/edge/ess-xtask/src/support.rs | 200 +++++++++++++++++++++++++++++++++++
 1 file changed, 200 insertions(+)
```

This pass covers base `60279c34737863b59e9b46fa4fb07deaf1b010a3` to subject `254db232b785ba3ed6166e9cd5f23e2ad7fe9679`, branch `impl/review-public-support-claims`, unit `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims`. The only working-tree change is 200 lines appended inside the existing `#[cfg(test)] mod tests` of `crates/edge/ess-xtask/src/support.rs`. The executable brief expressly assigns this private test block despite the generic charter's test-file rule. No production line, existing case, documentation, Taskfile, binding, planning record or Git state was changed. `final-input-verification.json` proves the original module prefix and old tests remain byte-identical. `attack.diff` is the complete actual diff, not a proposed fix. `subject.diff` retains all eight implementation-file changes.

1. Added cases and original focused observations

The seven cases were written before any test/build/CLI producer executed. Each was then selected alone with `--exact --nocapture`, before the unfiltered package run. All selected one case, none selected zero, and all passed. The README case is a behavioral observation supporting an ordinary-prose contradiction; it does not manufacture a red phrase-matching test. No Rust test failure is claimed for this pass.

| Receipt label | Test location | Assertion | Current result |
|---|---|---|---|
| focus-readme | crates/edge/ess-xtask/src/support.rs:697 | Adjacent README defaults into explicit-site index without any authored flags; unselected sibling Markdown and downloads remain absent. | 1 passed, 0 failed |
| focus-roots | crates/edge/ess-xtask/src/support.rs:725 | Actual explicit/combined maps retain every site path under distinct roots, local assets exist and docs-ir stays out of combined output. | 1 passed, 0 failed |
| focus-rows | crates/edge/ess-xtask/src/support.rs:750 | Every cell of all 20 real rows, removal and duplication of each row, 19 adjacent swaps and one extra row refuse: 120 mutations, with actual CLI-derived expected block. | 1 passed, 0 failed |
| focus-version | crates/edge/ess-xtask/src/support.rs:800 | Mutating a copy of the real Cargo workspace version changes the expected source claim while keeping surrounding release bytes identical. | 1 passed, 0 failed |
| focus-metadata | crates/edge/ess-xtask/src/support.rs:824 | Actual target-help metadata lists all four targets; removing that option metadata from a copy refuses despite neighboring possible values. | 1 passed, 0 failed |
| focus-refusal | crates/edge/ess-xtask/src/support.rs:843 | Actual CLI refusal for an unknown kind or absent specification cannot become successful projection facts. | 1 passed, 0 failed |
| focus-docsir | crates/edge/ess-xtask/src/support.rs:861 | Actual docs-ir has the exact explicit path and JSON format marker; a same-text description or nested object cannot replace the marker. | 1 passed, 0 failed |

Original focused run: focus-readme

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_adjacent_readme_is_selected_without_authored_flags -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: /home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136
actual CLI included adjacent README in index.html without --front-page, --include or --asset; siblings and downloads stayed absent
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 30.49s

```

stderr, verbatim:
```text
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling syn v3.0.4
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling syn v2.0.119
   Compiling serde_derive v1.0.229
   Compiling serde_json v1.0.151
   Compiling itoa v1.0.18
   Compiling typenum v1.20.1
   Compiling memchr v2.8.3
   Compiling hybrid-array v0.4.14
   Compiling serde_derive_internals v0.29.1
   Compiling thiserror v2.0.20
   Compiling schemars v0.8.22
   Compiling schemars_derive v0.8.22
   Compiling thiserror-impl v2.0.20
   Compiling hashbrown v0.17.1
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling indexmap v2.14.1
   Compiling crypto-common v0.2.2
   Compiling block-buffer v0.12.1
   Compiling unsafe-libyaml v0.2.11
   Compiling utf8parse v0.2.2
   Compiling const-oid v0.10.2
   Compiling ryu v1.0.23
   Compiling digest v0.11.3
   Compiling serde_yaml v0.9.34+deprecated
   Compiling anstyle-parse v1.0.0
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/specify/ess-primitives)
   Compiling cfg-if v1.0.4
   Compiling cpufeatures v0.3.1
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstyle-query v1.1.5
   Compiling pulldown-cmark v0.13.4
   Compiling anstyle v1.0.14
   Compiling sha2 v0.11.0
   Compiling anstream v1.0.0
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/specify/ess-domain)
   Compiling unicase v2.9.0
   Compiling anyhow v1.0.104
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling bitflags v2.13.1
   Compiling clap_lex v1.1.0
   Compiling clap_builder v4.6.6
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/specify/ess-compiler)
   Compiling clap_derive v4.6.4
   Compiling ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen)
   Compiling clap v4.6.6
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 15.20s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-readme/receipt.json`.

Original focused run: focus-roots

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: /home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1485522-1788757095485334854
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.78s

```

stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-roots/receipt.json`.

Original focused run: focus-rows

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: /home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1485680-1788757098951191381
all 20 actual rows attacked: 120 independent refusals
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 2.21s

```

stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-rows/receipt.json`.

Original focused run: focus-version

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
test support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.00s

```

stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-version/receipt.json`.

Original focused run: focus-metadata

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: /home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1486137-1788757100006691523
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.10s

```

stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-metadata/receipt.json`.

Original focused run: focus-refusal

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: /home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1486227-1788757100480343873
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.20s

```

stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-refusal/receipt.json`.

Original focused run: focus-docsir

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask --bin ess-xtask support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: /home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1486351-1788757101172716407
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.10s

```

stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/focus-docsir/receipt.json`.

2. Package execution and remaining local checks

The baseline 29 is the actual implementor handoff: 24 unit and 5 layout tests. It was not re-executed before adding cases. Its complete report, final-package receipt/stdout/stderr and immutable census are retained under `target/review-boundaries-14/public-support/implementation` and hashed by `implementor-command-readback.json` / `implementor-input-inventory.json`. The first package execution after additions ran 31 unit + 5 layout tests and passed. The final package run below used the final formatted source; the earlier package and first formatting refusal remain retained. Formatting changes only wrapped four newly added assertions/prints; the mutation-count print now says “mutation refusals.”

Command: `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/cargo test --locked -p ess-xtask`

stdout, verbatim:
```text

running 31 tests
test support::tests::a_complete_source_block_is_accepted_without_owning_release_prose ... ok
test support::tests::cargo_version_changes_invalidate_only_the_source_block ... ok
test support::tests::command_inventory_requires_a_complete_nonempty_unique_section ... ok
test support::tests::every_material_row_change_is_refused_with_its_location ... ok
test support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent ... ok
test support::tests::emitted_version_markers_are_parsed_and_must_agree_across_the_projection ... ok
test support::tests::html_requires_the_actual_output_root_and_both_nonempty_local_assets ... ok
test support::tests::help_inventory_reads_both_clap_layouts_without_swallowing_neighbor_options ... ok
test tests::a_named_but_untagged_version_is_not_an_incomplete_release ... ok
test support::tests::missing_duplicated_or_reordered_block_markers_refuse ... ok
test tests::a_version_tag_whose_commit_never_reached_main_is_refused ... ok
test tests::a_version_tag_with_no_release_behind_it_is_refused ... ok
test tests::an_empty_release_is_refused_by_name ... ok
test tests::an_undated_release_is_refused_by_name ... ok
test tests::exclusions_cover_only_the_named_subtree ... ok
test tests::generated_paths_must_stay_below_the_projection_root ... ok
test tests::only_bare_version_tags_are_release_tags ... ok
test tests::published_release_tags_come_from_the_json_report ... ok
test tests::release_notes_stop_before_the_next_release ... ok
test support::tests::support_check_is_an_available_maintenance_command ... ok
test tests::release_publication_is_evaluated_after_every_dependency_finishes ... ok
test tests::the_generated_index_includes_static_site_source ... ok
test tests::the_release_record_is_checked_after_every_release_run ... ok
test tests::workspace_version_comes_only_from_the_workspace_package_table ... ok
test tests::sync_checks_and_reconciles_in_both_directions ... ok
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values ... ok
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text ... ok
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation ... ok
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags ... ok
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots ... ok
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.38s


running 5 tests
test the_path_scan_reads_an_area_qualified_path ... ok
test every_workspace_crate_lives_under_an_area_directory ... ok
test the_path_scan_excludes_by_root_relative_path_and_reads_published_website_source ... ok
test every_literal_path_naming_a_workspace_crate_exists ... ok
test the_path_scan_finds_at_least_one_path_in_this_repository ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

```

stderr, verbatim:
```text
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 0.61s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-ec09005f8fd97d8f)
     Running tests/layout.rs (target/debug/deps/layout-2c4124a758c7a7b2)
```

Direct exit: 0. Receipt: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/commands/final-package/receipt.json`.

| Label | Direct exit | Seconds | Free before | Free after |
|---|---:|---:|---:|---:|
| focus-readme | 0 | 45.706316 | 12415762432 | 12083146752 |
| focus-roots | 0 | 0.859953 | 12078702592 | 12071084032 |
| focus-rows | 0 | 2.287110 | 12071055360 | 12069990400 |
| focus-version | 0 | 0.080515 | 12069978112 | 12069965824 |
| focus-metadata | 0 | 0.178001 | 12068282368 | 12068188160 |
| focus-refusal | 0 | 0.275788 | 12068052992 | 12067942400 |
| focus-docsir | 0 | 0.181270 | 12067799040 | 12067700736 |
| package | 0 | 3.239797 | 12060516352 | 12047564800 |
| format-check | 1 | 0.048877 | 12037373952 | 12037365760 |
| clippy | 0 | 7.354048 | 12036812800 | 12038160384 |
| final-format | 0 | 0.049420 | 12033634304 | 12033454080 |
| final-package | 0 | 3.044820 | 12033441792 | 12021743616 |
| support-check | 0 | 2.301140 | 12021714944 | 12018577408 |
| shared-projections | 0 | 0.519443 | 12018524160 | 12021063680 |

Final formatter, strict package Clippy (`--all-targets -- -D warnings`), actual `cargo xtask support --check` and shared `cargo xtask generate --check` all exited 0. Complete argv, cwd, the effective controlled environment, source/HEAD identity, direct exit, stdout/stderr hashes and resulting CLI/package executable hashes are in each `commands/<label>/receipt.json`; the complete outputs are adjacent files. The first formatter exited 1 solely for formatting my added code, retained as `commands/format-check`; it is not a production finding. One readback helper initially assumed the newly introduced support module existed at the base: its Git read exited 128 and helper exited 1. The original helper and copied verbatim stderr remain in `readback.py`, `readback-first.stderr`, and `readback-first-result.json`; `readback-v2.py` correctly handles the retained intended absence and verified all 46 owners. No test or support command failed because of that bookkeeping error.

3. Finding and reachability

| File:line | Category / severity | Verdict / origin | Finding |
|---|---|---|---|
| website/docs/concepts/ess.md:160 | contract-drift / blocker | NEEDS-CHANGE / introduced | The concept page says authored pages require explicit selection, but explicit site generation automatically includes an adjacent README.md as index.html without authored-selection flags. |

What was measured: `support::tests::adversary_adjacent_readme_is_selected_without_authored_flags`, support.rs:697, runs the actual public CLI through the changed xtask `cli_output` route. Its exact arguments are retained in `/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/argv.txt`. The observed JSON map `/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/actual-cli-stdout.json` (3728199 bytes, SHA256 `7718625882c23fddf2f7c3ac0cd744a5ef92867b739169c2402ff37b9d7145c9`) has `site/index.html.contents` containing `ADVERSARY_README_DEFAULT`. There is no `--front-page`, `--include` or `--asset` argument. Unselected sibling and download sentinels are absent. Direct test exit 0, one case executed; original stdout is quoted above. This passing positive control exposes a false prose guarantee, not an execution failure.

What reaches it: any valid specification directory with an adjacent README and `ess generate --kind site --path <directory>`. `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/main.rs`:149–152 documents this default; :2370–2372 dispatches explicit Site to the site renderer. `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/site.rs`:320–339 chooses `directory.join("README.md")` when `front_page` is absent and reads it as `index.html`; :340–348 separately restricts additional pages/assets to explicit inputs. The new paragraph at `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/concepts/ess.md`:159–160 broadly says “Authored pages and downloads require explicit selection.” The complete subject diff adds that sentence. Thus the contradiction is introduced by this unit even though the correctly observed default behavior already exists. The base was read without moving the checkout; no base test run is needed to attribute the newly added sentence.

The bounded correction is to state the adjacent README front-page default and reserve explicit-selection wording for additional authored pages/downloads. This belongs to the current public-claims unit; no new authored-discovery implementation, new manifest or follow-up story is needed. I left the production prose unchanged for its owner. “Blocker” here means this accuracy story should correct the false statement before delivery; it does not claim a new runtime defect or evidence of unintended publication in a live system.

4. Finite acceptance coverage and things not broken

All 27 accepted material statements and all 21 validation obligations were compared with the final diff and actual caller/owner boundaries. The previous candidate pass remains historical design input, not the verdict for this implementation.

| Accepted material rows | Attack and observed boundary |
|---|---|
| 1 | Real workspace version metadata; stale source row refuses, surrounding release bytes stay separate. |
| 2–3 | Full retained release JSON and dated readback agree on record 383642123, tag 0.20.0, publication timestamp and four archive names plus SHA256SUMS; no asset execution/checksum claim. Fresh remote verification belongs to root. |
| 4–9 | Actual five-generator map, explicit HTML roots/assets, Markdown/Mermaid, opt-in docs-ir and schema/OpenAPI/AsyncAPI markers; the actual support command and shared projection check pass. README prose exception is the one finding above. |
| 10–15 | Actual CLI import/project inventories plus exact unchanged OpenAPI accounting/refusal, Kubernetes observation/projection and BuildKit/Helm owners. Maintained prose retains partial support, credential, obligation, no-apply and no-live-provisioning limits. Existing semantic tests were source-read and preserved; full execution remains root's gate. |
| 16–18 | Four actual CLI targets, Clap ArgMatches/handler/dependency owner, Binary64 full-target refusal test. No claim that target availability proves generated business behavior. |
| 19–22 | Actual built-in target inventory, suite/report defaults, emitted detailed/standalone markers and suite/5 marker; exact strict/qualification owners retain the nonempty complete-inventory/no-in-scope-refusal boundary. Marker checks are not semantic conformance proof. |
| 23 | Current Unreleased changelog and retained exact release-source receipts support qualification relative to the dated 0.20.0 observation. Coverage-module absence at the old release is a Git existence result, not a failed test. |
| 24 | Replay is source-reviewed as presentation, without independent execution or publisher authentication; existing browser tests remain root-owned. |
| 25–26 | Runtime owner checks supplied identities, components, replicas and storage; explicit executor owner uses supplied state/credentials. The checker calls no external executor. No universal resource, provisioning or recovery guarantee was introduced. |
| 27 | Actual schema command inventory and current-source labels; command presence does not infer release publication. |

Validation rows 1, 2, 4–9 were executed through the focused cases / actual support and shared projection checks; authored boundary row 3 retains existing tests and adds the measured README-default observation. Rows 10–17 retain the specified semantic/refusal owners and their unchanged source hashes; they were not rerun as broader packages in this bounded lane. Rows 18–20 (rendered site, complete task check/site-build, fresh release and Website/Atlas delivery) are explicitly root-owned. Row 21 is satisfied at source scope by the exact eight-file implementation diff and unchanged public allowlist; it is not a claim about a live website.

The block comparator rejected all 60 real-cell changes, all 20 missing rows, all 20 duplicates, 19 neighboring swaps and one extra row. Existing parser cases additionally cover missing/duplicate markers, malformed marker types and differing emitted projection versions. The metadata-loss mutation uses a copy of actual CLI help. CLI nonzero status propagates as refusal. Output-map key/path identity is exercised through both real explicit and combined routes. Source inspection confirms all values come from Cargo metadata, existing generator metadata or public CLI output; no Rust-token search was introduced as support proof. The existing layout scan is a separate unchanged repository path test.

5. Subject, input identities and read extent

`inspected-inputs.json` carries each exact absolute path, SHA256, bytes, Git subject where applicable and exact selected source line numbers. `source-owner-readback.json` compares the 46 material owners with the exact base/subject; `selected-owner-lines.txt` retains the 1,827 selected source lines, all read during this pass. All eight implementation diffs and the complete original private checker module were read, along with its existing tests and direct callers. The source-owner excerpts supplement the full diff; a whole-file hash is not presented as a full semantic read of every unchanged line.

`tracked-inputs.json` inventories all 1,142 tracked payloads (38,268,471 bytes) used by the package/source closure, with full absolute paths and hashes. It is a whole-payload identity census, not a claim that every repository file was semantically reviewed. `original-source.json` and `final-source.json` distinguish original Git subject bytes from the added tests. The original implementation's 364 native evidence entries were checked unchanged in payload and non-atime identity after all producers; 48 original command receipts and their whole stdout/stderr were parsed/hashed separately. Historical fixture outputs are preserved.

The 378-entry full frozen Rust 1.98.1 sysroot manifest was parsed and every payload, mode and native name verified before the first producer; no floating stable tool was used. Its 87,168-byte manifest SHA256 is `387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d`. `toolchain-verified.json` records full verification, and each producer rechecks the Cargo/rustc/rustdoc/formatter/Clippy executable hashes. The original four relevant executable copies total 123,504,544 bytes and remain in `original-binaries/`; final copies are in `final-binaries/`. Their source/copy paths and exact hashes are bound by the adjacent manifests. Cargo runs against this unit's target, with no CARGO_TARGET_DIR, compiler wrapper or sccache server, jobs=2, offline, debug/incremental off and the assigned TMPDIR. CARGO_HOME is assigned scratch; its two literal links point to the existing offline registry and git caches.

Exact directly inspected input table (source rows are the original subject hashes; final support test bytes are separately recorded):

| Absolute input path | Bytes | SHA256 | Read extent |
|---|---:|---|---|
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/CHANGELOG.md | 63660 | `04c7a77afd7539d0333f4e1f5282aceb224d49abb2dd97c1a1d4e4f77d35d211` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 3–14,148–196 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/Cargo.lock | 33730 | `8ca4848311f5c82eef7e170f3b8562fe0132b874b9b6f630f2bd28e7d7fc5842` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/Cargo.toml | 3785 | `f806132b62b38f010175cfdcb8fbe627f25bf0d3aec08a923959224a66718c10` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 39–46 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/Taskfile.yml | 7315 | `3245661ae1bc707e5657abd4d7af13e1cb1e09806a95d684c5b315d7d4047e2c` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject; lines 38–46,95–131,144–160 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/b10x.docs.yaml | 2340 | `af67e8dbbd482583c9c72afc2cf7800f134dcda7ac6cd129381e21aaace0df49` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/coverage.rs | 8137 | `fa6bfe23553296e53cf7e8b60ff2021ce6ac886bff32cdd741607e4907a82982` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 203–205 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/main.rs | 139710 | `1332526c7fe67a3e92a23724e862ed0cc86ccd14c3f7ea5bfa0797c13937a029` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 140–157,379–384,449–450,453–461,501–520,576–580,584–601,615–638,640–649,1275–1279,1481–1486,1634–1745,2355–2379,2518–2522,2915–2968,2976–3058,3065–3075,3078–3091,3094–3099,3199–3207 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/schema.rs | 10135 | `345f81fc51f463bf2e68e1f28ea2a90c48e5cc4ef4d45391b3e8476f8838a8c2` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 15–36,73–83 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/src/site.rs | 11531 | `1e179ed7f2684bfd3c51e7c231a27371cc21e0d294c0a05aadd23b29489843bd` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 67–71,152–155,310–350 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/authored_site.rs | 8138 | `542c3fcecae0bd29ab58de89e947da89fc96a3258dc7b31a80b31860b70abec4` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 85–111 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/count_reports.rs | 6049 | `518bd521c0d361f080f552fdf1617b9ee3e7ba32a4fbc90077af7e98ae8a7a0b` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 5–34,62–117 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/coverage_browser.rs | 17051 | `27c79ffc2f5b74d9696fa0ffb1c245d0b3e46c9d2add1d613b7804e3bd62d7f9` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 9,211,311 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-cli/tests/coverage_cli.rs | 16006 | `6602c0ec47f090bbac6a0566fd9ded620f9d35494b3d814be1712982a5f7f424` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 246–299 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/Cargo.toml | 373 | `4f06e40288fd75aae6294ab55ef016d3d541edce2198b4f396ecaa0e7d180357` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 10–16 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/src/main.rs | 38863 | `c583a1c3487f6ad9f726704c1d028c463ae10f5deaef39bbb863405e915b6b2c` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject; lines 105–144,466–535 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/src/support.rs | 30361 | `ba8aafa8d6baf66b7178f8190756702254c6be0222bf2b92f59c55e84d84dc6c` | complete original Git subject module, all 676 lines, complete added test block and diff; subject hash shown, final hash in final-source.json |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-deployment/src/runtime.rs | 33627 | `b119a25325ba7e01bb533febbd6a59f4be26e122d72bfcdfc9fd1f8f4bc12138` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 482–531,622–685 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-deployment/tests/deployment.rs | 43648 | `189c6584925154335c7c2c7b1777f79222c5e2391202ab115ec2d8ef8f4cbbc3` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 313–334,447–458 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/asyncapi.rs | 38193 | `30fc11197be98c2faefee61fe01b17d12d640410167fc59c0823b62dec356034` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 154,187–190 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/docs.rs | 117964 | `1084bb9132beb7e7301e1896389039d18b0632fa180a5ed277734dd33a4debe6` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 69–105 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/document.rs | 16611 | `bf3c6a1ec618a2e7269a4e1eef51e03d341608d427981b16dac38929a92a0d6c` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 54–55 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/html.rs | 41762 | `966e43244a9910acba981def0173681a881c9231538ef4d90ea899e819943aa3` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 51–58,625–642 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/lib.rs | 3657 | `eb1b0e45d1c5ebb81dfb5e44b83e1417772d6935604fe1154d8323918c3a057e` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 46–59 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/openapi.rs | 58326 | `5843677bd6366f299fa3801cfb6859b4f613299519bd8e172110c830bf455262` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 210,239–254 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-gen/src/schema.rs | 10462 | `ab84031634a24d71e24643f321d7902230a4213b5964f0c7577e2903f6068f49` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 66,103–144 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-openapi/src/accounting.rs | 11074 | `65ada37ec0bd9bc917bf5a89b20c60742a0045b172447a99358c5324fe05b7b9` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 9–14,140–150,224–253 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-openapi/src/lib.rs | 52059 | `52105880e6cb92a786a81053de8a7a82ad042e5cec2a6b32611da0cfac5ec9dd` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 495–508,1232–1240 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-openapi/tests/accounting.rs | 11063 | `a885f1b32597fdb5323c7ee2d1fe754edb496de8c17124d5546b22ce0db7d168` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 15–60,77–88 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/src/clap/mod.rs | 6391 | `b29f950bf6e7acc80d6912469a179da316fffd6e3ff488b24b887f711e8440d5` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 23–32,62–65,101–105 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/src/clap/tree.rs | 18456 | `028b76d8d982388d1360e5cc886e010994623e4c369adf1303878e178918296c` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 453 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/src/lib.rs | 14718 | `d4e0b1da54d0a1c798c931c6251555805a209feeb75f9c03dccbd0fa760f2e7b` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 80–115 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/tests/clap.rs | 9950 | `c28481cf6d6ea2782b87145facac138c9d3b9502c315c574b27f6a567aca3c7f` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 249–279 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/generate/ess-synth/tests/feasibility.rs | 44608 | `2d3356a2ba97f0492b4bfa55144cdee4719b80eb0ca7b904489c0f4dbcc0649e` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 50–84 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/ess-kubernetes/src/lib.rs | 7832 | `e4ad23c6773b62cf59b56497b5620cb0b83abde5bbaaef2a4462f7f061fb4fc4` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 54–112 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/infra-project/src/lib.rs | 3875 | `9adacde58ea017936b1501279fefbc54fa3e808a543eaed5985c2a54cc504b3d` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 8–35 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/infra-project/src/project.rs | 66265 | `26324788584cf0ef89346f51cc936fa1b1901dbbf41444108aba4a81a926524b` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 552–574 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/infra/infra-project/tests/projection.rs | 24637 | `3abdb915961028a6dc532571d2cf3a1361404e2f128ec9cb37ff2d6ed5a72214` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 113–179,438–478 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/verify/ess-conformance/src/counts.rs | 17936 | `291d118f03e11fbc301db50d3f5c5a0481d7d550201255865fd43ae96fbdb76d` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 12–15,319–345 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/verify/ess-conformance/src/coverage.rs | 33620 | `4f244ec4872b5c7cdd0756d5d149eafd5b3ba988b6c30a6eff87b223fded3128` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 358–366 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/verify/ess-conformance/src/web_replay.rs | 8656 | `30a6a29bf8aabd3d62f6ba285dd4f4921ae6e7f86af77538ea4d27fb37cf0ba3` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 1,120–129 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/concepts/ess.md | 17908 | `866072d59e394eeceb88b2917b27cb0b6bb4d37b24063346d0999c3813f1ee24` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/guides/synthesize.md | 8742 | `2ceb397d668905b076e18e19e0e0356d566d195ae66a85281adf14187f6fbf95` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/guides/verify-conformance.md | 9713 | `035e4897f23f69e7e1ba236c807c000405d88edfe2a2a999135ac629746db52b` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject; lines 145–150 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/reference/cli.md | 16656 | `14d2339613e34e6087edbc080f0551406617a7688ed83ab32af18c348eb93f46` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/reference/formats.md | 38918 | `3301a66f41e6e3c6fcc0997dbc5889ddfaad9ae68b1307bcbedea16194e2b24e` | whole Git blob hashed and compared to exact base; semantic source review of listed lines; full eight-file subject diff reviewed; lines 8–15,39–79,163–176 |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/website/docs/status/where-this-stands.md | 10520 | `197040308ec97e642410f468fc3a85cf4b6d29155a1740aef46e1669b3e90edc` | complete three-dot change diff and relevant complete sections; full payload hash verified against exact Git subject |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/AGENTS.md | 7816 | `9f7275e26a79f61a1f5560cfaa64584c4e6df9461496c8005bb34a583b3419ec` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/.engineering/planning/story/review-public-support-claims.md | 6754 | `882faaf36f46d29c89653e7fdf7642daad0a91e0f79559b836d7011923bc6b0f` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/docs/design/review-public-support-claims.md | 19869 | `79840752b56b0a0995866240040411a7467ea7087b4caa83ba08714cd74defa7` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/brief.md | 4595 | `6854212c33786f1c86251ec8edab83415ee07025eefbad90c052a3d6236e8c64` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/environment.json | 3509 | `96719b0d9e049313032a7e6c735e15ff1c6484f62124e5250c651bf552b3f1eb` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1/toolchain-manifest.json | 87168 | `387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/brief.md | 6722 | `ae393081a4e44209cf266a57a38e117b11d6e6dab1eb25cfa7ce0aec99a9dd91` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/report.md | 41716 | `20f61ec77772038624a76beb43ebaa8289d95a9584cabe5c0100c60df608ed00` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/seal.json | 3348 | `bc5643d58336211d1027d0ba7b98d97b4d1b11f68315776521b818272d63ee86` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/matrix-accounting.json | 19043 | `23b3450692eac66f29469b8dce5c7c49ec19cfcdda34514bbbe2a5ce4497b244` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/readback-v2/link-readback.json | 7199 | `4842165c013f53307204fd26fe71e88107c6593a187f33c89a6589ebce121852` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/readback-v2/release-local-readback.json | 558 | `614f81b0dbfb9831824ec67d25c84cacf4d1f50d5ea0d503c377a9d37461b482` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/implementation/readback-v2/evidence-owner-hashes.json | 13819 | `1fc9df1bf28e24c19a3372796fe2c400e57812237c6695a27a00e3c13cbe9f9e` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/crates/edge/ess-xtask/tests/layout.rs | 13810 | `ea030972952a17c01f410a527ef4dd59f92766f627517c38096dc24baafdd328` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/.cargo/config.toml | 53 | `f151f199a14cb96356203380ebe3a999ac897365b507eddbd2f17d172a6c1ca2` | complete text or structured metadata read |
| /home/timo/.codex/plugins/cache/beyond10x/aep-drive/0.8.0/agents/adversary.md | 19212 | `680ac11700dc215bf39201d7019b36277f8b260b673f99c6f09e8a4eb051f795` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-14/preparation/unit-handoff/verified.json | 8393 | `89ece113f6a00f75e05255e3f81f9ad258b80d64e4ed91efc1d5b44de779c53c` | complete text or structured metadata read |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/latest-release-readback.json | 714 | `3ba3aac290074895cb94d45a87837941beb5d6a34eec596f0bb61c3f534dd5e8` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/latest-release.json | 18435 | `2ce2479a525e9ffbffb5778333109e16995d1a323df7fabc89c7365435f449a7` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/CHANGELOG.md | 62196 | `4ee501ce2b077d0a05febf8e53eff29302eecdae703b8be8973c9f9318d7d8fe` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/crates/edge/ess-cli/src/coverage.rs.stderr | 120 | `f7fc44a9abf1420095647c5f77b7dcb8259ef11a6646fb0a23d2c2fba76a2bcb` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/crates/edge/ess-cli/src/main.rs | 140226 | `f468192fd7aa10ca4b84a9615ef52e086ca5a649b0c355011209c3d1c8937cc6` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source/crates/verify/ess-conformance/src/coverage.rs.stderr | 130 | `a08b9661603d15723f6f4d293941459399f9b164e29dbf044a629618d27f0f89` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-source-readback.json | 970 | `ddc13436aaa9f9a0549dc48288f3053e0384ea6dca56f92d40bf1ca9dddc764f` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-tag-remote.json | 257 | `d9b9d7ddb341d35b89eb71c53c251f5b47d3f3a11a0dba94954234c18e1b4382` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-tag-remote.stderr | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |
| /home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/public-support/inputs/release/release-tag-remote.stdout | 119 | `6885346b1d437701244c3e3a3443ad34fdb7aaca04126a62723ed7891670e7db` | whole retained release payload hashed; structured release metadata parsed; release-source qualification reviewed, no release binary/network execution |

6. Resource, outside writes and quiescent seal

Every build/test/check producer records free space before and after. The minimum recorded producer value was 12018524160 bytes, above the 8,589,934,592-byte floor. Final input/copy preparation measured 11,850,391,552 bytes free. No resource refusal or agent cleanup occurred.

Assigned scratch is `/home/timo/.local/state/worktree/trees/b10x/ess/ess-public-support-claims/target/review-boundaries-14/adversary-pass-1` and the only assigned external producer temporary root is `/home/timo/.cache/ess-w14-support-adversary1-tmp`. Outside scratch/TMP, the tracked tests-only file changed and this unit's `target/debug` build products plus `target/ess-support-check` probe reports were written. No other tree's target, production/documentation source, Git/store/lifecycle state, network service or real executor was used. The full native census covers the ENTIRE unit target and assigned temporary root, not only scratch. It includes hidden names, empty directories, raw native path bytes, lstat identity, regular payload SHA256 and literal symlink targets without traversal. The census excludes only itself and the final seal to avoid self-reference. A second name/identity pass checks stability before sealing; those final control writes subsequently change their parent-directory metadata. Special files, if any, are metadata-only and explicitly identified.

Every retained external path, fully expanded:

```text
/home/timo/.cache/ess-w14-support-adversary1-tmp
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1486351-1788757101172716407
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1486351-1788757101172716407/actual-document.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1486351-1788757101172716407/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1487950-1788757121168985848
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1487950-1788757121168985848/actual-document.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1487950-1788757121168985848/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1500101-1788757261108486127
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1500101-1788757261108486127/actual-document.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-docs-ir-1500101-1788757261108486127/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1486137-1788757100006691523
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1486137-1788757100006691523/actual-help.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1486137-1788757100006691523/missing-metadata-help.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1486137-1788757100006691523/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1487950-1788757121511481480
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1487950-1788757121511481480/actual-help.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1487950-1788757121511481480/missing-metadata-help.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1487950-1788757121511481480/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1500101-1788757261398942050
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1500101-1788757261398942050/actual-help.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1500101-1788757261398942050/missing-metadata-help.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-metadata-1500101-1788757261398942050/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/README.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/actual-cli-stdout.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/argv.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/download.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/sibling.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1478052-1788757044487537136/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045/README.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045/actual-cli-stdout.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045/argv.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045/download.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045/sibling.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1487950-1788757121168940045/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954/README.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954/actual-cli-stdout.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954/argv.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954/download.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954/sibling.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-readme-1500101-1788757261108454954/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1486227-1788757100480343873
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1486227-1788757100480343873/actual-refusal.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1486227-1788757100480343873/missing-input-refusal.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1486227-1788757100480343873/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1487950-1788757121168855170
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1487950-1788757121168855170/actual-refusal.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1487950-1788757121168855170/missing-input-refusal.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1487950-1788757121168855170/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1500101-1788757261108355466
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1500101-1788757261108355466/actual-refusal.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1500101-1788757261108355466/missing-input-refusal.txt
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-refusal-1500101-1788757261108355466/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1485522-1788757095485334854
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1485522-1788757095485334854/observed-maps.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1485522-1788757095485334854/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1487950-1788757121168880702
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1487950-1788757121168880702/observed-maps.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1487950-1788757121168880702/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1500101-1788757261108418936
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1500101-1788757261108418936/observed-maps.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-roots-1500101-1788757261108418936/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1485680-1788757098951191381
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1485680-1788757098951191381/actual-expected.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1485680-1788757098951191381/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1487950-1788757123772970509
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1487950-1788757123772970509/actual-expected.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1487950-1788757123772970509/system.yaml
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1500101-1788757263483549360
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1500101-1788757263483549360/actual-expected.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-support-adversary-rows-1500101-1788757263483549360/system.yaml
```

The unchanged `sync_checks_and_reconciles_in_both_directions` test also creates and removes its own disposable fixture during each package run. These known transient outside paths come from its actual two process IDs and source construction; they are not falsely counted as retained census entries:

```text
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/docs
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/go
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/site
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/docs/index.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/docs/orphan.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/go/owned-elsewhere.go
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1487950-0/site/sidebar.json
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/docs
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/go
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/site
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/docs/index.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/docs/orphan.md
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/go/owned-elsewhere.go
/home/timo/.cache/ess-w14-support-adversary1-tmp/ess-xtask-sync-1500101-0/site/sidebar.json
```

The package's own temporary-fixture cleanup remains unchanged; I issued no cleanup/deletion operation. This is a final retained-state census, not a syscall history of compiler-internal transient names. No unassigned external write was performed intentionally or discovered. New behavioral fixtures deliberately have no cleanup and remain available to root.

All owned producers have exited. After creating `native-census.json` and `seal.json`, this agent relinquishes all source, target, scratch and temporary writes. The seal binds this exact report, all command/input manifests, complete diffs, original/final source and binary manifests, the original implementor immutability check and the final native census. Root may independently read a stable handoff, record the full report verbatim, route the one prose fix and later execute the complete integration/publication gates. This agent ran no planning, Git mutation, lifecycle, network or publication command and claims no approval or verifier independence.

```findings
- file: website/docs/concepts/ess.md
  line: 160
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The concept page says authored pages require explicit selection, but explicit site generation automatically includes an adjacent README.md as index.html without authored-selection flags.
```
