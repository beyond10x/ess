# Consumer coverage integration — 2026-09-08

The consumer-coverage remediation passed every repository gate step and site-build at
`49732ec39da306e277eaf4f1d820e246e08797cf`. The new gate extracts the actual model and consumer contract inventory,
checks explicit accounting, and executes the exact cases used to qualify support or refusal.
New or changed identities cannot inherit the fixed initial unknown allowance.

The existing baseline remains explicitly unproven: 1,806 models across 87 behavioral profiles,
157,122 cells, **54 Supported, 0 Refused and 157,068 BaselineUnknown**. Qualification of the initial
unknowns remains with the separate draft epic. This result does not claim complete consumer support.
The 22 qualifying cases are direct Rust assertions, with no nested runtime claim attached to them.

## Actual complete integration

| Lane | Actual exit | Seconds | Libtest summaries in lane log | Passed in lane log |
|---|---:|---:|---:|---:|
| site-build | 0 | 25.722343 | 0 | 0 |
| fmt-check | 0 | 2.468288 | 0 | 0 |
| clippy | 0 | 45.810306 | 0 | 0 |
| test | 0 | 356.929874 | 199 | 2311 |
| doc-check | 0 | 16.732259 | 0 | 0 |
| example-check | 0 | 1.687293 | 0 | 0 |
| projection-check | 0 | 16.890678 | 0 | 0 |
| support-check | 0 | 8.462503 | 0 | 0 |
| consumer-check | 0 | 589.016972 | 0 | 0 |
| release-check | 0 | 5.046613 | 0 | 0 |
| action-check | 0 | 1.655299 | 0 | 0 |
| planning | 0 | 40.601267 | 0 | 0 |

The full test lane reports **2,311 passed, 0 failed, 0 ignored** in 199 libtest summaries,
including all 30 browser replay cases and all 40 synthesis feasibility cases. The affected checker
package's 110 cases were already accepted after both independent source reviews. Prior partial
attempt counts are not added to the full-suite total. Optional runtime behavior is not inferred
from a passing outer case.

The actual consumer-check executed 72 successful native commands: six locked/offline builds,
22 exact listings, 22 ignored-case listings and 22 exact single-threaded case executions.
Their aggregate recorded native duration was 31.864073754 seconds.
All 157,122 final cell records equal the accepted correction2 result. Seven checker-owned native
images and seven separate root copies were completely hashed and read back, alongside the actual
coverage producer images and source snapshots. No imported green receipt substitutes for execution.

