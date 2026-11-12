---
format: aep.planning-md/3
id: story:two-errors-of-one-wire-name-and-different-fields-bind-on-one-status
kind: story
status: active
title: Two errors of one wire name and different fields bind on one status
summary: 'ESS-HTTP-025 refuses the pair without alike: (cannot be told apart) and with it (the wire tells them apart) on 0.93.0 (adopter report)'
tags:
- backlog
- defect
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
- informed_by: story:two-outcomes-answered-alike-may-share-a-status-and-body
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:23Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:25Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Defect

Two outcomes of one command refuse with errors that share a wire name and differ in their fields.
On one status with `{error: error}` no `ess-http/1` binding of the pair validates on 0.93.0, and
the two refusals give opposite reasons: without `alike:` the bodies "cannot be told apart", with
`alike:` "the wire tells them apart".

Model (`format: ess/37`, validates):

```yaml
errors:
  - name: demo.t.Boom
    naming: {wire: internal_error}
    fields:
      - {name: message, type: String}
  - name: demo.t.BoomMore
    naming: {wire: internal_error}
    fields:
      - {name: message, type: String}
      - {name: details, type: String}
```

Command `demo.t.Rename` has outcomes `boom` (`error: demo.t.Boom`) and `boom-more`
(`error: demo.t.BoomMore`). Binding:

```yaml
routes:
  - command: demo.t.Rename
    method: put
    path: /api/items/{id}
    parameters: {id: input.id}
    outcomes:
      no-such-item: {status: 404, body: {error: error}}
      boom: {status: 500, body: {error: error}}
      boom-more: {status: 500, body: {error: error}}
      renamed: {status: 200, body: response}
```

`ess specify http validate --path model --binding binding.yaml`:

```
error[ESS-HTTP-025]: routes[0].outcomes.boom: outcome `boom` of command `demo.t.Rename` answers `500` with `{error: error}`, and so does `boom-more` (`{error: error}`), with a body a reader cannot tell apart; two outcomes on one status are told apart by two `envelope` bodies, by `{<member>: error}` on one member with distinct error wire names, or by two redirects, one always writing a parameter the other never writes or both writing one parameter from `error.wire` with distinct error wire names
```

The same binding with `alike: [[boom, boom-more]]` on the route:

```
error[ESS-HTTP-025]: routes[0].alike: `alike[0]` (`[boom, boom-more]`): `boom` and `boom-more` answer `500` with `{error: error}`, which writes each error's own fields beside the member, and their errors' fields differ in name, wire name, type or order, so the wire tells them apart
```

## Acceptance

Decided by the fit review (2026-11-12): the pair validates without `alike:`.

- One function decides whether the wire tells two `{<member>: error}` bodies of one wire name
  apart: one error always writes a field, by wire name and with a type that is not `Optional`,
  that the other never writes. Both `ESS-HTTP-025` checks use it.
- The pair of the Defect section validates without `alike:`.
- With `alike: [[boom, boom-more]]` it is refused, and the message names the deciding field
  (`details`).
- A pair that differs only in an optional field, in a type or in order is refused in both forms.
  Both messages state the same fact (the bodies differ and no always-written field decides) and
  name the forms that validate: one error with the optional field and `alike:`, an always-written
  field, distinct wire names, or `envelope`.
- OpenAPI describes both bodies on the `500` response. The design's text on mapping an answer to
  its outcome states the field rule; no HTTP conformance target exists yet.
- The generated Rust and Go servers answer each outcome with its own fields; one test holds both.
- A pair with one wire name and identical fields keeps today's answers (refused without `alike:`,
  admitted with it); a binding without such a pair keeps its bytes.
- `docs/design/http-presentation-binding.md` states the rule under "Status per outcome" and in the
  `ESS-HTTP-025` row.

## Evidence

Both refusals were rerun for this story on `ess 0.93.0` and are quoted verbatim. The pair came
from a downstream adopter's retrofit, which counts 11 open notes on it. The rules read are the
acceptance of `story:two-outcomes-answered-alike-may-share-a-status-and-body`.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

