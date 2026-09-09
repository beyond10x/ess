---
format: aep.planning-md/1
id: review-result:cli-pipeline-decision-r1-20260909
kind: review-result
status: active
title: Independent review of authored CLI pipeline coverage decision
relations:
- reviews: story:cli-presentation-binding
revision: 1
---
# CLI authored-pipeline decision review, round 1

Verdict: approve.

Reviewed: docs/design/cli-consumer-pipeline-coverage.md, SHA-256
a5af3a5ad7b3f779ef6f63635cbcebe4d0a52dc0855a7966ed6c9a720fbe98b4.

This is a read-only design decision review, not an implementation adversary
attack. No Cargo, source/test/AEP edits or new execution evidence. Only this
ignored report was written.

## Decision assessment

The candidate makes the required boundary change explicitly. Lines 13-22
replace the three EssIr-start profiles with authored input admission, assembly,
ESS compilation and binding compilation, then add the selected terminal stage.
Lines 52-59 assign a refusal to the stage that actually returns it and require
both non-entry of later stages and an admitted terminal-success control.
That is an honest use of the existing Supported/Refused algebra. It does not
label a parser refusal as behavior of compile(&EssIr,...) alone.

The source entry sequence is real:

- RawSpecFile::parse is called for each source in
  crates/edge/ess-cli/src/load.rs:31-38.
- A source-reader failure stops before assembly at load.rs:41-47.
- Specification::assemble is called at load.rs:53, and its refusal stops before
  compilation at :55-63.
- ess_compiler::compile is called at load.rs:65.
- main.rs:1980-1986 exposes that load result to the CLI binding edge.
- The binding edge returns before reading/compiling the presentation on a model
  refusal, then calls Binding::from_yaml and ess_cli_contract::compile:
  crates/edge/ess-cli/src/cli_binding.rs:28-35.
- Artifact projection occurs only after that succeeds (:56-61).

The exact coverage entry identities for those common model stages are:

```text
ess_domain::lib(ess_domain)::spec::impl<RawSpecFile;>::parse
ess_domain::lib(ess_domain)::spec::impl<Specification;>::assemble
ess_compiler::lib(ess_compiler)::resolve::fn::compile
ess_cli_contract::lib(ess_cli_contract)::resolve::fn::compile
```

Emission adds
ess_cli_project::lib(ess_cli_project)::fn::project.
Direct execution adds
ess_cli_project::lib(ess_cli_project)::runtime::fn::run.

Specification assembly really runs validation
(crates/specify/ess-domain/src/spec.rs:219-262); ESS compilation additionally
revalidates before resolution
(crates/specify/ess-compiler/src/resolve.rs:857-875).
The candidate's sequence is therefore correct.

Binding::from_yaml remains a real reader/helper in this pipeline and must be
used by the authored-binding fixtures. The candidate need not classify every
helper as another independent bound entry. Conversely, this decision does not
qualify filesystem discovery/security, presentation-reader errors or the
generated installed binary's behavior merely by listing model admission stages.
Lines 24-29 already preserve the crucial installed-binary/runtime limitation.
The profile describes source-text semantics, not full coverage of every edge
I/O operation.

## Can the nine topology IDs now have honest tests?

Yes, under the newly selected complete pipeline boundary. Qualification remains
pending actual test authoring, exact attribution, extraction and execution.
The following finite cases give a concrete pilot; these are proposed assertions,
not observed test results.

