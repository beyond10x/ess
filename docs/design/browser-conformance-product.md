# Browser conformance product

Status: binding design for the complete browser conformance product; implementation and runtime
validation remain outstanding. This contract governs default ordinary and coverage
`ess verify conform web` emission. Its product, presentation and ABI identities are registered
separately from suite/input/report versions. Resource limits below are enforced and measured
(section 4); feature completeness is a completion obligation, not an implementation claim.

## 1. Authority, baseline and complete source admission

The product supplies useful declaration navigation immediately after emission and actual
conformance execution after an explicit independent Rust target installation is built into its
WASM host. Navigation is declaration inspection. Execution uses the existing Rust Runner and
its immutable ExecutedRun capability. No JavaScript model executor or fabricated report bridges
the two surfaces.

The observed baseline is successful ordinary **and** coverage authored direct-response emission,
followed by real browser failures: the ordinary page displays `9007199254740992` where original
suite bytes retain `9007199254740993`, and the coverage page aborts on an unsupported suite step.
Node diagnostics and actual Firefox/BiDi confirm these distinct defects. Those probes performed
no independent target execution. They are not evidence of an emission refusal.

The source distinction is precise: `response::used_by` recognizes `ExpectResponsePayload`,
whereas authored direct responses lower to `ExpectDirectResponse`. Existing response/fixture
emission fences apply to the forms their predicates select, not all response forms. The new
default product supports every admitted runner feature and response form within the same finite
resource envelope. It must not preserve those feature fences, legacy JS step whitelists or
reduced-model omissions as default product behavior.

Relevant source seams are [web emission](../../crates/verify/ess-conformance/src/web.rs),
[response predicates](../../crates/verify/ess-conformance/src/response.rs),
[authored lowering](../../crates/verify/ess-conformance/src/authored.rs),
[CLI loading](../../crates/edge/ess-cli/src/load.rs),
[original-byte admission](../../crates/verify/ess-conformance/src/admission.rs),
[coverage lineage](../../crates/verify/ess-conformance/src/coverage.rs),
[Runner](../../crates/verify/ess-conformance/src/runner.rs), and
[exact-run counts](../../crates/verify/ess-conformance/src/counts.rs).

The complete model authority is retained original specification source, recompiled in Rust.
EssIr is Serialize-only with private resolved handles; compact IR JSON is not an admitted input
from which a browser may construct compiler authority. The emitter retains the complete acquired
source list, original UTF-8 bytes and stable relative labels in discovery order alongside its
compilation. Reuse acquisition/parsing in load.rs; do not reread a potentially changed source
manifest, invent a SourceMap iterator, or parse files individually when cross-file declarations
require `RawSpecFile::parse_all`.

Emitter and host parse the full set together, use `Specification::assemble`, compile with
SourceMap, perform existing model admission, and compare provenance with the selected suite and
every ancestor through existing SuiteProvenance/ess-gen derivation. Do not guess a second digest
formula. All declarations remain represented: response mappings, one-time policy, constraints,
preserves/deletes/sets/affects, aggregates, periodic bindings, caller/grants, components and
preconditions. Browser compilation uses only supplied bytes, without ambient filesystem/import
fetches. Source labels contain no local absolute paths.

Mandatory runtime admission order is:

1. Validate bounded frame, closed manifest, exact file hashes/lengths and complete original bytes.
2. Ordinary: `AdmittedSuite::from_json(original)`. Coverage: `AdmittedInput::from_json(original)`
   with every original parent. Filtered coverage without its full carrier refuses.
3. Parse/assemble/compile full sources, check model admission/provenance and product/ABI identity.
4. Regenerate the typed presentation, verify its binding, and issue an immutable Loaded handle.

Before all four succeed, factory invocation, target identity and every target/fixture callback
remain zero. A previous successful load/run cannot authorize changed bytes under the same name.
Generated canonical source-built suite/input bytes are that emitted artifact's original bytes;
thereafter they are preserved verbatim, including their complete coverage lineage.

Ordinary suites **run all admitted scenarios**. Navigation selection only changes focus; it must
not call `AdmittedInput::from_suite/select` or fabricate coverage. Coverage Run selection uses
`AdmittedInput::select` with sorted unique IDs, retains newly derived selected bytes and the full
original parent chain, and re-admits before target construction. Reports identify the actual
selected digest and inherited inventory. Run all uses the original input. Selection does not
relabel another run or silently change its coverage claim.

