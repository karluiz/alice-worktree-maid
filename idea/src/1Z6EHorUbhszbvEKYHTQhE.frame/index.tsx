import React, { useMemo, useState } from 'react'
import { Loader2 } from 'lucide-react'

import { ART } from '../shared/alice/brand'
import { TopBar } from '../shared/alice/TopBar'
import { DISK, WORKTREES, type Verdict } from '../shared/alice/worktrees'
import { VerdictGroup } from './components/VerdictGroup'
import { RepoSidebar } from './components/RepoSidebar'

export const args = {
  state: 'default',
  provider: 'claude',
  repoCount: 14,
}

export const argTypes = {
  state: { control: 'select', options: ['default', 'scanning', 'spotless'] },
  provider: { control: 'radio', options: ['claude', 'kimi', 'copilot'] },
  repoCount: { control: 'number' },
}

type Props = Partial<typeof args> & { onSweep?: (count: number, gigabytes: number) => void }

export default function SweepReport(props: Props) {
  const { state, provider, repoCount, onSweep } = { ...args, ...props }
  const [selected, setSelected] = useState<Set<string>>(
    () => new Set(WORKTREES.filter((w) => w.verdict === 'sweep').map((w) => w.id))
  )
  const [sweeping, setSweeping] = useState(false)
  const [repo, setRepo] = useState<string | null>(null)

  const visible = repo ? WORKTREES.filter((w) => w.repo === repo) : WORKTREES
  const byVerdict = (v: Verdict) => visible.filter((w) => w.verdict === v)
  const picked = WORKTREES.filter((w) => selected.has(w.id))
  const pickedGb = useMemo(() => Math.round(picked.reduce((s, w) => s + w.gb, 0) * 10) / 10, [picked])
  const usedPct = (DISK.usedGb / DISK.totalGb) * 100
  const sweepPct = (pickedGb / DISK.totalGb) * 100

  const toggle = (id: string) =>
    setSelected((prev) => {
      const next = new Set(prev)
      next.has(id) ? next.delete(id) : next.add(id)
      return next
    })
  const setMany = (ids: string[], on: boolean) =>
    setSelected((prev) => {
      const next = new Set(prev)
      ids.forEach((id) => (on ? next.add(id) : next.delete(id)))
      return next
    })

  if (state === 'scanning') {
    return (
      <div className="flex h-screen flex-col bg-paper">
        <TopBar provider={provider} />
        <main className="flex flex-1 flex-col items-center justify-center gap-6 px-6 text-center">
          <img src={ART.sweeping} alt="" className="w-full max-w-xl" />
          <p className="flex items-center gap-2 text-sm text-ink-2">
            <Loader2 size={15} className="animate-spin" />
            Reading {repoCount} repos… 9 of 13 worktrees so far
          </p>
        </main>
      </div>
    )
  }

  if (state === 'spotless') {
    return (
      <div className="flex h-screen flex-col bg-paper">
        <TopBar provider={provider} />
        <main className="flex flex-1 flex-col items-center justify-center px-6 text-center">
          <img src={ART.rest} alt="Alice drinking tea on an empty hard drive" className="w-full max-w-sm" />
          <h1 className="mt-4 text-3xl font-semibold tracking-tight">Spotless.</h1>
          <p className="mt-2 max-w-sm text-sm text-ink-2">
            Every worktree in {repoCount} repos is either in use or already gone. She'll look again tomorrow at 9.
          </p>
        </main>
      </div>
    )
  }

  return (
    <div className="flex h-screen flex-col bg-paper">
      <TopBar provider={provider} title="~/code" right={<span className="hidden text-xs text-ink-3 lg:inline">Scanned 2 min ago</span>} />

      <div className="flex min-h-0 flex-1">
      <RepoSidebar active={repo} onPick={setRepo} />
      <div className="flex min-w-0 flex-1 flex-col">
      <div className="min-h-0 flex-1 overflow-y-auto">
        {/* Hero: how much disk she can give back */}
        <section className="mx-auto grid max-w-5xl grid-cols-1 items-end gap-6 px-5 pb-4 pt-7 sm:px-8 lg:grid-cols-[1fr_minmax(0,360px)]">
          <div className="min-w-0">
            <p className="font-hand text-lg text-sweep">she found the mess</p>
            <h1 className="mt-1 flex flex-wrap items-baseline gap-x-3 text-ink">
              <span className="text-5xl font-semibold tracking-tight tabular-nums sm:text-6xl">{pickedGb} GB</span>
              <span className="text-lg text-ink-2">ready to sweep</span>
            </h1>

            <div className="mt-6 max-w-xl">
              <div className="relative h-3 overflow-hidden rounded-full bg-wash" aria-hidden>
                <div className="absolute inset-y-0 left-0 bg-ink" style={{ width: `${usedPct}%` }} />
                <div
                  className="absolute inset-y-0 bg-sweep transition-[width,left] duration-200 ease-[var(--ease-out-strong)]"
                  style={{ left: `${usedPct - sweepPct}%`, width: `${sweepPct}%` }}
                />
              </div>
              <div className="mt-2 flex flex-wrap justify-between gap-x-4 text-xs text-ink-2">
                <span className="tabular-nums">
                  {DISK.usedGb} of {DISK.totalGb} GB used
                </span>
                <span className="tabular-nums text-ink">
                  {Math.round(DISK.usedGb - pickedGb)} GB after sweeping
                </span>
              </div>
            </div>
          </div>
          <img src={ART.sweeping} alt="Alice sweeping withered branches into a dustpan" className="hidden w-full lg:block" />
        </section>

        {/* The three piles */}
        <div className="mx-auto max-w-5xl px-5 pb-10 sm:px-8">
          <VerdictGroup verdict="sweep" items={byVerdict('sweep')} selected={selected} onToggle={toggle} onSetMany={setMany} />
          <VerdictGroup verdict="ask" items={byVerdict('ask')} selected={selected} onToggle={toggle} onSetMany={setMany} />
          <VerdictGroup verdict="keep" items={byVerdict('keep')} selected={selected} onToggle={toggle} onSetMany={setMany} />
        </div>
      </div>

      {/* Commit bar */}
      <footer className="border-t border-rule bg-paper px-5 py-3 sm:px-8">
        <div className="mx-auto flex max-w-5xl flex-wrap items-center justify-between gap-x-6 gap-y-2">
          <p className="text-xs leading-relaxed text-ink-2">
            Swept worktrees wait in Alice's bin for 7 days. Uncommitted work is saved as a patch first.
          </p>
          <button
            type="button"
            disabled={picked.length === 0 || sweeping}
            onClick={() => {
              setSweeping(true)
              setTimeout(() => onSweep?.(picked.length, pickedGb), 800)
            }}
            className="inline-flex items-center gap-2 whitespace-nowrap rounded-lg bg-ink px-5 py-3 text-sm font-medium text-paper transition-[transform,background-color,opacity] duration-150 ease-[var(--ease-out-strong)] hover:bg-ink/85 active:scale-[0.97] disabled:opacity-50"
          >
            {sweeping && <Loader2 size={15} className="animate-spin" />}
            {sweeping
              ? 'Sweeping…'
              : picked.length === 0
                ? 'Pick something to sweep'
                : `Sweep ${picked.length} worktree${picked.length === 1 ? '' : 's'} · ${pickedGb} GB`}
          </button>
        </div>
      </footer>
      </div>
      </div>
    </div>
  )
}
