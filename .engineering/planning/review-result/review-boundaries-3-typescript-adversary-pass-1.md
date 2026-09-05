---
format: aep.planning-md/1
id: review-result:review-boundaries-3-typescript-adversary-pass-1
kind: review-result
status: active
title: Wave 3 TypeScript adversary pass 1
relations:
- reviews: story:review-typescript-root-collision
revision: 1
---
unit: story:review-typescript-root-collision at 7a73ce9bf741a98f82c7141f4a80340451a704ed plus test-only working tree, adversary pass 1
verdict: nothing found
cases: executed 19→25, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

```text
$ git --no-pager diff --stat
 .../schema-contract/tests/typescript_typecheck.rs  | 113 +++++++++++++++++++++
 1 file changed, 113 insertions(+)
exit: 0
```
The tracked diff is append-only in a test file. The new untracked integration test is included explicitly below; no production, inline production tests, manifest, planning, design or lifecycle changes were made.

```text
$ git --no-pager diff --no-index --stat -- /dev/null crates/generate/schema-contract/tests/typescript_bindings.rs
 .../schema-contract/tests/typescript_bindings.rs   | 84 ++++++++++++++++++++++
 1 file changed, 84 insertions(+)
exit: 1 (new-file difference)
```
```text
$ git status --short
 M crates/generate/schema-contract/tests/typescript_typecheck.rs
?? crates/generate/schema-contract/tests/typescript_bindings.rs
exit: 0
```
The complete test-only patch is `target/review-boundaries-3/adversary-tests.patch`: two test files, 197 added lines, no removed lines. Existing cases and assertions are preserved.

The reviewed base is `45832cc885377b2d61845ee33af14f0293d99e67`. I read its complete committed diff through `7a73ce9bf741a98f82c7141f4a80340451a704ed`, the story and briefs, the full implementation report, the new registry.ts fixture and compiler test target, and the public projector and CLI caller before writing cases. No base execution or tree movement occurred.

The before counts come from the implementor's completed report and the adversary brief: default package 16, explicit compiler lane 3, documentation tests 0. No baseline suite was run. All six new cases were authored and the two allowed test files formatted before the first test command. Each exact isolated command below ran one case, with captured output and exit status, before either suite ran. There was no initial red run, typo correction, assertion weakening or retry.

New cases and isolated execution:

1. `crates/generate/schema-contract/tests/typescript_bindings.rs:7` — Five escaped, punctuation and non-ASCII definition keys collide with the exact normalized root; refusal pointer escapes JSON Pointer segments, repeat calls return equal errors and the input schema bytes remain untouched. Green on its first isolated run and in its final suite.

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --test typescript_bindings escaped_and_non_ascii_definition_keys_refuse_root_collisions_deterministically -- --exact > target/review-boundaries-3/adversary-case-1.log 2>&1
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.23s
     Running tests/typescript_bindings.rs (target/debug/deps/typescript_bindings-4484db42097fd431)

running 1 test
test escaped_and_non_ascii_definition_keys_refuse_root_collisions_deterministically ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

exit: 0
```

2. `crates/generate/schema-contract/tests/typescript_bindings.rs:37` — Array helpers rendered inside nullable alternatives and nested arrays of unused emitted definitions refuse both an Array root and a normalized _array_ definition; each refusal is deterministic. Green on its first isolated run and in its final suite.

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --test typescript_bindings array_helpers_in_alternatives_and_unused_definitions_cannot_be_shadowed -- --exact > target/review-boundaries-3/adversary-case-2.log 2>&1
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/typescript_bindings.rs (target/debug/deps/typescript_bindings-4484db42097fd431)

running 1 test
test array_helpers_in_alternatives_and_unused_definitions_cannot_be_shadowed ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

exit: 0
```

3. `crates/generate/schema-contract/tests/typescript_bindings.rs:60` — The Array definition remains usable when an items keyword on a string emits no helper; contextual root namespace and keyword/punctuation object properties retain their spelling, stable bytes and unchanged schema. Green on its first isolated run and in its final suite.

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --test typescript_bindings property_names_and_unemitted_items_do_not_reserve_bindings -- --exact > target/review-boundaries-3/adversary-case-3.log 2>&1
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/typescript_bindings.rs (target/debug/deps/typescript_bindings-4484db42097fd431)

running 1 test
test property_names_and_unemitted_items_do_not_reserve_bindings ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

exit: 0
```

4. `crates/generate/schema-contract/tests/typescript_typecheck.rs:250` — Seven distinct global-looking roots compile with nested nullable arrays, escaped definition references and emitted Array helpers. Green on its first isolated run and in its final suite.

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck nested_nullable_arrays_and_global_looking_aliases_typecheck -- --exact > target/review-boundaries-3/adversary-case-4.log 2>&1
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.25s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 1 test
test nested_nullable_arrays_and_global_looking_aliases_typecheck ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.92s

exit: 0
```

5. `crates/generate/schema-contract/tests/typescript_typecheck.rs:289` — Six contextual roots compile with Array, await, constructor, __proto__, readonly and foo/bar properties; required and quoted property spellings survive. Green on its first isolated run and in its final suite.

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck contextual_roots_and_keyword_properties_keep_compilable_spelling -- --exact > target/review-boundaries-3/adversary-case-5.log 2>&1
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 1 test
test contextual_roots_and_keyword_properties_keep_compilable_spelling ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.89s

exit: 0
```

6. `crates/generate/schema-contract/tests/typescript_typecheck.rs:330` — Three normalized definitions preserve order, escaped references, wire property names and stable repeat bytes; the accepted $Root projection compiles. Green on its first isolated run and in its final suite.

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck multiple_normalized_definitions_keep_references_and_binding_order -- --exact > target/review-boundaries-3/adversary-case-6.log 2>&1
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 1 test
test multiple_normalized_definitions_keep_references_and_binding_order ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.92s

exit: 0
```