## 2. Default useful navigation before any WASM build

The emitted page works immediately when served as an ordinary static directory. It requires no
prebuilt WASM binary to inspect declarations. Choose a **Rust-emitted presentation document**,
not a JavaScript parser/validator/interpreter of original suites or source declarations.

At emission Rust admits the complete suite/input and lineage, compiles retained original source
files, verifies model provenance, and derives one complete lossless declaration presentation.
Write it as `declarations.json`. The product manifest binds its exact digest and length alongside
all original files. It contains scenario/step indices, complete typed declaration cards, source
and selected/ancestor digests, inventory/refusal declarations, and complete model declaration
panels. Semantic numeric values are exclusively tagged decimal text; no payload number is a JSON
number. Small structural indices are bounded u32. Strings render through text nodes, never HTML.
The closed display value family is Null, Bool, Text, NumberText, ordered List and ordered Object
entries. NumberText preserves the exact original/canonical numeric spelling without a JavaScript
Number conversion. Complete typed serialization feeds the shared Rust presenter; no semantic
field is dropped by a JS whitelist. It generates the same representation during WASM Load.

JS may parse this **display-only** document after exact file-integrity checks and closed display
schema checks. It may index cards, filter titles, expand trees and implement Step/Back/Play/Reset/
Select as navigation. It must not parse execution source/suite/input JSON, infer effects, evaluate
conditions, synthesize a world, compute expected outputs, construct reports, or turn display
values back into target values. Render every admitted step as a complete card; richer summaries
must not replace the complete tree. Unknown display schema tags cause a visible presentation
error rather than dropped fields. This closed rendering schema is not a second model authority.

Label the page: **“Declarations admitted at emission. Not executed.”** The provenance panel shows
which source/input bytes those declarations describe. A visible Run panel states **“Execution
requires building browser_host.rs with your Rust target installation”**, links the emitted
instructions, and reports a missing module as `build_required`, not a conformance result.
No empty world panel pretends simulation happened. A declared effect appears as a declaration;
actual state/results appear only after a real run. This is the initial new product's navigation
contract, explicitly distinct from historical replay/1's reduced declared-world replay. Preserve
that historical product and its assertions as a separate legacy entrypoint.

Before enabling navigation, fetch manifest + declared blobs as raw bytes, enforce fetch budgets,
and verify their SHA256/byte counts with WebCrypto, including the presentation blob. JS hashing
is consistency checking, **not Rust admission or authentication**. Any mismatch visibly marks
stale/corrupt output and disables the normal cards/Run until a coherent bundle is loaded. The
manifest cannot authenticate an attacker who replaced every file; this static distribution has
no signature claim. Absolute/parent/external paths and duplicate references are refused before
fetch. Only same-directory relative resources are allowed. Once a runtime exists, it rechecks
all originals independently; presentation bytes can never mint a Loaded handle.

This decision avoids a mandatory reader-only build and avoids shipping a frozen universal
reader binary. An optional future reader-only WASM package is outside initial scope. Replacement
original bytes accepted by Rust Load obtain a newly Rust-derived presentation; the original
static cards are not silently relabeled as describing the replacement.

## 3. Closed bundle, presentation and ABI

The identities are `ess-conformance-browser/1`,
`ess-conformance-browser-presentation/1`, and `ess-conformance-browser-abi/1`.
These are product/ABI identities, **not new suite/input/report majors**.
Keep replay/1 bytes/admission/refusal behavior unchanged. Unknown format,
ABI minor, field, enum tag, trailing byte, duplicate key or contradictory length is a refusal;
there is no permissive fallback to legacy replay or untyped JSON.

Bundle manifest has exactly: `format`, `abi`, `sources`, `execution`, `presentation`,
`generator`. `sources` is an ordered array of BlobRef; `execution` is tagged ordinary-suite or
coverage-input plus BlobRef; `presentation` is BlobRef. BlobRef has `path`, lowercase SHA256,
`byte_length`. `generator` has package/version and product semantic revision. Labels use relative
UTF-8 paths with no empty, dot, dot-dot, backslash, colon, control or absolute segments. A blob
appears once; labels are unique. The original source byte set/order comes from one acquisition,
not a second filesystem read. Execution is one exact ordinary suite or one exact input/1
containing the full original ancestor strings. No JS parent reconstruction.

