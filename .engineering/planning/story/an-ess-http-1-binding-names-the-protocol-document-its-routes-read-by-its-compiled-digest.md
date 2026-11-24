---
format: aep.planning-md/3
id: story:an-ess-http-1-binding-names-the-protocol-document-its-routes-read-by-its-compiled-digest
kind: story
status: draft
title: An ess-http/1 binding names the protocol document its routes read, by its compiled digest
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:an-ess-protospec-1-message-declares-header-fields-absent-fields-fixed-values-and-an-application-body
revision: 1
---
## Need

Under the owner's note of 2026-11-24 an HTTP route that streams server-sent events, or that reads a
request header, takes its frame and header structure from an `ess-protospec/1` document. An
`ess-http/1` binding cannot name one.

- The binding's top-level keys are `type`, `specification`, `refines`, `refusals`, `servers` and
  `routes` (the refusal quoted in `story:an-http-binding-declares-how-a-caller-authenticates`).
- `refines:` names another `ess-http/1` binding by digest, with the file passed as `--refines`
  beside `--http` (`crates/edge/ess-cli/src/main.rs:296-299`, `:339-342`; `ui.rs:55-66`;
  `docs/design/http-presentation-binding.md:98-125`). It is the precedent for this link, and it is
  not a protocol document.
- `ess-protospec/1` is consumed by no projection (`crates/specify/ess-compiler/src/protocol.rs:4-5`).

Minimal reproduction (written fresh, not run):

```yaml
type: ess-http/1
specification: {system: demo, version: v1, source_digest: "sha256:<hex>"}
protocol: "sha256:<the digest `ess specify protocol validate --path p.yaml` prints>"
routes:
  - {command: demo.orders.PlaceOrder, method: post, path: /orders}
```

```console
$ ess specify http validate --path spec --binding b.http.yaml --protocol p.yaml
error: unexpected argument '--protocol' found
$ ess specify http validate --path spec --binding b.http.yaml
error: reading b.http.yaml as HTTP binding YAML: unknown field `protocol`, expected one of `type`, `specification`, `refines`, `refusals`, `servers`, `routes` …
```

## Outcome

- **The key.** `protocol: sha256:<hex>` is optional at the top level of `ess-http/1`. It is the
  compiled digest of an `ess-protospec/1` document (`CompiledProtocol::digest`), which is
  insensitive to formatting and is what a protocol trace pins.
- **The flag.** Every command that takes `--http` and `--refines` takes `--protocol <file>`:
  `ess specify http validate` and `compile`, `ess generate --kind openapi --http`,
  `ess generate synthesize --http`, and `ess ui … --http`. The change lists each one, read from
  `main.rs` and `ui.rs`. The file is compiled by `ess_compiler::protocol::parse_and_compile`, and
  its digest must equal the key.
- **The IR.** `ess-http-ir/1` records `protocol` after `refines` and omits it when absent. A binding
  without the key keeps its IR bytes and its OpenAPI bytes.
- **Use.** The key maps nothing by itself. Routes read protocol messages through their own keys:
  `stream:` (the SSE binding story) and `request.message` (the header-input story). Each of those
  stories refuses a protocol message it cannot use.
- **Refinement.** A refining binding names the same protocol digest as the binding it refines, or
  none when that one names none.
- `docs/design/http-presentation-binding.md` gains § "A protocol document": the link, the digest,
  the flag, and that header and frame structure are read only through route keys. It also states
  the split under PR-D6 A: request headers live in the protocol document, response headers in the
  binding.

## Acceptance

- **Red first.** The reproduction validates (exit 0) and compiles with `protocol` in
  `ess-http-ir/1`. Today the flag is an unexpected argument and the key an unknown field. The cases
  are in `crates/specify/ess-http/tests/compile.rs` and `crates/edge/ess-cli/tests/http_presentation.rs`
  (existing binaries).
- **Refusals.** Each is refused with one new code (the next free `ESS-HTTP-0NN` when the unit lands;
  `034` is taken by `limits`), naming the key or the flag:
  - `protocol:` without `--protocol`, with a message that names the digest;
  - `--protocol` without `protocol:`;
  - a file whose compiled digest differs, naming both digests;
  - a file `ess specify protocol validate` refuses, with the protocol diagnostics passed through;
  - a refining binding whose protocol digest differs from the refined binding's.
- **Old bindings.** Every existing `ess-http` fixture compiles to the same IR bytes, and
  `openapi_http_presentation.rs` keeps its bytes.
- Each test runs within 60 s, and each test step within 120 s.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (binding struct and IR envelope, `RouteSpec` 1561-1610 untouched)
- cited: `crates/edge/ess-cli/src/main.rs` (296-299, 339-342, 1802-1858, 3547-3640, 3755), `crates/edge/ess-cli/src/http.rs`,
  `crates/edge/ess-cli/src/ui.rs` (55-66)
- cited: `crates/specify/ess-http/tests/compile.rs`, `crates/edge/ess-cli/tests/http_presentation.rs`
- cited: `docs/design/http-presentation-binding.md`, `website/docs/reference/formats.md`,
  `website/docs/reference/cli.md` (regenerated), `CHANGELOG.md`
- Size: S, about 300 changed lines.

## Gate

- `task fmt-check`
- `cargo clippy -p ess-http -p ess-cli --all-targets --locked -- -D warnings`
- `cargo test -p ess-http --locked --test compile`
- `cargo test -p ess-cli --locked --test http_presentation`
- `cargo test -p ess-cli --locked --test generate_ui_http` (with `ESS_UI_HTTP_SHELF_SERVER` and
  `ESS_UI_HTTP_ITEMS_SERVER` set after `task ui-http-servers`)
- `task ci-lint` (public API and CLI)
- `task test-xtask` (diagnostic code, `CHANGELOG.md`, `docs/`)

Each test runs within 60 s, and each test step within 120 s. A build-pool job ends within 10 minutes
unless it names its time with `slot --timeout` (at most 30 minutes).

## Depends on

- `pr0-protocol-message-structure`, for `CompiledProtocol::message`. The digest alone exists today,
  so this unit can land right after it.
- PR-D1 and PR-D2 (architecture).
- It shares `crates/specify/ess-http/src/lib.rs` with `story:an-http-binding-declares-how-a-caller-authenticates`
  (active) and the member-body story `sl01-member-body`, so these merge one after another.

## Fit review

1. **Need.** A route cannot take frame or header structure from a protocol document, because the
   binding cannot name one. Reproduction above.
2. **Class: gap.** This is the link the owner's note requires.
3. **Already expressible?** No.
4. **Fit.** It follows the precedent of `refines:` (a digest in the document, the file beside the
   flag), and the precedent of `ess-http/1` keys that are absent when undeclared (stage 2, operation
   ids, `servers`, `limits`): no format version, and an older reader refuses the key as unknown.
5. **Second adopter.** A webhook API whose deliveries carry a signature header that a protocol
   document names.
6. **Cost.** One optional key, one CLI flag on the commands that read `--http`, one code.
7. **Considered.**
   - (a) A path to the protocol file inside the binding. That ties a published binding to a file
     layout.
   - (b) A digest over the file's bytes, as `refines:` uses. That breaks on reformatting, and a
     protocol trace pins the compiled digest.
   - Chosen: the compiled digest.

## Decisions

- PR-D1 and PR-D2 (architecture).
- PR-D6 (default A) is recorded on the design page.
