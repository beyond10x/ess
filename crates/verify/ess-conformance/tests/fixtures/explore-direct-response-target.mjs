// An in-memory implementation of `explore-direct-response.yaml`. It is copied into the package as
// `explore-external-target.mjs`, which `explore-external-driver.mjs` runs. `mode` selects the
// double:
//
//   (empty)      returns `token` and `label`, the input's label, and leaves the Optional `note` out
//   drops-token  leaves the REQUIRED `token` out
//   other-label  returns a `label` other than the input's, a value the model does not determine
//   with-note    returns the Optional `note` too
//
// `explore_direct_response_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let minted = 0;
  let version = 0;

  const reset = () => {
    minted = 0;
    version = 0;
  };

  return {
    identity: () => ({ name: 'explore-direct-response-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      if (command !== 'explorert.keys.Mint') {
        throw unsupported(`${command} is not a command of explore-direct-response.yaml`);
      }
      const label = typeof input.label === 'string' ? input.label : '';
      if ([...label].length > 8) {
        return { outcome: 'too-long', error: 'explorert.keys.TooLong', directEvents: [] };
      }
      minted += 1;
      version += 1;
      const response = { token: `tok-${minted}`, label };
      switch (mode) {
        case 'drops-token':
          delete response.token;
          break;
        case 'other-label':
          response.label = `label-${minted}`;
          break;
        case 'with-note':
          response.note = 'a note';
          break;
      }
      return { outcome: 'minted', consistency: `v${version}`, directEvents: [], response };
    },

    queryView({ view }) {
      throw unsupported(`${view} is not a view of explore-direct-response.yaml`);
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
