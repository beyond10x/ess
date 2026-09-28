---
format: aep.planning-md/3
id: review-result:adversary-c1-charset-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave correctness-1 unit charset
relations:
- reviews: story:a-field-constrained-by-a-charset-publishes-that-charset
revision: 1
---
unit: story:a-field-constrained-by-a-charset-publishes-that-charset
verdict: red
cases: executed 924→930, red 3 (ess-domain), plus 10 existing ess-xtask cases red (128 pass at base, 118 at head)
origin: introduced 1, pre-existing 3, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/charset/adv1/ (logs, base/ export, base-target/ 620M)
needs-coordinator: yes — reviewed-schema-metadata.json must be re-pinned; not the unit's file

Adversary pass 1 (`aep:adversary`), 2026-09-28, on head 4c369054f + untracked `crates/specify/ess-domain/tests/adversary_charset_pass1.rs`.

| case | asserts | now |
|---|---|---|
| `every_committed_document_the_parser_reads_is_admitted_at_the_new_charsets` :65 | 113 committed YAML documents RawSpecFile::parse reads get no refusal at the 5 new schema locations | green |
| `the_published_charset_agrees_with_the_constructor_at_the_boundaries` :123 | newline, a-1, Cyrillic, Arabic-Indic digit, fullwidth, NBSP, A, a--, -, 4096 chars | green |
| `a_transition_name_the_parser_refuses_the_schema_refuses` :157 | Transition.name refused by schema where serde refuses | red |
| `a_binding_written_with_its_id_alias_is_admitted_by_the_schema` :176 | binding with `id:` admitted | red |
| `a_selector_name_the_validator_refuses_the_schema_refuses` :197 | Selection.name refused by schema where validation refuses | red |

`cargo test -p ess-domain --no-fail-fast`: 927 passed, 3 failed, EXIT=101. `cargo test -p ess-xtask consumer_coverage`: head 118 passed / 10 failed; base 128 passed / 0 failed. ess-cli schema_bundle, schema_registry_identity(+adversary), empty_projection: green. `cargo xtask schema --check`: current.

- F1 introduced NEEDS-CHANGE: regenerated schema stales the pinned shape hash of `wire:RawSpecFile#/definitions` in reviewed-schema-metadata.json (10 xtask cases red); the implementor's xtask-pins.md said no byte change was needed; it also omits the fifth new node RawTopology/properties/workloads/propertyNames. Previous schema commit ac6fc6fe2 re-pinned in the same change.
- F2 pre-existing CONFIRMED: `Transition.name` (entity.rs:167) checked by deserialize_local_name but published bare — in the story's class; the Raw*-prefix enumeration missed it.
- F3 pre-existing CONFIRMED: `Selection.name` (selection.rs:236) and `SelectionInput.name` (:515) held to the field-name charset by validation, published bare.
- F4 pre-existing CONFIRMED note: the binding `id` alias (and component `component` alias) is unpublished; the schema refuses 21+ committed documents the parser reads.

Also at base, not charset: 7 committed documents use a string Ranking the schema types as an object.

```findings
[{"file":"crates/edge/ess-xtask/src/consumer_coverage/reviewed-schema-metadata.json","category":"contract-drift","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The regenerated schema stales the pinned shape hash of wire:RawSpecFile#/definitions, so 10 ess-xtask consumer_coverage tests go red (128 pass at base, 118 at head), contrary to the unit's xtask-pins.md."},{"file":"crates/specify/ess-domain/src/entity.rs","line":167,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Transition.name is refused at parse time by a single-segment QualifiedName charset but is published as a bare string, a member of the story's class that the Raw*-prefix enumeration missed."},{"file":"crates/specify/ess-domain/src/selection.rs","line":236,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"Selection.name and SelectionInput.name are held to the field-name charset by validation but are published as bare strings."},{"file":"crates/specify/ess-domain/src/binding.rs","line":206,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"The id alias for a binding name (and the component alias for a component name) is unpublished, so the schema refuses 21 or more committed documents the parser reads and the new name pattern never applies to that spelling."}]
```

Coordinator routing: F1 into the unit (re-pin, following ac6fc6fe2); F2, F3 into the unit (story class); F4 filed as its own story (not a charset defect).
