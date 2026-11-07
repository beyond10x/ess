---
format: aep.planning-md/3
id: review-result:ess-0900-u3-null-tag-adversary
kind: review-result
status: active
title: 'ESS 0.90.0 U3 null tag adversary: four findings fixed'
relations:
- reviews: story:a-null-tag-draws-no-chip
revision: 1
---
unit: ESS 0.90.0 U3, a null tag draws no chip (commit 771ad5c0b6d), adversary tests c8484db3a1f
verdict: findings fixed in the wave
origin: introduced 4 / pre-existing 0
fixes: 86ea448b247, merged f4806cd7d44

Findings:

1. The unit re-pinned PLAIN_PROJECT; the re-pin is undone.
2. A picked suggestion saved the text "null"; it no longer does.
3. The input showed an empty item for a held null; it no longer does.
4. A held null hid a suggestion named "null"; it no longer does.

Evidence: PR https://github.com/beyond10x/ess/pull/763 CI run 42328951670 green at 18e91e142b2; squash commit 9a2468ef48e on main; tag 0.90.0.
