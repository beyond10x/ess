---
format: aep.planning-md/3
id: dependency-blocker:release-0-52-await-other-batch
kind: dependency-blocker
status: cleared
title: Wait for the other ESS session batch before releasing 0.52.0
relations:
- blocks: task:release-0-52-0-20261003
revision: 2
transitions:
- {from: "open", to: "cleared", at: "2026-10-03T01:40:07Z", actor: "human:timo", revision: 2}
---
## Blocking condition

The operator explicitly selected “Wait for the other session’s batch” on 2026-10-03. #394 has merged as d0f22461dae326f59664bc9427efad3589641961 and #393 has PR397; other changes in that session may still be pending. A single PR completing is not evidence that its entire batch is done.

## Clears when

The other batch has landed, as established by its owner or an explicit batch completion record reconciled with GitHub. Then refresh the release candidate from main once, reconcile its changelog and generated release documentation, and execute the final local and remote release gates. #390–#395 remain outside this session’s implementation and regrouping scope.

## Retained work

PR398 contains the initial release preparation. The local managed release tree retains the main integration and corrected release documentation. No 0.52.0 tag has been pushed. Do not run new full gates or publish another candidate while this hold is open.
