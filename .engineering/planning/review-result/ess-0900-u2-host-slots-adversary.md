---
format: aep.planning-md/3
id: review-result:ess-0900-u2-host-slots-adversary
kind: review-result
status: active
title: 'ESS 0.90.0 U2 host slots adversary: five findings fixed'
relations:
- reviews: story:a-ui-document-declares-a-host-slot
- reviews: story:a-host-slot-names-the-generator-item-it-stands-in-for
revision: 1
---
unit: ESS 0.90.0 U2, host slots declared in ess-ui/1 and named for the generator item they stand in for (commits dac45294b19, 5e1d26606fd), adversary tests 550a42617ea
verdict: findings fixed in the wave
origin: introduced 5 / pre-existing 0
fixes: 6ae985a4d0e, merged 85ce1384ae8

Findings:

1. The host name was not held to the node-name pattern; it now is.
2. `reason` was not a prose key; it now is.
3. A page slot kept its kind's header refs; it now drops them.
4. The reason was not trimmed in the report; it now is.
5. A leading byte-order mark in the hosts file was not stripped; it now is.

A reused template holding a slot stays refused as declared twice in 0.90.0, the fail-closed reading of the ADR's section 1; story:a-host-slot-in-a-reused-template-is-one-registration takes it to 0.91.0.

Evidence: PR https://github.com/beyond10x/ess/pull/763 CI run 42328951670 green at 18e91e142b2; squash commit 9a2468ef48e on main; tag 0.90.0.
