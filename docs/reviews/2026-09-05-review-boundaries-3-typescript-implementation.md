unit: story:review-typescript-root-collision — Allocate TypeScript roots and definitions in one symbol space
verdict: green
cases: executed 9→16, red 4
origin: n/a
wrote-outside-worktree: none
needs-coordinator: no

## 1. Unit, acceptance and scope

Acceptance: a requested TypeScript root that collides with a referenced definition produces valid uniquely named output or a deterministic refusal.

The implementation deterministically refuses duplicate root/normalized-definition bindings, forbidden exported alias names and declarations that shadow the actually emitted Array helper. It retains the existing definition/definition collision error, definition normalization, reference resolution, property spelling and all valid baseline projection bytes. JSON Schema remains the runtime authority; the schema input is not mutated.

The entire assigned package scope was checked against the base `45832cc885377b2d61845ee33af14f0293d99e67`, this tree's AGENTS and story revision 8. No depends_on edge was present. Inferred manifest and integration-test paths were confirmed: this package can expose a required-feature test without editing root manifests, Taskfile, CLI, workflows or the lockfile. The feature is optional for the default Rust gate and mandatory once selected; it never silently skips. Node v22.23.1 and TypeScript 6.0.3 were measured locally. The configured compiler is invoked with strict, noEmit, explicit files and types: [], under this tree's target/tmp. No npm install was needed.

The inferred allocation mechanism was measured once after the first red cases: adding root collision refusal, an emitted-helper check, and the initially established keyword restrictions produced 15/15 Rust cases but left the compiler corpus red. The compiler then identified ten additional module binding restrictions (implements, interface, let, package, private, protected, public, static, yield, await). A new Rust module-reserved case was observed red before extending the refusal set. The mechanism hypothesis was therefore incomplete; the retained compiler output supplied the missing boundary.

Class and enumeration:
- One declaration namespace: caller root, every normalized top-level definition, existing definition/definition collisions, including punctuation normalization and escaped local references.
- Feasible bindings: the compiler corpus enumerates the pinned compiler's keyword vocabulary plus ordinary, contextual, primitive, helper, eval/arguments and __esModule controls. The refusal set is limited to names the exported module cannot bind. Valid contextual aliases have positive compiler controls.
- Emitted helpers: Array at the root and nested object array positions is checked at actual array rendering. Root and normalized-definition shadowing are refused; an unused Array declaration and annotation values mentioning arrays retain their valid bytes.
- Property names: lexical property quoting remains separate from type-binding feasibility. Keywords, Array, and a quoted wire-key retain their spelling.
- Compatibility: the complete noncollision Registry output is frozen in a fixture that passed against unchanged production before implementation. Existing insertion-order and vocabulary cases remain selected.

## 2. Observed diff

Source and tests are uncommitted. The ordinary diff excludes untracked test files, so those files are additionally shown with git's no-index diff statistics. Its exit 1 means a difference exists.

```console
git --no-pager diff --stat
```

```text
 crates/generate/schema-contract/Cargo.toml        |   8 +
 crates/generate/schema-contract/src/typescript.rs | 266 +++++++++++++++++++++-
 2 files changed, 266 insertions(+), 8 deletions(-)
```

Exit: `0`.

```console
git --no-pager diff --no-index --stat /dev/null crates/generate/schema-contract/tests/fixtures/registry.ts
```

```text
 .../generate/schema-contract/tests/fixtures/registry.ts   | 15 +++++++++++++++
 1 file changed, 15 insertions(+)
```

Exit: `1`.

```console
git --no-pager diff --no-index --stat /dev/null crates/generate/schema-contract/tests/typescript_typecheck.rs
```

```text
 .../schema-contract/tests/typescript_typecheck.rs  | 247 +++++++++++++++++++++
 1 file changed, 247 insertions(+)
```

Exit: `1`.

```console
git status --short
```

