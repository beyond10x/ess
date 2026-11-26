---
format: aep.planning-md/3
id: story:a-further-instance-of-an-input-that-declares-an-example-keeps-the-example-at-every-leaf-that-no-decision-and
kind: story
status: active
title: A further instance of an input that declares an `example:` keeps the example at every leaf that no decision and no checked write needs told apart, so a second row's creation, a second caller's refusal and a stale-revision refusal send values an implementation that validates the example's content accepts.
tags:
- external-request
- issues-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:a-suffixed-instance-is-created-with-the-inputs-the-specification-gives
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:30:49Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:30:51Z", actor: "human:timo", revision: 3}
---
Size: M (about 270 changed lines: 120 source, 150 test).
Fixed suite bytes of committed models: none expected. The 7 repository models that declare an input
`example:` were synthesized on 0.98.0. Their only sends carrying a type placeholder are an identity,
a scalar told apart on a further row (`adversary-u14/lockers.yaml`,
`related-refusal-shadowed.yaml`: `site-<n>`), and fresh-probe sends
(`explore-stored-rows.yaml`). None of these takes a path this story changes. The byte pins decide.
Patch-release candidate: no. Suites of models with the shape change their values.

## Need

A downstream application's implementation validates the content of what it is sent: an identifier
must be one it knows, a list member must name an existing row, a nested document must hold its
required parts. The specification gives each such input an `example:` that passes. Synthesis sends
that example on the first instance only. Every further instance of the input gets the type's
placeholder at every leaf: a second row created in setup, the step of a refusal for a second caller,
a stale-revision refusal. Placeholders here means the path text with a counter suffix, an empty list,
a structural default. The implementation refuses that step as invalid input before the branch under
test, so the scenario fails against an implementation that does what the specification says.

What makes a further instance of the leaf differ from the first is in most cases one leaf, or none.
Today the whole input is replaced.

## Reproduction

Reproduced on 0.98.0 (installed release). Model (run directory `run/v2-create-guard-reads-layout/`
of the assessment):

```yaml
format: ess/33
system: demo
version: v1
domain: demo.shelf
types:
  - {name: demo.shelf.ShelfId, kind: newtype, of: String}
  - {name: demo.shelf.AccountId, kind: newtype, of: String}
  - {name: demo.shelf.StyleRef, kind: newtype, of: String}
  - name: demo.shelf.Layout
    kind: struct
    fields:
      - {name: style, type: demo.shelf.StyleRef}
      - {name: labels, type: "List<String>"}
      - {name: note, type: "Optional<String>"}
entities:
  - name: demo.shelf.Shelf
    identity: {name: shelf_id, type: demo.shelf.ShelfId}
    fields:
      - {name: account_id, type: demo.shelf.AccountId}
      - {name: title, type: String}
      - {name: code, type: demo.shelf.StyleRef}
      - {name: layout, type: demo.shelf.Layout}
      - {name: rev, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.shelf.Clerk
    attributes:
      - {name: account_id, type: demo.shelf.AccountId}
    may: [demo.shelf.CreateShelf, demo.shelf.SaveShelf, demo.shelf.ShelfById]
errors:
  - {name: demo.shelf.ShelfExists, summary: A shelf of that id exists already.}
  - {name: demo.shelf.ShelfNotFound, summary: No such shelf in the account.}
  - {name: demo.shelf.StaleRev, summary: The shelf moved on.}
  - {name: demo.shelf.BadLayout, summary: The layout is refused.}
commands:
  - name: demo.shelf.CreateShelf
    input:
      - {name: shelf_id, type: demo.shelf.ShelfId}
      - {name: title, type: String, example: "Pantry"}
      - {name: code, type: demo.shelf.StyleRef, example: "walnut"}
      - name: layout
        type: demo.shelf.Layout
        example: {style: oak, labels: [jars], note: top}
    outcomes:
      - {name: bad-layout, when: "not defined(input.layout.note)", error: demo.shelf.BadLayout}
      - {name: exists, existing_instance: true, error: demo.shelf.ShelfExists}
      - name: created
        creates: demo.shelf.Shelf
        instance: shelf_id
        sets: {account_id: {caller: account_id}, title: input.title, code: input.code, layout: input.layout, rev: 0}
        emits: [demo.shelf.ShelfCreated]
        payload:
          demo.shelf.ShelfCreated: {shelf_id: input.shelf_id}
  - name: demo.shelf.SaveShelf
    input:
      - {name: shelf_id, type: demo.shelf.ShelfId}
      - {name: title, type: String, example: "Pantry"}
      - {name: code, type: demo.shelf.StyleRef, example: "walnut"}
      - name: layout
        type: demo.shelf.Layout
        example: {style: oak, labels: [jars], note: top}
      - {name: base_rev, type: Integer}
    outcomes:
      - {name: missing, unknown_instance: true, error: demo.shelf.ShelfNotFound}
      - name: other-account
        when_subject: {predicate: account_id != caller.account_id}
        error: demo.shelf.ShelfNotFound
      - name: stale
        when_subject: {predicate: rev != input.base_rev}
        error: demo.shelf.StaleRev
      - name: saved
        updates: demo.shelf.Shelf
        instance: shelf_id
        sets: {title: input.title, code: input.code, layout: input.layout}
        emits: [demo.shelf.ShelfSaved]
        payload:
          demo.shelf.ShelfSaved: {shelf_id: input.shelf_id}
events:
  - name: demo.shelf.ShelfCreated
    fields:
      - {name: shelf_id, type: demo.shelf.ShelfId}
  - name: demo.shelf.ShelfSaved
    fields:
      - {name: shelf_id, type: demo.shelf.ShelfId}
views:
  - name: demo.shelf.ShelfById
    source: demo.shelf.Shelf
    consistency: read_your_writes
    params:
      - {name: shelf_id, type: demo.shelf.ShelfId}
    filter: shelf_id == param.shelf_id
    fields:
      - {name: shelf_id, type: demo.shelf.ShelfId}
      - {name: title, type: String}
```

