export const ART = {
  greeting: 'https://cdn.magicpatterns.com/patterns/generated-images/bc897c85-4343-4b1d-a1fe-728e69312737.jpg',
  sweeping: 'https://cdn.magicpatterns.com/patterns/generated-images/b1b6cb0b-e3ed-4139-9c07-cd09b4c9b67e.jpg',
  rest: 'https://cdn.magicpatterns.com/patterns/generated-images/79b11f09-ca93-4d72-9642-e741acb8ac6b.jpg',
}

export type ProviderId = 'claude' | 'kimi' | 'copilot'

export const PROVIDERS: { id: ProviderId; name: string; plan: string; auth: string; model: string }[] = [
  { id: 'claude', name: 'Claude', plan: 'Pro or Max subscription', auth: 'Sign in with Anthropic', model: 'claude-sonnet' },
  { id: 'kimi', name: 'Kimi', plan: 'Moonshot API key', auth: 'Paste a key', model: 'kimi-k2' },
  { id: 'copilot', name: 'GitHub Copilot', plan: 'Individual or Business seat', auth: 'Uses your gh login', model: 'copilot-gpt' },
]

export const providerName = (id: string) => PROVIDERS.find((p) => p.id === id)?.name ?? 'Claude'
export const providerModel = (id: string) => PROVIDERS.find((p) => p.id === id)?.model ?? 'claude-sonnet'