Root readback: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/integration-preparation/final-integration-readback/readback.json`, SHA256 `6c62998a3c951d26d6df63efdb6837d803bde1cad2240ce3f239c47a543fbfa5`.
It verifies all 1,215 tracked source files, every lane's own direct status and full log hash,
the actual consumer command streams and case counts, source/provider equality and retained native
bytes. The readback took 4.863672 seconds (5.410664 seconds for the direct command). Original full gate records live at
`target/review-boundaries-18/gate-49732ec39da3-attempt2/`; continuation7 supplies the nine remaining
lanes and retains the original site/fmt/Clippy results. Their complete archival locations are
recorded separately during managed cleanup.

## Accepted source correction and remaining work

The source implementation is accepted at af8ae3b95317ef351e8e4251d883d818c6e71b0f and
integrated with the planning evidence in 49732ec39da306e277eaf4f1d820e246e08797cf. Findings from both source reviews were fixed:
consumer owners and associated contracts remain visible; required/default trait callables and
opaque signature/associated-declaration macros receive explicit accounting. All prior assertions
remain, with only a repeated blank line formatted away. The final correction added 42 finite
classifications without removing any of the 7,671 previous classifications or expanding e005.
The two independent review records and their fixed outcomes remain in AEP; no third attack ran.

This closes the implementation obligation for original story `review-consumer-coverage` after AEP
records this named test result and admits the terminal move. Publication and exact managed cleanup
are performed under the existing standing approval and recorded from their actual outcomes.
After publication, 27 of 31 original review stories are published. Output ownership, primitive
semantics, typed diagnostics and fuzzing remain original draft obligations; execution recovery is
separate. No release tag, version bump, deployment or downstream publication is selected.

## Integration execution history — 2026-09-08

The accepted source is merge commit `49732ec39da306e277eaf4f1d820e246e08797cf`
(parents af8ae3b95317ef351e8e4251d883d818c6e71b0f and
3124b1e843bceb3826b30ffe9f0af3499a834536). Root verified all 1,215 tracked files:
implementation bytes match accepted af8; the four integration planning files match 3124.
All source and assertion bytes stayed fixed through the following attempts.

Each original command retains its raw streams, actual direct status, launch identity and source
record in `target/review-boundaries-18/integration-preparation` or the gate directory. An
orchestrator termination is not a product assertion failure or a completed test pass. Partial
counts from retries are not added to the final full-suite count.

| Attempt | Outer exit | Seconds | Observed result |
|---|---:|---:|---|
| integration-v3 | 201 | 14.053523 | Site build received a native linker flag for WASM; actual task 201. Corrected only the site environment. |
| integration-v4 | 1 | 142.712176 | Site/fmt/Clippy completed; monitor encountered a removed Cargo temporary object and interrupted test compilation. |
| integration-continuation1 | 1 | 37.404299 | Allocation ceiling interrupted test compilation; no completed test summary. |
| integration-continuation2 | 1 | 242.521660 | Allocation ceiling interrupted the browser binary after 39 summaries (304 passed, 0 failed) and 11 browser assertions. |
| integration-continuation3 | 1 | 235.521939 | Allocation ceiling interrupted the browser binary after 39 summaries (304 passed, 0 failed) and 24 browser assertions. |
| integration-continuation4 | 201 | 32.749562 | General TMPDIR on tmpfs caused socket rename EXDEV at input_discovery_cases.rs:1044; 81 passed, 1 failed across five summaries. Monitor had no error. General TMPDIR restored to checkout filesystem for continuation5. |
| integration-continuation5 | 201 | 266.645923 | 1,673 passed and three failed across 114 completed summaries. All 30 browser cases passed. The three generated-WASM builds failed on inherited native `-fuse-ld=lld`; no monitor refusal. |

Three completed lanes remain the original results in `gate-49732ec39da3-attempt2/results.json`:
site-build, fmt-check and strict Clippy. Their direct child statuses, log hashes and unchanged
source are verified before continuation. The corrected site invocation clears native RUSTFLAGS
for WASM only. Its log preserves npm's 30 audit findings (9 moderate, 21 high), dependency integrity,
uuid deprecation and two installation-script approval warnings. Site compilation succeeded;
this wave makes no dependency-security remediation claim.

The first continuation allowance preparation also refused before writing a grant or launching a
producer; its original refusal is retained. The first continuation2 evidence archive readback
refused a copied executable's 4,096-byte allocation change with unchanged payload. No cleanup
occurred from that refusal. A separate fsynced census/archive/readback succeeded and retained the
original failed attempt. These are orchestration records, not extra product tests.

Continuation5 assigns general fixtures to its owned cache TMP on the checkout device and browser
profiles to its own executable `/dev/shm` directory. It reuses the complete independently read-back
Go cache there. Test threads are bounded to two; no test filter or assertion change is introduced.
The actual consumer-check retains its declared locked/offline provider profile. Source, native
tools, browser payload and prior successful logs are reverified throughout. Resource readings are
live samples, not a filesystem quota; disk, tmpfs and available-memory reserves are checked separately.

## Exact retained-record relocations during integration preparation

Every complete receipt below records successful archive, separate full payload/native readback
and exact retirement, or an explicitly documented reproducible dependency/cache cleanup. Source,
Git history and managed checkouts remain live pending publication and managed closure. Original
failure streams and counts remain in the archives; archives do not promote their results.
All paths are below `/home/timo/.cache/ess-review/2026-09-06-resume/`.

| Complete receipt directory | SHA256 of complete.json |
|---|---|
| `waves15-16-remaining-records-retirement` | `a278ebb892a58a46f75fcf61e1fb35711564366546e1e81a385a7077aeecdc05` |
| `consumer-final-records-retirement` | `61c34236c6fc9dc27be4c602f1d82ab3fcae3ae3f623f57b627a66d3293a6687` |
| `coordinator-website-dependencies-retirement` | `fcaf19cd0c65be1d13ccf56c62178a4885519915935e25fb6665955a0df2949d` |
| `consumer-integration-attempt1-retirement` | `014bf29bc028f0dbb36b562c0cb87060a352c17eeccc8199ae397ecf3895358d` |
| `coordinator-unused-toolchain-retirement` | `36fe5e15b340ff4bc600aa6bbc97b12b7330561338d8a382683aae1603ada1e2` |
| `coordinator-unused-go-cache-retirement` | `1d19086ad09b5a3aa6d9fbb599864dd9f73d26bcb60515b17688ed74adc24d04` |
| `consumer-completed-tmp-records-retirement-recovery` | `09013e72149b7f934dbf9f75da284ca757149872e9bce026c819a82eccd8a12b` |
| `unit-completed-site-dependencies-retirement` | `5aaae095c3e217c945f783c15051b37dbe19ad5fd5520a097c5a212d508d01bd` |
| `unit-completed-site-cache-retirement` | `d88bf2e39ecfa0ec29f934c1585dc884040df242de4f7e4116e5e10759777ab2` |
| `consumer-continuation3-records-retirement` | `231aa2eb10bcb4a8aad21d22cfeafb262bba08610c90c8c61700449a1e6177de` |
| `unit-idle-go-cache-retirement` | `3810dfcdafe5b1e29908c977e9bf5f7b0a2e32f7c9505ace2ada659265c45020` |

Protected processes with unreadable descriptors are recorded in each process census; no complete
machine-wide process-visibility claim is made. No unrelated cache or excluded downstream tree was
changed. Token and tool-use totals are unavailable from this harness and are not invented.

Continuation6 keeps general fixtures on the checkout device and uses target-specific
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS` for native linking. General RUSTFLAGS
is absent so nested WASM builds use their own linker. Consumer-check retains the Taskfile
profile explicitly. No source or assertion changes accompany this environment correction.

