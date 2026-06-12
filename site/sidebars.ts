import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const sidebars: SidebarsConfig = {
  // Explicit nav order — the onboarding arc, start to finish.
  handbookSidebar: [
    'getting-started',
    'working-with-claude',
    'onboarding-project',
    'rules-explained',
    'repo-setup',
    'play-testing',
    'tooling-tour',
  ],
};

export default sidebars;
