// A small in-memory implementation of `explore-retry/retry.yaml`, for the concurrent explorer lanes
// in `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore_concurrent_retry_target.go` is the same target in Go, line for line: the two lanes'
// histories are compared byte for byte. A request is the command, its `document` and its
// correlation; one answered `seeded` before is answered `replayed`, with the retained response and
// no event. The mutant `unretained` has an `invokeCommand` that looks a request up when it is
// invoked and retains it only when it returns, so a retry sent while its original is in flight is
// applied as a new request and creates a second record.

import { unsupported } from './dist/index.js';

const id = (n) => `00000000-0000-4000-8000-${String(n).padStart(12, '0')}`;
const keyOf = (request) =>
  `${request.command}\u0000${typeof request.input?.document === 'string' ? request.input.document : ''}\u0000${request.correlation}`;

export function newRetryTarget(mutant) {
  let records = 0;
  let retained = new Map();
  const reset = () => {
    records = 0;
    retained = new Map();
  };
  const seed = (key) => {
    records += 1;
    const result = {
      outcome: 'seeded',
      response: { revision_id: id(2 * records) },
      consistency: `seq:${records}`,
      directEvents: [{ event: 'retry.core.Seeded', payload: { record_id: id(2 * records - 1) } }],
    };
    if (!retained.has(key)) retained.set(key, result);
    return result;
  };
  const executeCommand = (request) => {
    if (request.command !== 'retry.core.Seed') {
      throw unsupported(`${request.command} is not a command of explore-retry`);
    }
    const key = keyOf(request);
    const answered = retained.get(key);
    if (answered !== undefined) {
      return {
        outcome: 'replayed',
        response: answered.response,
        consistency: answered.consistency,
        directEvents: [],
      };
    }
    return seed(key);
  };

  const target = {
    identity: () => ({ name: 'explore-retry-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),
    executeCommand,
    queryView: () => {
      throw unsupported('no views');
    },
    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('a replay is never forced');
    },
    redeliverEvent: () => {
      throw unsupported('no bindings');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
  if (mutant === 'unretained') {
    target.invokeCommand = (request) => {
      const unretained = !retained.has(keyOf(request));
      return {
        complete: () => {
          const key = keyOf(request);
          if (retained.has(key) && unretained && request.command === 'retry.core.Seed') {
            return seed(key);
          }
          return executeCommand(request);
        },
      };
    };
  }
  return target;
}