The manifest and presentation writers use typed pretty JSON with two-space indentation and one
final LF, preserving declared struct-field order and deterministic ordered collections (catalog
layout P). Blob SHA256 is bare lowercase hexadecimal over every exact referenced byte, including
any final LF. Original source/suite/input bytes are not recanonicalized to obtain these hashes.
The selected suite digest continues to use its existing original-byte profile. Manifest blob
hashes establish consistency, not signatures; there is no circular whole-output hash or new
semantic model digest. Presentation is display data, never an execution input capability.

The module exports memory and four functions:

```rust
// Required export signatures; this design does not claim implementation is complete.
fn ess_browser_abi_version() -> u32; // exactly 0x0001_0000
fn ess_browser_reserve(len: u32) -> u32; // owned request buffer or 0
fn ess_browser_dispatch(len: u32) -> u32; // owned response address; a fatal ABI failure is a tag-5 error response
fn ess_browser_response_len() -> u32;
```

Reserve invalidates the previous request/response spans; dispatch consumes the last reserved
request with the exact length. Callers cannot supply arbitrary pointers to dispatch. Buffers
remain module-owned; no JS free function. Rust checks bounds/overflow/state before every slice.
After a fatal reserve/dispatch/trap the worker is discarded; no stale handle remains usable.
Static/module initialization performs no installation callbacks.

All integer framing is little-endian u32. Request header is 24 bytes:
magic `ESBW`, major=1, minor=0, opcode, request_id, payload_length. No trailing bytes.
Response header is 28 bytes: same magic/major/minor, response tag, echoed request_id,
status code, payload_length. UTF-8 fields are length-prefixed bytes. Closed request opcodes:

1. **Load**: manifest byte string; blob_count; for each manifest blob in its declared order,
   path byte string and raw blob byte string. All originals supplied together. Rust checks the
   entire frame/manifest/digests; admits the full execution document/lineage; parses all source
   files together, assembles, compiles, performs model admission and provenance comparison;
   regenerates presentation and verifies the bound display document. Success returns a u32
   Loaded handle plus presentation bytes. Failure clears any earlier Loaded state. Factory
   and target callbacks remain zero throughout, including after a prior successful load.
2. **SelectCoverage**: Loaded handle; count; sorted unique scenario-id strings. Reject for
   ordinary suites. Calls AdmittedInput::select and re-admits its retained full-lineage carrier;
   success returns a replacement handle, selected digest and presentation. Original input and
   newly derived selected bytes stay distinct and retained. Empty selection follows existing
   admission/count semantics, never a fabricated green.
3. **Run**: Loaded handle; 16-byte nonce; u32 presentation generation. No expected values,
   source overrides, timing knobs or fixture results are accepted here. Exactly one active run.
   Constructs the installation only now, then invokes the existing generic Runner with its
   concrete Target/Clock. Produces actual ExecutedRun and CountReport/optional CountRun.
4. **Release**: Loaded handle; drops retained state. No target is alive between completed runs.

Each response tag is one of `loaded`, `selected`, `completed`, `released`, or `error`.
Load/Select response payload fields are exactly handle, selected digest, presentation length+bytes.
Completed payload is selected digest, nonce, generation, canonical CountReport length+raw bytes,
optional canonical CountRun length+raw bytes (zero length means absent), then sanitized display
length+bytes. JS downloads raw report bytes unchanged; it renders only the safe display DTO with
numeric text. Errors have a closed numeric code plus bounded safe category/location text, never
raw target/factory/panic strings. Codes distinguish invalid_frame, incompatible_abi,
invalid_bundle, admission_refused, invalid_handle, selection_not_available, installation_required,
resource_limit, execution_error, internal_failure. Stable code/category are sufficient; do not
leak raw offending values in error locations.

Handle 0 is invalid; other handles are non-reused within a worker. On u32 exhaustion restart the
worker. Requests are serialized. Run's nonce plus selected digest gives the Ids namespace also
supplied to the installation; reject nonce reuse in the same worker. Fresh worker/page uses a
fresh cryptographic nonce. Reset changes presentation generation, never actual target state.
Explicit cancellation terminates the worker and yields aborted/cleanup-unconfirmed, no CountReport.
Worker watchdog is the same terminal distinction. No unsupported async target protocol is implied.