```text
 M crates/generate/schema-contract/Cargo.toml
 M crates/generate/schema-contract/src/typescript.rs
?? crates/generate/schema-contract/tests/
```

Exit: `0`.

## 3. Test-first red evidence

The first default run below preceded all production edits; three new cases failed. Four unique new Rust refusal cases were observed red across the first and module-reserved runs. The compiler corpus was also observed red before production correction. Positive compatibility controls passed on the unchanged production baseline.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p schema-contract
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.40s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-e827a73da4bfb459)

running 15 tests
test typescript::tests::normalized_definition_collisions_keep_the_existing_deterministic_refusal ... ok
test typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition ... FAILED
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::noncolliding_projection_retains_the_complete_baseline_bytes ... ok
test typescript::tests::reserved_words_and_primitive_type_aliases_are_refused ... FAILED
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding ... FAILED
test typescript::tests::unused_array_name_and_keyword_properties_keep_their_valid_bytes ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok

failures:

---- typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition stdout ----

thread 'typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition' (605111) panicked at crates/generate/schema-contract/src/typescript.rs:573:39:
root must not shadow emitted Array helper: "// @generated by ESS schema-contract projection; do not edit.\n// Source: urn:example:array:1\n// JSON Schema remains authoritative for runtime validation and refinements.\n\nexport type Array = Array<string>;\n"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- typescript::tests::reserved_words_and_primitive_type_aliases_are_refused stdout ----

thread 'typescript::tests::reserved_words_and_primitive_type_aliases_are_refused' (605116) panicked at crates/generate/schema-contract/src/typescript.rs:562:13:
assertion `left == right` failed: break
  left: Ok("// @generated by ESS schema-contract projection; do not edit.\n// Source: urn:example:names:1\n// JSON Schema remains authoritative for runtime validation and refinements.\n\nexport type break = string;\n")
 right: Err(InvalidRootName("break"))

---- typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding stdout ----

thread 'typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding' (605117) panicked at crates/generate/schema-contract/src/typescript.rs:534:48:
root/definition collision must refuse: "// @generated by ESS schema-contract projection; do not edit.\n// Source: urn:example:collision:1\n// JSON Schema remains authoritative for runtime validation and refinements.\n\nexport type Item = string;\n\nexport type Item = Item;\n"


failures:
    typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition
    typescript::tests::reserved_words_and_primitive_type_aliases_are_refused
    typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding

test result: FAILED. 12 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p schema-contract --lib`
```

