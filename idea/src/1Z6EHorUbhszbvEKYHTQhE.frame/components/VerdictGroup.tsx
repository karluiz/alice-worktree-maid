import React, { useState } from 'react'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import { Check, ChevronDown, Lock } from 'lucide-react'

import type { Verdict, Worktree } from '../../shared/alice/worktrees'

type Props = {
  verdict: Verdict
  items: Worktree[]
  selected: Set<string>
  onToggle: (id: string) => void
  onSetMany: (ids: string[], on: boolean) => void
}

export function VerdictGroup({ verdict, items, selected, onToggle, onSetMany }: Props) {
  const meta = META[verdict]
  const total = Math.round(items.reduce((s, w) => s + w.gb, 0) * 10) / 10
  const allOn = items.every((w) => selected.has(w.id))
  const [open, setOpen] = useState<string | null>(null)
  if (items.length === 0) return null

  return (
    <section className="mt-10 first:mt-4">
      <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 border-b border-ink pb-2">
        <div className="flex items-baseline gap-3">
          <h2 className={`font-hand text-2xl font-bold ${meta.text}`}>{meta.title}</h2>
          <span className="text-sm text-ink-2">{meta.blurb}</span>
        </div>
        <div className="flex items-baseline gap-4 text-xs text-ink-2">
          <span className="tabular-nums">
            {items.length} · {total} GB
          </span>
          {verdict !== 'keep' && (
            <button
              type="button"
              onClick={() => onSetMany(items.map((w) => w.id), !allOn)}
              className="font-medium text-ink underline decoration-rule underline-offset-4 transition-colors duration-150 hover:decoration-ink"
            >
              {allOn ? 'Deselect all' : 'Select all'}
            </button>
          )}
        </div>
      </div>

      <ul className="divide-y divide-rule">
        {items.map((w) => (
          <Row
            key={w.id}
            w={w}
            on={selected.has(w.id)}
            open={open === w.id}
            locked={verdict === 'keep'}
            onToggle={() => onToggle(w.id)}
            onOpen={() => setOpen(open === w.id ? null : w.id)}
          />
        ))}
      </ul>
    </section>
  )
}

function Row({
  w, on, open, locked, onToggle, onOpen,
}: { w: Worktree; on: boolean; open: boolean; locked: boolean; onToggle: () => void; onOpen: () => void }) {
  const meta = META[w.verdict]
  const reduce = useReducedMotion()

  return (
    <li>
      <div className="grid grid-cols-[auto_1fr_auto] items-center gap-x-4 gap-y-1 py-3.5 md:grid-cols-[auto_minmax(0,1.3fr)_minmax(0,1.4fr)_88px_72px_auto]">
        {locked ? (
          <span className="flex h-5 w-5 items-center justify-center text-keep" title="Alice won't sweep this">
            <Lock size={14} />
          </span>
        ) : (
          <button
            type="button"
            role="checkbox"
            aria-checked={on}
            aria-label={`Sweep ${w.branch}`}
            onClick={onToggle}
            className={`flex h-5 w-5 items-center justify-center rounded border transition-colors duration-150 ${
              on ? `${meta.box} text-paper` : 'border-ink-3 bg-paper hover:border-ink'
            }`}
          >
            {on && <Check size={13} strokeWidth={3} />}
          </button>
        )}

        <button type="button" onClick={onOpen} className="min-w-0 text-left">
          <span className="block truncate font-mono text-sm text-ink">{w.branch}</span>
          <span className="block truncate text-xs text-ink-2">
            {w.repo} · by {w.agent}
          </span>
        </button>

        <p className={`col-span-3 col-start-2 row-start-2 font-hand text-[17px] leading-snug md:col-span-1 md:col-start-auto md:row-start-auto ${meta.text}`}>
          {w.note}
        </p>

        <span className="hidden text-right text-xs tabular-nums text-ink-2 md:block">{w.age}</span>
        <span className="col-start-3 row-start-1 text-right text-sm font-medium tabular-nums text-ink md:col-start-auto">
          {w.gb} GB
        </span>

        <button
          type="button"
          onClick={onOpen}
          aria-expanded={open}
          aria-label="Why?"
          className="hidden h-7 w-7 items-center justify-center rounded-md text-ink-2 transition-colors duration-150 hover:bg-wash hover:text-ink md:flex"
        >
          <ChevronDown
            size={16}
            className={`transition-transform duration-200 ease-[var(--ease-out-strong)] ${open ? 'rotate-180' : ''}`}
          />
        </button>
      </div>

      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            initial={reduce ? { opacity: 0 } : { height: 0, opacity: 0 }}
            animate={reduce ? { opacity: 1 } : { height: 'auto', opacity: 1 }}
            exit={reduce ? { opacity: 0 } : { height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: [0.23, 1, 0.32, 1] }}
            className="overflow-hidden"
          >
            <div className={`mb-4 ml-9 rounded-lg px-4 py-3 ${meta.wash}`}>
              <p className="text-xs font-medium text-ink">What she checked</p>
              <ul className="mt-1.5 space-y-1">
                {w.evidence.map((e) => (
                  <li key={e} className="flex gap-2 text-sm text-ink-2">
                    <span className={`mt-2 h-1 w-1 shrink-0 rounded-full ${meta.dot}`} aria-hidden />
                    {e}
                  </li>
                ))}
              </ul>
              <p className="mt-2 truncate font-mono text-xs text-ink-3">{w.path}</p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </li>
  )
}

const META: Record<Verdict, { title: string; blurb: string; text: string; box: string; wash: string; dot: string }> = {
  sweep: {
    title: 'sweep it',
    blurb: 'Merged, superseded or orphaned',
    text: 'text-sweep',
    box: 'border-sweep bg-sweep',
    wash: 'bg-sweep-wash',
    dot: 'bg-sweep',
  },
  ask: {
    title: 'ask me first',
    blurb: 'Unpushed or uncommitted work',
    text: 'text-ask-ink',
    box: 'border-ask bg-ask',
    wash: 'bg-ask-wash',
    dot: 'bg-ask',
  },
  keep: {
    title: 'leave alone',
    blurb: 'Open PRs and agents still working',
    text: 'text-keep',
    box: 'border-keep bg-keep',
    wash: 'bg-keep-wash',
    dot: 'bg-keep',
  },
}
