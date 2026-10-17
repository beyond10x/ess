// An in-memory implementation of `explore-codes.yaml` (beyond10x/ess#582, #583). It reads its own
// wall clock for every guard over `now`, as a deployed implementation does.
//
// `mode` selects the double:
//
//   (empty)        the specification
//   keeps-row      `RedeemCode` answers `redeemed` and the code stays in `Codes`
//   redeems-again  `RedeemCode` answers `redeemed` again for a code it already redeemed
//
// `explore_codes_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let codes = [];
  let version = 0;

  const reset = () => {
    codes = [];
    version = 0;
  };
  const instant = (value) => {
    const at = Date.parse(value);
    if (Number.isNaN(at)) throw new Error(`the explorer sent ${value}, which is no RFC 3339 instant`);
    return at;
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };

  return {
    identity: () => ({ name: 'explore-codes-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const now = Date.now();
      switch (command) {
        case 'explorecodes.auth.StoreCode': {
          instant(input.expiry);
          const id = `00000000-0000-4000-8000-${String(codes.length + 1).padStart(12, '0')}`;
          codes.push({ id, expiry: input.expiry, deleted: false });
          return answered('stored', 'explorecodes.auth.Stored', { code_id: id });
        }
        case 'explorecodes.auth.RedeemCode': {
          const code = codes.find((it) => it.id === input.code);
          if (code === undefined || (code.deleted && mode !== 'redeems-again')) {
            return refused('unknown-code', 'explorecodes.auth.UnknownCode');
          }
          if (!code.deleted && instant(code.expiry) < now) {
            return refused('expired', 'explorecodes.auth.Expired');
          }
          code.deleted = true;
          return answered('redeemed', 'explorecodes.auth.Redeemed', { code_id: code.id });
        }
        case 'explorecodes.auth.PlanCode': {
          if (instant(input.at) < now - 60_000) {
            return refused('in-the-past', 'explorecodes.auth.InThePast');
          }
          return answered('planned', 'explorecodes.auth.Planned', { at: input.at });
        }
        default:
          throw unsupported(`${command} is not a command of explore-codes.yaml`);
      }
    },

    queryView({ view }) {
      if (view !== 'explorecodes.auth.Codes') {
        throw unsupported(`${view} is not a view of explore-codes.yaml`);
      }
      return {
        rows: codes
          .filter((code) => !code.deleted || mode === 'keeps-row')
          .map((code) => ({ code_id: code.id, expiry: code.expiry })),
      };
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcomes');
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