Exit: `101`. Log: `target/review-boundaries-3/red-default.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.46s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 3 tests
test accepted_binding_collision_and_keyword_corpus_typechecks ... FAILED
test compiler_rejects_a_known_duplicate_binding ... ok
test keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting ... ok

failures:

---- accepted_binding_collision_and_keyword_corpus_typechecks stdout ----

thread 'accepted_binding_collision_and_keyword_corpus_typechecks' (611822) panicked at crates/generate/schema-contract/tests/typescript_typecheck.rs:81:5:
accepted bindings failed TypeScript:
../../../target/tmp/typescript-611821-1/case-12.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-12.ts(5,13): error TS2457: Type alias name cannot be 'void'.
../../../target/tmp/typescript-611821-1/case-12.ts(5,18): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-15.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-15.ts(5,13): error TS2457: Type alias name cannot be 'break'.
../../../target/tmp/typescript-611821-1/case-15.ts(5,19): error TS1003: Identifier expected.
../../../target/tmp/typescript-611821-1/case-16.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-16.ts(5,13): error TS2457: Type alias name cannot be 'case'.
../../../target/tmp/typescript-611821-1/case-16.ts(5,18): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-17.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-17.ts(5,13): error TS2457: Type alias name cannot be 'catch'.
../../../target/tmp/typescript-611821-1/case-17.ts(5,19): error TS1005: '{' expected.
../../../target/tmp/typescript-611821-1/case-18.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-18.ts(5,13): error TS2457: Type alias name cannot be 'class'.
../../../target/tmp/typescript-611821-1/case-18.ts(5,19): error TS1005: '{' expected.
../../../target/tmp/typescript-611821-1/case-19.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-19.ts(5,13): error TS2457: Type alias name cannot be 'const'.
../../../target/tmp/typescript-611821-1/case-19.ts(5,19): error TS1134: Variable declaration expected.
../../../target/tmp/typescript-611821-1/case-20.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-20.ts(5,13): error TS2457: Type alias name cannot be 'continue'.
../../../target/tmp/typescript-611821-1/case-20.ts(5,22): error TS1003: Identifier expected.
../../../target/tmp/typescript-611821-1/case-21.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-21.ts(5,13): error TS2457: Type alias name cannot be 'debugger'.
../../../target/tmp/typescript-611821-1/case-21.ts(5,22): error TS1005: ';' expected.
../../../target/tmp/typescript-611821-1/case-22.ts(5,13): error TS1359: Identifier expected. 'default' is a reserved word that cannot be used here.
../../../target/tmp/typescript-611821-1/case-22.ts(5,21): error TS1005: ';' expected.
../../../target/tmp/typescript-611821-1/case-23.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-23.ts(5,13): error TS2457: Type alias name cannot be 'delete'.
../../../target/tmp/typescript-611821-1/case-23.ts(5,20): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-24.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-24.ts(5,13): error TS2457: Type alias name cannot be 'do'.
../../../target/tmp/typescript-611821-1/case-24.ts(5,16): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-24.ts(5,25): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-24.ts(6,1): error TS1005: ')' expected.
../../../target/tmp/typescript-611821-1/case-24.ts(6,1): error TS1005: 'while' expected.
../../../target/tmp/typescript-611821-1/case-25.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-25.ts(5,13): error TS2457: Type alias name cannot be 'else'.
../../../target/tmp/typescript-611821-1/case-25.ts(5,18): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-26.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-26.ts(5,13): error TS2457: Type alias name cannot be 'enum'.
../../../target/tmp/typescript-611821-1/case-26.ts(5,18): error TS1003: Identifier expected.
../../../target/tmp/typescript-611821-1/case-27.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-27.ts(5,13): error TS2457: Type alias name cannot be 'export'.
../../../target/tmp/typescript-611821-1/case-28.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-28.ts(5,13): error TS2457: Type alias name cannot be 'extends'.
../../../target/tmp/typescript-611821-1/case-28.ts(5,21): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-29.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-29.ts(5,13): error TS2457: Type alias name cannot be 'false'.
../../../target/tmp/typescript-611821-1/case-30.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-30.ts(5,13): error TS2457: Type alias name cannot be 'finally'.
../../../target/tmp/typescript-611821-1/case-30.ts(5,21): error TS1005: '{' expected.
../../../target/tmp/typescript-611821-1/case-31.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-31.ts(5,13): error TS2457: Type alias name cannot be 'for'.
../../../target/tmp/typescript-611821-1/case-31.ts(5,17): error TS1005: '(' expected.
../../../target/tmp/typescript-611821-1/case-31.ts(5,26): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-31.ts(6,1): error TS1005: ')' expected.
../../../target/tmp/typescript-611821-1/case-31.ts(6,1): error TS1005: ';' expected.
../../../target/tmp/typescript-611821-1/case-32.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-32.ts(5,13): error TS2457: Type alias name cannot be 'function'.
../../../target/tmp/typescript-611821-1/case-32.ts(5,22): error TS1003: Identifier expected.
../../../target/tmp/typescript-611821-1/case-33.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-33.ts(5,13): error TS2457: Type alias name cannot be 'if'.
../../../target/tmp/typescript-611821-1/case-33.ts(5,16): error TS1005: '(' expected.
../../../target/tmp/typescript-611821-1/case-33.ts(5,24): error TS1005: ')' expected.
../../../target/tmp/typescript-611821-1/case-34.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-34.ts(5,13): error TS2457: Type alias name cannot be 'import'.
../../../target/tmp/typescript-611821-1/case-34.ts(5,20): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-35.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-35.ts(5,16): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-36.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-36.ts(5,24): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-37.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-37.ts(5,13): error TS2457: Type alias name cannot be 'new'.
../../../target/tmp/typescript-611821-1/case-37.ts(5,17): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-38.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-38.ts(5,13): error TS2457: Type alias name cannot be 'null'.
../../../target/tmp/typescript-611821-1/case-39.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-39.ts(5,13): error TS2457: Type alias name cannot be 'return'.
../../../target/tmp/typescript-611821-1/case-39.ts(5,20): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-40.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-40.ts(5,13): error TS2457: Type alias name cannot be 'super'.
../../../target/tmp/typescript-611821-1/case-40.ts(5,19): error TS1034: 'super' must be followed by an argument list or member access.
../../../target/tmp/typescript-611821-1/case-41.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-41.ts(5,13): error TS2457: Type alias name cannot be 'switch'.
../../../target/tmp/typescript-611821-1/case-41.ts(5,20): error TS1005: '(' expected.
../../../target/tmp/typescript-611821-1/case-41.ts(5,28): error TS1005: ')' expected.
../../../target/tmp/typescript-611821-1/case-41.ts(6,1): error TS1005: '}' expected.
../../../target/tmp/typescript-611821-1/case-42.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-42.ts(5,13): error TS2457: Type alias name cannot be 'this'.
../../../target/tmp/typescript-611821-1/case-43.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-43.ts(5,13): error TS2457: Type alias name cannot be 'throw'.
../../../target/tmp/typescript-611821-1/case-43.ts(5,19): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-44.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-44.ts(5,13): error TS2457: Type alias name cannot be 'true'.
../../../target/tmp/typescript-611821-1/case-45.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-45.ts(5,13): error TS2457: Type alias name cannot be 'try'.
../../../target/tmp/typescript-611821-1/case-45.ts(5,17): error TS1005: '{' expected.
../../../target/tmp/typescript-611821-1/case-46.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-46.ts(5,13): error TS2457: Type alias name cannot be 'typeof'.
../../../target/tmp/typescript-611821-1/case-46.ts(5,20): error TS1109: Expression expected.
../../../target/tmp/typescript-611821-1/case-47.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-47.ts(5,13): error TS2457: Type alias name cannot be 'var'.
../../../target/tmp/typescript-611821-1/case-47.ts(5,17): error TS1134: Variable declaration expected.
../../../target/tmp/typescript-611821-1/case-48.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-48.ts(5,13): error TS2457: Type alias name cannot be 'while'.
../../../target/tmp/typescript-611821-1/case-48.ts(5,19): error TS1005: '(' expected.
../../../target/tmp/typescript-611821-1/case-48.ts(5,27): error TS1005: ')' expected.
../../../target/tmp/typescript-611821-1/case-49.ts(5,1): error TS1128: Declaration or statement expected.
../../../target/tmp/typescript-611821-1/case-49.ts(5,13): error TS2457: Type alias name cannot be 'with'.
../../../target/tmp/typescript-611821-1/case-49.ts(5,18): error TS1005: '(' expected.
../../../target/tmp/typescript-611821-1/case-49.ts(5,26): error TS1005: ')' expected.
../../../target/tmp/typescript-611821-1/case-60.ts(5,13): error TS1005: '{' expected.


note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    accepted_binding_collision_and_keyword_corpus_typechecks

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s

error: test failed, to rerun pass `-p schema-contract --test typescript_typecheck`
```