```console
$ ess specify validate --path system.yaml
demo v1 — 1 file(s), valid
$ ess verify conform synthesize --path system.yaml --out suite.json
5 scenario(s) (0 authored), 2 refusal(s), written to suite.json
$ ess verify conform run --path system.yaml --suite suite.json --target interpreted --report-format 2 --report-out report.json
  5 scenarios: 5 passed, 0 failed, 0 error, 0 unsupported
```

The interpreted target passes because it checks no content. Observed, and expected after the
change:

| step | input | observed | expected |
|---|---|---|---|
| a second `CreateShelf` (`shelf_id-17` in `SaveShelf/outcome/other-account`, `shelf_id-1` in `CreateShelf/outcome/created`) | `layout` | `{style: layout.style-17, labels: [layout.labels.0-17], note: layout.note-17}` | `{style: oak, labels: [jars], note: top}`: the guard reads only whether `note` is present, and the search left it present |
| the same | `title`, `code` | `Pantry`, `walnut` | unchanged |
| `SaveShelf` under test in `…/stale` and `…/other-account` (step 7) | `layout` | `{style: layout.style-1, labels: [layout.labels.0-1], note: layout.note-1}` | differs from the held `{style: oak, labels: [jars], note: top}` in one leaf (D1 default: `note: layout.note-1`), with `style: oak` and `labels: [jars]` |
| the same | `title`, `code` | `title-1`, `code-1` | unchanged: each is a whole scalar the accepting sibling writes and the row already holds, so it must differ (D2) |

Variants that decide the condition (all under `run/` of the assessment):

- No guard anywhere (`v1`): the second create sends every example. The refusal steps still send
  placeholders at all three inputs.
- The accepting sibling writes only `title` (`v6`): the refusal steps send `title-1`, and the example
  at `code` and `layout`.
- Entity fields under other names (`v3`), and no input comparison on `SaveShelf` (`v5`): no change.
  The guard on the refusing command plays no part.
- `CreateShelf`'s guard reads `title` and `code` by value (`v4`): the second create sends `title-1`
  and `code-1`, and the example at `layout`.
- An update's own write (`v7`, `saved` alone beside `missing`): `SaveShelf` sends `title-1`,
  `code-1` and the placeholder `layout`.
- `StyleRef` as an enum with `wire:` spellings (`v8`): a moved `code` or `layout.style` takes another
  declared variant, a value the type admits.

## Outcome

A further instance of an input that declares an `example:` is the example at every leaf. The
exception is a leaf that something needs told apart:

- the creation's identity, or a leaf a branch decision reads by value;
- a leaf copied into a stored field that a view filter, order, grouping or aggregate reads, or that
  another command's stored-row guard reads (`told_apart`, kept as today but per leaf);
- a leaf whose write the scenario must see: a complete refusal's input that an accepting sibling
  writes, and an update's own write. Here the fewest leaves move that make the value differ from the
  one the row holds.