ABI minor 0 and product semantic revision 1 are exact matches between generated host and runtime
library constants. Acceptance builds use the exact reviewed workspace. Source/lock/module hashes
in a consumer build receipt document local provenance; they are not proof a malicious consumer
compiled that source. No guessed Cargo version or manually asserted source hash substitutes for
using the tested concrete runtime library. Evolving host semantics requires a coordinated ABI/
semantic revision and compatibility tests, not only retaining a numeric package version.

## 4. Finite browser resource profile

These product budgets are enforced at their exact values: each exact bound is admitted and one
byte or one item more is a `resource_limit` (`resource_profile_admits_each_exact_bound_and_refuses_one_over`,
`display_node_budget_admits_its_exact_bound_and_refuses_one_over`,
`payload_writers_fill_their_frame_exactly_and_refuse_one_byte_more`). In Firefox 156 the emitted
module reserves exactly 64 MiB and refuses one byte more, grows to exactly 8,192 pages and no
further, and admits and runs a valid Load frame of exactly 64 MiB with a measured high-water mark
of 3,095 pages (about 194 MiB); an ordinary small bundle peaks at 32 pages
(`wasm_frame_memory_and_fetch_budgets_are_enforced_and_measured_in_firefox`). The 300-second
watchdog ends a non-returning run as aborted/cleanup-unconfirmed
(`adversary_watchdog_timeout_ends_cleanup_unconfirmed_without_report_or_disclosure`). Adjust
coherently if later measurements show the profile cannot process its own advertised bounds.

| Bound | Value | Rationale / failure |
| --- | --- | --- |
| Entire Load frame | 64 MiB | Bounds original archives, escaped lineage carrier, and presentation together; checked before reserve. |
| Manifest | 256 KiB | Bounded closed metadata independently of large original documents. |
| Source documents | 1,024, within total byte bound | Supports multifile models without unbounded filesystem-style inventory. |
| Blob path / scenario id control string | 1,024 / 4,096 UTF-8 bytes | Limits control allocations; source semantics retain existing name validation. |
| Original parents | No additional count cap | Full lineage is mandatory; bounded by frame bytes and existing Rust admission. No dropping oldest parents. |
| Presentation document | 32 MiB, 1,000,000 display nodes, depth 1,024 | Bounds rendering work, includes the full declaration once rather than quadratic world snapshots. |
| One ABI response | 64 MiB | Includes exact reports plus sanitized display, with checked cumulative accounting. |
| Linear memory maximum | 512 MiB (8,192 WASM pages) | Leaves room above wire size for parsing, compiler IR, runner and outputs; not a proven worst-case multiplier. |
| Live capabilities/workers | One Loaded object and one worker per page | Select atomically replaces the capability; no unbounded retained modules. |
| Rendered DOM | 2,000 visible tree nodes, 200 cards per page | Pagination/virtualization preserves accessible full data; never semantic truncation. |
| Safe error text | 512 UTF-8 bytes | Closed categories/locations only; no raw errors. |
| Worker wall watchdog | 300 seconds per Run | A broken clock/host must terminate; normal Runner timing remains authoritative within it. |

Streaming fetch counts actual bytes and stops above limits, even if Content-Length lies; do not
first call unbounded arrayBuffer and only then check. Stream chunks are concatenated within the
stated cumulative budget before transfer. Rust independently checks again. JSON display parsing
is size/depth bounded by the generated profile; Rust validates the full regenerated representation
before runtime use. Native suite admission depth/type bounds remain unchanged; browser limits
must not redefine source value types. Over-budget emission/load is a visible `resource_limit`,
not a feature-specific refusal, omitted scenario, successful empty report or weaker replay.

If runtime output exceeds its budget after effects occurred, report a terminal product error
with cleanup state, never truncate or manufacture a complete run. Use the maximum-memory linker
setting plus runtime response bounds; allocation trap is an execution error/aborted worker, not
a conformance assertion failure. Measure bytes/high-water marks and retain evidence. This profile
supports every admitted feature within the same finite envelope; it is not license to exclude
responses, fixtures, callers, clocks, aggregates or protected windows.

## 5. Initial packaging contract: explicit consumer manifest, no helper

