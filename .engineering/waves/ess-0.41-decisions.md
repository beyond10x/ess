# Coordinator decisions for the 0.41.0 defect waves (2026-09-28)

Goal (operator): all defects from GitHub issues fixed and integrated via one integration branch, then
a new version cut. Defects in scope: #193, #195, #196, #198, #199, #201, #202, #203, #204, #205,
#209, #210, #211. Feature requests #194, #197, #200 are out of this goal. #206 is fixed by 0.40.0
(PR #208).

One new source format, `ess/18`, carries every new authored key in this batch (#201, #204, #195,
#211). The first unit that needs it (state-scoped, wave B) registers it; later units gate on
`FormatVersion::V18` and do not edit the registration files. One mutation report bump,
`ess-mutation-report/2`, carries #203 and #210 together.

| issue | decision |
|---|---|
| #196 Map witness `{}` | every Map input gets one entry: key = canonical spelling of the key primitive derived from the path (must pass `setup_map_key`), value = recursive witness recorded like a list element; project `<map>.count` as a fact so a count guard is decidable; `Distinction` carried into the key |
| #198 only first creator | seed every declared creator in declaration order (tie-break: declaration order, then score, then index); `MAX_NODES` applies per creator; a creator that cannot produce a row is skipped, refuse only if all fail (first cause kept) |
| #209 refusal needs a record | keep today's no-record send (documented precedence: input refusal before existence) AND add an arranged half, as `refusals_on_a_stored_row` does; creator chosen as #198; if no arrangement reaches the refusal, refuse at synthesis with a named cause, never skip at run time; `reads_identity` refusals stay plain sends |
| #199 state via when_subject branch | thread `Distinction` through `reach_state`/`search`; no fallback inside owner-arrangement recursion; `other_sources` unchanged |
| #202 sets retarget witness | option a: target-name priority, separate that pair first and pin it; pinned separations are never undone in the `'moved` loop |
| #193 identity/link-field view filter | opaque instance token facts (`instance:<name>`) bound for the identity field and every `Instance`-valued settled field; `bound()` supplies the instance for a param of the identity/link type; only `==`/`!=` decidable; aggregate `held()` reuses the tokens as group keys |
| #201 wrong_state per state | option B: `when_subject_state: [<states>]` on a refusal branch (a list), in `ess/18`; does not extend to `when_state_changes:`; the default branch must satisfy `validate_move` for every state the partition leaves to it |
| #204 state and stored field together | option B: `state` as a pseudo-field inside a `when_subject` predicate, in `ess/18`; precedence: guarded branches select before `wrong_state` applies (the #192 ruling) |
| #205 precondition list literal | option A: structured literals (lists, structs) admitted in precondition inputs and fixtures; `Json` and `Binary64` leaves stay refused |
| #203 mutate: gained refusals scored survived | new verdict `unwitnessed` with `added_refusals: [{code, scenario}]`, refusal identity (code, scenario id or subject); code `ESS-MUTATE-004`; in `ess-mutation-report/2` |
| #210 mutate --collect with skipped baseline | red = `Failed` or `Error`; skipped baseline scenarios are listed as not scored with reasons and a count; scenarios matched by id, a mutant scenario absent from the baseline's executed set is excluded and listed; `audit --target` gets the same relaxation; if nothing is scored the exit status is non-zero with "nothing scored"; in `ess-mutation-report/2` |
| #211 guard on another entity | new outcome key `when_related: {via: input.<field>, exists: false}` or `{via: input.<field>, predicate: <over related fields and input.>}`, usable on any branch including `creates:` and subjectless refusals, in `ess/18`; keyed by the other entity's identity, one hop, input-bound; a missing row makes a predicate Unknown, selecting only `exists: false`; non-identity lookup (rule 2) stays out of scope; combining with `when_subject*` on one branch is `conflicting_declaration` |
| #195 binding reads delivery context | spelling `context.<field>`; declared on the binding as `when.context_fields` with an authority naming the external channel; admitted only for events delivered by an external channel binding; the target protocol may deliver an event with context and may answer unsupported (scenario skipped with reason); redelivery carries the original context; in `ess/18` |

## Waves (collision rule: no two units in one wave edit the same function)

| wave | units |
|---|---|
| A | mutate (#203 + #210, `mutate.rs`), mapwitness (#196, `witness.rs`), arrangement (#198 then #209 then #199, `synthesize.rs` arrange/prepare + `subject_fact.rs` creator loop), preconditions (#205) |
| B | state-scoped (#201 + #204, registers `ess/18`), viewfilter (#193, `synthesize.rs` shows/bound + `aggregate.rs` held), distinguish (#202, `synthesize.rs` distinguished) |
| C | related-guard (#211), binding-context (#195) |

Integration branch: `integrate/ess-0.41`, cut from `main` after PR #208 merges (0.40.0).

## Revisions after implementation

- #210 (2026-09-28, mutate unit): excluded are only scenarios the baseline run reported `unsupported` or `skipped`; a scenario new to the mutant suite still counts. The literal rule ("absent from the baseline's executed set") turned three killed mutants in the repository's own audits (billing `from-drop/…Invoice.cancel/*`, one oracle `from-drop`) into survived.
- #203/#210: the manifest also moves to `ess-mutation-manifest/2` (its reader refuses unknown fields, so the refusal list needs a new version); `--collect` still reads `/1`, comparing refusal counts only. Report and manifest are registered together in `FORMAT_RELEASES` (`/2` unreleased).
- #198 (2026-09-28, arrangement adversary pass 1): creators are tried in command-name order, not declaration order — the IR keeps commands in a name-keyed map, so declaration order is not available there; name order is deterministic and keeps existing arrangements byte-identical.
- #210 F2 refined (2026-09-28, mutate adversary pass 2): only an excluded scenario whose mutant copy differs from the baseline copy makes an unkilled mutant `inconclusive`; an unchanged excluded scenario cannot kill it, so the mutant stays `survived`. Refusal keys for ESS-SYNTH-005 and ESS-SYNTH-014 use the view as subject.