Exit: `101`. Log: `target/review-boundaries-3/red-compiler.log`.

### Isolated mechanism measurement

No additional correction was made between applying the initial scoped mechanism and these measurements.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p schema-contract
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.38s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-e827a73da4bfb459)

running 15 tests
test typescript::tests::reserved_words_and_primitive_type_aliases_are_refused ... ok
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test typescript::tests::normalized_definition_collisions_keep_the_existing_deterministic_refusal ... ok
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition ... ok
test typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding ... ok
test typescript::tests::noncolliding_projection_retains_the_complete_baseline_bytes ... ok
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test typescript::tests::unused_array_name_and_keyword_properties_keep_their_valid_bytes ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests schema_contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Exit: `0`. Log: `target/review-boundaries-3/mechanism-default.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.46s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 3 tests
test keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting ... ok
test compiler_rejects_a_known_duplicate_binding ... ok
test accepted_binding_collision_and_keyword_corpus_typechecks ... FAILED

failures:

---- accepted_binding_collision_and_keyword_corpus_typechecks stdout ----

thread 'accepted_binding_collision_and_keyword_corpus_typechecks' (621716) panicked at crates/generate/schema-contract/tests/typescript_typecheck.rs:81:5:
accepted bindings failed TypeScript:
../../../target/tmp/typescript-621715-2/case-10.ts(5,13): error TS1214: Identifier expected. 'public' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-11.ts(5,13): error TS1214: Identifier expected. 'static' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-12.ts(5,13): error TS1214: Identifier expected. 'yield' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-16.ts(5,13): error TS1262: Identifier expected. 'await' is a reserved word at the top-level of a module.
../../../target/tmp/typescript-621715-2/case-4.ts(5,13): error TS1214: Identifier expected. 'implements' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-5.ts(5,13): error TS1214: Identifier expected. 'interface' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-6.ts(5,13): error TS1214: Identifier expected. 'let' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-7.ts(5,13): error TS1214: Identifier expected. 'package' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-8.ts(5,13): error TS1214: Identifier expected. 'private' is a reserved word in strict mode. Modules are automatically in strict mode.
../../../target/tmp/typescript-621715-2/case-9.ts(5,13): error TS1214: Identifier expected. 'protected' is a reserved word in strict mode. Modules are automatically in strict mode.


note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    accepted_binding_collision_and_keyword_corpus_typechecks

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s

error: test failed, to rerun pass `-p schema-contract --test typescript_typecheck`
```

