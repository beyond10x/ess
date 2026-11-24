// The identity of the measuring engine: a digest of its files and the commit that holds them, so
// a round says which code measured it. Generic: the engine is every file under the loop's root
// except its outputs and records.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

// Not engine: dependencies, run output, and what the engine writes or the loop records.
export const NOT_ENGINE = ['node_modules', '.out', 'rounds', 'findings', 'baseline.json', 'trend.md'];

export interface Engine {
  digest: string;
  commit: string;
  dirty: boolean;
}

export function engineFiles(root: string): string[] {
  const out: string[] = [];
  const walk = (dir: string) => {
    for (const name of readdirSync(dir).sort()) {
      const path = join(dir, name);
      if (dir === root && NOT_ENGINE.includes(name)) continue;
      if (statSync(path).isDirectory()) walk(path);
      else out.push(relative(root, path));
    }
  };
  walk(root);
  // Whole paths sorted as strings: the ratchet computes the same digest from the committed tree's
  // file list (crates/edge/ess-xtask/src/ux_ratchet.rs, `engine_digest`), T26 of the R-03 review.
  return out.sort();
}

export function engineDigest(root: string): string {
  const hash = createHash('sha256');
  for (const file of engineFiles(root)) {
    hash.update(file).update('\0').update(readFileSync(join(root, file))).update('\0');
  }
  return hash.digest('hex').slice(0, 16);
}

// The last commit that changed an engine file, and whether the tree differs from it there.
export function engineCommit(root: string): { commit: string; dirty: boolean } {
  const spec = ['.', ...NOT_ENGINE.map((n) => `:(exclude)${n}`)];
  try {
    const commit = execFileSync('git', ['log', '-1', '--format=%h', '--abbrev=12', '--', ...spec], { cwd: root }).toString().trim();
    const status = execFileSync('git', ['status', '--porcelain', '--', ...spec], { cwd: root }).toString().trim();
    return { commit: commit || 'none', dirty: status !== '' };
  } catch {
    return { commit: 'unknown', dirty: true };
  }
}

export function engine(root: string): Engine {
  return { digest: engineDigest(root), ...engineCommit(root) };
}
