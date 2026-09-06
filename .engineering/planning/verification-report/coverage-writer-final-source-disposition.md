---
format: aep.planning-md/1
id: verification-report:coverage-writer-final-source-disposition
kind: verification-report
status: draft
title: 'Coverage writer final source disposition: 648 package cases pass'
relations:
- verifies: story:review-conformance-coverage
revision: 1
---
# Final coverage source review disposition and root verification

Source attack 2 is recorded unchanged as review-result:coverage-writer-source-pass2 before
this disposition. Its full report SHA256 is 7d37308d86d53777c57e1051d3e8883b57765a7b84a421ed4d5fbf9fb8759482.
Root read the complete report prose and both complete new tests, then verified 19 seal entries,
17,996 native scratch entries, 1,653 external entries, 1,103 inherited source/snapshot pairs and
two additive tests. Every one of the 12 raw command logs appears verbatim in the report. File
hashes, native-byte link targets and all captured metadata matched. The 648-case red pass is not
reported as green: 647 passed and the new overbroad view-return assertion alone failed.

The actual findings comparison returns no carried or new findings and resolves both source-pass1
signatures: complete typed D1 rendering and the known-coverage Go diagnostic. The complete JSON
is retained in preparation/source-review-pass2-record/01.log. There was no third source attack.

Root applied only verification-report:coverage-writer-final-view-test-scope's explicit test
correction. Exact payload equality remains for all nine normalized owners; owners 6, 7 and 9
now pin the exact observed numeric tokens within the otherwise identical seven-key payload.
A twelve-owner length assertion was added; all twelve changed-survivor refusals and own-property
assertions remain. The message names story:review-browser-replay-fidelity and requires a deliberate
update when supported view execution consumes these fields. Root read the exact diff and verified
all 1,103 inherited files plus the other new test unchanged. No production source was changed.
The original red bytes remain in both review evidence and root's pass2-original-red-cli-test.rs.

All four final root verification lanes passed against 1,105 frozen source files. The complete
package lane executed 648 cases, all passing, none ignored, in 64 summaries. Both exact new test
binaries, the producer-test binary and CLI binary were retained separately after execution.
Full command receipts, raw output, snapshots and exports remain beneath the unit's
 target/review-boundaries-11/final-test-disposition-02.

| Lane | Exit | Seconds | Raw log SHA256 |
| --- | --- | --- | --- |
| fmt | 0 | 0.431503252 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| focused | 0 | 46.270740774 | 47337e39de2bb95120fc73779c985b0d5229c112a9098ea11a88301a861e15fd |
| package | 0 | 67.822408678 | cd77405342d285f35a090e2ff43cec1106f80603ed4386907fd7abaa88322af2 |
| clippy | 0 | 28.079273968 | 68a8ed44ef3f904694e17257453f9b7a6454181fb0be0ed4fbe11df8344f1acd |

An earlier root orchestration attempt stopped before tests: cargo fmt --all incorrectly included
14 byte-pinned generated Rust files excluded by Taskfile.yml:6-7. That check exited 1 without
changing files. Its full log and source snapshot remain in final-test-disposition. Root corrected
the command to the exact three-package formatter scope used by the adversary; no generated
source or test assertion was altered to satisfy it. The final run uses the separate v2 runner
and fresh scratch/browser roots. This command-selection mistake is not a product failure.

The two final additive tests are frozen by bot commit d3c2c1dd09ad279f5bf6b4c1050b589a4a7ad761;
both author and committer were verified. Source integration, the full repository gate, actual
independent producer mapping/AEP correspondence and publication remain subsequent work. Neither
this record nor the package check marks the coverage story implemented or changes defaults.