Package suites and gates, executed after all six isolated cases:

| Lane | Before | After | Failed | Source of before |
| --- | ---: | ---: | ---: | --- |
| Default package, including integration tests | 16 | 19 | 0 | Implementation report / adversary brief |
| Explicit TypeScript compiler integration | 3 | 6 | 0 | Implementation report / adversary brief |
| Documentation tests | 0 | 0 | 0 | Implementation report / adversary brief |
| Distinct executed cases across the two selected lanes | 19 | 25 | 0 | Sum of the separate lanes |

default package:

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract > target/review-boundaries-3/adversary-default-suite.log 2>&1
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-e827a73da4bfb459)

running 16 tests
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition ... ok
test typescript::tests::module_reserved_aliases_are_refused ... ok
test typescript::tests::reserved_words_and_primitive_type_aliases_are_refused ... ok
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test typescript::tests::noncolliding_projection_retains_the_complete_baseline_bytes ... ok
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding ... ok
test typescript::tests::normalized_definition_collisions_keep_the_existing_deterministic_refusal ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test typescript::tests::unused_array_name_and_keyword_properties_keep_their_valid_bytes ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/typescript_bindings.rs (target/debug/deps/typescript_bindings-4484db42097fd431)

running 3 tests
test array_helpers_in_alternatives_and_unused_definitions_cannot_be_shadowed ... ok
test property_names_and_unemitted_items_do_not_reserve_bindings ... ok
test escaped_and_non_ascii_definition_keys_refuse_root_collisions_deterministically ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests schema_contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

explicit TypeScript compiler lane:

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck > target/review-boundaries-3/adversary-compiler-suite.log 2>&1
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 6 tests
test contextual_roots_and_keyword_properties_keep_compilable_spelling ... ok
test keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting ... ok
test nested_nullable_arrays_and_global_looking_aliases_typecheck ... ok
test compiler_rejects_a_known_duplicate_binding ... ok
test multiple_normalized_definitions_keep_references_and_binding_order ... ok
test accepted_binding_collision_and_keyword_corpus_typechecks ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.36s

exit: 0
```

package formatting:

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo fmt -p schema-contract --check > target/review-boundaries-3/adversary-fmt.log 2>&1
exit: 0
```

strict package Clippy with compiler feature:

```text
$ env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo clippy --locked -p schema-contract --all-targets --features typescript-typecheck -- -D warnings > target/review-boundaries-3/adversary-clippy.log 2>&1
    Checking schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized] target(s) in 0.15s
exit: 0
```

The selected compiler lane executes `node /usr/lib/node_modules/typescript/lib/tsc.js`, checks its version is exactly 6.0.3, and uses strict/noEmit with explicit isolated files, `types: []`, ES2022 and ESNext. The helper at `tests/typescript_typecheck.rs:11–69` is unchanged. These are actual compiler executions: missing compiler or version mismatch fails a selected test. The three new compiler cases contributed fourteen accepted generated modules per run (7 + 6 + 1). The retained negative compiler control also passed by requiring real TS2300 duplicate-binding diagnostics. This pass does not claim the optional compiler lane is selected by default CI.

Judgement findings: none. There are no finding rows or origin classifications. These observations cover the committed implementation plus the exact retained tests; no unexecuted base origin is inferred.

Reachability: the added cases call public `schema_contract::typescript::project(&schema, root)` directly. The repository CLI at `crates/edge/ess-cli/src/schema.rs:125–141` reads JSON schemas selected by `$id` and passes the caller-supplied root to this same API; its stdout/check/write branches at lines 143–159 occur after projection. The fixtures use supported structural shapes and valid JSON Schema keyword placement. In particular, `items` on a string is inapplicable and does not emit an Array helper. CLI processes were not executed in this package-only assignment.

Attacked and could not break:

- Root/definition normalization and escaped diagnostic locations across five unusual but valid keys.
- Array helper reservations throughout nested and nullable structures, including definitions emitted despite being unused by the root.
- Distinct binding names resembling globals, keyword properties, contextual aliases and inapplicable items metadata.
- Declaration ordering, escaped references, repeated valid output bytes and schema/wire property preservation.
- The unit's existing complete-byte golden, registry fixture and real compiler duplicate-binding control, retained in the final suites.

Final scope and resource checks:

```text
$ git diff --check
exit: 0
```

```text
$ git diff --no-index --check -- /dev/null crates/generate/schema-contract/tests/typescript_bindings.rs
exit: 1 (new-file difference; no whitespace diagnostics)
```

```text
$ df -B1 --output=avail .
       Avail
140306722816
exit: 0
```

```text
$ du -sk target
297064	target
exit: 0
```

Available disk before building was 140,413,333,504 bytes; at return it is 140,306,722,816 bytes, above the required 8,589,934,592-byte reserve. The unit target occupies 297,064 KiB. Every command kept the unit-local target and prescribed compact settings; CARGO_TARGET_DIR was never set.

Outside authored paths: none. Logs, report and patch remain under the assigned worktree's target/review-boundaries-3; compiler-created fixtures/configurations and build outputs remain under its assigned target. The prescribed coordinator-owned sccache socket at /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock was used for shared compiler-cache access as required by resource-supplement.md; no outside authored scratch, build directory, cache purge or lifecycle mutation was performed. The installed compiler path was read only. No network, credentials, external services or permission changes were used. Duration and token usage were not measured.

```findings
[]
```

