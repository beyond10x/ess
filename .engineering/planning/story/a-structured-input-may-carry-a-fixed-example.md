---
format: aep.planning-md/3
id: story:a-structured-input-may-carry-a-fixed-example
kind: story
status: implemented
title: An open-struct or Json input may carry a fixed example that synthesis sends
summary: 'Accepted conformance steps cannot carry a fixed valid structured input; synthesis invents one and example: is refused (downstream deployment, 0.77.0).'
tags:
- adopter-report
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/structured_input_example.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/structured_input_example.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/undeclared_fields.rs
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: inferred
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T11:37:41Z", actor: "human:timo", revision: 14}
- {from: "proposed", to: "active", at: "2026-10-31T11:37:41Z", actor: "human:timo", revision: 15}
- {from: "active", to: "implemented", at: "2026-10-31T15:19:36Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Need

Reported by a downstream deployment's specification session (2026-10-30, against ESS 0.76.0 and
reproduced on 0.77.0). A create command takes a structured document with a required version member
and nested members, too large to type in full, so the specification types it loosely: an open
struct (`fields: []`, `undeclared_fields: ignored`) or `Json`. Every accepted conformance step of
that command then carries a document synthesis invented (`{}` for the open struct, `{"graph":
"graph"}` for `Json`), which the real service refuses for the missing version member. 45 of the
deployment's 65 failing real-service scenarios are this case. There is no way to give the input a
fixed valid value:

- `example:` on the input is refused: `ESS-COMMAND-002` "`Optional<…Graph>` reaches `…Graph`,
  which is not a scalar" for the open struct, and "`{…}` is not a value of `Optional<…Graph>`
  (Json)" for `Json`;
- `example:` on a struct field is an unknown key.

Brand-free reproduction (rewritten into a conformance fixture by the unit): a command
`CreateFlow(name: String, graph: Optional<Graph>, preset: Optional<Preset>)` with `Graph` an open
struct; outcome `no-definition` when neither `graph` nor `preset` is defined, else `created`.

## Triage on main (f692aab, 0.77.0)

Re-run 2026-10-31 with the 0.77.0 binary: output identical to the report. `synthesize` sends
`"graph": {}` at both accepted `execute_command` steps of `…/outcome/created`; the three `example:`
attempts are refused as above. Not fixed.

## Requester's proposal (theirs)

An open-struct or `Json` input (through `Optional` or a newtype) may carry an `example:` holding any
object; synthesis sends it at every accepted step that needs no other value, as it does for a
scalar's and a `List` input's `example:` (`ess/11`, `ess/31`). Optionally, an accepted step that
may omit the structured input could be told to use the alternative input instead.

## Where it lives

Lines on `main` f692aab (0.77.0), read by the scoper on 2026-10-31.

- `crates/specify/ess-domain/src/command.rs:6100-6121` `validate_examples`; `:6133-6160`
  `list_example_admitted` (the `ess/31` precedent, format-gated at `:6147-6155`); `:6229-6240`
  `example_admitted` refuses a struct, `List` and `Map` as "not a scalar".
- `crates/specify/ess-domain/src/types.rs:2755-2784` `Primitive::admits` has no `Json` arm.
- `command.rs:6376-6402` `struct_literal_admitted` refuses an undeclared member even on an open
  struct; it never reads `TypeBody::Struct.undeclared_fields` (`types.rs:1170-1185`).
- `crates/verify/ess-conformance/src/witness.rs:5767-5776` loads examples at the plain instance;
  `:5848-5850` (primitive, already `Json`) and `:5908-5910` (list) use them; the struct arm
  (`:6236-6253`) and `map` (`:6003-6040`) do not.
- `crates/verify/ess-conformance/src/input.rs:1739-1746` `project_value` refuses an undeclared member
  of any struct input, open or closed (`EssIr::undeclared_fields`, `ess-compiler/src/ir.rs:3443-3452`).
- `ess-compiler/src/ir.rs:2029-2033` `ResolvedCommand.examples` is a `BTreeMap<String, Node>` skipped
  when empty.

## Fit review

