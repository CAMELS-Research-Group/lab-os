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

Deployment is handled by the `deploy-site.yml` GitHub Action, which pushes the built site to GitHub Pages.
