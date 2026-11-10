---
format: aep.planning-md/3
id: story:a-guard-comparing-a-stored-list-with-an-input-list-is-arranged
kind: story
status: active
title: A guard comparing a stored list with an input list is arranged
summary: A when_subject predicate relating a stored list to an input list is refused ESS-SYNTH-003 after one candidate (beyond10x/ess#734)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#734
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/subject_field_in_input_list.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:34:12Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:34:13Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

A refusal guarded by `when_subject` over a stored list and an input list gets no scenario when the
creating command ties that stored list to another of its own inputs. Synthesis tries one arranged
row, the plain one with an empty list, and refuses `ESS-SYNTH-003`, although one creation with a
non-empty list reaches the branch.

Minimal reproduction (ess 0.92.0, `ess/35`): `Profile` stores `parties: List<String>`.

```yaml
- name: example.profile.DeclareProfile        # input: profile_id, parties, semantics: List<Semantic{name, roles: List<String>}>
  outcomes:
    - name: party-not-a-role
      when: {not: {forall: {in: parties, as: p, that: {exists: {in: semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}}}}}
      error: example.profile.PartyNotARole
    - {name: declared, creates: example.profile.Profile, instance: profile_id, sets: {parties: input.parties}}
- name: example.profile.ReviseProfile         # input: profile_id, semantics
  outcomes:
    - {name: unknown, unknown_instance: true, error: example.profile.UnknownProfile}
    - name: party-not-a-role
      when_subject:
        predicate: {not: {forall: {in: parties, as: p, that: {exists: {in: input.semantics, as: s, that: {exists: {in: s.roles, as: r, that: r == p}}}}}}}
      error: example.profile.PartyNotARole
    - {name: revised, updates: example.profile.Profile, instance: profile_id}
```

```text
$ ess verify conform synthesize --path spec --out suite.json
refused: refusal[ESS-SYNTH-003]: outcome example.profile.ReviseProfile/party-not-a-role has no scenario `example.profile.ReviseProfile/outcome/party-not-a-role`
  no candidate of the 1 tried satisfies ``example.profile.Profile` stored parties selecting this branch, over the rows 1 bounded arrangements left`
  help: write the branch's condition over values a candidate can carry, or supply a fixture for it
4 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

Every arranged `Profile` is created with `parties: []`. The same refusal appears with a flat
`roles: List<String>` input, and disappears when `DeclareProfile` loses its guard: the cause is the
creation's own input guard, not nesting. A witness: declare `parties: [client]` with one semantic
holding roles `[client]`, then revise with semantics holding roles `[server]`.

Cause, read in the code (inferred, not traced in a debugger):
`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:4992-5080` (`make_creations`) offers,
besides the plain witness, inputs from `hinted` and `spread` that write a stored collection with
several entries (beyond10x/ess#240, line 5052). Each is kept only if
`input_selects(.., creator.outcome, &input)` (line 5070): a candidate with `parties` non-empty and
`semantics` left at its witness value is refused by `DeclareProfile/party-not-a-role` and dropped,
so only the plain row (`parties: []`) is left, over which the refusal's predicate is false.

## Acceptance

Tests in `crates/verify/ess-conformance/tests/subject_field_in_input_list.rs`, run with
`cargo test -p ess-conformance --locked --test subject_field_in_input_list`, on the brand-free
fixture above, each failing on 0.92.0:

- `a_stored_list_tied_to_a_creation_input_is_arranged_non_empty`: `synthesize` reports
  `0 refusal(s)`; `example.profile.ReviseProfile/outcome/party-not-a-role` creates a `Profile` with
  a non-empty `parties` whose every element is a role of the `DeclareProfile` input, and sends
  `ReviseProfile` semantics whose roles miss at least one stored party; the interpreted target
  passes every scenario.
- `a_stored_list_tied_to_a_flat_creation_input_is_arranged_non_empty`: the same three results for
  the flat form (`roles: List<String>`, `exists: {in: input.roles, ...}`).
- `a_creation_guard_no_list_can_pass_still_refuses`: where the creation guard refuses every
  non-empty `parties` (`when: parties.count >= 1` as a refusal), the branch is still refused
  `ESS-SYNTH-003`, with the same text as today.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes on the change,
  and every row of `tests/fixtures/synthesis-suite-bytes-table.tsv` that differs from the base
  belongs to a model holding a stored list that a creation's input guard ties to another input.

## Decisions

- The tests go in the existing `subject_field_in_input_list.rs`, which already holds the
  `when_subject` list-membership cases, rather than in a new target.

Accept, redesigned. The arrangement search keeps a creation candidate that writes a stored list a
later guard reads by grounding the creation's other inputs from its own input guards, so the row is
reached by commands. The requester's second proposal, declaring seeds in `ess-inputs.yaml`, is
declined with the idiom: `--synthesis-seed FILE INSTANCE` already combines with `--path`, and the
seed design rules out a manifest extension. No design page must be written first; the `make_creations`
doc comment and the CHANGELOG entry record the rule. No format bump.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- inferred: crates/verify/ess-conformance/tests/subject_field_in_input_list.rs

confidence: medium

Shared surfaces: re-pins the suite-bytes rows it claims and proposes one `CHANGELOG.md` sentence.

`story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed`
(beyond10x/ess#727) and `story:per-state-refusals-are-witnessed-when-the-moving-guard-reads-an-input-list`
(beyond10x/ess#726) change the same file on other paths. No dependency either way; not safe to
implement at the same time.

## Fit review

1. **Need.** A command that refuses when a stored list holds an element the input list does not
   cover gets no scenario if the creating command validates that list against another of its
   inputs. Reproduction above; triage `repro/run.txt`, cases A to C. The requester asked that
   synthesis try a row with a non-empty list, and, failing that, that `--synthesis-seed` be
   declarable in `ess-inputs.yaml` because "the documented `synthesize --path <spec>` cannot use it".
2. **Class: defect** for the search. `make_creations` exists to offer rows whose stored collections
   hold several entries where a quantifier compares them with the input
   (`subject_fact.rs:5049-5052`, beyond10x/ess#240); it drops them when the creating branch's own
   input guard refuses the unrelated input values it was left with. Seeds are documented for states
   "no bounded arrangement of commands reaches" (`website/docs/guides/verify/synthesize-a-suite.md:40-43`);
   one creation reaches this one. **Convenience** for the manifest key: see 3.
3. **Already expressible?** A seed works with `--path` today:
   `ess verify conform synthesize --path spec --synthesis-seed seeds/profile.yaml one-party` gives
   `synthesis seeds: 1 selected, 1 applied`, `0 refusal(s)`, and the interpreted run passes 5 of 5
   (run.txt case D). So the manifest request is answered by that idiom. It is no answer to the search
   defect: a seed row is established by setup, not by `DeclareProfile`, and needs a target with the
   entity setup capability (`synthesize-a-suite.md:94-95`).
4. **Fit.** No authored surface. Grid R row `when_subject:` already reads the input and stored
   fields (`docs/design/language-grids.md:24`), `forall`/`exists` over lists are admitted
   (`website/docs/reference/predicates.md`); the change is in synthesis only. General: the fix grounds
   a candidate's other inputs from the creating branch's own input guards, for every stored
   collection a later guard reads (`hinted`, `spread`, the move guards at `subject_fact.rs:5053-5067`),
   not for lists of roles only. Other targets are unaffected: the interpreter answers the branch
   correctly (run.txt case D). The manifest key is refused as a whole row: seeds are selected on the
   command line only, by decision (`docs/design/synthesis-seeds.md:30-32`, "no directory scan,
   manifest extension, fragment syntax or implicit instance selection").
5. **Second adopter.** An order whose stored `lines` were checked against the catalogue items of the
   creating input, and a cancel command refusing when a stored line is not in the input's cancellable
   items. Same arrangement.
6. **Cost.** No source format, no key, no diagnostic. Suites of specifications with this shape gain a
   scenario where a refusal stood, of existing steps; suite format unchanged. Synthesis byte pins
   may move. The manifest key would cost an `ess-inputs` format version and reverse a recorded
   decision; not taken.
7. **Alternatives.** (a) Change nothing: the adopter keeps a seed or an authored scenario, and the
   refusal stays (beyond10x/ess#735). (b) Raise the arrangement budget: the one row is not a budget
   cut, the candidates are dropped before the search. (c) A seeds key in `ess-inputs.yaml`: refused,
   see 4. (d) Chosen: in `make_creations`, a hinted or spread candidate the creating branch's input
   guards refuse is grounded against those guards (the grounding `candidates` already does for
   input guards) before it is dropped. It adds no surface.

## Source

github:beyond10x/ess#734
