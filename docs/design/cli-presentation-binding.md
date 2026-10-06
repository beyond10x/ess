# CLI presentation binding (`ess-cli/1`)

This additive document presents typed callable interfaces as a binary. It does not
change `ess/1`, component ownership, legacy CLI synthesis, or composition bytes.
The selected compiled ESS model owns every input, result and error type. A local
action is presentation metadata with an explicit owner and action, not an entity.

## Closed input and complete example

All objects reject unknown fields, and `format` must be exactly `ess-cli/1`.
The following two files form a complete model and binding. The model needs no
component or invented lifecycle because it declares only value types.

```yaml
# system.yaml
format: ess/1
system: demo
version: v1
types:
  - name: demo.StoreInput
    kind: struct
    fields:
      - {name: profile, type: String}
      - {name: secret, type: String}
  - name: demo.Stored
    kind: struct
    fields:
      - {name: profile, type: String}
  - name: demo.Failure
    kind: struct
    fields:
      - {name: reason, type: String}
```

```yaml
# cli.yaml (outside the model directory)
format: ess-cli/1
binary: demo
about: Demonstrate a typed local interface
globals:
  config: config
  state: state-dir
  output: output
callables:
  store:
    target: {kind: local, owner: demo.cli, action: store-credential}
    input: demo.StoreInput
    result: demo.Stored
    errors: {store_failed: demo.Failure}
commands:
  - path: [credential, store]
    aliases: [[credential, set]]
    callable: store
    about: Store a credential through the application handler
    arguments:
      - field: profile
        source: {kind: option, long: profile}
      - field: secret
        source:
          kind: protected
          file: secret-file
          stdin: secret-stdin
          hidden_tty: secret-prompt
```

`globals` is required and declares the long flag names for process context. Config
and state are optional paths; output accepts `human` (default) or `json`. They are
global even after a subcommand and never enter the command payload. Long names,
binary names and command path tokens use lowercase ASCII words separated by `-`.
`help`, `version`, and `completions` are reserved command names; `help` and `version`
are reserved flags. Paths have one or two tokens. Aliases are complete explicit
paths, not protocol aliases or automatically synthesized spelling variants.
The binary name also refuses Cargo's reserved output-directory names: `build`,
`deps`, `examples` and `incremental`, before any package is projected.

Each callable explicitly declares `input`: a reference to a declared struct, or
`null` for an inputless call. Omission is refused. Inputless calls receive `{}` at
the handler seam and need no empty ESS struct. Each callable also has a result type
reference. References use ESS type syntax, including `Optional<T>`, `List<T>` and
`Map<String, T>` for results and nested fields. Errors map stable ASCII codes to
model types; the handler returns one declared code and typed data. There is no
arbitrary extension map. The supported projection includes String, Boolean,
Integer, enums, unconstrained newtypes/structs, Optional, List and string-keyed Map.
Integer values use exact JSON integer tokens within `i64::MIN..=i64::MAX`, including
negative argv values; decimal and exponent spellings are outside this projection's
integer codec. No integer is converted through floating point before dispatch.
Other primitives, unions, recursive types and invariants are explicit refusals in
this version, rather than validation promises the generated adapter cannot keep.

Targets are closed variants:

* `local`: `owner` and `action` identify an application-local handler. `owner` is
  explicit qualified metadata; it does not claim an ESS component exists.
* `service_forward`: `owner` identifies an existing ESS component and `operation`
  identifies an accepted command or a view in an owned domain. Input fields must
  exactly match that command input or view parameters. A view result is its row
  shape, or a list of that shape. The callable's result/error envelope is the
  application's typed adapter contract; transport and command-outcome mapping
  remain an explicit handler obligation. No ownership is transferred.
