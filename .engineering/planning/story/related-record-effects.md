---
format: aep.planning-md/3
id: story:related-record-effects
kind: story
status: active
title: A non-creating command may declare an effect on related records
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#229
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T19:20:22Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T19:20:22Z", actor: "human:timo", revision: 5}
---
## Outcome

A non-creating command may declare an effect on related records (the second half of beyond10x/ess#229).

## Acceptance

- Not yet specified: this story holds the half of #229 split out of `story:feature-request-229` so that story is checkable; it is drafted and not scheduled.

## Origin

beyond10x/ess#229.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# story:related-record-effects — fit review (beyond10x/ess#229, effect half)

Read tree: `gaps-270` (#229 guard half merged as `ess/20`, c1f4353df). CLI: `ess 0.44.0`; 0.45–0.48 and `ess/20` may differ.

1. **Need.** One outcome changes its subject and also moves rows of another entity that reference it.
   - Deactivating a user ends that user's live sessions.
   - Completing a sign-in moves the attempt and increments a counter on the bound identity (#229 comment).

   Repro `repro-related-record-effects/` (ess 0.44.0):
   - `v1-affects-sets.yaml` (`affects: [{entity: Session, where: user_id == subject.user_id, sets: {ended: true}}]`) is valid.
   - `v2-affects-moves.yaml` (`moves: demo.users.Session.end` inside `affects:`) is refused with `unsupported_construct` ESS-COMMAND-009, "a secondary effect sets fields in this cut and takes no transition". A spurious cascade comes with it: `empty_declaration` ESS-COMMAND-007, "declares no outcomes".
   - `v3-affects-increment.yaml` (`{increment: 1}` in `affects:`) is refused with ESS-COMMAND-009.
   - Requester's syntax: `ends: demo.signin.Session where user_id == input.user_id`, and in the comment "a secondary effect on a second record named by an input (or found by a lookup)".
2. **Class: gap**, for the transition only. The field-change half shipped in 0.38.0 as `affects:` (#175; CHANGELOG.md:1127; `docs/design/set-effects-over-filtered-instances.md:32-41`).
3. **Already expressible? Partly.**
   - Field changes on referencing rows: yes (`v1`). Synthesis writes `DeactivateUser/outcome/deactivated` when the filter reads `input.user_id` (`v1b-affects-sets-input.yaml`, `out-v1b/suite.json`).
   - Lifecycle move: no. A Boolean stand-in leaves `Session`'s state, and `EndSession`'s wrong-state handling, untouched.
   - A separate command with an `instances:` set move (`idiom/separate-command.yaml`, valid) states a different architecture (a second call or a binding). It is honest only where the service actually works that way.
   - **Defect found:** `where: user_id == subject.user_id`, where `subject.user_id` is the subject's identity, validates, but synthesis refuses with ESS-SYNTH-001: "no witness: `subject.user_id` … reads a subject field the arrangement does not determine" (`out-v1`, ess 0.44.0). The subject's identity is the command's `instance:` input. This is not in the open-issue list.
4. **Fit: the requester's `ends:` fails; `affects[].moves` passes.**
   - Requester's `ends:` is a new verb for `moves:` + filter (red flag 1).
   - Redesign: admit `moves: <Entity>.<transition>` inside an `affects:` entry.
     - The parser already reads the key and refuses it "in this cut" (`crates/specify/ess-domain/src/command/set_effects.rs:224-235`).
     - Semantics are those of the `instances:` set move: skip rows outside `from`, and count it as a lifecycle cause but not as an arrangement driver (design :22-24, :53-54).
   - Siblings: `instances:` already moves, and `affects:` is the only set construct that cannot.
   - Composes with:
     - `when_related` (#229 guard half, `ess/20`);
     - the read-back view rule (design :80-86);
     - Entity Runtime `SetEffectUnsupported` and Rust/Go/Web/Clap `MissingRepresentation`, both already by name (design :95-98);
     - `ess-diff/9` `outcome-set-effect-changed`, which gains a line.
   - Selection "by a reference to the subject" is already `where: <fk> == subject.<field>`. Selection by relation name stays out (design :103-104).
   - Against #197: both are "an outcome changes more than its subject". #197 changes the *subject* on an *error* branch and reverses `refusal_mutated_state`. This story changes *other* rows on an *accepting* branch and changes no rule. They share the secondary-move synthesis and read-back. This one goes first; #197, if ever accepted, reuses `affects[].moves`. Not duplicates.
   - `{increment}` over a set (the sign-in counter) is a separate design question about per-row semantics (`set_effects.rs:560`). Excluded.
5. **Second adopter: yes.**
   - Closing a project archives its open tasks (`Task.archive` where `project_id == subject.project_id`).
   - Cancelling an order cancels its pending shipments.
   - Also the original cases of #167 and #175.
6. **Cost.**
   - Format gate: `ess/20` if still unreleased, else the next.
   - No new key; one refusal is removed.
   - Synthesis adds an outside-`from` row to the `affects:` segment.
   - One diff line and docs.
   - Entity Runtime stays refused by name.
   - Breaking for nobody: the construct is refused today.
7. **Alternatives.**
   - (a) Change nothing: a `sets:` stand-in, or a separate set-move command.
   - (b) The requester's `ends: … where`: refused, because it is a duplicate spelling.
   - (c) `affects[].moves`: chosen.
   - (d) Relation-named selection or lookups: deferred, as in the ess/16 design.

## Decisions

- **accept, redesigned (proposed):**
  - **The construct:** an `affects:` entry may declare `moves: <Entity>.<transition>`, with the `instances:` set-move semantics.
  - **Changed from the request:**
    - The requester's `ends: … where` is dropped (a duplicate of `moves:` + `affects:`).
    - `{increment}` over a set and relation-named selection are excluded.
  - **File separately:** a defect. `where: <fk> == subject.<identity>` validates, but synthesis refuses it with ESS-SYNTH-001 "no witness" (repro `out-v1`, ess 0.44.0).
  - **Also:** the spurious ESS-COMMAND-007 cascade beside the `affects[].moves` refusal.
  - **Overlaps, not duplicates:**
    - #229 parent: the guard half is `story:feature-request-229`, merged as `ess/20`.
    - #175 / #167, closed: `affects:` / `instances:` shipped in 0.38.0. The field half is already fixed.
    - #197: related ("changes more than its subject"). Sequence this first.
    - #237, #283, #285: the guard side, not effects.
  - **Already fixed?** The field half yes (0.38.0); the transition half no.

- Coordinator (2026-10-01): adopted as proposed above.
