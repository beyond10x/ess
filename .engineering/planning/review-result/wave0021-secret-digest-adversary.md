---
format: aep.planning-md/2
id: review-result:wave0021-secret-digest-adversary
kind: review-result
status: active
title: 'Wave 0021: adversary review of the Secret-digest fix'
relations:
- reviews: story:full-scan-secret-digest-oracle
revision: 1
---
# Wave 0021: adversary review of the Secret-digest fix

Unit: `story:full-scan-secret-digest-oracle`, merged as ess#109 (`b3cb5959`) through the bot merge queue.

- Two adversary passes over the implementor's change. Every finding was fixed and each fix was mutation-checked: breaking the guarded condition made the named test fail.
- Result: a full scan writes `infra-observation/3` with `{"present": true}` per Secret key and no digest or length; `infra-ir/3` and `infra-drift/3` carry nothing derived from a value.
- Legacy `/1` inputs are read with their digests dropped, so no written or derived output is named by them; refusals no longer quote Secret content.
- Accepted consequence: drift no longer detects a rotated Secret value.
