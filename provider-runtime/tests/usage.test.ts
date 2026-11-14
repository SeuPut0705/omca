import { expect, test } from 'bun:test';
import type { AuthStorage, StoredAuthCredential, UsageProvider } from '@oh-my-pi/pi-ai';
import { accountUsage } from '../src/usage';

const row = (id: number): StoredAuthCredential => ({ id, provider: 'openai-codex', disabledCause: null, credential: { type: 'oauth', access: 'old', refresh: 'private', expires: 1, email: 'same@example.test' } });

test('each report uses the exact durable account, even with shared email', async () => {
  const seen: string[] = [];
  const provider: UsageProvider = { id: 'openai-codex', async fetchUsage(params) {
    seen.push(params.credential.accessToken ?? '');
    return { provider: 'openai-codex', fetchedAt: 1234, limits: [{ id: 'weekly', label: 'Weekly', scope: { provider: 'openai-codex', modelId: 'model-a' }, window: { id: '7d', label: 'Week', resetsAt: 5678 }, amount: { unit: 'requests', used: 12, limit: 10 } }] };
  } };
  // This narrow fake implements only the two public methods accountUsage consumes.
  const storage = { usage: { providerFor: () => provider }, oauth: { accessById: async (_provider: string, id: number) => ({ ok: true, credentialId: id, accessToken: `token-${id}`, apiKey: `token-${id}` }) } } as unknown as AuthStorage;
  const one = await accountUsage(storage, 'openai-codex', row(1), new AbortController().signal);
  const two = await accountUsage(storage, 'openai-codex', row(2), new AbortController().signal);
  expect(seen).toEqual(['token-1', 'token-2']);
  expect(one.limits[0]).toMatchObject({ used_fraction: 1.2, window_id: '7d', resets_at: 5, label: 'Weekly · model-a' });
  expect(two.fetched_at).toBe(1);
});

test('a mismatched credential cannot be assigned another account report', async () => {
  const storage = { usage: { providerFor: () => ({ fetchUsage: () => { throw new Error('must not fetch'); } }) }, oauth: { accessById: async () => ({ ok: true, credentialId: 2, accessToken: 'wrong' }) } } as unknown as AuthStorage;
  expect(accountUsage(storage, 'openai-codex', row(1), new AbortController().signal)).rejects.toThrow('Account unavailable');
});
