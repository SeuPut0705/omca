import { expect, test } from 'bun:test';
import type { StoredAuthCredential } from '@oh-my-pi/pi-ai';
import { canActOnAccount, modelList, supportsApiKey } from '../src/catalog';

test('hosted API key capability comes from catalog credential metadata', () => {
  expect(supportsApiKey('openai')).toBe(true);
  expect(supportsApiKey('google')).toBe(true);
  expect(supportsApiKey('anthropic')).toBe(true);
  expect(supportsApiKey('not-a-provider')).toBe(false);
});

test('model alias retains the requested provider id', () => {
  const models = modelList('openai-codex-device');
  expect(models.length).toBeGreaterThan(0);
  expect(models.every(model => model.provider_id === 'openai-codex-device')).toBe(true);
});

test('disabled credential can be removed but not selected', () => {
  const rows: StoredAuthCredential[] = [{ id: 13, provider: 'openai', credential: { type: 'api_key', key: 'test-only' }, disabledCause: 'disabled' }];
  expect(canActOnAccount(rows, 13, 'remove_account')).toBe(true);
  expect(canActOnAccount(rows, 13, 'select_account')).toBe(false);
  expect(canActOnAccount(rows, 14, 'remove_account')).toBe(false);
});
