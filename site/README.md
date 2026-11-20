# lab-os Handbook Site

The lab-os handbook is built with [Docusaurus 3](https://docusaurus.io/).

## Development

Install dependencies:
```bash
npm ci
```

Start the local dev server:
```bash
npm run start
```

Build static content:
```bash
npm run build
```

The production build enforces a link-rot gate — broken internal links cause the build to fail by design.

The site is not deployed: GitHub Pages is off for the org, so its deploy workflow was removed.
