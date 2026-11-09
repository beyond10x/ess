# ESS gap report, 2026-11-09

Source: 43 items found while writing a specification and UI document for an engineering workbench (recorded on ess 0.90.0).

## Tested on

- ess 0.92.0, the newest published release (2026-11-09). Tag commit `ce8a1d4ab3ea906c7e19404006e2f020e9c09b2d`; linux x86_64 tarball `ess-0.92.0-x86_64-unknown-linux-gnu.tar.gz`, `sha256:6ed7a9cb37229d0e85fce43d3dcf0285485e59ae244a87bfd04f2069989fa73f`.
- Every minimal example sits in `examples/<n>/` (`spec/`, `ui/`, `out.txt` or `out-ui.txt` hold the input and the exact output). `<n>` is the item number; `ui<nn>` for ess-ui items. Generators: `ess-gaps/gen/*.py`.
- Whole-model run on 0.92.0: `conform synthesize` gives 879 scenarios and 118 refusals (ESS-SYNTH-001 x27, -003 x5, -004 x50, -005 x14, -008 x9, -017 x8, -023 x3, -024 x2); the interpreted run passes 861, fails 15, 3 unsupported. Log: `ess-gaps/model/synth.log`.
- Releases between 0.90.0 and 0.92.0 (0.91.0, 0.91.1, 0.91.2) were read; only item 7 was fixed by them.

## Excluded (already filed or fixed)

| item | reason |
|---|---|
| 2 badge tone map on a column | #723 (open), finding C18 "columns cannot take `tone_by`"; still refused on 0.92.0 |
| 7 client pager sizes | fixed in 0.91.0 (client-paged `pager: {sizes}`); `ui07` passes on 0.92.0; no issue number |
| 14 `when_subject` reading through a reference | #564 (open; children #557, #558 closed): following a reference in guards, values, effects and view fields; still refused on 0.92.0 |
| 16 ratio of aggregate measures | #561 (open) and #563 (closed): aggregates over expressions, arithmetic over measures; still refused on 0.92.0 |

Related issues that do not cover a kept item, listed so the owner can link: #723 (also names the `as: duration` unit and the reorder drag handle, items 41 and 4), #560 and #556 (items 17, 19, 12), #547 (item 34), #542 (item 43), #415 (item 32), #735 (item 35), #428/#360/#193 (item 23), #361/#272 (items 20, 22, 24), #324/#322 (item 10), #636/#638 (item 27 is a different surface), #565 (group by a List field; not item 15), #473 (item 40 is another case).

## Not reproduced or not re-tested

- 36 (suite never sends exit 0 to a command branching on exit status): the minimal example's suite sends both 0 and 1. Not reproduced; recorded on 0.90.0.
- 11 (comma in a flow-mapping help text): YAML syntax, not an ESS gap. Not re-tested.
- 9 (enum choice options need the model): not re-tested without the model; `ui09` with an inline `types` enum passes. Recorded on 0.90.0.
- 27 (caller attribute `name`, server test login): the minimal example generates with 0 obligations; the whole-model case is not minimised. Kept below as "model only".

## Kept items

Format: construct; exact text on 0.92.0; what the model needs; example. Fix ideas are ideas, not requests.

### ess-ui / React