Exit: `101`. Log: `target/review-boundaries-3/mechanism-compiler.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p schema-contract
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.41s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-e827a73da4bfb459)

running 16 tests
test typescript::tests::normalized_definition_collisions_keep_the_existing_deterministic_refusal ... ok
test typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition ... ok
test typescript::tests::module_reserved_aliases_are_refused ... FAILED
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test typescript::tests::noncolliding_projection_retains_the_complete_baseline_bytes ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding ... ok
test typescript::tests::reserved_words_and_primitive_type_aliases_are_refused ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test typescript::tests::unused_array_name_and_keyword_properties_keep_their_valid_bytes ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok

failures:

---- typescript::tests::module_reserved_aliases_are_refused stdout ----

thread 'typescript::tests::module_reserved_aliases_are_refused' (624696) panicked at crates/generate/schema-contract/src/typescript.rs:614:13:
assertion `left == right` failed: implements
  left: Ok("// @generated by ESS schema-contract projection; do not edit.\n// Source: urn:example:names:1\n// JSON Schema remains authoritative for runtime validation and refinements.\n\nexport type implements = string;\n")
 right: Err(InvalidRootName("implements"))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    typescript::tests::module_reserved_aliases_are_refused

test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p schema-contract --lib`
```

Exit: `101`. Log: `target/review-boundaries-3/red-module-reserved.log`.

## 4. Baselines and final package gates

Runner counts for identical selected commands:
- Default package unit lane: executed 9 → 16, exit 0.
- Default package doc-test lane: executed 0 → 0, exit 0. No documentation examples were added; the new cases live in the unit and explicit compiler lanes.
- Explicit compiler integration lane: executed 0 → 3, exit 0. The new lane's before count is the runner's printed zero from a feature/empty-target scaffold before adding compiler cases or changing production; that zero run is baseline evidence, not a verification claim.
- Missing-compiler negative execution: 3 selected, 0 passed, 3 failed, 0 ignored, exit 101, as required.
- Package formatter, default Clippy and feature-enabled Clippy each exit 0.

Final Rust test execution took 0.01s after 0.39s compilation; the final compiler test runner took 1.06s after 0.47s compilation. These are runner-reported durations, not an inferred total task time.

The environment for final gates includes the coordinator's dedicated sccache socket. The earlier baseline and initial red runs preceded the resource supplement and used the original prescribed environment without that socket. TMPDIR and Cargo build output remained inside the assigned tree throughout; CARGO_TARGET_DIR was never set.

```console
node --version
node /usr/lib/node_modules/typescript/lib/tsc.js --version
```

```text
v22.23.1
Version 6.0.3
```

Exit: `0`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p schema-contract
```

