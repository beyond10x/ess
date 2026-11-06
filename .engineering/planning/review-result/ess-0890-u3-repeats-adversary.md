---
format: aep.planning-md/3
id: review-result:ess-0890-u3-repeats-adversary
kind: review-result
status: active
title: 'ESS 0.89.0 U3 repeated identities adversary: two introduced findings fixed in the wave'
relations:
- reviews: story:an-each-write-may-take-repeated-identities-in-list-order
prose_only: findings fixed in the wave are stated in prose with their fix commits
revision: 1
---
unit: ESS 0.89.0 U3, repeated identities in an each write (ess/35) (commits 287611c87f4, f6d5bf23793), adversary tests 9c166a199be
verdict: findings fixed in the wave
origin: introduced 2 / pre-existing 0
fixes: b03880c69e0, merged 029e6255378

Findings:

1. With several each entries over one entity, last-wins applied entry by entry rather than in list order; repeated: last_wins is now refused beside another each entry over the same entity.
2. The obligation for a repeated identity did not name its key; it now does, and the wording for moves and deletes is corrected.

Evidence: PR https://github.com/beyond10x/ess/pull/762 CI run 42274504324 green at 0307e8bc99a; squash commit ec2cb77cad2 on main.
