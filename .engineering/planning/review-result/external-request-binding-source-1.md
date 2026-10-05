---
format: aep.planning-md/3
id: review-result:external-request-binding-source-1
kind: review-result
status: active
title: Bounded request-binding source and adversarial review; package checks pending
relations:
- reviews: story:external-request-binding
revision: 1
---
unit: ESS412/414 checkpoint3, base 2f554561bef25125a93a1fb1d6517d50cb24ed20
verdict: nothing found in completed source and bounded adversarial review; whole-unit acceptance pending
cases: independent executed 0→2 distinct passing cases, semantic red 0, retained fixture failure 1
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: coordinator-executed generated fixtures in two assigned directories, listed below
needs-coordinator: owning package tests, all-target lint and refreshed-base source review

```text
 crates/generate/ess-synth/src/go/behaviour.rs      | 101 ++++++++++++++++++++-
 crates/generate/ess-synth/src/go/port.rs           |  47 ++++++++--
 crates/generate/ess-synth/src/rust/behaviour.rs    |  71 ++++++++++++++-
 .../ess-synth/tests/adversary_served_pass1.rs      |   5 +-
 .../ess-synth/tests/adversary_served_pass2.rs      |   5 +-
 .../tests/adversary_served_pass2_at_most_once.rs   |   5 +-
 .../generate/ess-synth/tests/declared_behaviour.rs |   4 +-
 .../fixtures/declared-behaviour-go-harness/main.go |   4 +-
 .../fixtures/declared-behaviour-harness/main.rs    |   5 +-
 .../tests/served_correction_pass2_bridge.rs        |   5 +-
 .../generate/ess-synth/tests/served_publication.rs |   5 +-
 .../ess-synth/tests/served_unfinished_committed.rs |   5 +-
 crates/generate/ess-synth/tests/synthesis.rs       |   4 +-
 13 files changed, 232 insertions(+), 34 deletions(-)
```
The diff above is the frozen author change, not reviewer production edits; it lists thirteen tracked paths. The new integration test and design document add the other two inventoried paths. Review changes are only copied test harnesses and evidence in assigned ignored scratch. No production, author-test or store edit occurred during review.

## Outcome and scope

Both independently authored cases now pass against exact checkpoint3. The source inventory SHA256 is `405d4b2f049f17c450e1ebdc24e4c4ba66bc0ec61e3047c054ae761868204bab`; all fifteen current source hashes match the frozen inventory. This is completed bounded review, not whole-unit acceptance, integration permission, or evidence for a later refreshed base. Owning package tests, all-target lint and the refreshed-base source review remain pending. ESS415 served collision support remains an explicit separate refusal, not delivered by this change.

Reviewed the complete fifteen-path unit, accepted story revision23 and scope amendment. Rust/Go request identity and payload allocation, required API migration, unchanged legacy assertions, no-use/owed emission, collision handling, typed branch routing, component-local allocation and exact ordinary-byte controls had no source finding. The synthetic suffix case is correctly described as a reservation-set test; package normalization does not create a claimed input/input_ pair. The existing Number/CAS/seed contracts are outside this unit.

## Independent execution and fixture correction

The probes were written before execution. Root executed the admitted copied-harness commands, preserving the author source as an exact prefix. The originally planned standalone sequence was explicitly narrowed by the coordinator to one combined two-case run to avoid redundant executions; after its fixture error, only the failed case was rerun. The executor was root, not this reporting turn. Original plans, probe, binary and run remain retained.

Execution1 compiled the harness successfully in0.392588seconds, then ran two cases in34.370686seconds: guard-order passed; mixed-case failed while compiling the reviewer-owned Go child fixture. The Go fixture attempted to assign an OwedOutcome interface into an earlier SendOutcome variable. This is a test-fixture typing error, not a semantic source finding or a product RED.

The only correction was a separately named `owedAnswer` local and the same nil assertion referring to it. All remaining outcome/sentinel/event/trace assertions are identical. Corrected probe SHA256 `43717ad6cb9693cf2260cc444f2a2cdc566704b243d4da79fde06802d2dcdf1f`; original SHA256 `abd25f239d00e172fbbc206a933a888281d3708afb5d275e275d5238d7f55251`. Exact diff:

```diff
--- probes.rs
+++ probes-amended-2.rs
@@ -1376,8 +1376,8 @@
     answer, err = service.Send(held.Send{Value:-7})
     if err != nil { panic(err) }
     if _, ok := answer.(held.SendOutcomeRefused); !ok || len(service.DrainOutbox()) != 0 { panic("refusal changed") }
-    answer, err = service.Owed(held.Owed{Value:exact+1})
-    if answer != nil || err == nil || err.Capability != "independent sentinel" || err.Source != "renewal.input.Owed" { panic("owed result changed") }
+    owedAnswer, err := service.Owed(held.Owed{Value:exact+1})
+    if owedAnswer != nil || err == nil || err.Capability != "independent sentinel" || err.Source != "renewal.input.Owed" { panic("owed result changed") }
     if len(service.DrainOutbox()) != 0 { panic("owed published") }
     want := []call{{"external",exact},{"external",-7},{"owed",exact+1}}
     if !reflect.DeepEqual(p.calls,want) { panic("crossed generated/owed boundary") }
```

Execution2 compiled the corrected harness in0.328496seconds and ran only the mixed case in8.598097seconds:1passed,0failed,13filtered. The previously passing guarded-order case was not rerun. Total distinct independently added passing cases:2. Do not count the three case invocations as three distinct tests, and do not add internal model vectors to Rust test-runner counts.

The first case executes true external answers with false/true input eligibility, exact request comparisons, declaration-order and first-taken callback traces, local refusals, state/effect preservation and positive success across Rust crate/workspace and Go, using both direct and HTTP routes. The second exercises generated and wholly owed commands through the real input-domain component in both Rust layouts and Go: precise large integer delivery, event/drain behavior, negative refusal, the owed sentinel and an exact callback/delegation trace. It also checks that owed commands are absent from the generated request enum/wrappers. These provide executed safety-fact evidence at level4 for the reviewed typed-input/dispatch boundary. They do not supply authentication, deep-copy Go aliasing guarantees or live runtime evidence.

## Exact commands and full outputs

Execution1's full compile argv/environment/admission is `execution-1/admission.json`; executed plan SHA256 `c0c3cd6cc6ffa054bd51579edfa250f3c003702755f7c6b8acce0f148f5340f7`. Runner:

```text
probes independent_ --test-threads=1 --nocapture

running 2 tests
test independent_external_when_keeps_request_and_declaration_order ... ok
test independent_mixed_generated_owed_input_port_keeps_boundaries ... 
thread 'independent_mixed_generated_owed_input_port_keeps_boundaries' (507523) panicked at .engineering/drafts/external-request-binding-412/adversary-review-1/probes.rs:1421:9:
fixture compiler result, not semantic RED (go): # independentmixed
./main.go:36:19: cannot use 1st function result (value of interface type input.OwedOutcome) as input.SendOutcome value in multiple assignment: input.OwedOutcome does not implement input.SendOutcome (missing method isSendOutcome)

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    independent_mixed_generated_owed_input_port_keeps_boundaries

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out; finished in 34.37s

```

Execution2's full compile argv/environment/admission is `execution-2/admission.json`; executed plan SHA256 `100fd0efde51b714251acded2b71788ef6fd80d49532feff80c0a2034cc967e1`. Runner:

```text
probes-amended-2 independent_mixed_generated_owed_input_port_keeps_boundaries --exact --test-threads=1 --nocapture

running 1 test
test independent_mixed_generated_owed_input_port_keeps_boundaries ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 8.60s

```

Both compile logs are empty and exit0 (SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855). Combined first-run log SHA256 `4c3af6278e311f97bebd23403949b44cc167eb4691685b2ef699249963ca94f1`; corrected mixed log SHA256 `9f097f16800f097e6479864023baf9ad9a440983a75508c59fa6d2f42b3cff63`. Both recorded source_unchanged=true; the reporting turn independently rechecked all fifteen hashes. Fresh admitted free bytes were12,904,460,288 and13,635,870,720 respectively. Neither execution timed out. No provider/model invocation or cost was measured or claimed.

## Author evidence and limits

The author targeted run independently records six selected integration passes (six filtered) and one allocator-unit pass; these are separate from the two review cases. Its raw logs and results remain at focused-run-3. Earlier passing author request-binding cases are retained. This report does not infer a completed owning package/lint gate from focused results. No extra tests or compilers were run while writing this report. Later base/source changes require renewed source binding and the coordinator's pending checks.

## Outside-worktree paths

The coordinator admitted and executed generated fixtures at `/var/tmp/ess-412-independent-probes-20261004` and `/var/tmp/ess-412-independent-probes-20261004-rerun2`. Original and corrected runs use different roots. The exact owner-built dependency paths are retained in the two admission records and final evidence inventory. This report writes only assigned ignored review artifacts; no outside-worktree data was changed in the reporting turn. No lease or compiler lane is retained.

```findings
[]
```