Such a leaf moves inside its declared type as today: an enum to another variant, a text to its
placeholder (D2). A leaf with no example, such as a member the example leaves out or an input
without `example:`, keeps today's witness. The guard is decided again on the result, as today
(`keeps_branch`, `selects`). Where it is not kept, the input falls back leaf by leaf to today's
value.

## Cause (read in the source, not traced)

Paths under `crates/verify/ess-conformance/src/`. The same code is in 0.97.4, 0.98.0 and the newest
branch (`8487e9d59`).

- `witness.rs:7286-7299` (`Builder::input`): an example is the base at `Distinction::PLAIN` only.
  So every `candidates(…, Distinction::further(n))` value of an example-bearing input is the type's
  placeholder at every leaf (`docs/design/string-alphabet-and-length.md:340-344`).
- Further-instance creation. `synthesize.rs:9230` `exampled`, called from `created_by` (`:8940-8942`),
  puts the plain value back per root input. It skips the whole input at `:9269-9277` when any guard
  of the creating command reads any path under it. `guarded` comes from `FactPath::namespace` at
  `:9261-9265`, `decision_inputs` at `:9266`, and `told_apart` at `:9267`, whose `inputs_read`
  (`:9376-9381`) also cuts each path to its root. A guard that reads only `defined(layout.note)`
  costs every leaf of `layout`.
