---
format: aep.planning-md/3
id: story:a-view-may-answer-each-row-as-one-scalar-value
kind: story
status: active
title: A view may answer each row as one scalar value, not an object
summary: A list whose items are bare strings has no spelling in a view or an ess-http/1 route on 0.93.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:28:01Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:28:02Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Need

A downstream adopter's service has list routes whose items are bare strings (`["vip", "sales"]`).
A view types each row as an object, so the model promises `[{name: "vip"}]` where the wire holds
`["vip"]`. 0.91.0 added `field: .` for this shape
(`story:a-column-may-show-a-plain-string-row`), but in `ess-ui/1` only; `ess ui check --model`
refuses it on a view of the model.

Refusals on 0.93.0 (`format: ess/37`, view `demo.tags.Tags` over entity `Tag` with identity
`name: TagName`):

```text
fields: [{name: ., type: demo.tags.TagName}]
  invalid field name identifier ".": must start with a letter, optionally after underscores
fields: [{name: name, type: demo.tags.TagName, field: .}]
  invalid type: string ".", expected struct ViewFieldSource
shape: demo.tags.TagName
  error[ESS-VIEW-002]: `demo.tags.TagName` is not a struct, so it cannot describe the fields of `demo.tags.Tags`
fields: demo.tags.TagName
  invalid type: string "demo.tags.TagName", expected a sequence
# ess-http/1 route of the same view:
body: rows.name | array.name | item.name | response.name
  error[ESS-HTTP-024]: view `demo.tags.Tags` answers `rows`, `array` or `single`, and `rows.name` is not a view's body
row: name
  unknown field `row`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`,
  `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals`
