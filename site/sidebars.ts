import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const sidebars: SidebarsConfig = {
  handbookSidebar: [
    {
      type: 'category',
      label: 'Get started',
      collapsible: true,
      collapsed: false,
      items: [
        {
          type: 'category',
          label: 'Getting Started',
          collapsible: true,
          collapsed: true,
          link: {type: 'doc', id: 'getting-started/index'},
          items: [
            'getting-started/terminal-basics',
            'getting-started/install-git',
            'getting-started/install-github-cli',
          ],
        },
        {
          type: 'category',
          label: 'Working with Claude',
          collapsible: true,
          collapsed: true,
          link: {type: 'doc', id: 'working-with-claude/index'},
          items: [
            'working-with-claude/plan',
            'working-with-claude/build',
            'working-with-claude/autonomous-loops',
            'working-with-claude/verify',
            'working-with-claude/review',
          ],
        },
        {
          type: 'doc',
          id: 'onboarding-project',
          label: 'Onboarding Project (superseded)',
        },
      ],
    },
    {
      type: 'category',
      label: 'Workshops',
      collapsible: true,
      collapsed: false,
      link: {type: 'doc', id: 'workshops/index'},
      items: [
        {
          type: 'category',
          label: 'Part 1 — Planning',
          collapsible: true,
          collapsed: false,
          items: [
            'workshops/planning/index',
            'workshops/planning/pre-work',
            'workshops/planning/prd-interrogation-worksheet',
            'workshops/planning/plan-decomposition-worksheet',
            'workshops/planning/completion-checklist',
            'workshops/planning/homework',
          ],
        },
        {
          type: 'category',
          label: 'Part 2 — Building',
          collapsible: true,
          collapsed: false,
          items: [
            'workshops/building/index',
            'workshops/building/pre-flight',
            'workshops/building/exercises',
            'workshops/building/execution-verification-worksheet',
            'workshops/building/quality-gates-worksheet',
            'workshops/building/completion-checklist',
            'workshops/building/homework',
          ],
        },
        {
          type: 'category',
          label: 'Part 3 — Closeout',
          collapsible: true,
          collapsed: false,
          items: [
            'workshops/closeout/index',
            'workshops/closeout/pre-work',
            'workshops/closeout/learnings-carry-forward-worksheet',
            'workshops/closeout/presentation-worksheet',
            'workshops/closeout/completion-checklist',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: 'Deep Dives',
      collapsible: true,
      collapsed: false,
      items: [
        'rules-explained',
        'repo-setup',
        'play-testing',
        'tooling-tour',
      ],
    },
  ],
};

export default sidebars;
