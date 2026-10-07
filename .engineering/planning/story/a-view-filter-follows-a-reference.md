---
format: aep.planning-md/3
id: story:a-view-filter-follows-a-reference
kind: story
status: archived
title: A view filter follows a declared references relation one hop
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#492
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "archived", at: "2026-10-07T23:36:56Z", actor: "human:timo", revision: 3}
---
## Outcome

Gap 4 of https://github.com/beyond10x/ess/issues/492: `exists: {in: param.queues, as: q, that:
queue.uuid == q}` over a `references` relation of cardinality one validates, and an `exists` over
the related rows for a many-relation. Today `ESS-VIEW-003` refuses it ("not a declared observable
root"). Gap 3 comes first: it states the same rows once the caller resolves the uuids.

Spec first; fit review owed.

## Fit review

Reproductions: `target/wave-scratch/fit/492/g4/` (ess 0.56.0, `format: ess/23`).

1. **Need.** A row holds a key of another entity (`desk_no`). The caller names those entities by a
   different field of theirs (a public uuid), and the read selects the rows whose referenced entity
   carries one of the named values. For a many-relation, the caller names a group or tag, and the
   read selects rows of its members. Minimal reproduction (`g4/requested.yaml`):

   ```yaml
   relations:   # on fit.desks.Segment
     - {name: desk, kind: references, target: fit.desks.Desk, cardinality: one, via: desk_no}
   views:
     - name: fit.desks.SessionsAtDesks
       source: fit.desks.Segment
       params: [{name: desks, type: Optional<List<Uuid>>}]
       filter:
         any:
           - not defined(param.desks)
           - exists: {in: param.desks, as: d, that: desk.public_id == d}
   ```

   `ess specify validate` exits 1: `error[ESS-VIEW-003]: `desk.public_id` reads `desk`, which is not a
   declared observable root; a view cannot select on something its source never observes`.
   Requester's syntax (theirs): `exists: {in: param.queues, as: q, that: queue.uuid == q}` over a
   `references` relation of cardinality one, and for a many-relation "an `exists` over the related
   rows".

2. **Class: convenience.** The rows are expressible today in two ways (question 3). The requester
   says so too: "Item 3 states the same rows once the caller resolves the uuids". A filter that reads
   another entity at query time is a join. The refusal is a recorded decision, not a defect:
   - "**Joins**: aggregates over more than one entity. `source:` stays one entity"
     (`docs/design/aggregate-views.md:1033`);
   - the row-set reads design: "It does not add a view join" (`docs/design/filtered-related-reads.md:5`);
   - #447 declined a view over a relation with idioms (`docs/design/read-api-view-idioms.md:174-196`).

3. **Already expressible? Yes, two idioms.**
   - **The caller resolves the names into keys** (`g4/i1-caller-resolves.yaml`). The view takes
     `desk_nos: Optional<List<DeskNo>>` and filters `desk_no` (item 3's shape). A second view,
     `Desks`, lists `desk_no` with `public_id`, so the caller can resolve, which is #447's two-view
     idiom (`read-api-view-idioms.md:186-187`). This gives the current mapping at read time, the
     same answer a live join gives. It validates (`fit v1 — 1 file(s), valid`). Synthesis refuses
     its list filter with `ESS-SYNTH-017`, which is exactly item 3 and is witnessed once item 3
     lands. The many-relation case is the same: resolve the group or tag into its current member
     keys through a membership view, then send the keys.
   - **Copy at write**, where the referenced value is fixed for the row's life or the write-time value
     is the fact wanted (`g4/i2-copy-at-write.yaml`):
     `desk_public_id: {related: {via: input.desk_no, field: public_id}}`, with the filter on the
     copy. It validates.
     - Witnessed by synthesis today with a scalar selector (`g4/i2c-copy-scalar.yaml`):
       `4 scenario(s) (0 authored), 0 refusal(s)`. The aggregate scenario opens desks, records
       segments through them and reads `{desk: <public id>}` → `sessions: 2` and a nonmatching id
       → `0`. `ess verify conform run --target interpreted` reports `4 scenarios: 4 passed`.
     - The list form runs as an authored scenario (`g4/scenarios/copy-list.yaml`, reads `{}` → 3,
       `[A]` → 2, `[]` → 0, `[B, A]` → 3): `4 scenarios: 4 passed`.

4. **Fit of the requested design.**
   - Vocabulary: `<relation>.<field>` would be a new filter root. Nowhere else in ESS does a view
     read another entity, and the value source that does read one is spelled `{related: {via, field}}`
     (`website/docs/guides/specify/values-and-views.md:39`). The same concept would get a second
     spelling.
   - Siblings: the request covers a filter only. A view field would still be unable to read the same
     value (#447), so it fails "works in one construct, not its sibling".
   - Targets: every view evaluator (interpreter, generated Rust, Go and TypeScript, web, Entity
     Runtime) would read a second entity at query time. `read_your_writes` would need a meaning
     across two entities' writes, and synthesis would arrange related rows per query. None of that
     exists.

5. **Second adopter.** An order report filtered by the customer's country, where the order row holds
   `customer_id`. A live join answers with the customer's current country; the order-time country
   is a copy at write. Either way it is one of the two idioms above, depending on which fact the
   report means.

6. **Cost.**
   - Declining: none. The idioms use released constructs.
   - Accepting as asked: a new filter root, a source format, an `ESS-VIEW-003` change, cross-entity
     consistency rules, every view evaluator, synthesis of related rows per query, and diff
     classifications for relation changes that now affect views.

7. **Considered.**
   - (a) The requester's one-hop filter root: refused, for the reasons in question 4. It reopens
     #447's decision for the filter side only.
   - (b) Copy at write as the whole answer: insufficient alone. It gives the write-time value, which
     differs for a mutable field or a group membership (`read-api-view-idioms.md:182-186`).
   - (c) Chosen: change nothing, and answer with the two idioms: caller resolution for the current
     mapping and for many-relations, copy at write for fixed values.

## Decisions

**Decline, with the idiom.**

- **Current values, and group or tag membership.** The caller resolves the names it receives into
  the row's keys by reading a view of the referenced entity, then sends those keys to item 3's
  list filter. Synthesis witnesses it once item 3's selector lands. Until then an authored scenario
  checks it.
- **Values fixed for the row's life, such as a public id.** Copy at write with
  `{related: {via: input.<key>, field: <field>}}` and filter on the copy. Synthesis witnesses it
  today with a scalar selector (`g4/i2c-copy-scalar.yaml`: 4 scenarios, 0 refusals, 4 passed on the
  interpreter).

If a live cross-entity filter is needed later, start from the shape #447 named for a view field:
`{related: {via: <reference field>, field: <field>}}` read at query time, one hop, absent where the
reference is, admitted as a filter operand and a view field together.

Still owed: a `specification` artifact stating these idioms with this output, per the assessment
procedure. It was not written here, because this review writes nothing to the planning store.
