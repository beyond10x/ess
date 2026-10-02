---
format: aep.planning-md/3
id: review-result:adversary-gaps-306-pass-2
kind: review-result
status: active
title: Adversary pass 2, downstream gaps unit feature-request-306
relations:
- reviews: story:feature-request-306
revision: 1
---
unit: story:feature-request-306, tree gaps-306
verdict: NEEDS-CHANGE, red 3 (introduced 3)

- blocker: an unchanged regenerate in a umask-002 clone rewrote state.json (mode taken from disk).
- warning: the printed adopt route refused on the differing files and left the owner as a placeholder.
- note: relocate read a file under a symlinked parent as absent.

Correction 2 (coordinator-verified): recorded mode kept when bytes match; refusals list differing files, owner keys and the full route; relocate uses publish's alias and ancestor checks. Coordinator decision: the two route cases follow the printed route literally, including its first step (move the differing files aside); adopt semantics unchanged.
Coordinator rerun in gaps-306: ownership_relocation_adversary_p2 10/10; implementor gate output_ownership 51/51, ess-cli 881 passed before the route-case amendment.