Initial CLI routes remain source-built `ess verify conform web`, ordinary and coverage. No new
suite-import CLI flags, xtask build action, CLI build helper, target transport or prebuilt reader
binary is included. Rust Load accepts equivalent complete original-byte bundles through the
above API; that does not promise an upload workflow or new CLI import route.

Default bundle contains `index.html`, `player.js`, `worker.js`, vendored existing Vue+license,
`browser.json`, `declarations.json`, `sources/...`, exact `suite.json` or `input.json`,
`rust/browser_host.rs`, `rust/Cargo.toml.example`, `rust/lib.rs.example`, and `README.md`.
There is no runner.wasm until the consumer builds it. Files remain in the existing owned-artifact
writer contract. Consumer installation source and Cargo.lock are outside generated owned paths.

Document/test a consumer-owned cdylib manifest with edition/rust-version supported by the actual
runtime, `[lib] crate-type = ["cdylib"]`, an actual path dependency on the complete local runtime
workspace's `crates/verify/ess-conformance`, and a path dependency on the independent installation
crate. A standalone host may use its own `[workspace]` to avoid accidental ancestor membership.
The consumer lib includes the **exact emitted** browser_host.rs and invokes its generated
installation macro with a concrete Installation type. The macro only binds the ABI exports to
Product<Installation>; it cannot construct Target during registration. Examples use explicit
clearly marked paths to be filled by the consumer, no home path/git HEAD/crates.io guess.

The documented commands are explicit Cargo operations: generate the host's lock offline once,
retain/review that lock; `cargo build --manifest-path <host>/Cargo.toml --locked --offline
--target wasm32-unknown-unknown --release`; copy the resulting consumer cdylib to `runner.wasm`
beside index.html. Document the WASM maximum-memory linker argument for the profile and record
rustc/cargo versions, lock hash, exact emitted module hash, runtime/installation revision or
content digest, WASM hash and command exit in the build evidence. Acceptance invokes these same
commands from Rust test orchestration. Missing target/dependencies or lock drift is a build
failure with corrective instructions; no network fetching, stub binary or library-only stand-in.
Static serving can use the consumer's existing HTTP server; acceptance uses existing Rust Server.

Without runner.wasm the visible navigation contract in section 2 already works. On an explicit
Connect runtime action, a worker loads WASM as raw bytes, checks ABI, passes originals through
Load, and only then offers Run. A missing module is build_required; invalid ABI/admission is a
visible connection refusal with factory count zero. A connected module lacking installation is
installation_required. The supported initial example installs a real concrete target; no fake
reader target or Interpreted default is generated. A compatible installation supplies Target and
Clock together, preserving Runner wall/budget-time alignment and namespace isolation.

## 6. Explicit independent installation

The generated module is included by a consumer-owned cdylib and binds a concrete installation:

```rust
pub trait Installation {
    type Target: ConformanceTarget;
    type Clock: Clock;
    fn create(context: &RunContext)
        -> Result<Installed<Self::Target, Self::Clock>, InstallError>;
}
pub struct Installed<T, C> {
    pub target: T,
    pub clock: C,
    pub config: RunnerConfig,
}
```

These are the product API's type relationships; implementation must preserve their authority
boundary. `Product<I>` admits independently, then invokes I::create only from Run after selection
and admission. Execution is `Runner::new(config, clock, Ids::seeded(namespace))` followed by
`run_admitted(admitted, &target)`. The existing runner's target parameter is Sized. Do not pass
`Box<dyn ConformanceTarget>` or refactor Runner to evade that constraint.

The factory receives namespace and installation configuration, never expected rows, suite DTOs,
response values or an implementation oracle. It supplies its own implementation, state and
matching clock. Do not generate an Interpreted/sample/fake-success default. An explicitly chosen
reference interpreter must be labeled model execution, not independent consumer conformance.
The initial example installs a real concrete target. There is no mandatory reader-only WASM
build: unbuilt navigation is the Rust-emitted presentation described above.

There must be no factory/target construction in static initializers, module instantiation or
registration. Tests instrument construction and every method. Consumer-linked Rust is trusted
code: the page does not claim to prevent malicious consumer module initializers from violating
the installation contract. Missing installation produces installation_required, not a scenario
pass or unsupported conformance result.

