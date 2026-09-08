# Specification fuzzing and the measured Go owner refusal

Status: accepted for story:fuzz-the-specification-surface on 2026-09-08 under
approval-record:review-remediation-standing-implementation. Wave20 selects this single unit.
Source baseline: `9ab8a1b16dec7c08ba2dd2ac3b507f48254514f5`.

The two candidate binding reviews remain the final two design reviews. The reviewed v3 carrier,
property, observation, mutation controls and finite campaign limits are retained. The recovered
publication draft SHA256 is `fc6adbeaac0a1f860efa6a03950f71b700c6f1c04804261ac79276e5065c3132`.
This acceptance refreshes source, launch and delivery authority; it adds no third design attack.
The fresh read-only aep-drive:story-scoper 0.8.1 assessment confirms the same six reservations.
Root retained that report and its exact binding-read addendum, verified 24 current inputs, and
verified all 237 recovered regular preparation files. Evidence identities and local locations
are recorded in the wave20 opening receipt.

The exact six write reservations are `fuzz`, `crates/generate/ess-synth`, `Taskfile.yml`,
`crates/edge/ess-cli/tests/target_failure.rs`, `docs/design/review-specification-fuzzing.md`,
and `website/docs/reference/formats.md`. Root Cargo files, compiler/domain and CLI production
remain outside the assignment. Preserve current consumer-check and support-check plus every other
existing gate lane, and preserve the integrated Rust snapshot-name allocation correction.
The Go ownership lookup and reserved CLI failure fixture remain unchanged from the earlier review.
Historical seed evidence remains bound to its original source; setup must capture fresh positive
Go maps before the repair. No successful harness or setup run is implied by accepting this page.

## Property and input domain

The harness feeds actual ess/1 source documents through parse, assembly/validation and compilation.
Every accepted compiled model reaches each current artifact generator, the separate docs-ir
serialization path and all four synthesis targets. An ordinary parse, validation, projection or
target Result refusal is an observed result. It must not prevent an independently callable later
target from running. A genuine panic is failure and stays a libFuzzer crash, never a skipped
document or an expected successful result. Do not unwrap ordinary Results to manufacture crashes.

The corpus unit is an ordered collection of UTF-8 source-label/text pairs. The encoding is a
closed harness-local JSON object with exactly a documents array, whose entries contain exactly
label and text strings, not a new ESS product format or filesystem import protocol. Reject
duplicate object fields, unknown fields and malformed scalar shapes before model parsing.
Document labels are diagnostic identities only. Reject duplicate labels and malformed encodings
as harness-input refusals; never open a path derived from fuzz bytes. Preserve document boundaries
when constructing SourceMap and RawSpecFile inputs. In particular, combining system.yaml and
core.yaml under domain: demo.core can erase the measured missing-owner trigger and is prohibited
for that mandatory seed.

Commit both readable source fixtures and their deterministic encoded regression inputs, with a
checked exact translation and stable labels. The three mandatory identities are a system-level
types declaration, String mapped to Optional<String>, and a specification with no on_failure:
retry binding. Preserve the current minimal synthetic fixtures and original bytes; no unavailable
external consumer checkout is needed. All three must validate and compile in stable replay.
Retain them even if corpus minimization would remove them.

The sampled input budget is 1–8 source documents, 1–64 UTF-8 bytes per opaque label, at most
16,384 bytes per source text, 32,768 total source-text bytes and 65,536 encoded bytes. Check the
encoded budget before decoding and the remaining bounds before model parsing. Preserve supplied
document order; names are not sorted to conceal input-order defects. The JSON carrier has a
fixed shallow schema; use a bounded decoder and reject nested wrong shapes. Raw YAML mutation
is bounded by byte, time and memory budgets, without pretending those impose a semantic YAML
depth bound or that indentation measures parser depth. Do not introduce production admission
limits. These are sampling bounds, not a claim that ESS rejects larger valid specifications.

