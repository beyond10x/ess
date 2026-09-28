# Direct library returns

An in-process command can return a value without publishing an event or changing a declared
entity. A returned value is observable directly. Requiring an invented event or a last-result
view would specify behavior the library does not provide.

## Source and scenario authority

`ess/17` adds `returns: true` to an outcome. The command must declare a nonempty typed `response`.
This outcome promises a successful return satisfying that response schema. It makes no claim
about persistence or side effects; those need their own behavioral declarations and subsequent
real API observations. It cannot also declare an error, `accepts: nothing`, or `replays`.
Existing declarations retain their canonical bytes: the new field is omitted when false.

`ess-scenario/4` adds an optional `response` mapping to an act. Its keys are declared response
fields, and each literal is checked against its complete declared type. The map is a partial
set of value assertions; `{}` requests only the complete response shape check. Every selected
`returns: true` outcome receives that shape check even without literal assertions. A declared
optional field may be absent subject to its presence policy; an asserted literal must be present
and equal. Lists retain order and duplicate multiplicity, and nested objects remain closed.

## Execution and compatibility

The standalone `expect_direct_response` step carries the exact command, optional selected
outcome, response field types, reachable finite named declarations, and literal expectations.
Admission checks this authority before any target callbacks. The runner compares only the
immediately preceding invocation's `SemanticCommandResult.response`. The target receives inputs,
never expectations, and supplies actual return values. A failed response is a named failed check,
not an adapter error or a skip. No event, state, view or persistence is manufactured.

The step requires ordinary suite/28 or inventory suite/29. Explicit older pins refuse the new
vocabulary; historical suites keep their format and bytes. Source readers predating ess/17 and
scenario readers predating scenario/4 refuse the new document tags. The Rust runner uses admitted
original bytes and the existing report/2 association. Go and TypeScript producers refuse this
observation until their runners implement it; no unsupported generated suite is handed out.

Binary64 admission is unchanged. Adapters must preserve exact integer values and cannot turn
unsupported numeric/token semantics into weaker response assertions.

Direct returns have a separate resource profile from observed event selection: at most 1 MiB
of serialized response bytes, a nesting depth of 128, and 65,536 members per collection.
Long strings and large ordered collections are checked completely within these bounds; no
payload is truncated. The schema and authored expectation together also have a 1 MiB bound.
Existing selection and response-mapping observations retain their earlier limits.

Suite/28 and /29 count an expected response field's depth from the field value, excluding
the fixed suite envelope. Every other JSON position and all older suite formats keep their
existing document depth limit. The structural reader checks these bounded paths before typed
deserialization. Authored YAML still uses the existing YAML parser's document recursion limit,
which includes the scenario envelope; it can refuse a deeply nested literal before that literal
reaches the response validator. This transport limit does not reduce the actual return-value or
admitted-suite profile, and no truncated or partial substitute is admitted.

## Port from the reviewed prototype

The unreleased prototype at `6e0b577df9fa4142dffc4d616d96fe2ab5881e24` was based on
ESS 0.37.0 and used source/16 and suites/26–27. ESS 0.38.0 shipped different semantics under
those identities. This port reserves source/17 and suites/28–29; no old document is relabeled
or reinterpreted. Prototype reports remain historical evidence under
`docs/evidence/direct-library-returns/historical-0.37/`.