- Complete refusal. `synthesize/subject_fact.rs:7511` `refused_writes_apart` (ess/23,
  beyond10x/ess#461): an input an accepting sibling writes and the row holds takes the further
  witness's value of the whole written field (`:7545-7556`).
- The same whole-field move: `freshened` (`synthesize.rs:14669`, value at `:14696-14699`; an
  update's own write, ess#111, reproduced by `v7`), `unread_apart` (`:14807`, `:14857-14860`, ess#212)
  and `moved_apart` (`:15076`, `:15086-15089`, ess#161).

## Acceptance

Each item is red on 0.98.0 except where marked. Each test synthesizes one small model in process,
expected under 5 s (bound 60 s per test, 120 s per test step). Time each one before adding it, and
name any over 5 s in the report.

1. `suffixed_instance_inputs`: in the reproduction, every further `CreateShelf` sends
   `layout: {style: oak, labels: [jars], note: top}`, `title: Pantry` and `code: walnut`, and only
   `shelf_id` differs from the plain create.
2. `suffixed_instance_inputs`: with the guard reading a member by value
   (`when: "input.layout.style == \"x\""`, or the value guard the reproduction admits), the further
   create keeps its own witness at `layout.style` and sends the example at `labels` and `note`.
3. `suffixed_instance_inputs`: with a view filtering on a stored copy of `layout.style`, the further
   create keeps its own witness at `layout.style` and sends the example at every other leaf. The
   instances stay told apart where the view reads.
4. `subject_fact_complete_refusal`: in the reproduction, `SaveShelf` in `…/stale` and
   `…/other-account` sends a `layout` that differs from the held value in exactly one leaf, with the
   example at the others. It sends `title-1` and `code-1` as today, and `base_rev` as today.
5. `subject_fact_complete_refusal`: where the stored value holds members the example leaves out and
   the search filled, those members move and every example leaf is sent verbatim.
6. `declared_behaviour`: the reproduction's `saved` alone (`v7`) sends a `layout` that differs from
   the held value in one leaf, with the example at the others, and its scenario still checks the
   write.
7. Every suite of items 1-6 passes on the interpreted target (all passed).
8. Green today, kept: `a_second_instance_without_examples_is_unchanged` and the two uniqueness and
   related-row models of `suffixed_instance_inputs` keep their steps. A model with no `example:`
   synthesizes byte for byte as before.
9. Synthesis time of the reproduction and of `explore-stored-rows.yaml` is measured before and
   after. A jump is fixed in this unit.
10. The byte pins under Gate keep every row. A moved row names this story beside its re-pin.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`:
  - `exampled`, `told_apart`, `inputs_read`: decide per leaf path, not per root input. A leaf a
    guard reads only through `defined`/`exists` takes the example's value where the sent value has
    it present.
  - One helper used by `freshened`, `unread_apart` and `moved_apart` that moves the fewest leaves of
    an example-bearing field: first the leaves the example does not give, then one leaf of the
    example (D1), then today's whole further value.
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`: `refused_writes_apart` calls that
  helper.
- `docs/design/string-alphabet-and-length.md:340-344`: the paragraph says where a further instance
  keeps the example. #736 already sends it on unread leaves, so the paragraph is behind the code.

## Owned files

- `crates/verify/ess-conformance/src/synthesize.rs` (the functions above only)
- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`refused_writes_apart` only)
- `crates/verify/ess-conformance/tests/suffixed_instance_inputs.rs`,
  `tests/subject_fact_complete_refusal.rs`, `tests/declared_behaviour.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest`
- `docs/design/string-alphabet-and-length.md` (that paragraph)
- `CHANGELOG.md`

## Not yours

- `witness.rs`: rule 5 is unchanged. `invariant_ladders` belongs to #809 (active
  `story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths`),
  and `fields`/minting to the draft #816 item 4 story. A leaf a guard reads by value keeps the
  search's witness, so #809's candidates are what such a leaf gets. The two meet only through
  `keeps_branch`.
- `subject_fact.rs` `refusal_input_from` (#726) and the creation-candidate grounding (#734).

## Gate

One `--test` at a time. No build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test suffixed_instance_inputs`, then
  `--test subject_fact_complete_refusal`, `--test declared_behaviour`, `--test adversary_461_pass1`,
  `--test synthesis`
- `cargo test -p ess-cli --locked` (synthesis in `ess-conformance`)
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler`
  `--test string_alphabet_and_length`; `ess-cli` `--test tutorial_page`. The byte comparison of
  `synthesis_suite_bytes_table` runs in the `slow` lane (`task test-slow`); run it there too.
- the synthesis-sources stamp: `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
- `task test-xtask` (the `CHANGELOG.md` entry)

## Depends on

Nothing. Lands after step 9 of the open-issue order (#801, the last `subject_fact.rs` step) and before
step 10 (#795). The second row of #795 is created through `created_by` → `exampled`, so its test
should pin values this unit has already settled.

## Changelog

Under `Fixed` (it moves to `Changed`, naming the rows, if a committed pin row moves):

- `ess verify conform synthesize` sends an input's `example:` on a further instance at every leaf
  that nothing needs told apart: a second row's creation whose guard reads a member, a refusal's
  input that an accepting branch writes, and an update's own write. Before, the whole input fell
  back to type placeholders and an implementation that validates the example's content refused the
  step.

## Fit review

1. Need, apart from syntax. A further instance of an input keeps the shape and validity its declared
   example shows, wherever nothing needs it apart. Minimal reproduction above, with no downstream
   file. The requester's proposals, labelled as theirs: (a) "a suffixed instance derived from the
   `example:`, with its text leaves told apart"; (b) "an `examples:` list, one per instance".
2. Class: defect. #736's own acceptance says "Only the identity, and any field a filter or
   uniqueness check needs to tell the instances apart, differ between instances"
   (`story:a-suffixed-instance-is-created-with-the-inputs-the-specification-gives`, Acceptance).
   `v2` changes every leaf of a struct when one member's presence is read. `v1`/`v6` show a refusal
   step changing every leaf of each input the sibling writes.
3. Already expressible? Partly. A closed set of identifiers declared as an enum with `wire:`
   spellings moves to another valid variant (`v8`; `website/docs/reference/spec-versions.md:75`,
   `ess/32`). That idiom answers a leaf that must differ. It does not answer a leaf that need not
   differ: a list of references, a nested document. Steering with a guard on the command removes
   values the suite should send (the requester measured that it lost every accepting save of one
   kind), so that is no idiom.
4. Fit. No authored surface, so it opens no cell of grid R or O. It is a value choice inside
   synthesis. It reuses the existing gates (`keeps_branch`, `selects`, `admitted`) and the existing
   meaning of "told apart" (`told_apart`, `decided_by`), now applied per leaf. The four sibling moves
   (`refused_writes_apart`, `freshened`, `unread_apart`, `moved_apart`) all get the helper. The
   ~70 other further-distinction calls build whole rows or callers at a further distinction and are
   untouched; `exampled` already reaches the creates among them through `created_by`. Targets:
   synthesis only. The interpreter and generated code read the suite as before; `ess verify diff`
   is not concerned.
5. A second adopter: an order service whose `PlaceOrder` carries
   `shipping: {country: DE, carrier: parcel}`, with a refusal for an order from another account.
   The refusal step sends `shipping.country-1`, and the service answers 400 before the ownership
   check.
6. Cost: no key, no format, no diagnostic, no diff class, no generated-API change. Suite values of
   models with the shape change. Synthesis time grows by at most one branch decision per example
   leaf on the paths above (acceptance 9).
7. Considered:
   - Change nothing. The defect stays: every refusal and second row of a validating implementation
     fails in setup.
   - Rule 5 starts a further instance from the example and moves every leaf (`Pantry-1`, the
     requester's (a)), in `witness.rs`. It touches all further-distinction calls and moves told-apart
     rows of committed models. An identifier still moves to a value no implementation knows.
     Rejected.
   - An `examples:` list (the requester's (b)): new authored key, a source format step, and the
     author has to know how many instances synthesis makes. Deferred (D3).
   - The fewest leaves move (chosen). It covers every leaf that need not differ, which is most of
     them, with no syntax.

## Decisions

- Decision: accept, redesigned. The need is accepted. Proposal (a) is taken only as "keep the
  example", not "derive and suffix every text leaf". Proposal (b) is deferred.
- D1 (decided: default, 2026-11-24), which example leaf moves when every leaf of a field comes from the example and the value must
  differ. (A) default: a leaf of a scalar non-newtype type (`String`, a number, `Boolean`) first, then
  a newtype, never a list element or a map key, in declaration order within each rank. (B) the
  first leaf in declaration order.
- D2 (decided: default, 2026-11-24), the value a leaf that must differ takes. (A) default: inside its declared type, as today (an
  enum's next variant, a text's placeholder). The reply names the enum-with-`wire:` idiom for a
  closed set of identifiers. (B) a text leaf with an example takes `<example>-<n>`. This moves the
  told-apart rows of `lockers.yaml` and `related-refusal-shadowed.yaml`, and an identifier is no more
  valid than with (A).
- D3 (decided: default, 2026-11-24), an authored list of instance values (`examples:`) or an `example:` on a newtype declaration,
  for an open set of identifiers a leaf must differ in. (A) default, taken: not in this story; left out and deferred to the next source-format decision, as a
  `decision-blocker` for the next source-format decision, naming this story's evidence. (B) in this
  story, with its format step.

## Reported after 0.99.0

Reported 2026-11-25 on 0.99.0 and later: a further instance still sent a placeholder at a leaf that
only presence reads touch. The leaf is read through `defined(…)` in the creating command's own
guard and through `defined(…)` in another command's guard on the stored row.

Cause. `decided_by` in `crates/verify/ess-conformance/src/synthesize.rs` counted a stored-row
guard's `defined()`-only read as a read of the value, so the leaf was told apart and the further
instance kept the search's witness there instead of the example. 0.99.0 narrowed the own-guard
read only and left this one. Wrong since 0.99.0; a pinned suite of such a model moves when it is
synthesized again.

Red test `f103c32ea9`:
`a_further_create_keeps_the_example_at_a_leaf_a_stored_row_guard_reads_only_for_presence`
(`suffixed_instance_inputs.rs`).

In the reported model every placeholder send sat in a setup step, none in the refusing step (the
reporter, 2026-11-25).

## Delivery

Red test `f103c32ea9`, fix `0951bf9362864fce743a82adaf4db67cc5b44d36` on the branch
`fix/n90-a-presence-read-keeps-the-example-at-a-further-instance`, pushed. `decided_by` now splits
by-value reads from `defined()`-only reads; `told_apart` returns the presence-only copies apart;
`exampled` keeps such a leaf only where the example and the further witness disagree on its
presence. The red test is green (18 tests in the target).

Gate of the branch, third run: every step exit 0 (stamp, `task fmt-check`, clippy,
`suffixed_instance_inputs`, the bytes table and its slow models and components lanes, `cli_read`,
`adversary_related_values_all_bytes`, `external_beside_held_guard`, `adversary_461_pass1`,
`subject_fact_complete_refusal`, `declared_behaviour`, `synthesis`); no committed byte row moved.
No pull request yet. A pinned suite of a model with this shape moves when it is synthesized
again.

Tagged 0.99.4 on `daae8ebd1d` (branch `hotfix/0.99.4`, release commit `78c4ba842c`); the tag's
release run 45621137834 takes the full gate and was in progress when this was written.

0.99.4 is released (2026-11-25): the tag's release run 45621137834 took the full gate and ended in
success; the release is public at https://github.com/beyond10x/ess/releases/tag/0.99.4 with four
archives and `SHA256SUMS`, authored by the organization bot. Pull request 832 brings
`hotfix/0.99.4` to `main` with `main` merged in (head `c788b3bb79`); it was open when this was
written.

0.99.3 and 0.99.4 are on `main` since 2026-11-25 20:02 CET (pull request 832, merge commit `760b555161`).