1. Generated React has no base path. `routes.ts` and `router.tsx` read `window.location.pathname` from `/`, `www/index.html` loads `/assets/main.js` and `/assets/main.css` absolutely, `pushState` and `<a href={props.to}>` carry no prefix. No refusal. Needs: publish under a subpath (static hosting under a project path). Example `ui00/react-out`. Idea: a `base_path` option of `generate ui`.
3. Row filter membership against a page-state list. `row.id in state.picked`: `filter_expr ... a filter must parse as a comparison, membership, not, and or or expression`. `in` takes literal lists only; `state.picked contains row.id` passes the check but `contains` is a text operator (not tested at runtime). Needs: filter rows by a multi-select held in page state. Example `ui03`, `ui03b`.
4. Dragging a card between groups. Reorder sends `{id, delta}` and `dropDelta` returns undefined when source and target groups differ. No refusal; reference text calls `group_by` plus `reorder` a kanban. Needs: a move command carrying the target group. Example `ui04`.
5. No kanban composite: `board` is a widget dashboard. Same example as 4. Needs: board of columns over one view.
6. Page requiring an id listed in navigation. `ess ui check` and `--model`: 0 errors; the nav link renders without an id. Needs: a check error. Example `ui06`.
8. Record field `as: metric` renders as text: `fields.tsx` has no `metric` case; no check error. Example `ui08`.
10. Overlay param read only as command input. `overlay_params ... \`params.id\` is passed to this overlay and read nowhere inside it; it reads no parameter`. Needs: a param bound into `does` counts as read. Example `ui10` (see #322, #324).
41. `as: duration` `unit: m`: `\`m\` is no duration unit: \`unit\` is \`ms\` or \`s\``. `format: percent` goes to `Intl` style percent in `core.tsx` and expects a fraction (0.5 shows 50%). Needs: minutes and hours, and a documented percent scale. Example `ui41`.
42. `graph_editor` highlight from another read: `unknown field \`highlight\`, expected one of \`key\`, \`label\`, \`kind_by\`, \`opens\``. Needs: mark a node chosen by another read. Example `ui42`.

### ESS model

12. View field values. (a) `due - now`: `ESS-VIEW-009 \`due - now\` reads \`now\` as a value; a value read at the current time has no clock a suite can fix yet`. (b) `param.k` in a computed field: `ESS-VIEW-001 \`param.k\` is no field of \`gap.demo.Item\`` and `ESS-VIEW-003`. Needs: waiting time published by a view. Examples `12a`, `12b`.
13. Aggregate view reading through a reference: `view.gap.demo.ItemsByParent.fields: ... reads through a reference of \`gap.demo.Item\`, and an aggregate view is not read through one in this cut`. Adjacent to #564, whose text does not name aggregate views. Example `13`.
15. No hour or week bucket of a Timestamp as a group key. A guess `expr: week(due)` gives `ESS-VIEW-009 ... \`week\` computes from one row; arithmetic over measures is not admitted yet`. Needs: a group key from a time bucket. Example `15`.
17. Invariant over several fields: `ESS-SPEC-012 cannot parse predicate "passed + failed == total": left-hand side must be a fact path`. Needs: sum of fields equals a field. Example `17`.
18. No `or` in a string `when:` or filter: `ESS-COMMAND-002 \`n < 0 or n > 100\`: operator \`<\` does not admit \`Integer\` (Number) and \`Text literal \`0 or n > 100\`\`` and `ESS-SPEC-012 ... invalid fact path identifier "(n"`. `any:` works in `when:` and view filters; `when_subject` has no `any`/`all` form. Needs: disjunction in `when_subject` and in string predicates, or a clearer refusal. Examples `18a`, `18b`.
19. Absent-optional refusal does not narrow later branches: `ESS-COMMAND-002 the subject's \`note\` has type \`Optional<String>\`, and \`note\` requires \`String\``. Same shape as #556 (closed), still present for a payload after a `not defined` branch. Example `19`.
39. A transition only moved as a side effect: `ESS-ENTITY-005 \`close\` moves \`gap.demo.Parent\` to \`Closed\`, and no command outcome takes it` (`missing_causation`). Needs: `affects: moves:` of another command counts as causation. Example `39`.
40. A creating outcome with `affects:`: `ESS-COMMAND-009 outcome \`created\` creates its subject and declares \`affects:\``. Needs: create one row and update another in one outcome. Example `40`.

### Synthesis and conformance

20. Aggregate measure `where:` on an Optional field: `ESS-SYNTH-017 ... a measure's condition: no arrangement of one group's rows gives the condition of \`with_x\` rows it admits and rows it refuses`. Example `20`.
21. Guard over the states of related rows validates, synthesis finds none: `ESS-SYNTH-001 ... its own row set is Unknown` (whole model, a merge command). A minimal shape: `ESS-SYNTH-001 no witness: ... is \`row set\`, which has no arrangement of rows its selector selects: the command creating them refuses a missing related row through the input the selector compares`. The first form is model only; the second is `21b`.
22. Average over a field the creating command does not set: `ESS-SYNTH-017 view gap.demo.MeanScore has no scenario`. Example `22`.
23. View filtered on an optional field, read by a command's scenario: `ESS-SYNTH-005 outcome ... has a scenario ... that skips its read of \`gap.demo.HighScores\``. Example `23`.
24. (a) Guard counting related rows whose `where` names no input: `ESS-SYNTH-001 no witness ... a refused subject is observed through a view of the entity projecting its identity, and none shows this row`. (b) Four state-filtered counts in one group: `ESS-SYNTH-017 ... no arrangement of one group's rows gives the condition of \`idle\`, \`paused\`, \`running\`, \`waiting\` rows`. Examples `24a`, `24b`.
35. `scenarios:` of `ess-inputs/2`: one file with several documents: `unreadable: deserializing from YAML containing more than one document is not supported`; a directory: `scenarios entry "scenarios/dir" is not a regular file; list a file rather than a directory or special node`. Examples `35b`, `35c` (control `35a`). Not the same as #735.
37. Interpreted target: `MarkConforming/outcome/conforming` observes another model digest (`DigestMismatch`), view stays `Validated`; 15 scenarios fail in the whole model. Model only; log `ess-gaps/model/interp3.clean.json`, `ess-gaps/fails.py`. Needs: setup that records the related run and makes the digests equal.
38. Interpreted target: `TakeDefault/outcome/no-default` ends `nothing reached by the runner's deadline, after 50 await(s)`. Model only.
43. System-issued creating command: `ESS-SYNTH-023` on both outcomes of a command a `system: true` actor holds. Example `43`.

### Rust server generation

25. `a view parameter (\`params:\`), which a generated query does not apply` (obligation; 77 obligations in the whole model on 0.92.0, 90 on 0.90.0). Example `25`.
26. `an order over the optional field \`due\`...` and `an order over the \`Timestamp\` field \`created\`, which the suite ranks by its instant...`. Examples `26`, `26b`.
27. Caller attribute `name`: the server's test login names an actor but no name (4 commands). Model only; minimal example `27` generates without an obligation.
28. Panic in synthesis: `thread 'main' panicked at crates/generate/ess-synth/src/rust/behaviour.rs:2850:28` "the plan admits `{subject:}` only where a row is held". Reproduced in the minimal example `28` and the whole model.
29. `gap.demo.FinishItem.self: outcome variant allocates invalid Rust identifier \`Self\` in \`outcomes:gap.demo.FinishItem\``. Example `29`.
30. `distinct item in outlines by item.id as string`, distinct list members no generated behaviour compares. Example `30`.
31. `an expression value ({expr: "subject.n + input.delta"})` in `sets:`; the same construct for a computed value of a rank command. Example `31`.
32. `gap.one.Things, gap.two.Things: \`things\` is allocated 2 times in \`port methods:gap-service\` (view method, view method); give one of them \`naming: { code: ... }\``. Same shape as #415 (closed, commands) now for view methods. Example `32`.
33. `the guard path \`title.count\``. Example `33`.
34. `effects on filtered rows (\`affects:\`)`. Same as #547 (closed, claimed fixed in 0.63.0), still an obligation. Example `34`.

### New on 0.92.0 (not in the raw list)

- Reads through a reference (`field: {related: {via, field}}`) in generated queries are obligations in 5 views of the model. Next to #564.

## Ideas (not requests)

- Item 1: base path option. Item 6: check that a navigation entry needs no id. Item 29: escape or refuse at validate time. Item 28: refuse with a named error instead of panicking.
