# ESS documentation site

This Docusaurus site is the adopter-facing ESS documentation. It is intentionally separate from the
repository-root `docs/` tree, which contains engineering designs, plans, and reviews.

From the repository root:

```console
task site-build
```

For local development:

```console
cd website
npm ci
npm start          # http://localhost:3000/ess/
```

The Pages workflow publishes only `website/build/`. Do not configure GitHub Pages to serve the
repository-root `docs/` directory.