Use two engine entries sharing the same production pipeline. LibFuzzer mutates the original
carrier bytes externally; the byte-carrier entry decodes them
without an additional source mutation. Identical original bytes replay the same ordered source
collection. The structured entry deterministically maps its bounded
input bytes to at most 32 declarations, 8 fields/variants per declaration and 4 nested type
constructors, then renders actual ess/1 source bundles inside the same carrier/text budgets.
Include Newtype, Struct, Enum and Union, supported primitive leaves, optional/collection nesting,
and explicit system-owned versus domain-owned named-type variants with stable references. There
is no Alias variant to invent. Classify the actual admitted Specification.system().format,
including production defaults; do
not infer it from a textual header search. A valid ess/2 model is outside this campaign, recorded
separately from a production validation refusal. Keep source-version mutation visible rather
than rewriting a mutated header back to ess/1.

The stable lane replays a finite structured vector set containing at least 16 distinct accepted
compiled bundles beyond the three mandatory seeds and covers all four named-type body families.
Check exact generated-source identities, so repeated copies cannot satisfy that diversity count.
Include intentional invalid/refused controls as distinct outcomes. The instrumented lane records
actual accepted/compiled callback counts for both entries, separately from stable replay. Each
live entry qualifies only if at least one actual engine callback compiles an in-domain model,
every compiled callback completes all required downstream invocations exactly once, the complete
observation stream admits, and the engine terminates successfully without crash, timeout or
observation failure. Zero compiled callbacks is insufficient evidence and makes that lane
non-green even if libFuzzer exits zero. Retain starting inputs that exercise both entries; report
startup and mutated work only where the actual engine evidence distinguishes them. Record source
diversity without inventing a minimum random acceptance rate. Stable replay's sixteen distinct
accepted bundles do not substitute for actual live callbacks.

## Pipeline and observability

Use RawSpecFile::parse for each document, Specification::assemble (which already validates),
ess_compiler::compile, ess_gen::artifact::run for every registry entry, actual docs-ir construction
and pretty serialization, then synthesize_for for each Target. Derive/check the generator set
against ess_gen::generators(), preserving the actual registry order. Calling Generator::generate
directly does not exercise the checked production boundary: artifact::run owns provenance and
duplicate-path checks. Its ordinary Err remains an observed refusal and later calls still run;
its genuine assertions remain failures. Explicitly dispatch docs-ir because it is not an
additional registry entry. Filesystem-authored site inputs remain outside this property. Derive/check
synthesis alternatives against the actual Target enum with exhaustive matching without a
wildcard and dispatch Rust, Go, Web and Clap. A new
registry entry or target cannot disappear behind a stale parallel list. An unsupported dispatch
shape fails loudly with its identity. Add a concrete CLI-offered-choice correspondence case in
the reserved target_failure.rs file: compare the actual binary's complete generation and synthesis
choices with the current registry plus docs-ir and public Target alternatives. This catches a new
CLI-only projection that a library registry check alone misses. Preserve the existing public CLI
production surface; if help does not expose a complete checked set, return the concrete alternative
before silently accepting a partial comparison. No implementation-only artifact or conformance runner is
substituted for these source pipeline boundaries.

Use a compact closed typed harness observation contract; it is not a new product format. Each
attempt has a monotonically increasing identity, entry identity and exact original input byte
identity. Structured inputs additionally retain the deterministic rendered documents, their
identities and supplied order. Records distinguish Decode, per-document Parse, combined
AssembleValidate, Compile, each generator identity, DocsIr and each synthesis Target. Assembly's
success can imply a derived validation success, but do not count or pretend to observe a separate
validation call. After successful assembly, classify the actual source version before compiling
for this campaign. Ordinary returned refusal, successful invocation and not reached are distinct.

Persist and flush an attempt start, then the stage-start identity before each production call,
then its actual returned result. Close a normal attempt exactly once. Use bounded framed records
with explicit maximum record size; refuse oversized or malformed records instead of truncating
them into apparent success. Retain original inputs and large details in task-owned artifacts
referenced by exact digest, with write errors treated as observation failures. Admission rejects
duplicate, missing, out-of-order and truncated records, overlapping stage invocations and a
successful process exit without its complete terminal observation. A crash or timeout can leave
a last unfinished stage; retain that partial failed evidence without synthesizing completion.
Observation I/O failure is an operational nonzero failure, never an ordinary production refusal.

