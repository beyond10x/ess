// An in-memory implementation of `explore-direct-response-json.yaml`, copied into the package as
// `explore-external-target.mjs`. `mode` selects the double:
//
//   (empty)          returns `token`, `label` and the REQUIRED `Json` member `body` as an object
//   null-body        answers `body` as JSON null, one of the `Json` type's own values
//   drops-body       leaves `body` out
//   undefined-token  answers the REQUIRED `token` as undefined (the Go target leaves it out)
//
// `explore_direct_response_json_target.go` is the same target in Go.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let minted = 0;
  let version = 0;

  const reset = () => {
    minted = 0;
    version = 0;
  };

  return {
    identity: () => ({ name: 'explore-direct-response-json-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      if (command !== 'explorert.keys.Mint') {
        throw unsupported(`${command} is not a command of explore-direct-response-json.yaml`);
      }
      const label = typeof input.label === 'string' ? input.label : '';
      if ([...label].length > 8) {
        return { outcome: 'too-long', error: 'explorert.keys.TooLong', directEvents: [] };
      }
      minted += 1;
      version += 1;
      const response = { token: `tok-${minted}`, label, body: { k: 'v' } };
      switch (mode) {
        case 'null-body':
          response.body = null;
          break;
        case 'drops-body':
          delete response.body;
          break;
        case 'undefined-token':
          response.token = undefined;
          break;
      }
      return { outcome: 'minted', consistency: `v${version}`, directEvents: [], response };
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of explore-direct-response-json.yaml`);
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external branches');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
