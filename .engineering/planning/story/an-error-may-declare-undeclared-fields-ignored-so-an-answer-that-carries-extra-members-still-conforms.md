---
format: aep.planning-md/3
id: story:an-error-may-declare-undeclared-fields-ignored-so-an-answer-that-carries-extra-members-still-conforms
kind: story
status: draft
title: 'An error may declare undeclared_fields: ignored, so an answer that carries extra members still conforms'
summary: 'undeclared_fields: ignored is refused on an error, admitted on a struct and a command response only, so an answer with extra members cannot conform (gap report, 2026-11-24)'
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:an-error-outcome-may-answer-its-error-s-fields-as-the-whole-json-body-with-no-member-naming-the-error
revision: 2
---
## Need

A service answers an error as an object whose declared members are stable and whose other members
vary: a readiness refusal `{"ready": false, "reasons": [...]}` that may also carry a `trace_id` or a
vendor member. The specification states the members it promises; a target that adds a member no field
names must still pass, and a client must still read the answer.

The key that says this exists, and is admitted in two places only (`crates/specify/ess-domain/src/undeclared_fields.rs:1-30`):
a `kind: struct` type, and a command with a non-empty `response:`. An error is a refusal there:

| written on | refusal |
|---|---|
| an entity, an event, an error, a view or an actor | `UnsupportedConstruct` |

So `undeclared_fields: ignored` on an error is refused by name, and a `struct` member inside the error
does not make the answer's top level open.

Reporter's wording (theirs): "an open error body", beside an error body that does not name the error in
one member (that half is the draft `story:an-error-outcome-may-answer-its-error-s-fields-as-the-whole-json-body-with-no-member-naming-the-error`).

## Outcome

An error may declare `undeclared_fields: ignored`. The object that answers it, wherever a binding
writes it, then admits members no field names: conformance and a generated client accept them, the
OpenAPI schema says `additionalProperties: true`, and a generated server writes only the declared
members.

## Acceptance

Format (D77): a new place for an existing key is a grid-row decision; it lands in the source format
whose decision opens the error row (not `ess/38` unless that decision names it); below it the key on an
error is refused as it is today, naming the format.

- At the new format, `undeclared_fields: ignored` on an error validates; `refused` is the same as
  absent; on an event, view or actor it stays `UnsupportedConstruct`.
- The compiled IR carries it on the error; an error without it keeps its IR bytes.
- A conformance run against a target whose error answer holds an extra member passes for an error
  declared `ignored` and fails for one that is not.
- `ess generate --kind openapi` writes `additionalProperties: true` on that error's schema and nothing
  new on any other; a generated Rust and Go server writes the declared members and no others.
- `docs/design/typed-response-outcome-payloads.md` (where the design cites the key) and the module
  table in `undeclared_fields.rs` list the error row as open, and `website/docs/reference/spec-versions.md` names the format.

## Scope

- cited: `crates/specify/ess-domain/src/undeclared_fields.rs` (the position table), `crates/specify/ess-domain/src/command.rs` (error declarations),
  `crates/specify/ess-compiler/src/ir.rs`, `crates/generate/ess-gen/src/openapi.rs`,
  `crates/verify/ess-conformance` (error observation).
- Size: M, about 350 changed lines.

## Fit review

1. **Need.** A record of the error row that admits members it does not declare. Reporter's syntax
   (theirs): an open error body.
2. **Class: gap** for the position (the key is documented, the error row is closed).
3. **Already expressible?** No for an error. For a success answer: yes, `undeclared_fields: ignored` on
   the command's `response:`. Declaring the extra members as fields moves a figure out of the
   specification (fix-by-idiom fails: the suite then requires members the service does not promise).
4. **Fit.** Grid row: the `undeclared_fields` positions. Siblings: event payloads and views are closed by
   design and stay refused (a producer-side record); an error is a consumer-read answer like a response.
   Every target: conformance, OpenAPI, the generated clients and the interpreter; a generated server
   writes only declared members.
5. **Second adopter.** An error carrying a `trace_id` extension member.
6. **Cost.** One new accepted position, no new key, a format step (D77).
7. **Alternatives.** Change nothing; list the extra members as optional fields; open every record (loses
   the closed-record check).

## Decisions

AC9-D6: where the key opens next. A: errors only. B: errors, events and views. C: decline; list the members as optional fields. Decided: A, 2026-11-24 (the coordinator, under the owner's delegation); accepted as proposed (the need is the existing key in a sibling position), landing at the next source format that opens this row.