The installed target uses the entire admitted synchronous ConformanceTarget interface, including
responses/events/errors, fixtures, setup, grants/caller, views, periodic controls, scans/readings,
clocks and replay. No feature dispatch table or async-JS transport is added. Missing installed
capability keeps its genuine Runner non-success; it cannot satisfy that feature's support proof.
Acceptance implementations must independently implement every capability their vectors exercise.
The fixture implementation is a real Rust crate shared by native tests and the generated-host
consumer; it is not copied into the generated module or fed expected suite results.

## 7. Worker, clock, namespace and cancellation

Execution runs in a dedicated Web Worker so navigation stays responsive during synchronous
Runner activity. Transfer bounded raw byte arrays and instantiate with
`WebAssembly.instantiate(arrayBuffer, imports)`. Do not require instantiateStreaming or invent
an asynchronous target transport. Initial packaging uses the existing Rust test HTTP server;
array-buffer instantiation avoids relying on an unprovided WASM MIME arm.

Installation creates Target and Clock together. A deterministic target shares a logical clock
with Runner; Clock::wall and implementation time predicates refer to the same epoch/control.
Real-time synchronous installation must provide progressing execution-budget time and matching
wall time explicitly. Never silently use AdvancingClock's historical epoch against a target
reading current wall time. Existing now_offset resolution remains Rust-owned and is reused for
subsequent references. Periodic/clock/coordinate authority comes from the actual target interface,
not JS Date.now. Any necessary wall/monotonic imports are explicitly supplied by installation
with checked exact integer units, such as split u32 words. The first acceptance implementation
uses a Rust-controlled aligned clock and requires no such imports.

Each Run has a fresh cryptographic 128-bit nonce, worker/presentation generation and increasing
request ID. Rust validates these, derives bounded ASCII namespace from nonce plus admitted digest,
and supplies the same namespace to Ids and RunContext. Fixture provisioning, setup and scenario
correlations share it. Do not reuse Ids::for_suite across runs. Installation owns namespace
isolation; successful begin_scenario is an obligation, not proof of isolation. Test repeat,
reload and parallel-page uniqueness, including fixture provisioning before begin_scenario.

Reset/navigation Select invalidates the UI presentation generation. Execution selection is a
separate serialized ABI operation; it cannot mutate an active run's capability. A stale completion
may be retained under its original run identity but cannot overwrite current navigation/selection.
Let runs finish so end_scenario executes when possible. Forced worker termination/watchdog means
aborted/cleanup-unconfirmed, never rolled-back, passed or completed. Installation owns external
cleanup leases; the page does not promise cancellation reverses effects. A new run uses a fresh
namespace and worker state. Never manufacture a partial CountReport after termination.

## 8. Execution receipts and one-time disclosure

Only `Runner::run_admitted` may mint the ExecutedRun used here. Produce CountReport with
`CountReport::from_run(&run, same_admitted)` and optional CountRun with its corresponding method.
Retain exact canonical report bytes for download and derive separate sanitized display data.
JS does not aggregate statuses, fill missing checks, relabel producer_profile, recompute coverage
or parse report payload numbers for display. Exact numeric values render as decimal text.
Zero-scenario/partial/refused inventory keeps its actual CountReport conformance status.
The product display/receipt context binds source/input/selected digests, namespace, installation/
build identity, generation and complete/aborted state; it does not masquerade as a new report/2.

One-time observations remain inside target/runner Rust execution. Use Runner's protected
diagnostics and target-identity masking. Never stringify SemanticCommandResult, Run internals,
target/factory errors, memory buffers or raw panic text into any response. Admission/factory/
module failures expose bounded stable codes and safe category/location text, not raw exceptions
or offending values. Presentation contains declaration policy/field names and sanitized results;
it must not receive actual protected values, hashes, derived markers or unfiltered identity.

No protected value may reach localStorage, console, downloads, progress messages, network-visible
diagnostics, DOM/world panels or declaration snapshots. Scan every output channel with a fresh
sentinel in healthy and faulty one-time browser cases. Preserve report check/status codes and
already sanitized diagnostics; defense-in-depth export checks must never turn a failure green
or rewrite coverage. Runtime errors and resource aborts are product failures, not assertion
mismatches or unsupported-feature passes.

