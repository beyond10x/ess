---
format: aep.planning-md/3
id: story:a-row-set-guard-that-selects-its-entity-s-row-by-identity-from-the-input-is-witnessed-on-both-sides-the
kind: story
status: draft
title: A row-set guard that selects its entity's row by identity from the input is witnessed on both sides, the passing branch on a row created under that identity, and a side it cannot arrange is refused naming why
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Need

The seventh report, attempt 2. A downstream application states "refused when no `Card` has this
id" as a row-set guard over the identity. That is the one validating spelling that names the
entity when the identity type is shared:

```yaml
- name: unknown-card
  when_related:
    entity: demo.cards.Card
    where: card_id == input.card_id
    exists: false
  error: demo.cards.UnknownCard
- name: read
  returns: true
```

It validates. On 0.97.4 synthesis refuses the passing branch:

```text
refused: refusal[ESS-SYNTH-001]: outcome demo.cards.ReadCard/read has no scenario `demo.cards.ReadCard/outcome/read`
  no witness: `demo.cards.ReadCard/read: `unknown-card` is not decidedly passed over: its row set is True` is `row set`, which has no arrangement of rows the declared commands produce on which the rows its selector selects take this branch
```

The language promises the opposite. The row-set strategy states that "every branch such a
command decides is witnessed on rows its scenario arranges through the declared creating
commands" (`crates/verify/ess-conformance/src/synthesize/row_set.rs:5-6`, and the synthesis row
of `docs/design/filtered-related-reads.md:198`). From `ess/23` a selector over the identity
scopes the scenario, and pinning the identity is followed as "one record"
(`row_set.rs:443-511`, `:552-583`, `:1781-1786`, beyond10x/ess#463).

What the refusal shows, read at `bcfc51555`:

- Counts are tried in the order 1, 2, 0, 3 (`row_set.rs:77`). The reason printed is the first
  arrangement that *stood* and then failed the consistency check (`:3835-3845`). An arrangement
  that `plan.arrange` could not make records no reason at all (`:3826-3833`).
- "Its row set is True" therefore means that one row and two rows under the sent identity were
  never arranged, and that the zero-row arrangement stood and was correctly rejected. The refusal
  prints the zero-row reason and hides the real cause.
- Two rows under one identity cannot exist, so count 2 always fails here. Count 1 failing is the
  defect or gap this story settles.

Three causes can be read from the code. Which one held for the requester's model is not
established, because nothing was run:

1. The creator mints the identity (`generated:`). `input_for` picks a literal for
   `input.card_id`, and the closed filter (`:3712-3716`) asks for a row under that literal, which
   no creation can carry.
2. The creator sets the identity across a declared conversion, from an input of another type.
   The arrangement steers only a field written from an input or a literal without a conversion.
   The same report shows that refusal text on a neighbouring command. Overlap:
   `story:a-stored-field-set-across-a-representation-preserving-conversion-decides-the-guards-that-read-it-and-a-conversion-that-does-not-names-the-field`
   (draft).
3. Something else in the creator's own guards. 0.98.0 changed two arrangement paths of row sets
   whose creator refuses a missing related row (`CHANGELOG.md` 0.98.0 `### Fixed`, last two
   entries), so 0.98.0 may already answer some variants.

For a row-set `exists: true` guard today, synthesis arranges the selected rows through the
creating command, with the compared field steered from the input (`row_set.rs:1914-1923`). The
fixture `tests/fixtures/set-update-creator-without-related-refusal.yaml:53-62` shows this: an
accepting default beside `exists: false` over `shelf_id == input.shelf_id`, which is not the
identity. That path exists. The identity-pinned variant is the one in doubt.

## Outcome

A row-set guard whose selector pins the selected entity's identity to the input is witnessed on
both sides:

- the refusal is sent an identity no row carries, beside decoys;
- every branch answered after it is sent the identity of a row the scenario created first;
- an identity the target mints is sent as the captured instance (decision RD-D3).

Where count 1 still cannot be arranged, the refusal names that cause (minted identity,
conversion, the creator's own refusal) rather than "its row set is True".

## Acceptance

The fixtures are written by the test: `format: ess/37`, domain `demo.cards`, `Card` with identity
`card_id: demo.cards.CardId` (a newtype of `String`), `ReadCard` as in the Need, and a
`read_your_writes` view of `Card`. The view keeps this story apart from `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`.
There are three variants of the creator:

- (i) `CreateCard` takes `card_id: demo.cards.CardId` and the identity is that input;
- (ii) it takes `card_id: demo.cards.NewCardId` with a declared conversion to `CardId`;
- (iii) the identity is minted, `{generated: true}` in its event.

The acceptance checks:

- **Reproduced first.** This is the failing test. Synthesize each variant on the tree the story
  starts from, and record which variants refuse `ReadCard/outcome/read` with "is not decidedly
  passed over: its row set is True". At least one does, or the story closes as "fixed by 0.98.0"
  and records which change fixed it. Test:
  `crates/verify/ess-conformance/tests/row_set_identity_pinned.rs` (new), case
  `the_passing_branch_of_an_identity_pinned_absence_guard_is_reproduced`.
- **Variant (i) is synthesized.** After: `ReadCard/outcome/read` creates a `Card` under the sent
  literal before the send, and `ReadCard/outcome/unknown-card` is sent an identity no row carries,
  beside a decoy. Both pass on `--target interpreted`. Case
  `an_identity_from_the_input_is_created_before_the_read`.
- **Variant (iii) follows RD-D3.** Default A: the send names the captured instance, and the
  scenario passes on the interpreted target. Case `a_minted_identity_is_sent_as_the_captured_row`.
  Under B, the refusal names "the selector pins an identity the target mints".
- **Variant (ii) is answered or named.** It is synthesized where the conversion preserves the
  representation. Otherwise the refusal names the conversion and the field. Case
  `an_identity_across_a_conversion_is_answered_or_named`. Where the conversion story lands first,
  this case only checks that the two agree.
- **No zero-row reason hides a failed one-row arrangement.** After: where every count above zero
  failed in `plan.arrange`, the refusal states the reason of the one-row attempt. Case
  `a_failed_one_row_arrangement_is_the_reason_given`. Mutation check: printing the zero-row reason
  again fails it.
- **Other suites keep their bytes.** `row_set_struct_identity.rs`, `adversary_463_pass1.rs` and
  `a_row_set_branch_arranges_rows_past_the_creators_guard.rs` keep their verdicts. The byte pins
  of `AGENTS.md` ("A gate of named test targets includes the repository-wide byte pins") are run.

Each case synthesizes a model of a few commands and is expected to take under 5 s. The test step's
bounds apply: 60 s per test, 120 s for the step, and 10 minutes for any build-pool job.

## Fit review

1. **Need.** The passing branch of "refused when no row of E has this id", written as a row-set
   selector over E's identity, must be witnessed on a row the scenario created. The reproduction
   above is brand-free. The requester's option (b) is theirs: "a row-set `exists: false` guard
   whose passing branch synthesis arranges by creating the row".
2. **Class.** Defect where variant (i) fails, because the row-set module and #463 promise it.
   Gap for variant (iii). The diagnostic half is a defect either way: it reports a reason that is
   not the cause.
3. **Already expressible?** Yes, and it validates. Only synthesis falls short.
4. **Fit.** No new surface. The identity-pinned selector is the row-set spelling of an identity
   lookup. `docs/design/filtered-related-reads.md:39` makes the row-set mapping the alternative
   to the identity-addressed `via` form, which is why the requester's option (a), `entity:` beside
   `via:`, is not taken (RD-D2).
5. **Second adopter.** Any "exists by key" check whose key type is shared by an entity and the
   record that extends it one to one: an `Account` and its `AccountSettings` keyed by the same
   `AccountId`.
6. **Cost.** No format, and the suite major does not move. Suites move only where a model had
   this refusal. It adds no validation cost, so it does not add to the slow-validation defect of
   the sixth list's G14, which is a validation cost. It does add the row-set search (up to four
   counts and the decoys) to synthesis of such a command.
7. **Alternatives.** (a) Change nothing, so the requester uses `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`. (b) This
   story. (c) Rewrite an identity-pinned row set to the `via` lookup inside synthesis. That was
   refused, because two strategies would decide one branch.

Covers: the seventh report, attempt 2. Neighbours, not covered: the first report's items 10 to 13
(`gaps-uxui/p2a`: list guards and the addressed row inside related guards), and the second
report's G13 (`sl13-provided-synthesis`: rows with no creating command; where `Card` had no
creator, that feature applies instead).

## Decisions

- RD-D3 (minted identity): A, send the captured instance, as default; B, refuse naming it.
- Verdict: **accept, as a defect with a reproduction first.** Its priority is below
  `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`, which answers the requester without this story.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/row_set.rs`: `prepare_with` `:3622-3941`
  (the reason a failed `plan.arrange` leaves), `arrange_rows` `:1730-1934`, `input_for`
  `:626-669`, `pins_identity` `:555-583`
- inferred: `crates/verify/ess-conformance/src/synthesize/row_set/copied.rs` (variant iii, under A)
- inferred: `crates/verify/ess-conformance/tests/row_set_identity_pinned.rs` (new)
- `CHANGELOG.md` `## [Unreleased]` `### Fixed`

Size: M, about 450 changed lines. This is a guess until the first acceptance says which variants
fail.

## Depends on

None. It lands after `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view` by priority, not by code. If it runs beside the wave of
2026-11-24: s3 does not edit `row_set.rs`. s4 may touch it at one reader (its scope cites
`row_set.rs` near the old line 427), so land this story after s4 or rebase on it.
