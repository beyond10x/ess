# Parameterized event channel addresses

Status: proposed for beyond10x/ess#391 (2026-10-03), revision 2 after independent design
review. Design source baseline: `168521d870e23f75d29fc5df971e58e4202db30c`.

## Problem

A producer can publish an event whose required payload says which service and environment produced
it, but `ess-transport/1` can bind that event only to one literal NATS subject. It cannot state that
the concrete subject `usage.<service>.<environment>` takes its two tokens from
`event.source.service` and `event.source.environment`, or require those values to be one legal NATS
subject token.

Writing `subject: usage.{service}.{environment}` in `/1` does not express that relation. The 0.52.0
CLI accepts it, carries the braces unchanged into `ess-transport-ir/1`, emits no AsyncAPI
`parameters`, and generates that exact string as the Rust and Go subject constant. Retained
brand-free evidence is in
the retained transport design probe evidence recorded in story:feature-request-391.

This is a transport fact. The event remains the same fact when another broker carries it under a
different address, so the source ESS model does not gain an address expression.

## Decision

`ess-transport/2` adds `parameters` beside a channel's `subject`. The subject uses AsyncAPI channel
address expressions; each parameter maps to a required String field or bounded required-struct path
in that channel's event.

```yaml
type: ess-transport/2
specification:
  system: routing
  version: v1
  source_digest: sha256:<hex>
brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: routing.events.UsageRecorded
    broker: events
    subject: 'usage.{service}.{environment}'
    parameters:
      service: event.source.service
      environment: event.source.environment
    envelope: array
    delivery: at_most_once
    batch: {max_items: 100, max_delay_ms: 5000}
streams:
  - name: USAGE
    broker: events
    subjects: ['usage.>']
    storage: file
    retention: limits
    owner: external
```

`subject` remains a string because it is still the NATS subject syntax. `parameters` uses the same
target-to-source map shape and `event.` source spelling as a binding's `mapping:`. There is no
second nested template object and no new expression language.

### Closed authored grammar

A `/2` subject is either:

- a concrete publishable NATS subject, with `parameters` absent; or
- a template containing at least one address expression and a nonempty `parameters` map.

The subject is split on `.`. Each token is exactly one of:

- a static concrete token: nonempty, no whitespace, `.`, `*`, `>`, `{` or `}`; or
- one whole expression `{name}`. An expression cannot be embedded in static text. Parameter names
  are case-sensitive and match the AsyncAPI Parameters Object key grammar
  `^[A-Za-z0-9_-]+$`.

Every distinct expression name has exactly one `parameters` entry, and every entry is used by the
subject. Repeating the same expression in the subject is allowed and reads the same event value each
time. An empty or unused parameters map is refused. `/2` reserves braces for expressions; literal
braces remain possible only under `/1`, where they retain their released meaning.

Different expression names may map to the same source path. After path resolution, repeated names
and source aliases belong to one source equivalence class. Every occurrence in that class renders
the same token. Authored parameter names identify AsyncAPI address expressions; the resolved
semantic event path supplies identity for equality, stream analysis and generated access.

The only admitted parameter source is:

```text
event.<field>
event.<struct>.<field>
event.<struct>.<struct>.<field>
```

This reuses the binding mapping's existing one-to-three-member bounded event access. Every
intermediate member is a required struct. The terminal member is exactly required primitive
`String`; Optional, List, Map, enum, union, newtype and non-String terminals refuse in `/2`. Paths
use semantic field names. Compilation resolves them against the channel's exact event and retains
the semantic segments in IR.

`context.*` is not admitted. Existing `context.<field>` means the delivery context of an external
event arriving at a consumer binding; it is not producer configuration. ESS currently declares no
typed producer context beside an outgoing event. A future constructor/configuration source needs
its own authority and format review; #391 requires only payload paths.

### Runtime token rule

The value read from each source must be one concrete NATS subject token: nonempty and containing no
`.` character, whitespace, `*`, or `>`. Generated publishers check this before encoding, enqueueing
or calling `Transport`. The failure names the parameter and rule but does not include the rejected
value. It performs no I/O and buffers nothing.

Compilation checks source existence, structure, presence and String type. It cannot prove the value
of an arbitrary String, so runtime token validation is part of the generated publisher contract,
not a fabricated static proof.

### `naming.wire`

A literal `/2` subject retains `/1`'s rule: where the event declares `naming.wire`, the subject must
equal it.

A parameterized subject is refused when the event declares `naming.wire`. `naming.wire` is one
literal wire name; silently reinterpreting it as a template would change the ESS model's released
meaning. Authors of a parameterized transport leave the event wire name absent and let the sibling
transport document own the broker address.