Drop target/run state after each run; where module memory is retained for Loaded state, do not
retain raw observations or reuse them in later results. Replacing a module additionally limits
accidental memory reuse. Neither dropping allocations nor replacing modules guarantees secure
erasure or secrecy from hostile same-origin scripts/devtools. The boundary is intentional
product disclosure, not isolation from the browser owner.

## 9. Compatibility and source scope

Historical `ess-conformance-replay/1` remains an explicit legacy reader/emitter with unchanged
closed reduced-model schema, bytes and refusals. Preserve its existing declaration-replay tests;
new default bundles do not enter it as an admission gateway. The new product's declaration-only
cards make their different navigation contract explicit rather than pretending to simulate the
old world panel. Neither product's navigation is implementation execution evidence.

The `--history` branch stays the existing history product, with no target execution attached.
Existing --suite-format spelling continues to select ordinary/coverage production; display the
actual emitted suite version instead of asserting the option label is its major. No new suite
or report versions, import CLI flags, prebuilt reader, xtask/helper, transport rewrite or general
deployment service is included. Every new executable implementation/test is Rust, except the
necessary established embedded browser asset glue.

Implementation source scope is the following. New paths are marked explicitly; neighboring
regression files only change where their dispatch expectations require it.

- `crates/verify/ess-conformance/src/web.rs`: new default product emitter with retained source
  input; keep legacy emission available under explicit legacy functions for historical callers.
- `crates/verify/ess-conformance/src/lib.rs`: export the new product API.
- New `crates/verify/ess-conformance/src/web_execution.rs`: public Installation/Product API,
  typed product errors, same Runner/CountReport path.
- New `crates/verify/ess-conformance/src/web_execution/{bundle,presentation,abi,host}.rs`:
  closed archive/source admission, shared lossless presenter, checked linear-memory protocol,
  exact generated Rust host/example text. No compiler/domain IR constructor changes expected.
- New `crates/verify/ess-conformance/assets/browser-{index.html,player.js,worker.js}`:
  rendering/static integrity/worker glue only. Legacy `index.html`, `player.js`,
  `coverage-player.js`, `coverage-admission.js`, and `src/web_replay.rs` remain byte/behavior
  compatible; new default emitter must not route through their reduced admission.
- `crates/edge/ess-cli/src/{load.rs,main.rs,coverage.rs}`: retain exact acquired sources once and
  pass them to ordinary/coverage default emission. Preserve history dispatch and option names.
- New `crates/verify/ess-conformance/tests/browser_product_admission.rs` and
  `browser_product_presentation.rs`: full-input rejection/callback counters, ABI framing/budgets,
  exact values, presentation determinism and disclosure. ABI tests may live here through safe
  exposed test seams or internal module tests, no unsafe public test-only admission bypass.
- New `crates/edge/ess-cli/tests/browser_response_conformance.rs`: real CLI/emitted module build,
  actual Firefox/BiDi, independent healthy/fault implementation, ordinary and coverage products.
- New standalone fixture package `crates/edge/ess-cli/tests/fixtures/browser-target/{Cargo.toml,
  src/lib.rs}`: independent Rust implementation and shared aligned Clock, usable from native
  tests and generated host. Consumer test manifest/lock/build artifacts are temporary outputs,
  not a second checked-in guessed runtime dependency graph. Scope fixture Cargo metadata/lock
  according to existing repository dependency rules when adopted.
- `crates/edge/ess-cli/tests/support/browser.rs`: only needed product readiness/receipt helpers.
- `crates/edge/ess-cli/tests/{replay_fidelity_browser,coverage_browser,one_time_browser,
  browser_startup_refusal_boundary,browser_startup_slow_serve_boundary}.rs`: retain historical
  route assertions; migrate default-product expectations explicitly, keep declaration accuracy.
- `crates/edge/ess-cli/tests/{conform_web_history,conform_web_history_adversary}.rs`: unchanged
  regression execution expected; edits only if the test harness needs explicit product dispatch.
- New `docs/design/browser-conformance-product.md`; amend `docs/design/review-replay-subset.md`,
  `docs/design/typed-response-outcome-payloads.md`, `docs/design/review-format-catalog.md` and the
  emitted README template. Planning and changelog remain integration-owned.

No ess-synth generated application host, transport code, compiler semantics, new suite major,
Runner trait-object refactor, xtask, CI workflow or general browser deployment tooling is scoped.
If an actual compiler-to-WASM dependency cannot build, record the specific blocker and scope its
narrow correction; do not silently replace the implementation with JS semantics.

