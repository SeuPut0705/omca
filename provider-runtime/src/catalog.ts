import { getBundledModels } from '@oh-my-pi/pi-catalog';
import { authPolicyFor } from '@oh-my-pi/pi-catalog/compat/auth';
import { providerEntry } from '@oh-my-pi/pi-catalog/compat/providers';
import { getOAuthCredentialProvider, type StoredAuthCredential } from '@oh-my-pi/pi-ai';

export function supportsApiKey(id: string): boolean {
  const policy = authPolicyFor(id);
  if (!policy || policy.disabled) return false;
  if (policy.login?.kind === 'api-key') return true;
  if (policy.apiKeyFormat !== 'bearer') return false;
  const names = [...(providerEntry(id)?.envVars ?? []), ...(policy.env && 'vars' in policy.env ? policy.env.vars : [])];
  return names.some(name => /API_KEY$/.test(name) && !policy.oauthTokenEnv?.includes(name));
}

export function modelList(id: string) {
  return getBundledModels(getOAuthCredentialProvider(id)).map(model => ({ provider_id: id, id: model.id, name: model.name, context_window: model.contextWindow, reasoning: model.reasoning }));
}

export function canActOnAccount(rows: readonly StoredAuthCredential[], id: unknown, action: 'select_account' | 'remove_account'): boolean {
  return Number.isSafeInteger(id) && rows.some(row => row.id === id && (action === 'remove_account' || !row.disabledCause));
}
