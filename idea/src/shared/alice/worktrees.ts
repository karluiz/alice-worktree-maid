export type Verdict = 'sweep' | 'ask' | 'keep'

export type Worktree = {
  id: string
  branch: string
  repo: string
  path: string
  agent: string
  age: string
  gb: number
  verdict: Verdict
  /** Alice's one-line read, in her own voice. */
  note: string
  /** What she looked at to decide. */
  evidence: string[]
}

export const DISK = { totalGb: 512, usedGb: 489 }

export const WORKTREES: Worktree[] = [
  {
    id: 'w1', branch: 'feat/checkout-retry', repo: 'storefront', path: '~/code/storefront/.worktrees/checkout-retry',
    agent: 'claude-code', age: '6 days', gb: 7.4, verdict: 'sweep',
    note: 'merged in #482. done, done, done.',
    evidence: ['Squash-merged into main as a41f0c2, 3 days ago', 'No uncommitted changes', 'node_modules and .next are 6.9 GB of it'],
  },
  {
    id: 'w2', branch: 'agent/fix-flaky-e2e-3', repo: 'storefront', path: '~/code/storefront/.worktrees/fix-flaky-e2e-3',
    agent: 'codex', age: '11 days', gb: 6.1, verdict: 'sweep',
    note: 'attempt 3 of 3. attempt 2 won.',
    evidence: ['Sibling branch agent/fix-flaky-e2e-2 merged in #476', 'Last commit is identical to the merged diff minus a console.log', 'No uncommitted changes'],
  },
  {
    id: 'w3', branch: 'agent/fix-flaky-e2e-1', repo: 'storefront', path: '~/code/storefront/.worktrees/fix-flaky-e2e-1',
    agent: 'codex', age: '11 days', gb: 5.8, verdict: 'sweep',
    note: 'same story. the loser of the race.',
    evidence: ['Superseded by #476', 'Remote branch deleted', 'No uncommitted changes'],
  },
  {
    id: 'w4', branch: 'chore/bump-prisma', repo: 'api', path: '~/code/api/.worktrees/bump-prisma',
    agent: 'claude-code', age: '9 days', gb: 4.2, verdict: 'sweep',
    note: 'merged. left its docker volume behind too.',
    evidence: ['Merged into main in #219', 'Dangling docker volume api_wt_prisma, 1.1 GB', 'No uncommitted changes'],
  },
  {
    id: 'w5', branch: 'spike/rsc-cache', repo: 'storefront', path: '~/code/storefront/.worktrees/rsc-cache',
    agent: 'cursor', age: '24 days', gb: 5.3, verdict: 'sweep',
    note: 'a spike nobody came back for.',
    evidence: ['Never pushed, never opened a PR', 'Clean working tree', 'Findings already copied to docs/rsc-notes.md on main'],
  },
  {
    id: 'w6', branch: 'agent/i18n-strings', repo: 'web-admin', path: '~/code/web-admin/.worktrees/i18n-strings',
    agent: 'claude-code', age: '5 days', gb: 3.9, verdict: 'sweep',
    note: 'merged in #91.',
    evidence: ['Merged into main in #91', 'No uncommitted changes'],
  },
  {
    id: 'w7', branch: 'agent/refactor-auth-2', repo: 'api', path: '~/code/api/.worktrees/refactor-auth-2',
    agent: 'aider', age: '17 days', gb: 2.6, verdict: 'sweep',
    note: 'branch is gone upstream. orphan.',
    evidence: ['Remote branch deleted after #214 closed unmerged', 'Clean working tree'],
  },
  {
    id: 'w8', branch: 'agent/docs-sweep', repo: 'docs', path: '~/code/docs/.worktrees/docs-sweep',
    agent: 'codex', age: '8 days', gb: 1.9, verdict: 'sweep',
    note: 'merged. tiny, but it all adds up.',
    evidence: ['Merged into main in #58', 'No uncommitted changes'],
  },
  {
    id: 'w9', branch: 'agent/pdf-export', repo: 'web-admin', path: '~/code/web-admin/.worktrees/pdf-export',
    agent: 'claude-code', age: '13 days', gb: 1.4, verdict: 'sweep',
    note: 'closed as wontfix. sorry, little branch.',
    evidence: ['PR #88 closed by you: "not doing this"', 'Clean working tree'],
  },
  {
    id: 'w10', branch: 'agent/payments-webhooks', repo: 'api', path: '~/code/api/.worktrees/payments-webhooks',
    agent: 'claude-code', age: '19 days', gb: 4.8, verdict: 'ask',
    note: '3 commits never pushed. yours?',
    evidence: ['3 local commits not on any remote', 'Untouched for 19 days', 'Touches src/billing, which changed on main since'],
  },
  {
    id: 'w11', branch: 'feat/search-v2', repo: 'storefront', path: '~/code/storefront/.worktrees/search-v2',
    agent: 'cursor', age: '27 days', gb: 3.7, verdict: 'ask',
    note: '41 uncommitted files. I won\u2019t touch those.',
    evidence: ['41 modified files, nothing staged', 'Untouched for 27 days', 'I can stash them to a patch before sweeping'],
  },
  {
    id: 'w12', branch: 'feat/onboarding-emails', repo: 'web-admin', path: '~/code/web-admin/.worktrees/onboarding-emails',
    agent: 'claude-code', age: '2 days', gb: 2.8, verdict: 'keep',
    note: 'PR #94 is open and green. still cooking.',
    evidence: ['PR #94 open, checks passing', 'Commit 2 hours ago'],
  },
  {
    id: 'w13', branch: 'agent/rate-limiter', repo: 'api', path: '~/code/api/.worktrees/rate-limiter',
    agent: 'codex', age: '1 day', gb: 2.2, verdict: 'keep',
    note: 'an agent is in here right now.',
    evidence: ['codex process holds a lock on this worktree', 'Files changed 4 minutes ago'],
  },
]
