import React from 'react'

import { WORKTREES } from '../../shared/alice/worktrees'

type Props = { active: string | null; onPick: (repo: string | null) => void }

/** Repos under the watched folder, heaviest first. Hidden below md. */
export function RepoSidebar({ active, onPick }: Props) {
  const repos = Object.entries(
    WORKTREES.reduce<Record<string, { gb: number; sweep: number }>>((acc, w) => {
      const r = (acc[w.repo] ??= { gb: 0, sweep: 0 })
      r.gb += w.gb
      if (w.verdict === 'sweep') r.sweep += 1
      return acc
    }, {})
  ).sort((a, b) => b[1].gb - a[1].gb)
  const total = WORKTREES.reduce((s, w) => s + w.gb, 0)

  return (
    <aside className="hidden w-56 shrink-0 flex-col border-r border-rule bg-wash md:flex">
      <nav className="flex-1 overflow-y-auto px-2 py-4">
        <p className="px-2 pb-2 text-xs text-ink-3">Worktrees by repo</p>
        <Item label="All repos" gb={total} on={active === null} onClick={() => onPick(null)} />
        {repos.map(([name, r]) => (
          <Item
            key={name}
            label={name}
            gb={r.gb}
            sweep={r.sweep}
            on={active === name}
            onClick={() => onPick(name)}
          />
        ))}
      </nav>
      <div className="space-y-1 border-t border-rule px-4 py-3 text-xs text-ink-2">
        <p className="flex justify-between">
          <span>Alice's bin</span>
          <span className="tabular-nums">0 GB</span>
        </p>
        <p className="flex justify-between">
          <span>Next look</span>
          <span>Tomorrow 9:00</span>
        </p>
      </div>
    </aside>
  )
}

function Item({
  label, gb, sweep, on, onClick,
}: { label: string; gb: number; sweep?: number; on: boolean; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors duration-150 ${
        on ? 'bg-paper font-medium text-ink shadow-[0_0_0_1px_var(--color-rule)]' : 'text-ink-2 hover:bg-paper/60 hover:text-ink'
      }`}
    >
      <span className="min-w-0 flex-1 truncate">{label}</span>
      {!!sweep && <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-sweep" aria-label={`${sweep} to sweep`} />}
      <span className="shrink-0 text-xs tabular-nums text-ink-3">{Math.round(gb * 10) / 10} GB</span>
    </button>
  )
}
