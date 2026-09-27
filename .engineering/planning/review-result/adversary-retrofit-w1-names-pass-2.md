---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w1-names-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: two readers outside the loader refuse newtype map keys'
relations:
- reviews: story:field-names-underscore-and-newtype-map-keys
revision: 1
---
Adversary pass 2 against story:field-names-underscore-and-newtype-map-keys (beyond10x/ess#141, #143), aep:adversary, 2026-09-27, after correction round 3.

verdict: CONFIRMED (1 warning, 1 note)
cases: executed 663→667, red 2
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_names_pass2.rs` and
`crates/specify/ess-cli-contract/tests/adversary_names_pass2.rs`. Red: an `ess-scenario/3` fixture
typed `Map<demo.orders.ItemId, Boolean>` is refused as unreadable; a CLI binding contract
`Map<demo.Label, String>` is refused. No remaining copy of the old field-name rule was found
anywhere; no identifier collision in any emitter; every specification loader is scoped.

Coordinator routing: both to the implementor in a final correction round, with the two files
assigned.

```findings
[{"file": "crates/verify/ess-conformance/src/authored.rs", "line": 1534, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "An ess-scenario/3 fixture typed with a newtype map key the model admits is refused as Unreadable, because authored scenarios are deserialized outside any MapKeyNewtypes scope."},
 {"file": "crates/specify/ess-cli-contract/src/resolve.rs", "line": 140, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "A CLI binding value contract with a newtype map key is refused because contract() parses the type with no key-newtype scope."}]
```
