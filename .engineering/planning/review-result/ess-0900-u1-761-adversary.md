---
format: aep.planning-md/3
id: review-result:ess-0900-u1-761-adversary
kind: review-result
status: active
title: 'ESS 0.90.0 U1 list-of-identity guard adversary: blocker fixed, two pre-existing gaps absorbed or taken in'
relations:
- reviews: story:a-list-identity-guard-beside-an-updates-subject-is-witnessed
revision: 1
---
unit: ESS 0.90.0 U1, list-of-identity guard witnesses beside an updates or creates subject (issue 761; commits 46c22e67c00, 0d69cf8f32a), adversary tests c1360e09900
verdict: findings fixed in the wave
origin: introduced 1 / pre-existing 2
fixes: 0d69cf8f32a, merged d91b3a039c2

Findings:

1. Blocker: where the guarded branch was the only route, the creating step sent a list that did not satisfy its guard. The creating step now sends a satisfying list.
2. Pre-existing: a forall list guard was witnessed vacuously. Fixed by the same round and absorbed into this story; story:a-forall-list-guard-is-witnessed is archived.
3. Pre-existing: a guard needing several literals in one list was not witnessed. Taken into the wave as U4 (story:a-guard-over-several-list-literals-is-witnessed).

Evidence: PR https://github.com/beyond10x/ess/pull/763 CI run 42328951670 green at 18e91e142b2; squash commit 9a2468ef48e on main; tag 0.90.0.