Output (verbatim):

```text
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling autocfg v1.5.1
   Compiling serde_core v1.0.229
   Compiling libc v0.2.189
   Compiling serde v1.0.229
   Compiling memchr v2.8.3
   Compiling cfg-if v1.0.4
   Compiling zerocopy v0.8.56
   Compiling version_check v0.9.5
   Compiling zmij v1.0.23
   Compiling getrandom v0.3.4
   Compiling regex-syntax v0.8.11
   Compiling ref-cast v1.0.27
   Compiling serde_json v1.0.151
   Compiling parking_lot_core v0.9.12
   Compiling scopeguard v1.2.0
   Compiling once_cell v1.21.4
   Compiling smallvec v1.16.0
   Compiling itoa v1.0.18
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling unicode-general-category v1.1.0
   Compiling bit-vec v0.8.0
   Compiling borrow-or-share v0.2.4
   Compiling equivalent v1.0.2
   Compiling lock_api v0.4.14
   Compiling heck v0.5.0
   Compiling thiserror v2.0.20
   Compiling bytecount v0.6.9
   Compiling vsimd v0.8.0
   Compiling micromap v0.3.0
   Compiling outref v0.5.2
   Compiling percent-encoding v2.3.2
   Compiling ahash v0.8.12
   Compiling num-traits v0.2.19
   Compiling bit-set v0.8.0
   Compiling num-cmp v0.1.0
   Compiling data-encoding v2.11.1
   Compiling hashbrown v0.17.1
   Compiling aho-corasick v1.1.5
   Compiling uuid-simd v0.8.0
   Compiling syn v3.0.4
   Compiling syn v2.0.119
   Compiling num-integer v0.1.47
   Compiling num-complex v0.4.6
   Compiling num-bigint v0.4.8
   Compiling num-iter v0.1.46
   Compiling parking_lot v0.12.5
   Compiling regex-automata v0.4.18
   Compiling jsonschema-regex v0.52.1
   Compiling num-rational v0.4.2
   Compiling strum_macros v0.28.0
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror-impl v2.0.20
   Compiling num v0.4.3
   Compiling fraction v0.17.0
   Compiling strum v0.28.0
   Compiling regex v1.13.1
   Compiling fancy-regex v0.19.0
   Compiling fluent-uri v0.4.1
   Compiling email_address v0.2.9
   Compiling jsonschema-value v0.52.1
   Compiling referencing v0.52.1
   Compiling jsonschema v0.52.1
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 9.04s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-e827a73da4bfb459)

running 9 tests
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests schema_contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Exit: `0`. Log: `target/review-boundaries-3/baseline-default.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.33s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Exit: `0`. Log: `target/review-boundaries-3/baseline-compiler.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p schema-contract
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.39s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-e827a73da4bfb459)

running 16 tests
test typescript::tests::module_reserved_aliases_are_refused ... ok
test typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition ... ok
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test typescript::tests::normalized_definition_collisions_keep_the_existing_deterministic_refusal ... ok
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::noncolliding_projection_retains_the_complete_baseline_bytes ... ok
test typescript::tests::reserved_words_and_primitive_type_aliases_are_refused ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test typescript::tests::unused_array_name_and_keyword_properties_keep_their_valid_bytes ... ok
test typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests schema_contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Exit: `0`. Log: `target/review-boundaries-3/final-default.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo fmt -p schema-contract --check
```

Output (verbatim):

```text
```

Exit: `0`. Log: `target/review-boundaries-3/final-format.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p schema-contract --all-targets -- -D warnings
```

Output (verbatim):

```text
    Checking memchr v2.8.3
    Checking cfg-if v1.0.4
    Checking regex-syntax v0.8.11
    Checking smallvec v1.16.0
    Checking scopeguard v1.2.0
    Checking once_cell v1.21.4
    Checking itoa v1.0.18
    Checking serde_core v1.0.229
    Checking libc v0.2.189
    Checking zmij v1.0.23
    Checking zerocopy v0.8.56
    Checking equivalent v1.0.2
    Checking borrow-or-share v0.2.4
    Checking foldhash v0.2.0
    Checking allocator-api2 v0.2.21
    Checking num-traits v0.2.19
    Checking bit-vec v0.8.0
    Checking ref-cast v1.0.27
    Checking outref v0.5.2
    Checking vsimd v0.8.0
    Checking num-cmp v0.1.0
    Checking percent-encoding v2.3.2
    Checking micromap v0.3.0
    Checking lock_api v0.4.14
    Checking bytecount v0.6.9
    Checking unicode-general-category v1.1.0
    Checking bit-set v0.8.0
    Checking strum v0.28.0
    Checking data-encoding v2.11.1
    Checking thiserror v2.0.20
    Checking hashbrown v0.17.1
    Checking aho-corasick v1.1.5
    Checking uuid-simd v0.8.0
    Checking num-integer v0.1.47
    Checking num-complex v0.4.6
    Checking num-bigint v0.4.8
    Checking num-iter v0.1.46
    Checking getrandom v0.3.4
    Checking parking_lot_core v0.9.12
    Checking parking_lot v0.12.5
    Checking regex-automata v0.4.18
    Checking jsonschema-regex v0.52.1
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking num-rational v0.4.2
    Checking fluent-uri v0.4.1
    Checking email_address v0.2.9
    Checking num v0.4.3
    Checking fraction v0.17.0
    Checking ahash v0.8.12
    Checking regex v1.13.1
    Checking fancy-regex v0.19.0
    Checking referencing v0.52.1
    Checking jsonschema-value v0.52.1
    Checking jsonschema v0.52.1
    Checking schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized] target(s) in 5.34s