## 10. Required independent validation

Preserve the real ordinary integer-display and coverage unsupported-step reds as regression
assertions, then prove the emitted default product green in actual Firefox/BiDi. Record actual
CLI exits/artifacts, explicit consumer Cargo commands, toolchains, source/lock/module/WASM hashes,
HTTP requests, BiDi receipts, DOM states and canonical Rust reports. Baseline emission succeeds;
its defects are subsequent display and coverage admission, not a claimed CLI refusal.

Before building WASM, both default products must navigate all declarations, exact large integers
and complete lineage; show build_required; make zero target calls; and reject stale/corrupted
original or presentation bytes. This is static consistency and emission-admitted display proof,
not browser Rust admission. After building, exercise the **exact emitted browser_host.rs** with
a real independent target crate. A hand-written equivalent host or copied target implementation
does not prove packaging. Node-only, native-only or WASM-without-Firefox tests cannot close the
product story.

Every matrix row needs actual native and emitted-browser execution where applicable, with
relevant independent fault controls. Existing Go/TS vectors supplement those comparisons; they
do not substitute for the browser product.

| Family | Required evidence |
| --- | --- |
| Responses | Healthy, wrong-response, wrong-event, missing-response, wrong-type, extra-field, stale-result and wrong-outcome controls; direct, observed and nested mappings; optional absent/null; list order/duplicates; nominal values and exact i64 values above 2^53. Compare scenario/check codes with native runs. |
| Protected policies and creation identity | One-time return/capture/replay/leakage; distinct repeated identities; stale cross-scenario results; response-owned identities and output sentinel scans. |
| Original bytes and lineage | At least three lineage levels, narrowed child, retained outside/refusal counts; changed numeric token, duplicate key, unknown member, malformed UTF-8, missing/forged parent, changed scenario body, source/digest mismatch. Factory and every method remain zero on refusal, including after a prior healthy Load/Run. |
| Ordinary/coverage distinction | Ordinary AdmittedSuite positive; coverage-input positive; ordinary wrapped as coverage negative; filtered coverage lacking parents negative. Ordinary focus never changes executed subset; downloads/reports bind exact executed bytes. |
| Full declaration fidelity | Preserves/deletes/sets/affects/aggregates/periodic/caller/fixtures/one-time/nested declarations. Load and navigation invoke no target methods; all cards remain accessible without model interpretation in JS. |
| Scheduling and selection | Navigation while worker runs; stale completion cannot overwrite a new focus/selection; serialized coverage selection, failed Load invalidates prior capability, abort/cleanup semantics. |
| Isolation and clocks | Repeated/parallel runs use fresh namespaces and real begin/end isolation. Fixtures occur only after admission in the same namespace. Eventual timeout, current-time guards, now_offset, periodic controls and aligned wall/budget clocks execute independently. |
| Every other admitted Runner capability | Setup, grants/caller, no-input/no-op, stored/related/set effects, views, bindings/retries/context, scans, clocks/readings, periodic and replay; each existing focused fixture executes through this product with a relevant fault. Keep an explicit capability inventory. Unsupported results cannot count as implemented support. |
| ABI, paths and resources | Unknown versions/tags/fields, duplicate labels, invalid paths, framing overflow/trailing bytes, invalid/reused handles/nonces, buffer lifetime misuse, fetch lies, byte/depth/count boundaries, memory/output exhaustion and watchdog behavior. Measure peak usage before validating proposed budgets; no truncation or fabricated report. |
| Disclosure | Fresh protected sentinel absent from DOM, worker messages, console, network diagnostics and downloads for healthy and faulty targets, factory failure, trap and abort; safe check/status codes retained. |
| Legacy and neighboring products | Retained replay/1, replay_fidelity_browser, coverage_browser, one_time_browser, conform_web_history/adversary and browser startup boundaries retain meaningful feature assertions. Never replace fidelity checks with only “page loaded.” |

Exhaustive typed presentation/emitter checks catch newly added declarations. A common Runner
makes new admitted steps executable through the same path but does not itself prove independent
feature parity. Completion requires every admitted feature within the same bounded product,
with healthy/fault evidence; no default response refusal or narrower special-case demo remains.
