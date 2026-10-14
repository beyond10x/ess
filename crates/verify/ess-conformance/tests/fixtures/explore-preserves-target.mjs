// An in-memory implementation of `explore-preserves.yaml` (beyond10x/ess#536).
//
// `mode` selects the double:
//
//   (empty)         the specification
//   again-resumes   `PauseAgain` answers `still-paused` and moves the session back to `Active`
//   paused-relabels `Pause` answers `already-paused` and clears the session's label
//
// `explore_preserves_target.go` is the same target in Go, line for line.

import { unsupported } from './dist/index.js';

export function newTarget(mode = '') {
  let sessions = new Map();
  let created = 0;
  let version = 0;

  const reset = () => {
    sessions = new Map();
    created = 0;
    version = 0;
  };
  const refused = (outcome, error) => ({ outcome, error, directEvents: [] });
  const answered = (outcome, event, id) => {
    version += 1;
    return {
      outcome,
      consistency: `v${version}`,
      directEvents: [{ event, payload: { session_id: id } }],
    };
  };
  const kept = (outcome) => ({ outcome, consistency: `v${version}`, directEvents: [] });

  return {
    identity: () => ({ name: 'explore-preserves-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command, input }) {
      const session = sessions.get(input.session_id);
      switch (command) {
        case 'explorekeep.desk.Start': {
          created += 1;
          const id = `00000000-0000-4000-8000-${String(created).padStart(12, '0')}`;
          sessions.set(id, { state: 'Active', label: input.label });
          return answered('started', 'explorekeep.desk.Started', id);
        }
        case 'explorekeep.desk.Pause': {
          if (session === undefined) throw new Error(`no session ${input.session_id}`);
          if (session.state === 'Paused') {
            if (mode === 'paused-relabels') session.label = '';
            return kept('already-paused');
          }
          session.state = 'Paused';
          return answered('paused', 'explorekeep.desk.Paused', input.session_id);
        }
        case 'explorekeep.desk.PauseAgain': {
          if (session === undefined) throw new Error(`no session ${input.session_id}`);
          if (session.state !== 'Paused') return refused('not-paused', 'explorekeep.desk.NotPaused');
          if (mode === 'again-resumes') session.state = 'Active';
          return kept('still-paused');
        }
        case 'explorekeep.desk.Resume': {
          if (session === undefined) throw new Error(`no session ${input.session_id}`);
          if (session.state !== 'Paused') return refused('not-active', 'explorekeep.desk.NotActive');
          session.state = 'Active';
          return answered('resumed', 'explorekeep.desk.Resumed', input.session_id);
        }
        default:
          throw unsupported(`${command} is not a command of explore-preserves.yaml`);
      }
    },

    queryView({ view }) {
      const all = [...sessions];
      switch (view) {
        case 'explorekeep.desk.Sessions':
          return {
            rows: all.map(([id, session]) => ({
              session_id: id,
              state: session.state,
              label: session.label,
            })),
          };
        case 'explorekeep.desk.PausedSessions':
          return {
            rows: all
              .filter(([, session]) => session.state === 'Paused')
              .map(([id, session]) => ({ session_id: id, label: session.label })),
          };
        default:
          throw unsupported(`${view} is not a view of explore-preserves.yaml`);
      }
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