```

Exit: `0`. Log: `target/review-boundaries-3/final-clippy.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p schema-contract --all-targets --features typescript-typecheck -- -D warnings
```

Output (verbatim):

```text
    Checking schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized] target(s) in 0.12s
```

Exit: `0`. Log: `target/review-boundaries-3/final-clippy-compiler.log`.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck
```

Output (verbatim):

```text
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.47s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 3 tests
test compiler_rejects_a_known_duplicate_binding ... ok
test accepted_binding_collision_and_keyword_corpus_typechecks ... ok
test keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s

```

Exit: `0`. Log: `target/review-boundaries-3/final-compiler.log`.

### Missing compiler must fail

The explicit nonexistent configured path below was never created. The lane failed all three selected cases and ignored none.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 ESS_TYPESCRIPT_COMPILER="$PWD/target/missing-tsc.js" cargo test --locked -p schema-contract --features typescript-typecheck --test typescript_typecheck
```

Output (verbatim):

```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/typescript_typecheck.rs (target/debug/deps/typescript_typecheck-1436018e6e437030)

running 3 tests
test compiler_rejects_a_known_duplicate_binding ... FAILED
test keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting ... FAILED
test accepted_binding_collision_and_keyword_corpus_typechecks ... FAILED

failures:

---- compiler_rejects_a_known_duplicate_binding stdout ----

thread 'compiler_rejects_a_known_duplicate_binding' (629078) panicked at crates/generate/schema-contract/tests/typescript_typecheck.rs:16:9:
ESS_TYPESCRIPT_COMPILER must name an installed tsc.js: /home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/target/missing-tsc.js
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting stdout ----

thread 'keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting' (629079) panicked at crates/generate/schema-contract/tests/typescript_typecheck.rs:16:9:
ESS_TYPESCRIPT_COMPILER must name an installed tsc.js: /home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/target/missing-tsc.js

---- accepted_binding_collision_and_keyword_corpus_typechecks stdout ----

thread 'accepted_binding_collision_and_keyword_corpus_typechecks' (629077) panicked at crates/generate/schema-contract/tests/typescript_typecheck.rs:16:9:
ESS_TYPESCRIPT_COMPILER must name an installed tsc.js: /home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/target/missing-tsc.js


failures:
    accepted_binding_collision_and_keyword_corpus_typechecks
    compiler_rejects_a_known_duplicate_binding
    keyword_properties_and_valid_contextual_aliases_typecheck_without_rewriting

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p schema-contract --test typescript_typecheck`
```

Exit: `101`. Log: `target/review-boundaries-3/missing-compiler.log`.

### Lint correction retained

Feature-enabled Clippy initially found the compiler corpus function exceeded the line limit. Moving the unchanged corpus to a module constant resolved it; no check was weakened, allowed, ignored or removed. The final formatter, feature Clippy and compiler tests above ran after that refactoring.

Command:

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p schema-contract --all-targets --features typescript-typecheck -- -D warnings
```

Output (verbatim):

```text
    Checking schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-typescript-root-collision/crates/generate/schema-contract)
error: this function has too many lines (120/100)
  --> crates/generate/schema-contract/tests/typescript_typecheck.rs:76:1
   |
76 | fn accepted_binding_collision_and_keyword_corpus_typechecks() {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
   = note: `-D clippy::too-many-lines` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: could not compile `schema-contract` (test "typescript_typecheck") due to 1 previous error
```

Exit: `101`. Log: `target/review-boundaries-3/red-clippy-compiler.log`.

```console
git diff --check
```

```text
```

Exit: `0`.

## 5. Deliberate boundaries and handoff

- No CLI, Taskfile, root manifest/lock, shared workflow, planning, dependency, schema namespace, generated repository output or Rust synthesis changes were needed.
- No published TypeScript spelling was renamed. Refusal is the story-permitted answer to infeasible bindings. The existing definition/definition error remains intact; root and helper conflicts use deterministic InvalidShape pointers/messages.
- The compiler feature does not run in the current default Rust CI unless selected and supplied its compiler; the exact successful selection above is the coordinator's repeatable target gate. The brief explicitly assigned this separate lane.
- Full workspace task check and any required site build belong to the coordinator. No claim is made that they ran here.
- No AEP, Git staging/commit/push or worktree lifecycle operation was performed. This tree, build artifacts and report are retained for the coordinator/adversary.
- This unit addresses declaration binding feasibility; it does not change the supported JSON Schema structural vocabulary or introduce a persisted construct/version.

Before the post-supplement builds, free space was 140530360320 bytes. The final measurement below exceeds the 8589934592-byte reserve.

```console
df -B1 .
```

```text
Filesystem        1B-blocks         Used    Available Use% Mounted on
/dev/nvme0n1p2 910126964736 723368538112 140451074048  84% /
```

Exit: `0`.

## 6. Writes outside the assigned tree

None. Authored files are limited to crates/generate/schema-contract/** and assigned target/review-boundaries-3 scratch; compiler fixtures/configuration and build products are under target. The prescribed compiler cache server is coordinator-owned and was accessed through its assigned socket. No outside scratch, temporary checker, proposed shared patch or cleanup was created.