Stable replay reports actual attempts, compiled models and per-stage invocation/result counts;
every compiled in-domain attempt, including all three mandatory fixtures, must complete all ten
downstream calls. Each completed compiled attempt contributes exactly one invocation/result to
each of the five generator identities, DocsIr and four synthesis identities, irrespective of
ordinary Result success/refusal. On a successful lane, each downstream identity's completed count
equals the completed compiled-attempt count; partial counts belong only to failed evidence.
Distinguish cases, iterations, callbacks and subprocess counts. The live lane uses one worker and
retains its actual typed callback stream plus raw libFuzzer termination output. Derive the summary
from the admitted stream and report the measured relationship to libFuzzer counters; do not
assume its summary count equals callback/startup count or replace live evidence with stable replay.

Stable replay runs each bundle in a child under a ten-second monotonic parent deadline. On expiry,
the parent terminates and reaps the child before continuing, preserving the original input and
last stage observation. Classify timeout, crash/nonzero exit, launch failure and observation
failure separately from ordinary production refusals. Child exit zero without a complete
admitted observation is a harness failure. A controlled stalled-child case must prove termination
and reaping, not just a printed timeout. This guard is independent of the enclosing CI timeout.

No generated program compilation is part of this no-panic property. Target feasibility owns that
different guarantee. Existing Optional assignment or no-retry historical compiler defects do not
become proved fixed because the generator returned without panicking. Any new panic retains its
exact corpus input, command/toolchain/exit and target location for a concrete owner repair decision.

## Bounded Go prerequisite repair

The measured valid system-level newtype demo.Code panics at go/layout.rs:290 during owner lookup.
Go workspace already calls the Binary64 guard before Layout::of. Layout::of allocates names and
uses ownership internally, so the new owner prerequisite must run before its construction.

Compare every ir.types() identity against actual domain.types roster identities before Go name
allocation. Valid system-level Newtype, Struct, Enum and Union can lack these owners; validation
already checks domain ownership of entities, commands, events, views, errors and actors. Do not
invent additional valid-source owner gaps from those nouns. Collect every missing type owner
deterministically and return the existing
TargetFailureCode::MissingTypeOwner through Go's existing ess-target-failure/2 envelope. Emit one
cause per missing qualified type, with exactly that qualified name as its source and the detail
`Go cannot assign a package to type <qualified-name>: no domain owns it`, substituting the actual
name. Use the existing constructor's deterministic sorting/deduplication; these new Go detail
bytes are the selected contract, not a claim about the old Rust diagnostics. Preserve
the neutral SynthesisPlan and existing Binary64 preflight precedence. Do not guess a Go package,
call Rust's naming/empty-module checker, return a partial workspace, or catch/discard a panic.
Keep owner lookup's internal invariant where appropriate after the checked prerequisite.

Permanent library tests must cover Newtype, Struct, Enum and Union that can be system-owned,
both referenced and unreferenced where the language admits those forms, plus multiple missing
owners and deterministic order. Mark invalid input shapes as validation refusals, not execution
of an accepted Go case. Positive domain-owned controls must preserve existing canonical Go bytes;
Go's valid empty-domain behavior must not inherit Rust's empty-module refusal. Test both the
public synthesize_for facade and direct go::workspace, including an ownerless Binary64 ess/2
control for existing precedence; this does not widen the document fuzz domain beyond ess/1.
Capture complete pre-change owned artifact maps/plans and compare after the correction, rather
than comparing two post-change emissions. Existing all-target Binary64 refusal tests and every
original assertion remain unchanged.

Add a separate Go system-type case in the exact CLI target_failure.rs file. Its existing
Rust/Web empty-domain case and ess-target-failure/1 assertions remain intact. The new case must
execute both flat and generate-area spellings with text, JSON and YAML; assert Go's /2 envelope,
the exact MissingTypeOwner source/cause and neutral plan, exit status 1 and empty stderr. JSON
and YAML assert the typed /2 envelope; text asserts the actual target, source and selected detail
without demanding a machine-format discriminator or code that Display does not print. Cover no
--out, an absent output tree with an absent parent, and a pre-existing destination sentinel;
refusal must leave destinations absent or unchanged.
CLI production behavior needs no change if the existing Result boundary carries the new cause.

Update only the public formats reference's existing /2 description and affected cause explanation
to acknowledge Go missing-owner refusal. Preserve the separate Binary64 and normalization claims.
There is no new serialized shape, cause vocabulary, version bump or default switch selected.