```

## Acceptance

One of the two forms; the scoper picks, and the story names the one it did not take.

- Model form: a view may declare `row: <field>`. Each row is that field's value. The IR, the JSON
  Schema and OpenAPI give the row schema as the field's type (`items: {type: string}`), the
  interpreted target and synthesis answer and compare bare values, and `ess-ui/1` `field: .` reads
  such a view's rows. `order_by`, `filter` and `paging` keep reading the source row.
- Binding form: an `ess-http/1` view route may declare `body: rows.<field>` or `array.<field>`,
  answering each row as the value of that field, with the OpenAPI items schema being the field's
  type. A field that is `Optional` or a struct is refused by name.
- A model or binding without the key keeps its bytes.

## Evidence

The refusals above, observed by the reporting session on the 0.93.0 release binary on 2026-11-11.
Not rerun in this repository; the scoper reruns them before the story is proposed.

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is written below. Verdict
as of 2026-11-25: accept, redesigned. The need is a gap, and the reporter's spelling is one option
of BA-D1 (a second report of the same need, 2026-11-25, on 0.99.2).

**BA-D1. How does a view answer each row as one bare value?** (security-neutral; it adds a key to
the released format `ess-http/1` in options A and B, to the model format in option C)

- A: the binding names the field in the body form: `body: array.<field>` and `body: single.<field>`
  on a view route, and `{<member>: array.<field>}` inside the wrapped forms. Admitted only when
  `<field>` is the view's one and only field, not `Optional`; any other view is refused
  `ESS-HTTP-024`, naming the fields left out. Cost: a new value of the existing key `body`, one
  refusal rule, targets read the form (OpenAPI items schema, Rust and Go servers, React and
  terminal apps, `verify diff`, docs); the model and synthesis are untouched. Spelling follows
  `response.<field>`. Fails closed: an older reader refuses the value by name, and a view with
  more fields never loses one silently.
- B: the reporter's spelling, `body: {array: <field>}`, same admission as A. Cost: the same as A
  plus a clash: `{<member>: array}` already is an object of one member holding a body word, and
  `{array: <field>}` differs from it only by the value, so a view field called `single`, `array`
  or `rows` reads two ways; the same one-member object means two things in one key. Does not
  fail closed: the ambiguity is resolved by a rule the reader must know.
- C: the model form of the story's acceptance: a view declares `row: <field>`, each row is that
  field's value in the IR, the schema, OpenAPI, the interpreter, synthesis and `ess-ui/1`, and
  every route of the view answers bare. Cost: a model format key, the widest change (every
  target of the grid, a diff class, no migration), and a bare answer on every route of
  the view, wanted or not. Fails closed: an older reader refuses the key by name.
- Recommended: **A**. It is the smallest surface that matches `response.<field>`, it fails closed,
  it leaves the model and synthesis as they are, and it can widen later to a view of several
  fields by lifting one refusal. C is the choice if the model must say the row is a bare value
  for every reader (the UI, synthesis).
- Open with the owner. A release that carries this key names this pick in its pull-request line.

**BA-D1 = A (D135):** `body: array.<field>` and `single.<field>`, for a view of one field only,
anything else refused (fails closed); taken by the coordinator on 2026-11-25; a release that
carries this key names this pick in its pull-request line.

BA-D2 = yes (D156, the coordinator, 2026-11-26): beside the answered field only a field whose value the route's path carries may stand; every other second field stays refused.

## Fit review

1. **Need.** A list view whose row has one field must answer a bare JSON array of that field's
   values, `["vip","sales"]`; no form of the binding answers it. Reduction: entity `Tag` with
   identity `name: TagName`, view `demo.tags.Tags` over it with the one field `name`, route
   `GET /tags` with `body: array`. It answers `[{"name":"vip"},{"name":"sales"}]`
   (`docs/design/http-presentation-binding.md:496`); the wire wanted is `["vip","sales"]`. The
   reporter's proposal, labelled theirs: `body: {array: <field>}`.
2. **Class.** Gap. The binding names a view's answer by form: `rows`, `single`, `array`, and the
   wrapped `{<member>: …}` of each (`docs/design/http-presentation-binding.md:494-500`); none of
   them carries a row as a bare value, and a bare field value is spelled only for a command's
   outcome, `response.<field>` (:490, refused on a view by `ESS-HTTP-024`, :696).
3. **Already expressible.** No. `ess-ui/1` reads a plain string row with `field: .`
   (`website/docs/reference/ess-ui-test.md:67`), a UI reading form only; the model refuses a
   non-struct row (`ESS-VIEW-002`, the refusals in the evidence above). A route left unbound
   moves the answer's shape out of the specification, which
   `docs/design/generic-language-features.md:590` (Policy, 1) does not accept as an idiom. The
   other binding forms misstate the wire shape (`array`, `rows`) or answer a different object.
4. **Fit.** Grid R, the view-field row (`docs/design/language-grids.md:32`): the field is read
   from the source row as today, and nothing computed is added. Vocabulary: `response.<field>`
   (:490) is the bare value of one named field of a command's answer; the same dotted spelling
   on a view is its sibling, and the wrapped forms (:507-516) take it through their one rule,
   `{<member>: array.<field>}` holding what the bare form answers. Siblings: `single.<field>` is
   the same question for the one-row form and is admitted in the same change, else the refusal
   names why not. Targets: OpenAPI writes the field's schema as the items schema; the Rust and Go
   servers write each row's field bytes; the generated React and terminal apps read rows
   (`story:a-generated-ui-reads-a-view-s-rows-from-the-member-its-route-declares`) and must read
   a bare value or refuse the binding by name; `ess verify diff` and the generated docs name the
   body form; the interpreter and synthesis read the model, not the binding, in options A and B,
   and both change in option C. I do not know how conformance reads an HTTP list answer today;
   the scoper reads it before the story is proposed.
5. **Second adopter.** A catalogue service answers `GET /colours` as `["red","green"]`, and a
   permissions service answers `GET /roles/{id}/grants` as `["read","write"]`: each is a list
   view with one field and no other member on the wire.
6. **Cost.** Option A or B: a new value of an existing key in `ess-http/1`, so a format key
   (the format as it stands, 0.99.2, has none), one refusal rule of `ESS-HTTP-024`, no change to a
   binding without it. Option C: a key of the view in the model format, the IR, the schema,
   OpenAPI, the interpreter, synthesis, the UI and `ess verify diff`. A release that carries this
   key names this pick in its pull-request line.
7. **Considered.** Change nothing: the route stays unbound, or is bound with a form that misstates
   the wire (two views of the reporter's specification do that). The three options of BA-D1. The
   reporter's spelling is option B and is not the recommended one, see BA-D1.

## Second reported case

A second report of the same need (2026-11-25, a downstream application, `ess-http/1` at 0.99.2).
It does not change BA-D1.

- A view's row has two fields: one the route's path carries (`prefix`), one a list of strings
  (`keys`).
- The service answers `GET …/{prefix}` with only the bare array of the list field:
  `["k1","k2"]` (the one list field of a single row).
- `single.<field>` would fit if a field the path carries may stand beside the answered field. The
  choice is between "exactly one field" and "exactly one field the request does not already
  carry". A row field that is neither answered nor carried by the path stays refused (fails
  closed).

BA-D2 (open, the coordinator's): may a view field whose value the route's path already carries
stand beside the answered field? Recommended: yes, and only then; every other second field stays
refused.

## Delivery

Branch `issues/open-2026-11-26-d` at `fa096b6ab4`, merged into the open-issues wave; not released when written. BA-D1 = A is built: `body: array.<field>` and `single.<field>` on a view route, with the new `ViewAnswer` field; red test `a_view_answers_each_row_as_one_field_value`. The generated Rust and Go servers answer the bare values byte for byte (test `field_answers`). The gate ran every step with exit 0 and no committed pin moved.

Left: the interpreted target and synthesis do not compare bare values. Neither reads an `ess-http` binding (BA-D1 A: the interpreter and synthesis read the model, not the binding), and no HTTP conformance target exists (`docs/design/http-presentation-binding.md`, Stage 3), so no comparison exists to change; the generated servers' test `field_answers` holds the bare answers.
