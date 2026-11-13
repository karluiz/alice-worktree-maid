import React from 'react'
import { ArrowLeft, RotateCcw } from 'lucide-react'

import { ART } from '../shared/alice/brand'
import { TopBar } from '../shared/alice/TopBar'
import { DISK, WORKTREES } from '../shared/alice/worktrees'

export const args = {
  provider: 'claude',
  count: 9,
  gigabytes: 38.6,
  daysInBin: 7,
}

export const argTypes = {
  provider: { control: 'radio', options: ['claude', 'kimi', 'copilot'] },
  count: { control: 'number' },
  gigabytes: { control: 'number' },
  daysInBin: { control: 'number' },
}

type Props = Partial<typeof args> & { onBack?: () => void }

export default function Swept(props: Props) {
  const { provider, count, gigabytes, daysInBin, onBack } = { ...args, ...props }
  const swept = WORKTREES.filter((w) => w.verdict === 'sweep').slice(0, count)
  const freeAfter = Math.round(DISK.totalGb - DISK.usedGb + gigabytes)

  return (
    <div className="flex h-screen flex-col bg-paper">
      <TopBar provider={provider} title="~/code" />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <main className="mx-auto grid min-h-full max-w-5xl grid-cols-1 items-center gap-8 px-5 py-8 sm:px-8 md:grid-cols-[minmax(0,1fr)_minmax(0,1fr)] md:gap-12">
          <section className="min-w-0">
            <p className="font-hand text-lg text-keep">all tidy. she's having tea.</p>
            <h1 className="mt-1 text-5xl font-semibold tracking-tight tabular-nums text-ink sm:text-6xl">
              {gigabytes} GB back
            </h1>
            <p className="mt-3 text-base text-ink-2">
              {count} worktree{count === 1 ? '' : 's'} swept. You now have{' '}
              <span className="font-medium text-ink tabular-nums">{freeAfter} GB</span> free.
            </p>

            <ul className="mt-7 divide-y divide-rule border-y border-rule">
              {swept.map((w) => (
                <li key={w.id} className="flex items-center justify-between gap-4 py-2.5">
                  <span className="min-w-0 truncate font-mono text-sm text-ink-2 line-through decoration-sweep decoration-2">
                    {w.branch}
                  </span>
                  <span className="shrink-0 text-xs tabular-nums text-ink-3">{w.gb} GB</span>
                </li>
              ))}
            </ul>

            <p className="mt-4 text-sm text-ink-2">
              Everything sits in Alice's bin for {daysInBin} days, then she takes it out.
            </p>

            <div className="mt-6 flex flex-wrap gap-3">
              <button
                type="button"
                onClick={onBack}
                className="inline-flex items-center gap-2 whitespace-nowrap rounded-lg bg-ink px-5 py-3 text-sm font-medium text-paper transition-[transform,background-color] duration-150 ease-[var(--ease-out-strong)] hover:bg-ink/85 active:scale-[0.97]"
              >
                <ArrowLeft size={15} />
                Back to the report
              </button>
              <button
                type="button"
                className="inline-flex items-center gap-2 whitespace-nowrap rounded-lg border border-rule px-5 py-3 text-sm font-medium text-ink transition-[transform,border-color] duration-150 ease-[var(--ease-out-strong)] hover:border-ink active:scale-[0.97]"
              >
                <RotateCcw size={15} />
                Undo sweep
              </button>
            </div>
          </section>

          <img
            src={ART.rest}
            alt="Alice drinking tea on top of an emptied hard drive, a full rubbish bag beside her"
            className="mx-auto w-full max-w-md"
          />
        </main>
      </div>
    </div>
  )
}