## Compiled form

An admitted `/2` document compiles to canonical `ess-transport-ir/2`. Collections remain ordered as
in `/1`; `parameters` is omitted for a literal channel and ordered by parameter name when present.

```json
{
  "format": "ess-transport-ir/2",
  "specification": {
    "system": "routing",
    "version": "v1",
    "source_digest": "sha256:<hex>"
  },
  "brokers": {
    "events": {"protocol": "nats", "jetstream": true}
  },
  "channels": {
    "routing.events.UsageRecorded": {
      "broker": "events",
      "subject": "usage.{service}.{environment}",
      "parameters": {
        "environment": {"kind": "event_path", "path": ["source", "environment"]},
        "service": {"kind": "event_path", "path": ["source", "service"]}
      },
      "envelope": "array",
      "delivery": "at_most_once",
      "batch": {"max_items": 100, "max_delay_ms": 5000},
      "stream": "USAGE"
    }
  },
  "streams": {
    "USAGE": {
      "broker": "events",
      "subjects": ["usage.>"],
      "storage": "file",
      "retention": "limits",
      "owner": "external"
    }
  }
}
```

`event_path` is the only source kind in IR2. Keeping it tagged prevents a later producer-context
source from being smuggled in as a string with an unstable prefix. Two parameter entries with the
same normalized `event_path.path` are aliases and impose exact-token equality at all of their
subject positions; the compiler and generators key that equality by the resolved semantic path,
not by the authored parameter name. Wire JSON paths and target code accessors are derived from the
pinned `EssIr`, which the AsyncAPI and publisher generators already receive; they are not duplicated
as unchecked strings in transport IR.

## Exact stream coverage

A parameterized channel must be proved safe for **every** legal runtime substitution. Compiling one
sample concrete subject is insufficient.

Let the channel language be its fixed-length token sequence. A static token denotes itself. A
parameter position is a variable keyed by its normalized, resolved semantic event path and ranges
over all legal concrete NATS tokens. Positions with the same key must contain the same exact token,
whether they repeat one expression name or use different names that alias one source. A stream
subject is the existing NATS pattern language: exact tokens, `*` for one token, and terminal `>` for
one or more remaining tokens.

The compiler uses exact structural predicates:

- A stream pattern **covers** the channel language when its fixed prefix covers every channel token:
  an exact stream token equals a static channel token, while `*` covers either a static or variable
  position. Without `>`, lengths are equal. A terminal `>` covers the nonempty remaining suffix.
  Every occurrence of every variable must be covered by `*` or `>`; an exact token at even one
  variable occurrence restricts that equivalence class and therefore cannot cover all assignments.
- A stream pattern **intersects** the channel language by exact-token constraint unification. First,
  its length must equal the channel length, or terminal `>` must have at least one remaining channel
  token. Compare the fixed prefix and, for `>`, treat the remaining positions as wildcards. A static
  channel token conflicts only with a different exact stream token. A wildcard adds no constraint.
  An exact stream token at a variable position binds that variable's resolved-source key; another
  exact token at any occurrence of the same key must be identical. A conflicting binding proves the
  pattern disjoint. If the scan finishes without a conflict, the pattern intersects because the
  remaining variables have a legal assignment.

For example, `usage.{x}.{x}` does not intersect `usage.a.b` when `a != b`, while it intersects
`usage.a.a` and `usage.*.a`. The same result holds for
`usage.{left}.{right}` when both parameters resolve to the same event path. If they resolve to
different paths, `usage.a.b` intersects. This correlation is part of the exact proof and is never
approximated by treating placeholder occurrences independently.

On a JetStream broker, exactly one stream must cover the whole channel language and every other
stream on that broker must be disjoint from it. No covering stream is `ESS-TRANSPORT-015`; a second
covering or partially intersecting stream is `ESS-TRANSPORT-016`. With unrestricted String tokens
and NATS's exact/`*`/`>` patterns, multiple disjoint patterns cannot collectively cover the infinite
parameter domain unless one already covers it, so this rule is equivalent to “every expansion is
captured by exactly one stream.”

Literal channels continue through the existing concrete matching path and retain their diagnostics
and canonical bytes.

## AsyncAPI 3 projection

AsyncAPI 3 channel addresses expressly admit `{name}` expressions, require a `parameters` map for
them, and define `Parameter.location` as a runtime expression. ESS projects each parameter with the
standard location and one closed extension preserving the stronger ESS claim:

