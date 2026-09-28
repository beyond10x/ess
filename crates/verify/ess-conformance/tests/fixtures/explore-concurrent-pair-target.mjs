// A small in-memory implementation of `explore-pair/pair.yaml`, two entities each created by its
// own command, for the concurrent explorer lanes in `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore_concurrent_pair_target.go` is the same target in Go, line for line. It has no
// `invokeCommand`, so every call takes effect at its return instant. The mutants:
//
//   box-lost                 every `OpenBox` opens its box and its answer never arrives
//   box-lost-phantom-crate   as `box-lost`, and each such `OpenBox` also opens a crate nobody asked for
//
// A lost box creation accounts for a box nobody names, never for a crate: under
// `box-lost-phantom-crate` a read of `Crates` showing the phantom is a read of an instance no
// operation made.

import { indeterminate, unsupported } from './dist/index.js';

export function newPairTarget(mutant = '') {
  let boxes = [];
  let crates = [];
  let minted = 0;
  const reset = () => {
    boxes = [];
    crates = [];
    minted = 0;
  };
  // The next identity, shared by boxes and crates so no two instances share one.
  const mint = () => {
    minted += 1;
    return `00000000-0000-4000-8000-${String(minted).padStart(12, '0')}`;
  };

  return {
    identity: () => ({ name: 'explore-pair-target', version: '1' }),
    beginScenario: () => reset(),
    endScenario: () => reset(),

    executeCommand({ command }) {
      switch (command) {
        case 'pair.core.OpenBox': {
          const id = mint();
          boxes.push(id);
          if (mutant === 'box-lost-phantom-crate') crates.push(mint());
          if (mutant === 'box-lost' || mutant === 'box-lost-phantom-crate') {
            throw new Error('the answer was lost', { cause: indeterminate('the answer was lost') });
          }
          return {
            outcome: 'opened',
            directEvents: [{ event: 'pair.core.BoxOpened', payload: { box_id: id } }],
          };
        }
        case 'pair.core.OpenCrate': {
          const id = mint();
          crates.push(id);
          return {
            outcome: 'opened',
            directEvents: [{ event: 'pair.core.CrateOpened', payload: { crate_id: id } }],
          };
        }
        default:
          throw unsupported(`${command} is not a command of explore-pair`);
      }
    },

    // `Boxes` and `Crates` are every instance, in the order it was created.
    queryView({ view }) {
      if (view === 'pair.core.Boxes') return { rows: boxes.map((id) => ({ box_id: id })) };
      if (view === 'pair.core.Crates') return { rows: crates.map((id) => ({ crate_id: id })) };
      throw unsupported(`${view} is not a view of explore-pair`);
    },
    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw unsupported('no external outcome');
    },
    redeliverEvent: () => {
      throw unsupported('no bindings');
    },
    observeInvocations: () => {
      throw unsupported('no bindings');
    },
  };
}
