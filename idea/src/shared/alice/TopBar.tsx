import React from 'react'

import { providerModel, providerName } from './brand'

type Props = {
  provider?: string
  title?: string
  right?: React.ReactNode
}

/**
 * The window's unified title bar. In Rune this is the drag region of a
 * hidden-titlebar window: the OS draws the traffic lights, we leave room.
 */
export function TopBar({ provider, title, right }: Props) {
  return (
    <header className="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-rule bg-paper pl-4 pr-4 sm:pr-5">
      <div className="flex min-w-0 items-center gap-4">
        <div className="flex shrink-0 gap-2" aria-hidden>
          <span className="h-3 w-3 rounded-full bg-[#ff5f57]" />
          <span className="h-3 w-3 rounded-full bg-[#febc2e]" />
          <span className="h-3 w-3 rounded-full bg-[#28c840]" />
        </div>
        <span className="font-hand text-xl font-bold leading-none text-ink">alice</span>
        {title && <span className="hidden truncate text-xs text-ink-3 sm:inline">{title}</span>}
      </div>
      <div className="flex items-center gap-4">
        {provider && (
          <span className="flex items-center gap-2 whitespace-nowrap text-xs text-ink-2">
            <span className="h-1.5 w-1.5 rounded-full bg-keep" aria-hidden />
            Thinking with <span className="font-medium text-ink">{providerName(provider)}</span>
            <span className="hidden font-mono text-ink-3 md:inline">{providerModel(provider)}</span>
          </span>
        )}
        {right}
      </div>
    </header>
  )
}
