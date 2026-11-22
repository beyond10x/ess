// The page list of a built site, from its sitemap.xml, as paths below the origin.
import { existsSync, readFileSync } from 'node:fs';

export function sitemapPaths(file: string): string[] {
  if (!existsSync(file)) {
    throw new Error(`${file} is missing: build the site once before the specs run (task site-build)`);
  }
  const xml = readFileSync(file, 'utf8');
  const paths = Array.from(xml.matchAll(/<loc>([^<]+)<\/loc>/g), (m) => new URL(m[1]).pathname);
  return Array.from(new Set(paths)).sort();
}
