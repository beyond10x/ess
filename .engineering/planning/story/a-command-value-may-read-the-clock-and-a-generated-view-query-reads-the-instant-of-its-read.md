---
format: aep.planning-md/3
id: story:a-command-value-may-read-the-clock-and-a-generated-view-query-reads-the-instant-of-its-read
kind: story
status: draft
title: A command's value may read the clock, now with a Duration, and a generated Rust or Go view query reads the instant of its read
summary: now is refused as a value, and a generated query keeps a now-filtered view owed so the generated server does not start (downstream report 2026-11-26, gaps 6 and 7; TC-D1 to TC-D4)
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:synthesis-reads-a-view-whose-filter-orders-a-minted-timestamp-against-now-by-observing-the-minted-value-and
- informed_by: story:a-suite-fixes-the-targets-clock
- informed_by: story:served-store-and-entry
revision: 1
---
## Need

Reported 2026-11-26 by a downstream application on 0.99.2 (gaps 6 and 7 of a report of ten).

1. **A value from the clock.** A command stores a due instant counted from the moment it is
   handled: `due_at = now + 24h`, or `now` plus a duration the input carries. `now` is refused as a
   value (`website/docs/guides/specify/values-and-views.md:244`; `website/docs/reference/spec-versions.md`,
   row `ess/27`: "`now` refused as a value (`unsupported_construct`)"; `docs/design/language-grids.md`
   Grid R: "No position reads `now` as a value"). The author writes `{generated: true, present: true}`,
   and the rule "due one day after the request" is not in the model.
   `story:synthesis-reads-a-view-whose-filter-orders-a-minted-timestamp-against-now-by-observing-the-minted-value-and`
   reads such a minted instant by observing it and keeps "a source spelling tying a minted instant to
   `now` plus an input offset" out (its DF-D6 option B, not taken).
2. **A generated query that reads the clock.** From `ess/30` a view's `filter:` may order a
   `Timestamp` field against `now`, the instant the view is read (`website/docs/reference/predicates.md`
   § "A `Timestamp` against the current time"). Generated Rust and Go view queries read no clock and
   keep such a view owed (`crates/generate/ess-synth/src/view_query.rs:59-66`), and a generated
   network entry refuses to start while a view it reaches is owed
   (`website/docs/guides/synthesize.md:381-384`), although its memory context already reads the
   system clock for generated timestamps (same page, line 380). An overdue view, the common shape of
   such a filter, needs a hand-written query before the generated server starts.

## Fit review

1. **Need.** A stored instant derived from the moment of the decision; a generated view query
   decided at the moment of the read. The reporter proposed no syntax.
2. **Class.** Part 1 a gap: the domain fact cannot be stated. Part 2 a gap of the generated targets:
   the pages say the queries read no clock, so it is not a defect.
3. **Already expressible?** Part 1: `{generated: true, present: true}` moves the checked figure out
   of the specification, which `docs/design/generic-language-features.md` § Policy, policy 1, makes no
   answer. Part 2: a hand-written query linked through the ports, by design.
4. **Fit.** Cells: Grid R column `now`, rows `sets:`, event `payload:`, error `payload:` and the
   generated view query. The grid's target for the column is "every command position, every view
   position (a request clock), not invariants" (`language-grids.md`, Target), so this opens the
   column's remaining command cells, not one cell. Vocabulary: `now` with the existing offsets inside
   `{expr: …}`, where `Timestamp + Duration` already exists (`values-and-views.md`, the expression
   table). Composition: one command decision reads one instant (`predicates.md`, same section), so
   the stored value and every guard of the decision read the same instant. Siblings: a set effect's
   `sets:` keeps refusing an expression, and an invariant keeps refusing `now`. Targets: the
   interpreter reads the command clock it is handed; generated Rust and Go behaviours read their
   context's command clock once per decision already (`crates/generate/ess-synth/src/go/behaviour.rs:695-696`);
   Entity Runtime refuses (`CurrentTimeUnsupported`); `ess verify diff` and generated docs print the
   expression.
5. **Second adopter.** A library stores a loan's due instant 336 hours after lending; a session
   service stores `expires_at = now + 30m`; a desk lists the tickets whose response deadline passed.
6. **Cost.** Part 1: the next open source format; no new keyword; a suite expectation of a stored
   instant relative to the step's clock, for which I don't know whether a value form exists (the
   held read clock of
   `story:a-conformance-suite-s-view-read-may-hold-the-target-s-clock-at-an-instant-set-from-an-observed-value-and-every`
   is the nearest). Part 2: no source format; the generated Rust and Go `Context` gains a read clock
   (TC-D3 decides whether it is additive).
7. **Designs.** (a) Change nothing: the minted-value idiom with an observed held-clock read (the
   story above). (b) A source of its own, `{now: {offset: 24h}}`. (c) `now` as an operand of
   `{expr: …}`: chosen, no new source.

## Decisions

Each option list names its default, the recommended option A.

- **TC-D1, the value.** A: `now` as an operand of `{expr: …}` in a command's `sets:`, an event's
  and an error's `payload:`, the decision's one instant (recommended). B: a source of its own,
  `{now: {offset: <duration>}}`. C: change nothing.
- **TC-D2, the witness.** A: where the target admits a held clock the suite holds it for the step
  and requires the exact instant; otherwise it requires an instant between the send and the answer,
  plus the offset (recommended). B: the interval only.
- **TC-D3, the generated context.** A: an additive read-clock method whose default answers the
  command clock, so a hand-written `Context` still compiles (recommended). B: a required method.
- **TC-D4, units.** A: two units, the generated query first (no source format), then the value at
  the next open source format (recommended). B: one unit.

## Acceptance

1. At the next open source format `sets: {due_at: {expr: "now + 24h"}}` and
   `{expr: "now + input.term"}` validate; at the format below they are refused naming the format; into
   a field that is not a `Timestamp` they are `type_mismatch`; in a set effect's `sets:` and in an
   invariant they stay refused by name.
2. The interpreter stores the decision's instant plus the offset, the instant every guard of that
   decision reads.
3. Synthesis asserts the stored and the published instant; a target that adds the offset to another
   instant, or leaves it out, fails the suite.
4. Generated Rust and Go behaviours compute the value from their context's command clock; Entity
   Runtime refuses it by name.
5. A view with `filter: [state == Open, due_at < now]` gets a generated Rust and Go query that reads
   the instant of the read from its context; the generated network entry starts on its memory
   context; the suite's held-clock read of that view passes against both.
6. `docs/design/language-grids.md` (Grid R `now` column), `values-and-views.md`, `predicates.md`,
   `synthesize.md` and `spec-versions.md` say it.

Every new test runs under 5 s; none may pass the 60 s bound, and the test step stays within 120 s.
