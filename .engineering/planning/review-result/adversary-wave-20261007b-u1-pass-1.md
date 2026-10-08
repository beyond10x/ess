---
format: aep.planning-md/3
id: review-result:adversary-wave-20261007b-u1-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 2026-10-07b unit 1 (alphabet and text count lowering)
relations:
- reviews: story:entity-runtime-lowers-alphabet-and-text-count
revision: 1
---
unit: story:entity-runtime-lowers-alphabet-and-text-count (unit commit 1510faa858 plus one untracked test file)
verdict: red
cases: executed 136→143, red 3
origin: introduced 3, pre-existing 0, undecided 1
wrote-outside-worktree: none
needs-coordinator: yes (F1's natural fix sits in ess-domain, outside the unit; F3's origin needs a run against the base)

Cases added in `crates/generate/ess-entity-runtime/tests/adversary_alphabet_count_pass1.rs`:

| case | asserts | now |
|---|---|---|
| `adv_a_sets_literal_outside_the_alphabet_…` | ESS admits `sets: note: "kept!"` against alphabet `kep xactly`; the lowering refuses it by name, or entity-core takes the branch ESS takes | red |
| `adv_alphabets_that_overlap_pairwise_…` | layers `ac`/`bc`/`ab` pass ESS's pairwise check; the result is a named refusal or an exact lowering, never `TargetDefinitionRefused` | red |
| `adv_an_input_only_text_length_invariant_…` | an unstored input of a type with `value.count <= 3`: ESS rejects `"abcd"`, entity-core must too | red |
| `adv_text_length_comparisons_…_every_boundary` | `< <= == != >= >` against 2–5, stored and input text, combining and precomposed strings, each answer compared with `Predicate::evaluate` | green |
| `adv_an_ordinal_list_element_text_length_…` | `lines.0.description.count > 3` decides the same as the ESS evaluator | green |
| `adv_an_alphabet_holds_through_list_map_union_…` | entity-core rejects a character outside the alphabet in a list item, map value, union payload and optional struct member, and nowhere else | green |
| `adv_a_response_alphabet_behind_an_optional_struct_member_…` | `AlphabetUnsupported` at exactly `Run.response.wrapped.inner` | green |

Red output, each case run alone:

```
F1 …pass1.rs:360: ESS admits `sets: note: "kept!"` and takes `completed`; the lowered creation fails every request: Err(Validation([ValidationError { path: "fields.note", message: "character '!' (U+0021) at position 5 is not in the alphabet" }]))
F2 …pass1.rs:394: a valid specification lowered to a definition entity-core refuses: [ … 4 × TargetDefinitionRefused "invalid field definition at 'schema.note' / 'create.arguments.input.note': alphabet must declare at least one character" ]
F3 …pass1.rs:608: entity-core takes `Run` with `label: "abcd"`, which ESS does not admit as a `Label`
```

Suite: `cargo test -p ess-entity-runtime --locked --no-fail-fast` exit 101, 143 run, 140 passed, 3 failed (all in `adversary_alphabet_count_pass1`). `cargo clippy -p ess-entity-runtime --all-targets --locked -- -D warnings` exit 0.

| # | file:line | verdict | origin | what reaches it |
|---|---|---|---|---|
| F1 | `ess-entity-runtime/src/lib.rs:1162` | CONFIRMED | introduced | any valid spec with a `sets:` literal outside the field's alphabet; ESS checks only a literal's prefix (`ess-domain/src/command.rs:4499`); no shipped model does this today |
| F2 | `ess-entity-runtime/src/lib.rs:1162` | CONFIRMED | introduced | three or more nested alphabets that overlap pairwise but share no character; ESS's check (`ess-domain/src/types.rs:1328`) is pairwise; ESS's own witness treats the empty case as unsupported (`interpret/protected.rs:188`) |
| F3 | `ess-entity-runtime/src/lib.rs:936` | CONFIRMED | undecided | any input whose type has invariants and is not stored; covers every invariant, not only `.count`; base callers read as the same two, not run against the base |
| F4 | `ess-entity-runtime/src/subset.rs:75` | CONFIRMED | introduced | the row and `website/docs/reference/entity-runtime-lowering.md:52,113` say a text length in a list or map element is refused; an ordinal read lowers, correctly |

Attacked without a break: intersection order (ESS rejects a repeated character), combining versus precomposed strings, every comparison boundary, alphabets through Optional, List, Map, union and struct, the response-field refusal behind an optional struct, relations.

```findings
- file: crates/generate/ess-entity-runtime/src/lib.rs
  line: 1162
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a sets literal outside the field's alphabet, which ESS admits and stores, lowers to a creation entity-core rejects on every request
- file: crates/generate/ess-entity-runtime/src/lib.rs
  line: 1162
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: pairwise-overlapping alphabets with no common character intersect to an empty alphabet and surface as TargetDefinitionRefused instead of a named refusal or max_length 0
- file: crates/generate/ess-entity-runtime/src/lib.rs
  line: 936
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: invariants of an unstored input's type, including value.count, are dropped, so entity-core accepts input ESS rejects
- file: crates/generate/ess-entity-runtime/src/subset.rs
  line: 75
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the refused row says a text length in a list or map element is refused, but an ordinal element read lowers and decides correctly
```
