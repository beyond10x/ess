---
format: aep.planning-md/3
id: decision-blocker:idle-output-record-without-binding
kind: decision-blocker
status: cleared
title: An idle output record without a binding cannot tell an edited owned file from a copied record
relations:
- blocks: story:an-idle-output-record-carries-no-machine-path
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-07T06:33:56Z", actor: "human:timo", revision: 3}
---
## Question

Once an Idle `.ess-output/state.json` carries no root binding (beyond10x/ess#484), the reader cannot
tell "same folder, an owned file was edited" from "the record was copied into another folder". Today
a matching binding lets `ess generate` repair an edited owned file
(`crates/edge/ess-cli/src/output_ownership/mod.rs:938-958`), and a mismatched one runs the #306 copy
check, which refuses when owned files hold other bytes (`mod.rs:502-505`, `:569-616`). Found by the
U3 implementor of wave 2026-10-07b in phase 1.

| option | what | cost |
|---|---|---|
| A | the ledger decides everywhere: edited owned files are overwritten in any folder | drops the #306 guarantee that copying `.ess-output` into a foreign root never replaces an authored file; 4 relocation tests flip |
| B | the copy check runs everywhere: an owned file with other bytes refuses, naming the files and the re-enroll route | a hand-edited generated file in the same folder is refused instead of silently repaired; the `--check` advice changes; 3 tests flip |
| C | a machine-local binding outside the committed record | a new local file and its lifecycle |

Recommended: B.

## Decision (2026-10-07)

B. The copy check runs in every folder: an owned file whose bytes differ from the ledger refuses,
naming the files and the re-enroll route. A is refused because it opens a path to overwrite an
authored file. C stays out of scope. The release notes list under "Changed" that regenerating no
longer repairs a hand-edited generated file in silence and name the re-enroll command; the
`--check` advice text says the same.
