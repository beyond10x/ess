---
format: aep.planning-md/3
id: story:an-outcome-may-append-to-a-stored-list
kind: story
status: implemented
title: An outcome may append one element to a stored list
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T07:33:19Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-23T07:33:19Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:30Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Need

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22). An outcome adds one element to a stored list: the adopter's artifact-append command stores `coalesce(subject.artifacts, []) + [input.artifact]` (adopter `ess/domains/tasks.yaml:1409-1415`). ESS refuses the expression, so the outcome cannot say the list grows by one, and the stored list is left unspecified.

## Reproduction

Minimal spec on 0.68.0: an entity with `items: Optional<List<String>>` and an updating outcome with `sets: {items: {expr: "coalesce(subject.items, []) + [input.item]"}}`. `ess specify validate`:

```text
error[ESS-SPEC-012]: `coalesce(subject.items, []) + [input.item]`: `[` is no part of an expression; it is written with paths, literals, `+ - * /`, parentheses and function calls
  `ess-domain` refuses this as `unparsable_predicate`
```

## Where it lives

- The expression lexer refuses `[`: `crates/specify/ess-primitives/src/expr.rs:1453-1457`, traced through the message.
- No `sets:` source appends to a list (searched `ess-domain` and `ess-compiler`: no append form), inferred from the search.

## Acceptance (provisional)

- An outcome may store a list that is the subject's list, absent read as empty, with one element added at its end, written either as an expression with a list literal or as a dedicated `sets:` source; the design picks one.
- Validation types the element against the list's element type and refuses a mismatch with a named code.
- Generated Rust and Go store the appended list; the interpreter and conformance check the row holds the earlier elements and the new one last.

## Fit review

Source: an adopter's server implementation (wave 3, 2026-10-22). Its artifact-append command stores
`coalesce(subject.artifacts, []) + [input.artifact]`, where `artifacts` is an `Optional<List<…>>`
of a struct type.

1. **Need.** A command that adds one element to the end of a stored list cannot say so. The domain
   fact is "after this outcome the row's list is the list it held before, absent read as empty, with
   the input's element last". ESS can say neither "keep the earlier elements" nor "the new one is
   last". Minimal reproduction (no adopter text): an entity with `items: Optional<List<String>>`, an
   updating outcome with `sets: {items: {expr: "coalesce(subject.items, []) + [input.item]"}}`.
   On 0.68.0 `ess specify validate` refuses it as `ESS-SPEC-012` (`[` is no part of an expression),
   the story's `## Reproduction`. The adopter's proposed syntax is that expression.
2. **Class.** Gap. No `sets:` source writes a list holding anything: `[]` (ess/31) is only the empty
   list (`crates/specify/ess-domain/src/command.rs`, `PayloadSource::EmptyList`, "a list holding
   anything keeps its refusal"), and the expression grammar has no list value
   (`crates/specify/ess-primitives/src/expr.rs:14-19`, `1452-1457`).
3. **Already expressible?** No idiom keeps the figure checked.
   - `items: input.items`, with the caller sending the whole new list, compiles. But "the earlier
     elements are kept" then becomes the caller's promise, not the specification's: a server storing
     only what it is sent passes. That moves the checked figure out of the specification
     (`docs/design/generic-language-features.md:546-551`, policy 1), so it is no answer.
   - `{expr: …}` cannot hold a list or a struct at all. `Kind` is the scalar primitives only, and "an
     enum, a `Uuid`, a `Binary64`, a struct — is no operand" (`expr.rs:47-66`). The adopter's element
     is a struct.
4. **Fit.** Grid O (`docs/design/language-grids.md` § Grid O): the List / Map column has
   `forall`/`exists`/`distinct`/`.count` and nothing that builds a list. G1's collection family is
   `count`, `exists`, `forall` and `distinct` only (`generic-language-features.md:98`); no list
   construction was decided. Grid R: row `sets:` (subject) already reads input and subject. The
   chosen source reads exactly those two roots, so it opens no column of grid R.
   - Vocabulary. It mirrors `{increment: n}`, the existing source that is "the subject's value before
     this outcome, changed": a `sets:` source only, on an existing subject, refused on `creates:`,
     on payloads, inside a nested mapping and on set effects (`value_expression.rs:1307-1330`;
     `set_effects.rs:1717`). The element is spelled `input.<path>`, as every other `sets:` input
     read is.
   - G1's spelling table already holds mapping forms that lower to expressions, `{increment: n}` ↦
     `subject.f + n` (`generic-language-features.md:152`). `{append: input.x}` gets the same row:
     `coalesce(subject.f, []) + [input.x]`, once G1 has list values.
   - Targets:
     - validation types the element against the list's element type (`type_mismatch`);
     - the compiler lowers it to a new IR variant `ResolvedPayloadValue::Append`, carried only by a
       model that uses it;
     - the interpreter stores the earlier list plus the element;
     - generated Rust and Go append to a copy of the previous list;
     - synthesis expects the earlier elements and the new one last, read from the arranged row;
     - Entity Runtime refuses it by name, as it refuses `{increment: …}`;
     - the browser player skips it, as it skips every subject-reading source;
     - `ess verify diff` and the generated docs describe it.
5. **Second adopter.** An audit log: `AppendNote` stores `notes: {append: input.note}` on a ticket,
   so a target that overwrites the notes fails. Also a membership roster that `AddMember` grows by
   one.
6. **Cost.**
   - One new keyword, `append`, in a `sets:` mapping.
   - No new `ValidationCode`: refusals use `type_mismatch`, `conflicting_declaration`,
     `unsupported_construct` and `unsupported_format_version`.
   - One additive IR variant. Models without it keep their IR bytes. No new suite format: the
     expectation is an ordinary literal row value.
   - It joins the open `ess/31` row (`FORMAT_HISTORY`, release `None`), whose first addition is the
     sibling `[]` source. Both answer "a `sets:` entry stores a list value", so this is the same row,
     not a new decision.
   - Below `ess/31` the mapping `{append: input.x}` reads as the nested mapping it always was (a
     struct with a member `append`), so no older document changes meaning. Over a list field it is
     refused naming `ess/31`.
   - The changelog counts it one-to-one.
7. **Alternatives.**
   - (a) Change nothing. The adopter keeps the list unspecified, and a target that drops the earlier
     elements passes.
   - (b) The adopter's expression: list literals, `+` over lists and `coalesce` over lists in G1.
     Not chosen. It needs list and struct kinds in the expression checker and in each evaluator
     (`prim/expr.rs`, the Go and TypeScript predicate runtimes, the generated Rust and Go expression
     renderers). By the grid rule it would open the whole List column of grid O in every position,
     guards included: `==` on lists, list literals in `when:`. G1 has not decided that, and it is a
     language decision, not one request's.
   - (c) A dedicated `sets:` source, `{append: input.<path>}`. Chosen: smallest surface that keeps
     the figure checked, same shape as `{increment: n}`, and it lowers to (b) if G1 later opens the
     List column.

## Decisions

**Accept, redesigned.** The need is real (a gap). The adopter's expression is not adopted: list
values are not part of G1's expression language, and the element is a struct, which no expression
holds. The design is a dedicated `sets:` source:

```yaml
sets:
  items: {append: input.item}
```

- **Target.** An entity field typed `List<T>` or `Optional<List<T>>` of the outcome's existing
  subject (`updates:`/`moves:`). The stored value is the list the row held before the outcome
  (absent read as empty) with the element last. An `Optional` target is present afterwards.
- **Element.** `input.<path>`, from `ess/22` a path into a struct input. The read type must be `T`
  exactly. An `Optional` read is refused, because an absent element has nothing to append; a
  declared conversion is not applied.
- **Refused** (each refusal names the source):
  - on `creates:` (`conflicting_declaration`, no row before it);
  - in an event or error `payload:` (`conflicting_declaration`, a `sets:` source only);
  - inside a nested mapping (`type_mismatch`);
  - on a set effect's `sets:` (`unsupported_construct`, as `{increment: …}`);
  - over a field that is no `List<…>` (`type_mismatch`);
  - an element of another type (`type_mismatch`, naming both types);
  - an element that is not `input.<path>` (a reader error naming `input.<path>`).
- **Format.** `ess/31`, an addition to its open row. Below it the shape is the nested mapping it
  always was, and over a `List<…>` field it is refused as `unsupported_format_version` naming
  `ess/31`.
- **Other element sources.** Subject, related, caller and literal elements are not admitted in this
  cut. The refusal names `input.<path>`. They open together with G1's list values, not one by one.
- **Reply to the adopter.** "Accepted, redesigned: write `artifacts: {append: input.artifact}` under
  `format: ess/31`; the expression form is not adopted because ESS expressions hold no lists or
  structs."

## Acceptance

ess-domain, `crates/specify/ess-domain/tests/append_set.rs`:
- `an_update_may_append_an_input_to_a_stored_list`: `{append: input.item}` over `List<String>` and
  over `Optional<List<String>>` at `ess/31` validates, and the source is `PayloadSource::Append`.
- `an_element_of_another_type_is_refused_as_type_mismatch`: an `Integer` input appended to
  `List<String>` is refused as `type_mismatch`, naming both types.
- `append_over_a_field_that_is_no_list_is_refused`: a `String` target is refused as
  `type_mismatch`.
- `append_on_a_creating_outcome_is_refused`: `conflicting_declaration`.
- `append_in_an_event_payload_is_refused`: `conflicting_declaration`.
- `append_below_ess_31_is_refused_naming_ess_31`: a list target at `ess/30` is refused as
  `unsupported_format_version` naming `ess/31`.
- `a_struct_member_named_append_below_ess_31_is_a_nested_mapping`: a struct target whose member is
  `append` still validates at `ess/30`.

ess-conformance, `crates/verify/ess-conformance/tests/append_set.rs`:
- `the_interpreter_appends_the_input_last`: after two appends the view row holds the earlier element
  and then the new one, and an absent `Optional` list reads as empty.
- `the_synthesized_suite_passes_on_the_interpreted_target`: every synthesized scenario passes, and
  the appending branch's scenario asserts the row's list exactly.
- `models_without_it_keep_their_ir_bytes`.

ess-synth, `crates/generate/ess-synth/tests/append_set.rs`:
- `generated_rust_appends_to_the_previous_list`: the behaviour is generated and stores the previous
  list with the element pushed last.
- `generated_go_appends_to_the_previous_list_and_builds`: the same in Go, and the module builds.
