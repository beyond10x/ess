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
| C | related-guard (#211), binding-context (#195), linkguard (#193 part 2 guard case) |

Integration branch: `integrate/ess-0.41`, cut from `main` after PR #208 merges (0.40.0).

## Revisions after implementation

- #210 (2026-09-28, mutate unit): excluded are only scenarios the baseline run reported `unsupported` or `skipped`; a scenario new to the mutant suite still counts. The literal rule ("absent from the baseline's executed set") turned three killed mutants in the repository's own audits (billing `from-drop/…Invoice.cancel/*`, one oracle `from-drop`) into survived.
- #203/#210: the manifest also moves to `ess-mutation-manifest/2` (its reader refuses unknown fields, so the refusal list needs a new version); `--collect` still reads `/1`, comparing refusal counts only. Report and manifest are registered together in `FORMAT_RELEASES` (`/2` unreleased).
- #198 (2026-09-28, arrangement adversary pass 1): creators are tried in command-name order, not declaration order — the IR keeps commands in a name-keyed map, so declaration order is not available there; name order is deterministic and keeps existing arrangements byte-identical.
- #210 F2 refined (2026-09-28, mutate adversary pass 2): only an excluded scenario whose mutant copy differs from the baseline copy makes an unkilled mutant `inconclusive`; an unchanged excluded scenario cannot kill it, so the mutant stays `survived`. Refusal keys for ESS-SYNTH-005 and ESS-SYNTH-014 use the view as subject.
- #193 (2026-09-28, viewfilter unit): the `when_subject` guard comparing a link field with an input (ESS-SYNTH-003, issue part 2) needs instance-valued inputs in the subject-fact input search and a second arranged owner for `!=`; it lands in `subject_fact.rs`, which state-scoped edits. Split into unit linkguard, wave C, based on the integration head after state-scoped and viewfilter merge. Tokens and `bound()` from viewfilter are reused; probe `~/.cache/ess-wave-n2/viewfilter/guard-case-probe.rs` is its red test.
- #202 (2026-09-28, distinguish adversary pass 1): option a (target-name priority) is replaced by the joined-source rule, which reads the mutated model: within one outcome, where an input feeds two or more `sets:` targets and an input of the same type feeds none, that pair is separated first and pinned; no other pins (unmutated cross-named entries, e.g. a rotation, pin nothing). The rule applies both in `distinguished` and on branches whose input the stored-row search chooses (`subject_fact` prepare). A pair that cannot be separated is recorded as a note on the scenario, never dropped silently. Issue #202 Expected names the adopter shapes with differently named fields, so they are in scope.
- #211 (2026-09-29, related-guard adversary pass 1): (1) `existing_instance` may sit beside `when_related` on one command; `existing_instance` answers first (the command's own identity is checked before a related row is read). `when_subject*`, `wrong_state`, `unknown_instance` and `input_absent` beside `when_related` stay refused in 0.41. (2) A missing related row is answered by the `exists: false` branch before any other branch; an `exists: false` branch may not carry `when:` (`conflicting_declaration`), so every missing row has exactly one answer. (3) Synthesis of a related row searches toward the predicate goal like the stored-row search does, and carries the arranging chain so a self-referencing entity stops at the cycle.
- #202 (2026-09-29, distinguish adversary pass 2): the joined-source rule is widened: within one outcome, every `sets:` source is paired with each input of its type that no `sets:` entry of that outcome reads (the mutated model leaves the original source unread even when the new source feeds one target only, because the mutation generator retargets to the first same-typed input, which may be read only by a payload). Every such pair is separated and pinned, not only the first; pairs that cannot all be separated get a note each. Notes are kept only for scenarios still in the finished suite.
- #195 (2026-09-29, binding-context adversary pass 1): a context change in `ess verify diff` needs a new report format; the binding-context unit owns `ess-diff/10` (unreleased, registered in `FORMAT_RELEASES` beside `ess/18`), and a cause change with an `external` cause is refused below `/10` with `unsupported_format_version`. Context witness values are made distinct from every payload field and from each other inside delivery-context synthesis, not by changing the shared field-name witness, so models without the key keep their suite bytes.

## Scope extension (2026-09-29)

Defects filed after the batch was scoped join it: #227 (filed 2026-09-28), #216 and #217 (filed 2026-09-29 from a consumer hardening run). #212, #228 and #229 ask for new capability (mutation classes, a scoped multi-field key, cross-entity guards on non-creating commands and effects on related records) and stay out, as #194, #197 and #200 do.

| issue | decision |
|---|---|
| #227 input-guarded refusal beside a subject-state branch | admit a `when:` + `error:` refusal naming no subject beside `when_subject_state:`/`when_subject` branches that act on an existing record; precedence as #209 documents: an input refusal is answered before existence and before the held state; synthesis witnesses it with a plain send and on an arranged row in each state a sibling runs from; no format bump if the key set is unchanged (validation relaxation), otherwise gate on `ess/18` |
| #216 caller sources change the interpreted spec_digest | find which of the two digest paths (suite synthesis, interpreted target) hashes the model differently when actor `attributes:` and `{caller: …}` sources are present; both must digest the same compiled model; a regression test over a model with caller sources |
| #217 overlapping accepting guards | declared precedence, not a refusal (a refusal would break specifications that validate today): among accepting guarded branches, the first declared whose guard holds answers; documented in `input-guard-overlap-precedence.md`; synthesis witnesses each decidable overlap with an input in it asserting the first-declared branch, so a target answering the later one fails; interpreter and Entity Runtime checked to agree, and made to agree where they do not |

| wave | units |
|---|---|
| D | refusal-beside-state (#227), caller-digest (#216), overlap-precedence (#217); dispatched after binding-context and linkguard merge |

- Scope extension, second batch (2026-09-29, two consumer hardening runs): #218 (mutate scores an unsatisfiable-guard mutant survived) and #219 (verify diff leaves a newtype prefix change unclassified) are defects and join wave D. #218: a mutant whose guard no input satisfies (decided over the finite witness domain, as `boundaries` decides) is scored `equivalent` with the unsatisfiable guard named, in `ess-mutation-report/2`, never `survived`. #219: `type/<T>/prefix-added` (narrowed), `prefix-removed` (widened), `prefix-changed` (both), in `ess-diff/10` beside the binding-context cause. #220–#225 are feature requests and stay out; #229 received a comment with the second consumer's rules.

| wave | units |
|---|---|
| D (revised) | refusal-beside-state (#227), caller-digest (#216), overlap-precedence (#217), mutate-dead-guard (#218), diff-prefix (#219) |

- Scope extension, third batch (2026-09-29): #202 is reopened in scope for list inputs: two same-typed `List<…>` inputs a `sets:` entry reads are witnessed `[]` both, so a retarget survives; unit distinguish-lists gives each such input a distinct non-empty witness (checked first on the merged tree; if the merged distinguish rule already separates them, no unit). #218 widens: a mutant on an outcome the baseline suite does not witness (refused at synthesis) is scored `unwitnessed` naming the baseline refusal, not `survived`. #226 (a branch selected by a stored counter at its limit, ESS-SYNTH-003) is a defect in the #198/#199 class and joins wave D as unit counter-limit: the stored-row search repeats the raising command up to the guard literal; beyond the search bound the refusal names the bound.

| wave | units |
|---|---|
| D (revised 2) | refusal-beside-state (#227), caller-digest (#216), overlap-precedence (#217), mutate-dead-guard (#218), diff-prefix (#219), distinguish-lists (#202), counter-limit (#226) |

- 2026-09-29: #213, #214 and #215 were filed under a personal account; recreated by the App as #227, #228 and #229 and the originals closed. References above use the new numbers.

- 2026-09-29 (operator): 0.41.0 is cut from the merged batch (#193 view-filter part, #195, #196, #198, #199, #201, #202, #203, #204, #205, #209, #210, #211). Units still open (#193 guard case, #216, #217, #218, #219, #226, #227, #202 lists) continue on a new integration branch cut from main after the 0.41.0 PR merges, for 0.42.0. #230 (a relation carried by the identity field refused as ESS-ENTITY-005) is a defect and joins that batch; comments added to #227, #228 and #229 from a second adopter.

- #219 (2026-09-29): `ess-diff/10` ships in 0.41.0, so the prefix kinds cannot join it; they need `ess-diff/11` (unreleased, 0.42.0). Relation for `prefix-changed` is decided by comparison as `alphabet-changed` is (extends → narrowed, shortened → expanded, otherwise changed). The `reading` residual fix is accepted.

- #218 (2026-09-29): `ess-mutation-report/2` and `ess-mutation-manifest/2` ship in 0.41.0, so the `equivalent` verdict (ESS-MUTATE-005), `baseline_refusals`, `unsatisfiable_guard` and `counts.equivalent` need `ess-mutation-report/3` and `ess-mutation-manifest/3` (unreleased, 0.42.0); `--collect` keeps reading `/1` and `/2`. Precedence accepted: a failing scenario kills first, a dead guard (`equivalent`) comes before the unwitnessed reasons, so the #203 fixture mutant is now `equivalent` with its added refusal still named.

- #218 (2026-09-29, adversary pass 1): a dead guard never overrides `inconclusive` (the fallback scenario may kill it); only an otherwise `survived` or `unwitnessed` mutant becomes `equivalent`. Transition-class mutants (`from-drop`, `transition-to`) on a transition that only baseline-unwitnessed or dead outcomes perform are scored `unwitnessed` naming the baseline refusal; in scope of #218.

- #217 (2026-09-29, correction 1): an external branch declared after accepting `when:` branches that cover its inputs is unreachable and refused at synthesis; the committed `guarded_external.rs` model declares `rejected` first (its intent kept). This matches Entity Runtime source order; CHANGELOG lists it under Changed.

- Scope, 0.42 batch (2026-09-29): issues #231–#237 were filed by other sessions through the App. Defects join 0.42: #232 (Entity Runtime reports a guard refusal as FulfillmentKeysMismatch), #234 (synthesis builds struct inputs that violate the entity or struct invariants over them), #235 (the generated explorer checks wrong_state before input guards, against Entity Runtime and the design doc). Feature requests stay out: #231 (Entity Runtime lowering of further constructs, each refused honestly today), #233 (value expressions), #236 (mutate --component), #237 (distinct list members and cross-record counts).

- #230 (2026-09-29, adversary pass 1): no format gate stands only if no model that validated before is refused; the identity counts as a relation carrier only where it carries a `references`, `cardinality: one` relation to another entity; a self-reference through the identity is refused (`conflicting_declaration`).

- #227 (2026-09-29, adversary pass 1): overlapping input refusals — the first declared answers (Entity Runtime order, consistent with #217). The documented precedences (#209, #211, #227) are replaced by one total order, taken from what Entity Runtime does, written once in `cross-record-and-stored-field-guards.md` and linked from the other design docs; the interpreter follows it, including a `when_related` command (where `exists: false` answers a missing row before input refusals, per the #211 revision).

- 2026-09-29 (operator goal: every externally submitted finding and issue addressed, a new ess version cut, issues closed, store in sync, worktrees cleaned): defects are fixed and released in 0.42.0; feature requests are addressed by a recorded decision (a GitHub comment naming it a feature request outside the defect batch, and a draft story in the store) and stay open. #239 (when_related over a field the creator leaves absent) and #240 (stored Map fields arranged only as {}) are defects and join 0.42. #232 is in Entity Runtime (`entity-core/src/runtime.rs`, `FulfillmentKeysMismatch`), not in ess: it moves to beyond10x/entity-runtime and is closed on ess with a link.

- #227 (2026-09-29, correction 1 stop): total precedence order, keeping every rule released in 0.41.0: (1) on a command with a `when_related:` branch, `existing_instance` then `exists: false` (#211 revision); (2) input-guarded refusals, the first declared whose guard holds (#209, #227); (3) existence of the addressed row (`unknown_instance`, and `existing_instance` on commands without `when_related`); (4) the held state (`when_subject_state`, `when_subject`, `wrong_state`); (5) accepting and external branches in declaration order (#217). No cycle: (1) applies only to `when_related` commands, which Entity Runtime does not lower. The Entity Runtime executor answering RevisionConflict before input refusals contradicts #209 and is filed on beyond10x/entity-runtime. Assertions encoding "two overlapping refusals are both the answer / a conflict" (the pass-1 domain adversary cases, committed `adversary_arrangement_pass2.rs:314`) are superseded by first-declared and are updated to it.

- #235 (2026-09-29): step 4 of the total order is refined to what Entity Runtime does (`admit_state`, and `cross-record-and-stored-field-guards.md:276`): the held state selects through `when_subject_state`/`when_subject` at step 4, and `wrong_state` answers only when the branch selected by step 5 moves from a state its move does not start from; an accepting branch that moves nothing answers in every state. The #227 unit's order section is edited to say so at its merge.

- #240 (2026-09-29): `exists`/`forall` over a map binds `r` to its values (compiler and Entity Runtime agree), documented in predicates.md. The synthesis test that used a map quantifier as its undecidable-guard example now expects both branches synthesized; ESS-SYNTH-002 keeps no synthesis-level example (no other legal undecidable input guard was found).

- 2026-09-29 (third consumer batch): #242 ({$instance} inside list inputs refused by author) and #243 (authored step expecting an external branch compiles without configure_external_outcome) are defects and join 0.43; #244 (elapsed-time and calendar-window guards) is a feature request; the stored-field comparison ask went to #233 as a comment. Stories are created on integrate/ess-0.43 (the 0.42 branch is frozen as the release PR head).

- #234 (2026-09-29, adversary pass 1): every blocker is fixed in the unit except the `touched` default refused beside a `when_subject` over an unset optional struct member (acceptance line 3): that follows #239's absent-field rule and is re-checked once #239 merges into integrate/ess-0.43. An invariant repair cannot satisfy is refused per outcome naming the invariant (never sent).

- 2026-09-29 #239: observation before the command does not require an absent Optional field (runner compares top-level fields exactly; a view may omit the key). An 'absent or null' runner test needs a suite-format change; not in 0.43.
- 2026-09-29 #243: new ESS-AUTHOR-037 enters the coverage inventory without a coverage-format bump, as 034-036 did; released readers refuse an inventory carrying it rather than misjudge it.
- 2026-09-29 #243 corr1: an authored act whose claims (events, escalations, no_events) only an external branch satisfies — of its own command or of a command a binding reached from it invokes — is refused ESS-AUTHOR-037 naming those branches, whether or not outcome: is written, unless outcome: itself names an external branch of the act's own command (then it is armed). No authored key for a downstream external answer in 0.43; synthesis covers escalations. The 'decided elsewhere' exemption counts only commands the act reaches.
