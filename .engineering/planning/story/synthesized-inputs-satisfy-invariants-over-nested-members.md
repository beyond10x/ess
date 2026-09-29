---
format: aep.planning-md/3
id: story:synthesized-inputs-satisfy-invariants-over-nested-members
kind: story
status: active
title: Synthesis violates invariants over struct-input members and misses guards over nested input paths
refs:
- provider: github
  reference: beyond10x/ess#234
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T05:37:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T05:37:18Z", actor: "human:timo", revision: 3}
---
## Outcome

Synthesis builds command inputs that satisfy the invariants over them and witnesses guards over nested input paths (beyond10x/ess#234): entity invariants over struct-input members, struct-type invariants on an input, a `when_subject` reading a member of a stored optional struct, a wrong-state guard comparing a stored field with a nested input path, and a `when:` equality between an input and a member of another input.

## Acceptance

- a creating command whose entity invariant constrains a struct input's members is sent members that satisfy it (no placeholder that violates it), so its scenarios run;
- a struct type with invariants used as a command input is synthesizable;
- a `when_subject` over a member of a stored optional struct refuses at most its own branch, never the command's other outcomes;
- a wrong-state scenario whose guard reads `input.<struct>.<member>` is witnessed;
- a `when:` equality between an input and another input's member is satisfied by the witness;
- each item has a test that fails on 0.41.0; committed suites regenerate byte-identical or the change is listed in CHANGELOG.