## Workspaces, dependencies and actual gates

Choose a stable standalone fuzz workspace containing the shared pipeline and regression replay,
and a nested independently locked fuzz/engine workspace containing libfuzzer-sys and the fuzz
entry. This keeps nightly-only dependencies out of the stable replay graph. Neither workspace
joins the root workspace; both manifests/locks remain inside fuzz. Actual offline resolution of
both graphs is a first-stage unit prerequisite, distinct from archive/tool availability. If it
fails, retain the error and return the smallest concrete setup decision, without adding a fetch
to the default offline gate.

The Taskfile adds one stable fuzz-check step to task check without removing or weakening existing
steps. It explicitly runs standalone formatting, strict Clippy and deterministic replay/tests,
all using the exact manifest and locked offline Cargo. The local fuzz workspace is not covered
automatically by the root formatter/member enumeration. Ordinary workspace tests must not recurse
into task check. CI already delegates to task check; no nightly workflow provisioning is implied.

The retained complete Rust 1.98.1 snapshot is a candidate stable launch authority. Its manifest
is target/review-boundaries-12/preparation/toolchain-snapshot.json, SHA256
387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d: 378 entries and
1,392,727,899 regular-file bytes, with prior native/WASM probes and complete gate readbacks.
The floating stable installation changed since the older Rust 1.98.0 receipt and is not pinned
by that historical observation. The dated nightly, cargo-fuzz, Clang and five crate archives
still matched their earlier hashes during the refresh. Neither fuzz graph has resolved or built;
nightly source-library components remain absent. Fresh setup must bind the chosen exact compiler,
verify its full required payload and capture complete positive Go artifact maps before repair.
Historical output directory presence and successful command receipts do not substitute for that
source-pinned before/after artifact comparison.

The separate local instrumented lane uses installed nightly-2026-07-28, ordinary
AddressSanitizer, two build jobs, a finite run count, maximum input length, explicit per-input
timeout and RSS ceiling. Run each of the two entries sequentially with -runs=2048,
-max_total_time=120, -max_len=65536, -timeout=10, -rss_limit_mb=2048,
-malloc_limit_mb=512 and -seed=1592590347. The time limit can stop before the run cap; report the
actual completed count. Preflight the installed dated compiler commit against the scope record
and retain exact binary/version identities and flags. Do not use fork-mode ignore-failure flags.
This is bounded sampled evidence, not a proof of universal termination. Do not use MemorySanitizer,
careful mode or build-std without their missing component prerequisites. cargo-fuzz 0.13.2 has no
--locked flag in the inspected surface: enforce offline operation and verify the engine lock's
exact before/after bytes, without claiming a nonexistent locked invocation.

Mutable corpus, target, coverage and crash output live only under ignored task-owned paths. The
committed mandatory regression directory must not be lost to cargo-fuzz's default corpus ignore.
Keep exact original crashes and minimized derivatives as separate records, with no blanket tree
deletion. Disk and process preflight remains the wave coordinator's responsibility.

Minimum mutation controls demonstrate missing docs-ir dispatch, a skipped target, a swallowed
panic, a corpus translation that erases document boundaries, an engine adapter that never calls
the shared pipeline (or forces decode refusal for every input), and a missing terminal or stage
record despite exit zero. The no-work engine control must fail live qualification while stable
replay remains green. These controls verify the chosen boundaries, not random acceptance rates. Use a narrow injected test seam
inside the harness for controlled stage failures and count checking; the actual regression and
instrumented lanes still call the production APIs. Restore real execution to turn each control
green. Retain all original reds rather than weakening the mandatory seed assertions.

Two independent binding reviews are already retained as review-result:fuzz-specification-binding-pass1
and review-result:fuzz-specification-binding-pass2. This source/setup refresh is not a third
binding attack and changes none of the property, carrier, mutation-control or campaign limits.
The coordinator compared the final integrated source and scope and accepts this binding under
the standing implementation approval. Fresh resource assignment precedes each producer. Stage one of the
selected unit resolves the two offline lock graphs, validates tool readiness and captures the
pre-change positive maps before changing Go; those setup results are not a completed harness.
The integration source runs every task check lane plus site-build. Under the current source
completion boundary, this wave ends after verified ESS main publication and owned-worktree cleanup.
Downstream Website/Atlas delivery is excluded by the operator's latest scope instruction.