1. **Need.** An accepted step cannot carry a fixed valid value for a struct, `Map` or `Json` input;
   synthesis invents one. Minimal reproduction: `CreateDoc(name, body: Optional<Open>, preset:
   Optional<Kind>)`, `Open` an open struct with no fields; `synthesize` sends `"body": {}`.
2. **Class: gap**, plus one defect: the interpreted target refuses an undeclared member of an open
   struct in a command input ("`nodes` is not declared there", a probe of an edited suite), against
   the `ess/24` rule that an open struct is open wherever it is reached (`types.rs:1072-1078`).
3. **No idiom.** `fixture_inputs:` on the input is refused once a guard reads it (`ESS-COMMAND-009`);
   declaring the members gets an invented text value; typing the whole document is what the adopter
   cannot do.
4. **Fit.** The row "an input of type T may carry an `example:`": `ess/11` opened scalars, `ess/31`
   `List`; struct, `Map` and `Json` remain. A closed struct is already admitted inside a `List`
   example and refused at the top level, so opening only the open struct would keep that
   inconsistency. A union stays refused by name (no union-literal checker, `command.rs:6360-6367`);
   `example:` on a struct field stays refused (`command.rs:6853-6860`). `ess verify diff` already
   compares a `Node` (`ess-diff/src/change.rs:2490`).
5. **Second adopter.** `RegisterWebhook(target: Url, config: Json)` whose provider requires
   `{"version": 2}` in `config`.
6. **Cost.** One source format (`ess/33`, as `ess/31` gated the list example); no new key,
   diagnostic code, IR field or suite format; no migration.
7. **Considered.** (a) change nothing: no idiom; (b) as proposed, open struct and `Json` only: fails
   sibling generality; (c) the whole row: chosen.

## Decisions

**accept, redesigned** (wave D325). Format: `ess/33`, allocated by this unit.

- From `ess/33`, `example:` is admitted on an input whose terminal type is a struct (open or closed),
  a `Map<…>` or `Json`, through any newtype or `Optional`, checked as a precondition literal is. On an
  open struct, undeclared members are admitted and kept verbatim; declared members keep their type,
  required-ness and invariants; a closed struct refuses an undeclared member. A `Json` example is any
  value but `null`. The open-struct rule also opens list-example elements and precondition literals
  from `ess/33`. Below `ess/33` each keeps its refusal, its hint naming `ess/33`.
- Synthesis sends the example at the plain instance, as for scalars and lists: a `Map` example as
  written; a struct example's declared members as examples at `<path>.<member>` (so guards reading
  them keep their ladders) with undeclared members carried verbatim; an `Optional` member the
  example leaves out stays out. Other instances keep the invented witness.
- The interpreted target admits the undeclared members of an open struct in a command input, at
  every format (defect).
- The requester's "use the alternative input instead" half is out: synthesis already sends `preset`
  alone in its own accepted step, and the other steps pass once the example is valid.
- Models without a structured example keep their IR and suite bytes.
- CHANGELOG `[Unreleased]`: Added (`ess/33` structured examples) and Fixed (the interpreted target
  and an open struct's undeclared input members).

## Acceptance

- `crates/specify/ess-domain/tests/structured_input_example.rs` (new):
  `a_struct_map_or_json_example_is_admitted_at_ess_33_and_round_trips`;
  `an_open_struct_example_keeps_undeclared_members_and_a_closed_one_refuses_them` (also a
  `List<Open>` element with an extra member, refused today, admitted after);
  `a_structured_example_below_ess_33_keeps_its_refusal_and_names_ess_33` (a union example stays
  refused with its reason).
- `crates/verify/ess-conformance/tests/undeclared_fields.rs`:
  `an_open_struct_command_input_admits_undeclared_members_on_the_interpreted_target`; a closed
  sibling is still refused.
- `crates/verify/ess-conformance/tests/structured_input_example.rs` (new):
  `a_struct_or_json_input_with_an_example_is_sent_as_the_example` (before `{}` and
  `{"body":"body"}`; after the example; a guard on a declared member keeps its refusing scenario);
  `the_minimal_reproduction_synthesizes_and_passes_on_the_interpreted_target` (0 refusals, the
  accepted steps that carry `body` send the example, the interpreted run all passed).