```yaml
channels:
  routing.events.UsageRecorded:
    address: usage.{service}.{environment}
    parameters:
      service:
        description: Equals event.source.service and is one concrete NATS subject token.
        location: $message.payload#/0/source/service
        x-ess-source:
          kind: event_path
          path: [source, service]
          scope: every_item
          constraint: nats_subject_token
      environment:
        description: Equals event.source.environment and is one concrete NATS subject token.
        location: $message.payload#/0/source/environment
        x-ess-source:
          kind: event_path
          path: [source, environment]
          scope: every_item
          constraint: nats_subject_token
```

For `envelope: single`, `location` omits `/0` and `scope` is `one_item`. For `array`, the standard
location identifies the first item that supplies the address and `scope: every_item` states ESS's
additional equality requirement: every item in that message has the same value at the source path.
JSON Pointer segments use the fields' wire names and RFC 6901 escaping; `x-ess-source.path` retains
semantic names. Every authored expression remains a separate AsyncAPI parameter entry. Entries with
the same normalized `x-ess-source.path` are aliases and therefore declare the same address token;
no second equality identifier is needed. Literal channels emit no `parameters` and remain
byte-identical under `/1`.

The projection follows the AsyncAPI 3.0 Parameter Object rather than inventing an ESS-only
parameters shape: <https://www.asyncapi.com/docs/reference/specification/v3.0.0#parameterObject>.

## Generated Rust and Go publishers

The public publish method signatures stay payload-based. A dynamic operation emits
`*_SUBJECT_TEMPLATE`, not a constant named `*_SUBJECT`. Before serialization it reads each distinct
compiled event path, validates its token and substitutes that value at every expression occurrence
in the path's equivalence class, rendering one owned concrete subject.

For `envelope: single`, the publisher encodes and sends the payload on that subject.

For `envelope: array`:

- one operation worker loop (a thread in Rust and a goroutine in Go) manages a map from concrete
  subject to buffer; it does not create one thread, goroutine or persistent timer per subject;
- each subject has its own FIFO payload buffer and oldest-item deadline;
- `max_items` and `max_delay_ms` apply independently per concrete subject, and no message mixes
  subjects;
- timer scheduling is derived from the deadlines of the currently nonempty buckets. When a
  max-items or timer drain empties a bucket, the worker removes its map entry before transport I/O
  and cancels or recomputes any wake-up associated with it. The owned `(subject, batch)` being sent
  is not a bucket and retains no subject timer;
- `flush` and `close` drain every subject bucket in lexical subject order, preserving FIFO within
  each subject and returning the first failure by that deterministic order. They clear each drained
  bucket and all associated deadline state before transport I/O; `close` exits with an empty map;
- if a publish for the same subject arrives after its old bucket was removed while that batch is in
  flight, it creates a fresh bucket. The single worker finishes the older owned batch first, so
  per-subject FIFO remains intact. Transport or encoding failure still retires the drained bucket;
  at-most-once behavior does not retain it for retry;
- background failures still reach `on_error` and the failed at-most-once batch is dropped;
- `publish_*_now([])` remains a no-op. A nonempty `_now` call renders every payload first and refuses
  before encoding or I/O if they do not all select the same subject; otherwise it sends one array,
  retaining the existing one-call/one-message promise.

The dynamic generated package uses an owned `String`/`string` in `PublishError.Subject`; invalid
tokens and mixed-subject calls use the template as the subject and name the parameter/rule without
echoing its value. A package generated entirely from `/1` retains its existing Rust
`&'static str`, Go source, constants, report and bytes.

`ess-client-report/2` is selected when any operation is parameterized. It retains the operation's
`subject` template and adds the normalized `parameters` map. Static operations in that report omit
`parameters`. A client with only literal `/1` operations continues to write `ess-client-report/1`.

The generated NATS adapters do not change their seam: they receive one rendered concrete subject.

## Compatibility and version selection

`ess-transport/1`, `ess-transport-ir/1` and `ess-client-report/1` were released in ESS 0.52.0
(`crates/edge/ess-xtask/src/docs.rs:263-265`). Their meaning and canonical bytes are frozen.

- The new reader accepts authored `/1` with its exact old strict shape and compiles it to IR1.
- Authored `/1` with `parameters` remains an unknown-field parse refusal. Braces remain literal in
  `/1`.
- The new reader accepts authored `/2` and compiles it to IR2. `/2` may carry literal channels, but
  braces always mean address expressions there.
- The released 0.52.0 reader refuses the `/2` example before compilation; retained evidence is in
  `probe-evidence.md`.