**Fit review (2026-11-12): accept as proposed, the form without `alike:`.** The draft left open
which of the two bindings validates. The review chose the one without `alike:`, by the rule the
design already uses for two redirects: one answer always writes something the other never
writes. A pair that differs only in an optional field stays refused in both forms, with one
message that names the idiom.

Notice to the requester: no session of the requester is reachable; this section is the notice.

## Fit review

Source read at main `582524ae95`, 2026-11-12. The two refusals are those of the Defect section,
rerun on 0.93.0.

1. **Need.** A service answers two refusals of one command on one status with one error wire
   name; one of the two bodies carries a field the other never has. A specification of that
   service cannot bind the pair, and the two refusals give opposite reasons. The requester
   proposed no syntax. Minimal reproduction: the model and binding in the Defect section.
2. **Class.** Defect. One diagnostic, `ESS-HTTP-025`, answers one question (does the wire tell the
   two answers apart?) with two rules. The status check counts error wire names only (`docs/design/http-presentation-binding.md:409-411`;
   the refusal text at `crates/specify/ess-http/src/lib.rs:6397`).
   The `alike` check also counts the errors' fields (`errors_apart` in the same file: "writes each
   error's own fields beside the member, and their errors' fields differ"). The bodies of the pair
   do differ, so the first message, "a body a reader cannot tell apart", is false for it.
3. **Already expressible?** No. The forms that validate today all change the wire: two statuses,
   two wire names, or `envelope`. A specification written for a shipped service cannot change it.
4. **Fit.** The binding has no language grid; the triage is on the design's rule list ("Status
   per outcome", `docs/design/http-presentation-binding.md:404-441`).
   - Vocabulary: the design already tells two redirects apart when "one of them always writes a
     parameter the other never writes", a parameter being always written when its source is not
     `Optional` (`:411-413`; `Written::apart`, `crates/specify/ess-http/src/lib.rs:6249`). The fields an error object
     writes beside its member are the same kind of evidence. The rule for two `{<member>: error}`
     bodies on one member with one wire name: they are told apart when one error always writes a
     field, by wire name and with a type that is not `Optional`, that the other never writes.
   - One function decides it, and both checks call it. The status check admits such a pair. The
     `alike` check keeps refusing it and names the deciding field.
   - A pair whose fields differ only in an optional field, in a type or in order: an answer
     without the optional field fits both, so no reader decides, and the bodies are not one
     answer, so `alike` does not hold either (`:429-430`). It stays refused in both forms. The
     idiom for an optional field is one error with that field `Optional`, reported by both
     outcomes, with `alike:`; the message names it.
   - Targets: `ess specify http validate`; OpenAPI (both schemas on the status); the generated
     Rust and Go servers (each outcome's own fields; they write them today). No HTTP conformance
     target exists yet (`:778-780`); the design's text on mapping an answer to its outcome
     (`:776-780`) gains the field rule.
     `ess verify diff`: I don't know whether it classifies this change; the scoper checks.
5. **Second adopter.** A payment service that answers `422` with wire name `validation_failed`
   for two refusals: one body `{message}`, the other `{message, field_errors}`.
6. **Cost.** No new key. A binding the rule admits is refused by an older `ess`; the reference
   notes the release, as the design's keys table does (`:577`, column "since"). The text of
   `ESS-HTTP-025` changes for these pairs; tests that quote it change. Existing bindings keep
   their bytes.
7. **Alternatives.** (a) Change nothing: the pair cannot be bound and the messages contradict
   each other. (b) Admit the pair with `alike:`: one response schema cannot describe both bodies,
   and the caller is told it cannot tell apart what it can; refused. (c) Chosen: admit without
   `alike:` by the always-written field. (d) Count any difference of fields as telling apart: an
   answer without an optional field fits both outcomes; refused.
