---
format: aep.planning-md/3
id: story:synthesis-rule-determined-is-generated
kind: story
status: active
title: The synthesis guide states that what the specification determines is generated
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:20:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:20:03Z", actor: "human:timo", revision: 3}
---
## Outcome

The synthesis guide, the generated PLAN.md and the stated principle say what is generated now:
what the specification fully determines is generated, what it cannot is an obligation, and storage
is a port the implementor provides.

## Acceptance

- `website/docs/guides/synthesize.md` states the rule, the ports and the list of constructs that
  keep a command an obligation; the "never behavioural" sentence is replaced.
- The generated PLAN.md has a section naming the storage and context ports as the implementor's to
  provide.
- Every page stating the old rule is changed (`grep -rn "never behavioural\|Behaviour is never
  generated" website docs README.md` prints nothing outside the blog and design history).
