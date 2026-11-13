import React, { useState } from 'react'

import Connect from '../vWpfj6tAFJDwWfjtexRrdk.frame'
import SweepReport from '../1Z6EHorUbhszbvEKYHTQhE.frame'
import Swept from '../u5BaazZPiiHCFLNmtWgtdU.frame'

const CONNECT = 'vWpfj6tAFJDwWfjtexRrdk'
const REPORT = '1Z6EHorUbhszbvEKYHTQhE'
const SWEPT = 'u5BaazZPiiHCFLNmtWgtdU'

/** The size every frame draws at, unless one exports its own. */
export const viewport = { width: 1200, height: 780 }

export const args = {
  provider: 'claude',
}

export const argTypes = {
  provider: { control: 'radio', options: ['claude', 'kimi', 'copilot'] },
}

type Props = Partial<typeof args> & { start?: string }

export default function Alice(props: Props) {
  const merged = { ...args, ...props }
  const [step, setStep] = useState(() => merged.start ?? CONNECT)
  const [provider, setProvider] = useState(merged.provider)
  const [result, setResult] = useState({ count: 9, gigabytes: 38.6 })

  if (step === SWEPT) {
    return (
      <Swept
        provider={provider}
        count={result.count}
        gigabytes={result.gigabytes}
        onBack={() => setStep(REPORT)}
      />
    )
  }
  if (step === REPORT) {
    return (
      <SweepReport
        provider={provider}
        onSweep={(count, gigabytes) => {
          setResult({ count, gigabytes })
          setStep(SWEPT)
        }}
      />
    )
  }
  return (
    <Connect
      provider={provider}
      onConnect={(p) => {
        setProvider(p)
        setStep(REPORT)
      }}
    />
  )
}