| Exact model ID | Appropriate pilot and actual refusal stage |
|---|---|
| wire:RawSpecFile#/definitions/RawReplicas/additionalProperties | Add extra: 1 inside a replica object. Assert the particular unknown-field reader error, then no assembly/compilation/terminal call. A control removes only that key and reaches the selected terminal. |
| wire:RawSpecFile#/definitions/RawTopology/additionalProperties | Add extra: 1 beside workloads; assert the corresponding source-reader unknown-field refusal and admitted control. |
| wire:RawSpecFile#/definitions/RawWorkload/additionalProperties | Add extra: 1 beside replicas/stateless/requires; assert the corresponding source-reader refusal and admitted control. |
| wire:RawSpecFile#/definitions/RawReplicas/required | Remove min from a present replicas object that still has max: 3; assert the specific missing-min reader error. Restoring a valid min must reach the terminal. |
| wire:RawSpecFile#/definitions/RawReplicas/required/0 | The same case can qualify the exact required-array member min, provided the case explicitly identifies the missing member rather than an unrelated parse failure. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/min/minimum | min: -1 must be refused by the unsigned raw reader. Separately assert min: 0 successfully parses and is refused by assembly's nonzero-floor rule. A positive floor reaches the terminal. Name the different stages; never claim zero violates the schema's minimum. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/max/minimum | max: -1 must be refused by the raw reader. Separately assert max: 0 with min: 1 parses but assembly refuses a ceiling below its floor. A ceiling at least equal to the positive floor reaches the terminal. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/min/format | min: 4294967296 must produce the actual out-of-u32-range reader refusal. An otherwise valid stateless workload with min: 4294967295 should reach the terminal; preserve another ordinary admitted control as needed. Attribute this to the concrete reader's uint32 representation. |
| wire:RawSpecFile#/definitions/RawReplicas/properties/max/format | Use max: 4294967296 for the raw reader's range refusal, and max: 4294967295 with a valid positive min for the terminal-success control. Attribute the reader representation, not JSON Schema evaluator behavior. |

Owning declarations are
crates/specify/ess-domain/src/topology.rs:36-69:
three deny_unknown_fields readers, required min: u32 and optional max: u32.
The nonzero-floor and ceiling-versus-floor constraints are :130-161.
Topology conversion runs its validation at :318-320.
RawSpecFile::parse uses the actual serde_yaml reader, not a JSON Schema
validator (crates/specify/ess-domain/src/spec.rs:141-154).

For each malformed model, record a stage trace or equivalent explicit counters
around the real calls. The stage that returns a refusal is entered; later stages
must remain unentered. Terminal-success controls must really assert the binding,
entire artifact map or actual direct runtime result/recorded invocation relevant
to that profile. An early-return-only test or a separately unrelated happy-path
test does not satisfy the candidate's stated control requirement.

## Numeric/schema attribution is the important constraint

The draft does not assert that the source reader enforces a JSON Schema.
Its minimum-zero example at lines 35-37 correctly identifies workload validation
as the refusal owner. Its finite attribution rule at lines 61-66 also explicitly
leaves unexercised distinctions outstanding. I found no incorrect enforcement
claim that requires revising this candidate.

Keep that precision in the pilot:

1. The schema's minimum is 0.0, while the complete model requires a positive
   floor and any present ceiling at least equal to that floor. Zero is inside
   the schema's numeric lower bound yet can be semantically invalid. That
   difference must be tested and reported, not removed from either contract.
2. The schema emits format: uint32, and the reader's Rust field is u32.
   An overflow refusal establishes the latter's actual range contract.
   It does not establish that an arbitrary JSON Schema validator enforces
   format as an assertion. There is no JSON Schema validation call in this
   admission path.
3. Exact schema spelling checks may corroborate attribution to these wire IDs,
   but cannot stand alone as consumer behavior. The schema-to-reader relationship
   must be described concretely; do not silently assert the whole emitted
   schema and reader accept exactly the same documents.
4. A requirement using multiple numeric cases must identify the distinct named
   stages in its reason/refusal attribution. Do not collapse negative, zero and
   overflow outcomes into an unexplained generic "invalid input" claim.

These are applications of the candidate's existing attribution rules, not
requests to extend coverage policy.

## Scope, compatibility and handoff

Keeping the 87 existing profiles and accepted baseline unchanged while
fingerprinting only the three new profiles is coherent. The existing binder
supports finite entry lists and fingerprints their definitions/entrypoint
declarations (consumer_coverage/proposal.rs:180-199). The candidate explicitly
requires new receipts and refuses borrowing prior qualification (:70-75).
No new disposition or persisted format is needed for this decision.

Approval authorizes this bounded profile decision, not advance qualification
of the nine IDs, the other outstanding model families, or full ESS gates.
The next concrete step is the nine-ID pilot under freshly declared profiles.

Worktree: cli-binding-ess-20260909.
Own lease cli-pipeline-decision-review-20260909 released after this report.
Root retains source, AEP and coverage-policy ownership.

