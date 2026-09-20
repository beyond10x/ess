import fs from 'node:fs/promises';
import path from 'node:path';
import type {Plugin} from '@docusaurus/types';

/**
 * Point each shared document at its unified-site copy.
 *
 * The same documents are rendered twice: here, under `/ess/`, with this project's navigation and
 * its interactive pages; and on `beyond10x.github.io/docs/ess/`, inside the organization's shell
 * and its search index. Two addresses for one document is the thing a canonical link exists for,
 * and the unified copy is the one the ecosystem links to, so it is the one named here.
 *
 * A document the unified site does not carry stays canonical to itself. `b10x.docs.yaml` is the
 * only record of which those are, so it is read rather than repeated: it excludes
 * `docs/examples/specification-to-contracts.md`, whose MDX imports the browser lab and which the
 * passive documentation collection refuses.
 */
export default function canonicalToUnifiedDocs(): Plugin {
  return {
    name: 'b10x-canonical-to-unified-docs',
    async postBuild({outDir, routesPaths, siteConfig}) {
      const manifest = await fs.readFile(
        path.resolve(outDir, '..', '..', 'b10x.docs.yaml'),
        'utf8',
      );
      const excluded = [...manifest.matchAll(/^ *- (docs\/[^\s]+)$/gm)]
        .map((match) => match[1].replace(/^docs\//, '').replace(/\.md$/, ''))
        .map((entry) => entry.replace(/\/\*\*$/, ''));
      const origin = siteConfig.url.replace(/\/$/, '');
      let rewritten = 0;
      for (const route of routesPaths) {
        const documented = route.match(/^\/ess\/docs(\/.*)?$/);
        if (!documented) continue;
        const slug = (documented[1] ?? '').replace(/^\/|\/$/g, '');
        if (excluded.some((entry) => slug === entry || slug.startsWith(`${entry}/`))) continue;
        // `trailingSlash: false` emits `docs/reference/cli.html`; a route that is also a directory
        // keeps `index.html`, so both spellings are tried rather than assumed.
        const base = path.join(outDir, 'docs', ...slug.split('/').filter(Boolean));
        let held: string | undefined;
        let target: string | undefined;
        for (const candidate of [`${base}.html`, path.join(base, 'index.html')]) {
          const contents = await fs.readFile(candidate, 'utf8').catch(() => undefined);
          if (contents !== undefined) {
            held = contents;
            target = candidate;
            break;
          }
        }
        if (held === undefined || target === undefined) continue;
        const canonical = `${origin}/docs/ess/${slug}${slug ? '/' : ''}`;
        // Docusaurus emits unquoted attributes in production HTML, so the value is matched by
        // position rather than by a quoting style this build does not use.
        const next = held.replace(
          /(<link[^>]*\brel=("canonical"|'canonical'|canonical)[^>]*\bhref=)("[^"]*"|'[^']*'|[^\s>]+)/,
          `$1${canonical}`,
        );
        if (next === held) continue;
        await fs.writeFile(target, next);
        rewritten += 1;
      }
      if (rewritten === 0) {
        throw new Error('no document page carried a canonical link to rewrite');
      }
    },
  };
}
