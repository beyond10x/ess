---
format: aep.planning-md/3
id: decision-blocker:ess-058-opens-format-24
kind: decision-blocker
status: open
title: 'Which source format does 0.58.0 ship, now that #500 is first in it?'
relations:
- blocks: story:response-and-struct-admit-undeclared-fields-when-declared-ignored
- serves: vision:O2
revision: 1
---
## Question

https://github.com/beyond10x/ess/issues/500 is now ordered first for 0.58.0, and it adds an
authored key (`undeclared_fields: ignored`), so it needs source format `ess/24`.
`release-plan:ess-24-one-language` plans `ess/24` as one bump shared by the language-consistency
renames, the typed open questions and `ess specify upgrade`. A key added to `ess/24` after a release
ships it would need `ess/25`. Which source format does 0.58.0 ship?

| option | does | costs |
|---|---|---|
| A | 0.58.0 opens `ess/24` with #500 alone | the rest of the `ess/24` scope becomes `ess/25`; adopters see two bumps |
| B | 0.58.0 opens `ess/24` with #500 and https://github.com/beyond10x/ess/issues/498 (the two adopter keys) | as A, and #498's units (`command.rs`, `synthesize*`) join the wave; both issues close in 0.58.0 |
| C | #500 waits for the whole `ess/24` scope | #500 stays open until that unscheduled release; 0.58.0 ships no format bump |

Recommendation: B. Two adopter-requested keys share one bump, and the renames, which need
`ess specify upgrade`, become `ess/25` with that command.

## Decided