- IR stays serialize-only. Its `format: ess-transport-ir/2` is the compatibility fence for external
  consumers; no unchecked polymorphic IR deserializer is added.
- No `ess/N`, compiler IR, conformance suite, report, diff or entity-runtime format changes. Commands
  that do not accept a transport document cannot silently observe or ignore these parameters.

## Diagnostics

Existing `ESS-TRANSPORT-001`–`016` keep their families. `011` also refuses a parameterized channel
whose event declares literal `naming.wire`; `015` and `016` use the exact language coverage rules
above. New stable diagnostics are:

| code | path | meaning |
|---|---|---|
| `ESS-TRANSPORT-017` | `channels[i].subject` | malformed address expression, expression embedded in a token, invalid parameter name, or brace in a `/2` static token |
| `ESS-TRANSPORT-018` | `channels[i].parameters[.<name>]` | missing, unused or empty parameter mapping |
| `ESS-TRANSPORT-019` | `channels[i].parameters.<name>` | source is not an admitted `event.` path, has invalid depth, or names no field through required structs |
| `ESS-TRANSPORT-020` | `channels[i].parameters.<name>` | terminal source is not required primitive String, or an intermediate member is not a required struct |

Runtime invalid-token and mixed-subject errors are generated `PublishError`s, not transport compile
diagnostics.

## Acceptance

1. `/1` fixture bytes, IR1, AsyncAPI and generated Rust/Go publisher bytes are unchanged; a `/1`
   apparent template remains literal. The 0.52.0 binary refuses `/2`, and the new reader refuses
   `/2` fields under `/1` and accepts both valid versions.
2. Parsing/admission covers unmatched braces, embedded expressions, invalid expression spelling,
   repeated use of one expression with one mapping, missing/extra/empty mappings, `context.*`, path
   depth, unknown fields, Optional and non-struct intermediates, and every non-required-String
   terminal.
3. IR2 is deterministic and normalized; semantic paths survive field wire/code renames without
   changing authored syntax. One-change invalid documents report the exact new code and path.
4. Stream tests exhaust structural cover/intersection classes: exact static prefix, `*`, terminal
   `>`, short/long patterns, no cover, two full covers and one full cover plus a partial overlap.
   Correlation vectors prove that `usage.{x}.{x}` is disjoint from `usage.a.b` for distinct `a` and
   `b`, intersects `usage.a.a` and `usage.*.a`, and is fully covered only when every variable
   occurrence is wildcard-covered. The same vectors run with two expression names mapped to one
   resolved source path, plus a control where the names map to independent paths.
5. AsyncAPI validates as 3.0 and contains every address parameter, correct single/array runtime
   locations, semantic and wire-renamed paths, every-item equality and NATS-token metadata. Literal
   `/1` output is unchanged. Differently named parameters mapped to one semantic source emit the
   same normalized `x-ess-source.path` and wire location, making their exact-token alias explicit.
6. Generated Rust and Go fake transports prove concrete rendering, repeated/source-aliased
   substitution, invalid-token refusal before encode/enqueue/I/O, field code renames, two-subject
   isolation, per-subject max-items and timers, deterministic flush/close, same-subject `_now`,
   mixed `_now` refusal, closed publisher behavior, encoder failures and transport failures. After
   each max-items, timer, explicit-flush, close and failed-send drain, empty buckets and their
   deadline state are absent. A long sequence of unique subjects, each followed by `flush`, leaves
   zero bucket keys and zero subject timer state rather than growing with historical subjects; a
   same-subject enqueue racing an in-flight drain preserves FIFO and eventually retires both buckets.
7. An actual NATS JetStream test creates an externally owned stream bound to `usage.>`, then runs
   each generated adapter. For each language, 150 `service-a/eu` and 150 `service-b/us` payloads
   yield separate concrete subjects and arrays of at most 100 whose every item matches its subject;
   invalid tokens and mixed `_now` perform no publish. No generated code creates, updates or deletes
   the stream. The committed harness is Rust.
8. `task check`, focused strict lint, generated-output drift, format/reference checks and independent
   review pass on the final held bundle head.

## Out of scope

- Producer configuration or typed context parameters.
- Consumer subscription templates, durable consumers, acknowledgements or queue groups.
- Optional/defaulted parameter values, enum/newtype parameters, transforms or literals in the
  parameters map.
- Embedded expressions such as `usage-{service}`.
- Retry support or `delivery: at_least_once` publishers.
- Binding causal-completion authority for aggregate observations. Address rendering does not prove
  completion of transitive binding effects.
