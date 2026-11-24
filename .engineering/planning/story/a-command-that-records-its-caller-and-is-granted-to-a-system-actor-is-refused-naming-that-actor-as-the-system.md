---
format: aep.planning-md/3
id: story:a-command-that-records-its-caller-and-is-granted-to-a-system-actor-is-refused-naming-that-actor-as-the-system
kind: story
status: draft
title: A command that records its caller and is granted to a system actor is refused naming that actor as the system and the idiom, and the reference states who is a system actor
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:a-system-issued-creating-command-is-witnessed
revision: 1
---
## Need

A command records who did something: `approved_by: {caller: account_id}` in its `sets:` or an
event's `payload:`. It is granted to people (each declaring `attributes: [{name: account_id}]`) and
to an automated policy that approves the routine cases. The author marks the policy `system:
true`, because no person stands behind it, and the document is refused twice:

- `system: true` beside `attributes:` is `conflicting_declaration`: "`<Policy>` is the system
  itself and carries no credential, so it declares no `attributes:`", hint "a `{caller: …}` read
  on a command the system issues has nothing to read; drop `attributes:` or `system: true`"
  (`crates/specify/ess-domain/src/actor.rs:82-118`; design
  `docs/design/system-issued-commands.md:85-105`).
- Without `attributes:`, the caller read is refused: "`<Approve>` reads the caller's `account_id`,
  which `<Approver>` declares, and `<Policy>` may invoke it without declaring it"
  (`crates/specify/ess-domain/src/command/caller_value.rs:214-262`, "an attribute is readable only
  where every actor that may invoke the command declares it", `:9-11`). The message does not say
  that `<Policy>` is a system actor and can never declare it, nor what to write instead.

Reported by a downstream application on 0.98.0 (sixth list, item G13, rank P2): the policy actor
is modelled as a non-system actor with a credential of its own. The same report's second sentence,
that the application's automated actors are non-system because synthesis refuses a system-issued
creating command (`ESS-SYNTH-023`), is the active story
`story:a-system-issued-creating-command-is-witnessed`, not this one.

What `system: true` means, as designed and built:

- A system actor is "the system itself" with no credential (`actor.rs:63-84`). A command is
  system-issued, and conformance awaits instead of driving it, only when **every** actor granted it
  is a system actor (`system-issued-commands.md:85-91`). A command granted to people and to the
  policy stays an ordinary, driven command, so `system: true` on the policy changes nothing about
  how that command is tested.
- Who is recorded as having done something is read from the credential (`{caller: …}`), "not the
  command's input: the caller is not the authority on them, the credential is" (`actor.rs:66-68`).
  An actor that holds a credential, a service account or a policy with its own identity, is a
  caller, whatever process runs it.

So the rule is deliberate, and the application's model is the idiom: an automated actor that is
recorded by identity carries a credential, declares its attributes, and is not `system: true`. What
is missing is that the refusal and the reference say so.

## Outcome

Default of SX-D7 (C): no construct is added.

- The caller-read refusal names a system actor as such: "`<Approve>` reads the caller's
  `account_id`, and `<Policy>` may invoke it; `<Policy>` is a system actor (`system: true`), which
  carries no credential and so no `account_id`", with the hint "an automated actor recorded by its
  identity holds a credential: declare `attributes:` on it and drop `system: true`; a command
  granted to it and to other actors is driven either way". Its code stays
  `conflicting_declaration` or `undeclared_reference` as today.
- The `system: true` beside `attributes:` hint names the same idiom.
- The reference states who is a system actor, in the actors section of the specification guide and
  in `website/docs/reference/glossary.md`: the system acting without a credential, never an
  automated caller with an identity; and that `system: true` changes testing only for a command
  every grant of which is a system actor.

## Acceptance

- First: `crates/specify/ess-domain/tests/caller_values.rs` (or the crate's existing caller-value
  test target) gains a case with a command reading `{caller: account_id}` granted to an attributed
  actor and to a `system: true` actor. Today the message names the system actor only as an actor
  "without declaring it". After: it names it a system actor and the hint gives the idiom; the code
  is unchanged.
- The same document with the policy declaring `attributes:` and no `system: true` validates (a pin
  that the idiom works, run in the test).
- The `system: true` beside `attributes:` refusal's hint names the idiom; its code and location are
  unchanged.
- `task test-xtask` passes (diagnostic text pins; `docs/design/review-typed-diagnostics.md` counts
  per file are unchanged because no refusal is added).
- The guide and the glossary carry the paragraph; `CHANGELOG.md` names the clearer refusal under
  Changed.
- No format, IR or suite bytes move: `an_old_model_keeps_its_bytes` over the repository's examples.
- Each new test is timed and expected under 5 s; none may run over 60 s, the crate's test step stays
  within 120 s, and each build-pool job within 10 minutes.

## Fit review

Verdict: **decline the construct, with the idiom** (SX-D7 default C), plus a message and
documentation change. Tree `bcfc51555`, read-only.

1. **Need.** A command records which identity acted, for people and for an automated approver.
   Requester's framing (theirs): "a system actor with attributes".
2. **Class.** Deliberate rule. A system actor has no credential to take attributes from
   (`system-issued-commands.md:105`); the recorded identity is the credential's, never an input.
   The documentation gap (what a system actor is) and the unclear message are defects of wording.
3. **Already expressible?** Yes: the automated approver as a non-system actor with `attributes:`.
   The idiom keeps the checked figure in the specification: the recorded `approved_by` is still
   read from the caller and checked by every scenario (policy 1 holds).
4. **Fit.** Grid R column "caller", row by row, is unchanged: a caller attribute is readable where
   every granted actor declares it. Opening it to system actors (options A, B) would add a second
   source of "who acted" beside the credential.
5. **Second adopter.** A scheduler that closes stale tickets and is recorded as `closed_by`: it
   runs with a service identity, so it is an attributed caller; a sweep with no identity records a
   literal (`closed_by: system`) in its own command, which needs no caller read.
6. **Cost.** None in formats. This touches who is recorded as having done something, an audit and
   authority boundary: options A and B would let the specification, not the credential, name the
   actor of a recorded act.
7. **Considered.** (A) Constant attributes on a system actor, `attributes: [{name: account_id,
   type: String, value: policy}]`: the specification asserts an identity no credential proves;
   needs `ess/39`, IR, synthesis expecting the literal, a diff class. (B) A fallback on the read,
   `{caller: account_id, else: "system"}` for actors that declare none: a command granted to people
   and to the system records a literal for the latter; needs `ess/39`, a new key on the caller
   source in `payload:` and `sets:`, synthesis of both sides. (C) Decline, with the idiom and a
   clearer refusal. Chosen: (C).

Covers sixth list, item G13 (its first sentence). Its second sentence maps to
`story:a-system-issued-creating-command-is-witnessed` (active).

## Decisions

- **SX-D7. Item G13: may a command record a caller attribute when a system actor is granted it?**
  (A) A system actor declares constant attributes. (B) The caller read names a fallback value for
  actors that declare the attribute nowhere. (C) Decline, with the documented idiom (an automated
  actor with an identity is a caller with `attributes:`, not `system: true`) and a refusal that
  says so. **Recommended: (C).** Cost of A and B: they move who is recorded as having done
  something from the credential to the specification, an audit and authority boundary.

## Scope

- cited: `crates/specify/ess-domain/src/command/caller_value.rs` (`unreadable` `:214-262`)
- cited: `crates/specify/ess-domain/src/actor.rs` (`validate_system` hint `:103-115`)
- inferred: the crate's caller-value test target under `crates/specify/ess-domain/tests/`
- inferred: `website/docs/guides/specify/` (the page that documents actors and `attributes:`),
  `website/docs/reference/glossary.md`, `CHANGELOG.md`
- Size: S, about 120 changed lines. Under option A or B: L, a design page first, then about 600
  lines over domain, synthesis and diff, written as new units.

## Depends on

- None.