The first target-environment attempt (continuation6) was root-interrupted before any test binary
ran: actual rustc argv showed two copies of the native flag because the ancestor Cargo target
configuration already supplied it. Original direct Task 201/outer 1, KeyboardInterrupt, raw logs
and the interruption intent remain retained. Exactly 531 reproducible compiler-cache files with
that duplicated-flag fingerprint were independently hashed, checked against the stopped producer,
scanned for visible process references and removed. All source, evidence and fingerprint metadata
remain; cleanup-complete records the exact list and native identities.

Continuation7 instead uses the existing ancestor target configuration, pinned as
/home/timo/.cargo/config.toml SHA256
e41aa5953bfbdb6337fe0a2642bd87723b6bd581504797160af2f2aaef444e29.
It removes both global and redundant target flag environment settings for mixed-target lanes;
the actual consumer lane retains its exact declared global native profile. The warm test build
completed in 5.67 seconds before tests. The earlier capacity preparation refused before launch;
a subsequent first launcher selected v6 with v7's grant and refused immediately at the root-path
assertion (outer 1, 0.054498 seconds), before any product command. Original launch/grant/streams
remain separate from integration-continuation7-corrected records. These do not count as tests.

The prospective capacity revision changes the coordinator-selected primary free reserve from 8 GiB
to 6 GiB with an explicit 2 GiB additional primary allocation cap; previous observed full-test growth
was 447,524,864 bytes. Tmpfs allocation and free/available-memory guards remain 8 GiB. No repository
assertion, test filter, source check or required lane is changed by that operational policy.

Additional complete retention: `consumer-continuation5-records-retirement/complete.json`, SHA256 `8602bc9a51e1a428581a1f81dd0669c92290686bcbad4e7b8cee5e2f60e06cba`.

Additional complete retention: `waves5-6-completed-records-retirement/complete.json`, SHA256 `d77be04bbaee517bd08172a5f462ecfde8c5d8e644d59eee36095957eaca0b65`.

AEP closure initially rejected the fractional timestamp `2026-09-08T06:16:16.923229Z`
with `is not an instant this build can read`. The same observed readback was recorded at
whole-second precision, `2026-09-08T06:16:16Z`; the precise direct timestamp remains in the
original readback result. The three successful body/scope updates were not repeated.
