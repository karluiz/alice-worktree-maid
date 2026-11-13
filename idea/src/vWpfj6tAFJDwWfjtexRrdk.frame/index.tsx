import React, { useState } from 'react'
import { Check, FolderOpen, Loader2 } from 'lucide-react'

import { ART, PROVIDERS } from '../shared/alice/brand'
import { TopBar } from '../shared/alice/TopBar'

export const args = {
  state: 'default',
  provider: 'claude',
  folder: '~/code',
  reposFound: 14,
}

export const argTypes = {
  state: { control: 'select', options: ['default', 'connecting', 'error'] },
  provider: { control: 'radio', options: ['claude', 'kimi', 'copilot'] },
  folder: { control: 'text' },
  reposFound: { control: 'number' },
}

type Props = Partial<typeof args> & { onConnect?: (provider: 'claude' | 'kimi' | 'copilot') => void }

export default function Connect(props: Props) {
  const { state, provider, folder, reposFound, onConnect } = { ...args, ...props }
  const [picked, setPicked] = useState(provider)
  const [connecting, setConnecting] = useState(false)
  const busy = state === 'connecting' || connecting

  return (
    <div className="flex h-screen flex-col bg-paper">
      <TopBar title="First run" />
      <div className="min-h-0 flex-1 overflow-y-auto">
      <main className="mx-auto grid min-h-full w-full max-w-6xl grid-cols-1 items-center gap-8 px-5 py-8 sm:px-8 lg:grid-cols-[1fr_1fr] lg:gap-14">
        {/* The maid, introduced */}
        <section className="relative order-2 lg:order-1">
          <img
            src={ART.greeting}
            alt="Alice, a maid robot with a steel broom, bowing beside a plug and socket"
            className="mx-auto w-full max-w-md select-none"
          />
          <p className="absolute right-[6%] top-[8%] hidden rotate-[-4deg] font-hand text-lg text-sweep sm:block">
            plug in a brain,
            <br />
            she does the dirty job
          </p>
        </section>

        {/* The form */}
        <section className="order-1 min-w-0 lg:order-2">
          <h1 className="text-3xl font-semibold leading-[1.1] tracking-tight text-ink sm:text-4xl">
            Your agents leave worktrees everywhere. Alice cleans up.
          </h1>
          <p className="mt-4 max-w-md text-base leading-relaxed text-ink-2">
            She reads every worktree, decides what was merged, what is still cooking and what is just
            taking disk, then asks before she sweeps. Bring the model you already pay for.
          </p>

          <fieldset className="mt-9">
            <legend className="text-sm font-medium text-ink">Bring your own subscription</legend>
            <div className="mt-3 divide-y divide-rule border-y border-rule">
              {PROVIDERS.map((p) => {
                const on = picked === p.id
                return (
                  <label
                    key={p.id}
                    className="flex cursor-pointer items-center gap-4 py-3.5 transition-colors duration-150 hover:bg-wash/70"
                  >
                    <input
                      type="radio"
                      name="provider"
                      className="sr-only"
                      checked={on}
                      onChange={() => setPicked(p.id)}
                    />
                    <span
                      className={`flex h-5 w-5 shrink-0 items-center justify-center rounded-full border transition-colors duration-150 ${
                        on ? 'border-ink bg-ink text-paper' : 'border-ink-3'
                      }`}
                      aria-hidden
                    >
                      {on && <Check size={12} strokeWidth={3} />}
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block text-[15px] font-medium text-ink">{p.name}</span>
                      <span className="block text-sm text-ink-2">{p.plan}</span>
                    </span>
                    <span className="hidden whitespace-nowrap text-xs text-ink-3 sm:block">{p.auth}</span>
                  </label>
                )
              })}
            </div>
            <p className="mt-2 text-xs text-ink-3">OpenAI and local Ollama are next.</p>
          </fieldset>

          <div className="mt-7">
            <label htmlFor="folder" className="text-sm font-medium text-ink">
              Where should she clean?
            </label>
            <div className="mt-2 flex items-center gap-3 rounded-lg border border-rule bg-wash px-3 py-2.5 focus-within:border-ink">
              <FolderOpen size={16} className="shrink-0 text-ink-2" />
              <input
                id="folder"
                defaultValue={folder}
                className="min-w-0 flex-1 bg-transparent font-mono text-sm text-ink outline-none"
              />
              <span className="whitespace-nowrap text-xs text-ink-2">{reposFound} repos</span>
              <button
                type="button"
                className="whitespace-nowrap rounded-md border border-rule bg-paper px-2.5 py-1 text-xs font-medium text-ink transition-colors duration-150 hover:border-ink"
              >
                Choose…
              </button>
            </div>
          </div>

          {state === 'error' && (
            <p role="alert" className="mt-4 rounded-lg bg-sweep-wash px-3 py-2.5 text-sm text-sweep">
              That sign-in didn't stick. Your session may have expired, try again.
            </p>
          )}

          <div className="mt-7 flex flex-wrap items-center gap-x-5 gap-y-3">
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                setConnecting(true)
                setTimeout(() => onConnect?.(picked as 'claude' | 'kimi' | 'copilot'), 700)
              }}
              className="inline-flex items-center gap-2 whitespace-nowrap rounded-lg bg-ink px-5 py-3 text-sm font-medium text-paper transition-[transform,background-color] duration-150 ease-[var(--ease-out-strong)] hover:bg-ink/85 active:scale-[0.97] disabled:opacity-70"
            >
              {busy && <Loader2 size={15} className="animate-spin" />}
              {busy ? 'Waking Alice up…' : 'Send Alice in'}
            </button>
            <p className="max-w-xs text-xs leading-relaxed text-ink-3">
              Only branch names, commit messages and file stats go to your model. Code stays on disk.
            </p>
          </div>
        </section>
      </main>
      </div>
    </div>
  )
}
