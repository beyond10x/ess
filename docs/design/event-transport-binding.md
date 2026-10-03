# Event transport binding

Status: accepted for beyond10x/ess#390 and beyond10x/ess#392 (2026-10-03).

## Problem

An event is a fact; a channel is one way to carry it (`crates/generate/ess-gen/src/asyncapi.rs`,
module doc). The model therefore holds no broker, and the AsyncAPI projection says so: empty
`servers`, an address that "is a name and not a topic", no retention, no envelope.

That is right for a model and wrong for a **message contract**, whose whole substance is the
transport: "publish these items to subject `usage`; the persistent stream `USAGE` captures that
subject for 90 days; one message carries a JSON array of items; producers batch and do not retry;
producers never create the stream". None of it can be stated, so it lives in prose beside the
specification, and a generated client cannot be derived from prose.

## Decision

A sibling document, `ess-transport/1`, binds events of **one exact ESS** to a broker. It is to an
event what `ess-realization/1` is to a component: authored separately, pinned to the
specification's source digest, compiled against the resolved IR into a deterministic
`ess-transport-ir/1`, and never part of `ess/N`. The model does not change when the same event is
carried by another broker; a transport document changes instead.

```yaml
type: ess-transport/1
specification:
  system: metering
  version: v1
  source_digest: sha256:<hex>
brokers:
  - id: events
    protocol: nats
    jetstream: true
    host: nats:4222            # optional
channels:
  - event: metering.items.UsageRecorded
    broker: events
    subject: usage
    envelope: array            # single | array
    delivery: at_most_once     # at_most_once | at_least_once
    batch:                     # envelope: array only; the producer's flush policy
      max_items: 100
      max_delay_ms: 5000
streams:
  - name: USAGE
    broker: events
    subjects: [usage]
    storage: file              # file | memory
    retention: limits          # limits | interest | work_queue
    max_age_seconds: 7776000   # optional
    owner: external            # external | publisher
```

### Vocabulary

| key | meaning |
|---|---|
| `brokers[].protocol` | closed set; `nats` is the only member today. A second protocol is a vocabulary change in this format, not a free string |
| `brokers[].jetstream` | the broker persists subjects into streams; a stream may name only such a broker |
| `brokers[].host` | where the broker listens, when the contract fixes it; absent, AsyncAPI gets a `{host}` server variable |
| `channels[].subject` | the NATS subject one message of this event is published to |
| `channels[].envelope` | `single`: one message is one event payload. `array`: one message is a JSON array of payloads, also for one |
| `channels[].delivery` | the same words bindings use (`ess_domain::binding::Delivery`), here as the publisher's promise |
| `channels[].batch` | how a publisher fills an `array` envelope: flush at `max_items` or after `max_delay_ms`. A producer default, not a wire limit, so it is not `maxItems` |
| `streams[].owner` | `external`: an operator owns the stream and publishers must never create, update or delete it. `publisher`: a publisher may create it when missing |

### What compiling checks

1. `type` is `ess-transport/1`; `specification` names the IR's system, version and source digest.
2. Broker ids and stream names are unique; one channel per event.
3. A channel's event exists and its broker exists.
4. A subject is a publishable NATS subject: non-empty `.`-separated tokens, no whitespace, no `*`
   or `>`. Where the event declares `naming.wire`, the subject equals it, so the model's wire name
   and the transport's subject cannot disagree.
5. `batch` appears only with `envelope: array`; both numbers are at least 1.
6. A stream names a JetStream broker; its subjects are NATS subjects in which `*` is a whole token
   and `>` only the last one.
7. On a JetStream broker every channel's subject is captured by **exactly one** stream. None is the
   failure the server reports as "no stream found for given subject" at publish time; two is a
   configuration NATS refuses. Both are refused here, before anything runs.

Refusals carry stable codes `ESS-TRANSPORT-001` onward and the path of the offending key.

## AsyncAPI

`ess generate --kind asyncapi --transport <file>` projects the same documents, plus:

- `servers`: one per broker a channel of the component uses, `protocol: nats`, the `host` or a
  `{host}` variable, and a description naming JetStream and the streams;
- each bound channel: `address` is the subject, `x-ess-address-source: transport`, `servers`
  referencing its broker, and `x-ess-stream` carrying the capturing stream's name, subjects,
  storage, retention, maximum age and owner;
- each bound message: `envelope: array` makes `payload` an array whose `items` reference the event
  payload schema, with `x-ess-envelope: array`;
- each bound `send` operation: `x-ess-delivery` and `x-ess-batch`.

AsyncAPI 3 has no JetStream binding (its NATS operation binding names only a queue group), so the
stream travels as an extension, as `delivery` and `on_failure` already do. The document's
description no longer says the specification declares no transport; it names what the transport
document states and what it does not (security, message keys, partitioning). Without
`--transport`, output is byte-identical to before.

## Not decided here

- An address template whose tokens bind to payload or context fields (beyond10x/ess#391).
- A consumer-side binding (durable consumers, acknowledgement, queue groups).
- Brokers other than NATS.
- Generated clients, which read `ess-transport-ir/1` (beyond10x/ess#395).
