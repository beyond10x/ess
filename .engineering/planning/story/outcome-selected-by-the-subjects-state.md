---
format: aep.planning-md/1
id: story:outcome-selected-by-the-subjects-state
kind: story
status: archived
title: An outcome may be selected by the state the subject is already in
summary: 'when: reads the input, wrong_state: needs a state the transitions exclude, external: says nothing outside the system decided it. A branch chosen by the subject''s own state has no spelling, and the measured case is a system''s most travelled path.'
relations:
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
revision: 3
---
**Folded into `task:ess-gaps-measured-in-a-consumer-specification` on 2026-09-11**, at the operator's
request that all eight measured gaps arrive as one task for the maintainer to decompose. The body below is
kept because it carries the full argument, the measurements and the candidate resolutions; the task
summarises it. Archived rather than deleted so the record of what was drafted survives.

---

## What is missing

A command outcome may be selected by `when:` over the **input**, by `wrong_state:` (the states its
transitions do not run from), or by `external:` (something outside the system decided). There is no way to
say *this branch is selected by the state the subject is already in*.

Measured in an internal specification repository, 2026-09-11. Two commands report a campaign
membership's status — one from a push, one from a poll every two seconds. Which outcome applies depends on
the state the membership already holds, not on the reported value:

- `when:` cannot say it: it reads the input, and the input is the same in both cases.
- `wrong_state:` cannot say it: the command's transitions start from every declared state, so the set of
  states that refuse it is empty and the branch is `UnreachableBranch`.
- `external:` says something false. It means "inject the fault; no input can produce it", and a conformance
  target is asked to `ConfigureExternalOutcome` for it. The branch is the system's most travelled path.

That specification now carries an unconditional branch with an `UNMAPPED:` marker naming this gap, and two
permanent `ESS-SYNTH-003` refusals as a consequence (see the sibling story on exhaustiveness).

## What would settle it

A guard over the subject, checkable the same way `when:` is:

```yaml
outcomes:
  - name: confirmed
    given: subject.state == Idle        # the spelling is the design's to choose
    updates: outbound.CampaignMembership
```

Properties worth keeping: the left side resolves against the subject entity's declared fields and lifecycle
states, so a typo is `UnobservableFact`; exhaustiveness is computed over input *and* subject guards together,
so a set of branches that covers every combination needs no unconditional fallback; and synthesis arranges
the subject into the named state rather than injecting a fault.

## Acceptance

- [ ] An outcome may be selected by the subject's state, refused when it names a state the entity does not declare.
- [ ] Exhaustiveness counts subject guards, so a fully guarded command needs no unconditional branch.
- [ ] Synthesis arranges the subject into the guarded state instead of reporting `ESS-SYNTH-003`.
- [ ] the internal specification repository' two campaign report commands lose their `UNMAPPED:` markers and their two refusals.

## Out of scope

A guard over another entity, a view, or any expression beyond one comparison against a declared state.

## Provenance

Filed 2026-09-11 from measurements in an internal application repository wave 3
(`review-result:adversary-conformance-suite-pass-1`, finding F2, and `-pass-2`, finding A).
