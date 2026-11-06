---
format: aep.planning-md/3
id: approval-record:ess-0890-wave
kind: approval-record
status: approved
title: 'ESS 0.89.0 runs wave-2026-11-06-ess: eight ride-along pull requests, issue 748 in two units and a proposed host-slot ADR, one pull request and release 0.89.0'
summary: Coordinator approval of 2026-11-06 under the operator's grant, with the units and build terms
tags:
- non-interactive
relations:
- decides: story:a-command-route-takes-one-input-as-its-bare-request-body
- decides: story:a-literal-path-segment-takes-precedence-over-a-variable-one
- decides: story:an-each-write-may-take-repeated-identities-in-list-order
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-06T10:55:29Z", actor: "human:timo", revision: 2}
---
## Decision

2026-11-06, the coordinator, under the operator's grant (class C: a wave, bot commits, a bot pull
request and its merge on green, a release through ess's own process), answered the wave proposal:

> Go for wave-2026-11-06-ess (0.89.0) as proposed. Everything listed under "what the go authorises"
> is approved, including closing PRs 741-747 and 749 as superseded through the bot after the wave PR
> merges.
> Accept the three issue-748 drafts with your verdicts (bare request body widened to any
> non-Optional input; literal segment first; repeated: last_wins). Move them draft->proposed->active
> citing this message.
> 749's Breaking line is accepted: format 3 is still read, and consumers move with their ESS pin.
> U3 is first out if it reaches its stop limit; it then leads 0.90.0.
> Limits as you stated (U1 25/40, U2 and U3 30/60 cargo min, peaks 8-10G); gate 10 stages, discard
> between stages.
> Adversary per unit before merge; one gate, one bot PR (squash), merge on green, tag after main is
> green, release.

The wave lead moves the three stories draft -> proposed -> active on those words and accepts none on
its own. The ADR `a-ui-document-declares-the-host-components-it-does-not-generate` stays proposed for
the operator.

This run is non-interactive: the wave lead is a sub-agent with no operator turn. Skill: aep
`implementing` 0.26.0, wave mode. Agents: `aep:implementor`, `aep:adversary`, at most 3 at once.

## Units

| unit | content | surface |
|---|---|---|
| U1 ride-alongs | pull requests 745, 741, 742, 743, 744, 746, 747, 749 merged into the integration branch; the red check of 746 fixed; digests re-pinned; 749's generated output re-recorded | ess-ui crates; ess-gen stamps, generated trees, projection corpus |
| U2 HTTP | a-command-route-takes-one-input-as-its-bare-request-body, then a-literal-path-segment-takes-precedence-over-a-variable-one | ess-http, ess-gen, ess-synth servers, ess-ui clients |
| U3 repeats | an-each-write-may-take-repeated-identities-in-list-order (ess/35) | ess-domain, ess-compiler, ess-conformance, ess-synth plan, ess-diff |
| U4 ADR | host slots in ess-ui/1, proposed | store only |

## Terms

- Cargo limits: U1 tell 25, stop 40 cargo minutes, peak 8G; U2 and U3 tell 30, stop 60, peaks 9G
  and 10G. Slot waits do not count. Every cargo call through the shared build-slot script.
- Gate: ten stages on the integration head, cache discarded between stages; CI runs the full suite.
- Merge: squash through the bot; the tag goes on the squash commit after `main`'s push run passes.