* `dynamic`: `owner`, `operation_field`, `schema_field`, and `payload_field` identify
  a local dynamic handler and three distinct String fields in its typed outer
  input. Payload is JSON text. The runtime must resolve the supplied schema
  identity for the operation and validate input, result and declared error data.
  The generated adapter fails closed unless that validator is installed.
  `DynamicValidator::validate` returns `Result<(), DynamicError>`: `InvalidValue`
  refuses native input with exit 2 and result/error data with exit 1; `Unavailable`
  refuses unresolved, stale or unavailable schema authority with exit 1;
  `Interrupted` uses exit 130. These finite failures never include native values
  or underlying resolver diagnostics. Payload text that repeats a key in any one
  object is refused before the validator runs, because a last-key-wins reading
  would validate a value other than the text the handler receives.

A callable may declare `invalid_input: <code>`, naming one of its own declared
`errors` as the answer for every invalid input: a typed value that fails its shape
(otherwise `cli_input`), and an unparsable, empty or duplicate-keyed dynamic payload
or one the validator refuses with `InvalidValue` (otherwise `cli_dynamic_input`).
The answer is that code with `{}` data and exit 2, so it never carries input text.
Admission refuses an undeclared code, an error type that does not accept `{}` (a
struct with a required field), an inputless callable, and `cli_parse`, which fails
before a callable is known. Acquisition failures (`cli_source`) and an unavailable
or interrupted validator keep their own codes. The generated reference names the
answer. Without the key, the adapter codes are unchanged.

Each input field has exactly one argument mapping. `option` has `long` and consumes
a value; Boolean values use explicit `true`/`false`. `positional` has a consecutive
one-based `index`. Scalar String/newtype/enum values are text; other values use JSON
syntax. Requiredness follows Optional in the model. `protected` has `file`, `stdin`
and `hidden_tty` long flag names, and supports String fields only. The three sources
are mutually exclusive; exactly one is required for a required field. Protected
values have no argv value channel. At most one field per command can read stdin.
Files/stdin are bounded UTF-8 reads (1 MiB). Hidden TTY uses `rpassword` without echo
and rejects captured values larger than 1 MiB; its capture buffer is not bounded
while typing. Acquisition failures and parser errors never include provided argument
values or source errors. Paths to files are argv metadata, not secret values.

Protected files on Unix must be regular files owned by the effective user, with
no group/other permissions and exactly one hard link. The runtime opens the final
path component without following symlinks, then checks metadata on that same open
file. Parent-directory symlinks are not rejected. Platforms without this admission
implementation refuse protected files. A `document` source is nonsecret String
text and declares mutually exclusive `inline`, `file`, and `stdin` flags, such as
`{kind: document, inline: input-json, file: input-file, stdin: input-stdin}`. Documents
have the same accepted size and UTF-8 limits; their files require regular-file
status but do not use protected-file ownership admission. Document text is not
automatically JSON-decoded unless a dynamic target validates its native payload.
On Unix, document files open nonblocking and regular-file admission checks that
same descriptor, so a FIFO is refused without waiting for a writer. Document
symlinks and hard links to regular files remain admitted. Other platforms retain
the standard file-open behavior followed by regular-file admission; this version
does not claim a nonblocking-open guarantee there.

### Trailing arguments

`{kind: trailing}` has no other fields. It binds one field to every argv word after
the first `--`, verbatim, for a command that hands those words to another program.
The field must be exactly a required `List<String>` (an unconstrained String newtype
counts as String); `Optional<List<String>>` is refused, because absent and empty
would be two spellings of one value. A command has at most one trailing source. It
reserves no flag and does not read stdin. Positionals keep their consecutive-index
rule and are filled only before `--`; the trailing list takes the next index after
them. Words are never JSON-decoded and never parsed as options: `--output json`
after `--` is a value, and so is a second `--`. With no `--`, or nothing after it,
the field is `[]`. A word before `--` that no positional takes is a parse failure
(`cli_parse`, exit 2), as is a word after it that is not UTF-8; neither failure
names the word. Help renders the list as `[-- <field:args>...]`.

```yaml
# launch-system.yaml
format: ess/1
system: demo
version: v1
types:
  - name: demo.LaunchInput
    kind: struct
    fields:
      - {name: connection, type: String}
      - {name: args, type: 'List<String>'}
  - name: demo.Launched
    kind: struct
    fields:
      - {name: status, type: String}
```

