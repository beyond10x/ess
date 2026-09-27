// An in-memory implementation of `explore-external.yaml`, a test double that arranges external
// outcomes the way the synthesized scenarios ask it to.
//
// An arrangement holds for its command until the scenario ends. `mode` selects the double:
//
//   (empty)              arranges every external branch it is asked to
//   cannot-arrange       `configureExternalOutcome` is not supported
//   ignores-arrangement  accepts every arrangement and then takes the ordinary branch anyway
//
// `explore_external_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

const PENDING = 'Pending';

export function newTarget(mode = '') {
  let payments = new Map();
  let created = 0;
  let version = 0;
  let forced = new Map();

  const reset = () => {
    payments = new Map();
    created = 0;
    version = 0;
    forced = new Map();
  };
  const answered = (outcome, event, payload) => {
    version += 1;
    return { outcome, consistency: `v${version}`, directEvents: [{ event, payload }] };
  };
  const arranged = (command, outcome) =>
    mode !== 'ignores-arrangement' && forced.get(command) === outcome;

  return {
    identity: () => ({ name: 'explore-external-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      switch (command) {
        case 'exploreext.pay.Authorize': {
          if (input.amount >= 1000 && arranged(command, 'declined')) {
            return { outcome: 'declined', error: 'exploreext.pay.Declined', directEvents: [] };
          }
          if (!(input.amount >= 1)) {
            return { outcome: 'invalid', error: 'exploreext.pay.InvalidAmount', directEvents: [] };
          }
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          payments.set(id, { payment_id: id, amount: input.amount, state: PENDING });
          return answered('authorized', 'exploreext.pay.Authorized', {
            payment_id: id,
            amount: input.amount,
          });
        }
        case 'exploreext.pay.Capture': {
          const payment = payments.get(input.payment_id);
          if (payment.state !== PENDING) {
            return {
              outcome: 'wrong-state',
              error: 'exploreext.pay.PaymentStateConflict',
              directEvents: [],
            };
          }
          if (arranged(command, 'failed')) {
            payment.state = 'Failed';
            return answered('failed', 'exploreext.pay.CaptureFailed', {
              payment_id: payment.payment_id,
            });
          }
          payment.state = 'Settled';
          return answered('captured', 'exploreext.pay.Captured', {
            payment_id: payment.payment_id,
          });
        }
        default:
          throw unsupported(`${command} is not a command of explore-external.yaml`);
      }
    },

    queryView({ view }) {
      if (view === 'exploreext.pay.PendingPayments') {
        return {
          rows: [...payments.values()]
            .filter((payment) => payment.state === PENDING)
            .map((payment) => ({ payment_id: payment.payment_id, amount: payment.amount })),
        };
      }
      throw unsupported(`${view} is not a view of explore-external.yaml`);
    },

    observeEvents: () => [],
    configureExternalOutcome({ command, outcome }) {
      if (mode === 'cannot-arrange') throw unsupported('this double arranges no external outcome');
      forced.set(command, outcome);
    },
    redeliverEvent: () => {
      throw unsupported('no redelivery');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