## Integrated setup and implementation sequence

There are two stages within this existing story and its six reservations. Stage one is a required
prerequisite to the Go repair and full harness implementation; it is not a separate story, format,
completed harness or substitute for final stable/live observations.

1. **Establish setup evidence before repair.** In the selected managed unit, first resolve the
   standalone stable graph and the independently locked nested engine graph entirely offline.
   Keep both manifests and locks under `fuzz`, outside root workspace membership. Create only the
   minimum owned manifest/readiness scaffold needed to establish actual tool readiness: bind
   exact full required compiler/runtime payloads, verify their hashes and actual versions, and
   compile/link/run the minimal dated-nightly ordinary-ASan readiness probe. A probe is setup,
   never a qualified final engine callback. Preserve real failures and return the smallest
   concrete setup requirement instead of silently fetching or substituting tools. Before any
   Go production repair, capture complete source-pinned pre-change valid Go artifact maps and
   plans, including the required domain-owned/family and empty-domain positive controls. Retain
   originals and full byte inventories, not just stdout receipts or output-directory existence.
   These three prerequisites—both actual offline graphs, actual tool readiness, and complete
   positive Go maps—must all be evidenced before stage two. If one fails, leave Go production
   unchanged and report the exact unmet prerequisite to the coordinator.
2. **Implement and prove the whole binding.** Once stage one is complete and the root's assigned
   resources remain valid, establish meaningful failing verifiers, make the bounded Go repair,
   and complete the shared harness, mandatory and structured replay, every mutation/watchdog
   control, the exact two live campaigns, CLI correspondence/refusal cases and all assigned
   package/standalone checks. Compare positive Go bytes with the pre-change maps. Setup success
   alone never completes this story. The source review and complete ESS integration/site gates remain separate coordinator-owned steps.

The dispatch brief names all actual unit, scratch, TMP, cache, compiler and target authorities.
All build outputs stay inside that unit. Never set `CARGO_TARGET_DIR` or share another unit's
build directory. Root workspace, standalone stable and engine build targets are separately
recorded; the engine's actual default target path must be established during setup, not guessed
from a different cargo-fuzz installation. Coordinator ownership includes creating/removing roots.

The retained ancestor Cargo configuration enables `/usr/bin/sccache`. An isolated CARGO_HOME
and unset wrappers did not disable it. The final unit environment must explicitly set
`RUSTC_WRAPPER=""`, `RUSTC_WORKSPACE_WRAPPER=""`, `CARGO_BUILD_RUSTC_WRAPPER=""`, and
`CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=""`; preserve and inventory ancestor configuration and verify
actual compiler launches. Keep CARGO_NET_OFFLINE=true, two build jobs, incremental compilation
disabled and task-owned temporary paths. Historical Rust 1.98.0 probes do not pin floating stable;
the retained complete Rust/Cargo 1.98.1 snapshot remains a possible stable authority, subject to
fresh full required-payload verification. The existing dated-nightly, ordinary-ASan, cargo-fuzz
interface and exact engine-lock requirements above remain unchanged.

Measure available filesystem bytes before and after each producer. Never start another producer
below 8,589,934,592 available bytes or without the coordinator's fresh unit reservation. Stop and
report a crossed floor; retain existing evidence and do not remove targets, caches or other
sessions' outputs to recover capacity. The preparation's historical measurements are not a launch
reservation. Root selects one unit at a time. Consumer coverage, browser delivery and output ownership are
already integrated at the source baseline; preserve their existing checks and behavior.

## Wave20 implementation evidence boundary

Stage one is retained separately from implementation: both independent graphs resolved offline,
the selected payloads were verified, and nine positive Go fixtures produced complete facade/direct
artifact maps and plans before the repair. Their original bytes are checked by the stable workspace.
The implementation's replay and engine share the source pipeline; live callback receipts are
independently matched to exact observed attempts. The sixteen finite rendered-source identities
are pinned under `fuzz/regressions`, and the stable Taskfile lane names that standalone workspace
explicitly. These mechanisms establish no generated-program execution claim. Actual campaign,
mutation and package outcomes belong to the unit's command records and coordinator review.