```yaml
# launch-cli.yaml
format: ess-cli/1
binary: demo
about: Launch an operator-pinned program
globals: {config: config, state: state-dir, output: output}
callables:
  launch:
    target: {kind: local, owner: demo.cli, action: launch}
    input: demo.LaunchInput
    result: demo.Launched
commands:
  - path: [launch]
    callable: launch
    about: Launch the program with every word after --
    arguments:
      - {field: connection, source: {kind: option, long: connection}}
      - {field: args, source: {kind: trailing}}
```

`demo launch --connection c -- --flag -x v` hands the handler
`{"connection":"c","args":["--flag","-x","v"]}`. `ess-cli/1` and `ess-cli-plan/1`
gained the kind in 0.54.0 without a format change; an older reader refuses it by
name. Every generated package's `src/wire.rs` and `src/runtime.rs` change with it, so
`ess generate cli --check` reports drift until the package is regenerated.

## Runtime contract and process policy

The generator emits a standalone Rust/Clap Cargo package named
`{binary}-cli-contract`, with library `cli_contract` and binary `{binary}`, a
deterministic manifest, resolved `binding.json`, source files, bash completion
text, `help.txt` and a generated `README.md` listing commands, aliases, types and
runtime obligations. `src/lib.rs`
exposes `command()`, `run(args, sources, handler, dynamic_validator)` and typed
`Context`, `Invocation`, `HandlerReply`, `ProcessOutput`, `Sources` and `Handler`
interfaces. The handler receives JSON values only after model-type validation,
alongside the target and separate context. It is the application's implementation
seam. The generated binary installs an unavailable handler and therefore cannot
claim credential persistence, process launching, or provider execution.

The runtime validates the handler's success result and declared error data before
rendering. JSON success is `{"ok":true,"result":...}`; failures are
`{"ok":false,"error":{"code":...,"data":...}}`. Internal failures carry stable
codes and empty data, never a handler-supplied freeform error. Success uses stdout
and exit 0; application/internal failures use stderr and exit 1; argument/source
and input validation failures use stderr and exit 2. A handler may return the
finite `UsageError { code, data }` reply for a typed application usage failure,
such as invalid configuration: it uses exit 2 after the same declared code/data
validation as `Error`, which uses exit 1 for operational failures. Unknown codes
and wrong-shaped data remain internal failures with exit 1. The application owns
the semantic classification and must choose the reply consistent with its modeled
failure kind; the adapter does not infer an exit status from payload fields.
Explicit acquisition/handler
interruption uses exit 130 and stable `cli_interrupted`. Human mode uses compact JSON
for typed values and a concise stable error code. Selected JSON mode also governs
parser failures, including malformed commands; argv is never copied into errors.
Help and completions use stdout and exit 0 in either mode. Secret inputs are never
automatically included in any output or Debug-derived invocation. Typed handler
results remain the application's declared output contract.

## Library integration freeze

`ess_cli_contract::Binding::from_yaml(&str) -> Result<Binding, Error>` is the closed
reader. `ess_cli_contract::compile(&EssIr, &Binding) -> Result<CompiledBinding,
Error>` resolves and validates every used model type and callable. The compiled
binding serializes deterministically through `to_canonical_json()` and reports
runtime obligations. It is read-only to callers and constructible only by compile.

`ess_cli_project::project(&CompiledBinding) -> BTreeMap<String, String>` returns
the entire deterministic artifact map without filesystem mutation. Edge commands
compile first, then compare or write that map. The edge is
`ess specify cli --path MODEL --binding FILE [--format json]` and
`ess generate cli --path MODEL --binding FILE --out DIR [--check]`.

The source and package fixtures compile and execute the emitted crate offline;
recording handlers prove dispatch, context separation, typed input/output/error
checks, secure source selection, dynamic-validator refusal, parser JSON policy,
help, aliases and deterministic bytes. The existing synth/composition suites remain
the evidence for unchanged legacy behavior; this document does not rewrite them.
