import { expect, test } from 'bun:test';
import { AuthStorage } from '@oh-my-pi/pi-ai';
import { assertDirectTransport, directCredentialResolver, supportsDirectLogin } from '../src/direct-auth';

const initial = { error: undefined, lastChance: false };

test('only metadata-backed interactive login flows are offered', () => {
  expect(supportsDirectLogin('openrouter')).toBe(true);
  expect(supportsDirectLogin('openai-codex')).toBe(true);
  expect(supportsDirectLogin('openai-codex-device')).toBe(true);
  for (const provider of ['github-copilot', 'cursor', 'snowflake', 'mistral', 'kilo', 'devin-windsurf', 'alibaba-coding-plan', 'alibaba-token-plan', 'cloudflare-ai-gateway', 'cloudflare-workers-ai', 'xiaomi']) {
    expect(supportsDirectLogin(provider)).toBe(true);
  }
  expect(supportsDirectLogin('devin-cli')).toBe(false);
  expect(supportsDirectLogin('perplexity')).toBe(false);
  expect(supportsDirectLogin('not-a-provider')).toBe(false);
  expect(() => assertDirectTransport('amazon-bedrock', 'bedrock-converse-stream')).toThrow();
  expect(() => assertDirectTransport('bedrock-mantle', 'openai-responses')).toThrow();
});

test('stored account selection never resolves an ambient key or a sibling account', async () => {
  const storage = await AuthStorage.create(':memory:');
  const oldKey = process.env.OPENAI_API_KEY;
  process.env.OPENAI_API_KEY = 'ambient-fake-key';
  try {
    await storage.credentials.reload();
    expect(() => directCredentialResolver(storage, 'openai', undefined)).toThrow('No connected account');
    await storage.credentials.upsert('openai', { type: 'api_key', key: 'direct-fake-key', source: 'login' });
    const row = storage.credentials.list('openai')[0];
    expect(row).toBeDefined();
    expect(await directCredentialResolver(storage, 'openai', row!.id)(initial)).toEqual({ apiKey: 'direct-fake-key', credentialId: row!.id });
    expect(() => directCredentialResolver(storage, 'openai', row!.id + 100)).toThrow('Selected account unavailable');
  } finally {
    if (oldKey === undefined) delete process.env.OPENAI_API_KEY;
    else process.env.OPENAI_API_KEY = oldKey;
    storage.close();
  }
});

test('OAuth selection resolves the exact stored account and never rotates to a sibling', async () => {
  const storage = await AuthStorage.create(':memory:');
  try {
    await storage.credentials.reload();
    await storage.credentials.upsert('openai-codex', { type: 'oauth', access: 'fake-token-one', refresh: 'fake-refresh-one', expires: Date.now() + 3600000, email: 'one@example.test' });
    await storage.credentials.upsert('openai-codex', { type: 'oauth', access: 'fake-token-two', refresh: 'fake-refresh-two', expires: Date.now() + 3600000, email: 'two@example.test' });
    const rows = storage.credentials.list('openai-codex');
    const row = rows.find(item => item.credential.type === 'oauth' && item.credential.email === 'two@example.test');
    expect(rows).toHaveLength(2);
    expect(row).toBeDefined();
    const resolve = directCredentialResolver(storage, 'openai-codex', row!.id);
    expect(await resolve(initial)).toMatchObject({ apiKey: 'fake-token-two', credentialId: row!.id });
    expect(await resolve({ error: new Error('failure'), lastChance: true })).toBeUndefined();
  } finally {
    storage.close();
  }
});
