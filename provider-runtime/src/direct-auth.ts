import { authPolicyFor } from '@oh-my-pi/pi-catalog/compat/auth';
import type { ApiKeyResolver, AuthStorage, StoredAuthCredential } from '@oh-my-pi/pi-ai';

export class DirectAuthError extends Error {}

const directCustomLogins: Readonly<Record<string, string>> = {
  'alibaba-coding-plan': 'alibaba-coding-plan',
  'alibaba-token-plan': 'alibaba-token-plan',
  'cloudflare-ai-gateway': 'cloudflare-ai-gateway',
  'cloudflare-workers-ai': 'cloudflare-workers-ai',
  'cursor': 'cursor',
  'devin-windsurf': 'devin-windsurf',
  'github-copilot': 'github-copilot',
  'kilo': 'kilo',
  'mistral': 'mistral-vibe-sign-in',
  'openai-codex-device': 'openai-codex-device',
  'snowflake': 'snowflake',
  'xiaomi': 'xiaomi',
};

export function supportsDirectLogin(id: string): boolean {
  const policy = authPolicyFor(id);
  if (!policy || policy.disabled || !policy.login) return false;
  const login = policy.login;
  return login.kind === 'oauth-code' || login.kind === 'device-code' ||
    (login.kind === 'custom' && directCustomLogins[id] === login.hook);
}

export function assertDirectTransport(provider: string, api: string): void {
  if (authPolicyFor(provider)?.nativeAuthApis?.includes(api)) throw new DirectAuthError('Provider requires ambient authentication');
}

export function directCredentialResolver(storage: AuthStorage, provider: string, selection: number | undefined): ApiKeyResolver {
  const rows = storage.credentials.list(provider);
  const row: StoredAuthCredential | undefined = selection === undefined
    ? rows.find(item => !item.disabledCause)
    : rows.find(item => item.id === selection && !item.disabledCause);
  if (!row) throw new DirectAuthError(selection === undefined ? 'No connected account' : 'Selected account unavailable');
  return async ({ error, lastChance, signal }) => {
    if (lastChance) return undefined;
    if (row.credential.type === 'api_key') {
      const key = row.credential.key;
      if (!key || (provider === 'google-vertex' && (key.startsWith('<') || key === 'N/A'))) throw new DirectAuthError('Selected account unavailable');
      return error === undefined ? { apiKey: key, credentialId: row.id } : undefined;
    }
    const access = await storage.oauth.accessById(provider, row.id, { signal, forceRefresh: error !== undefined });
    if (!access || !access.ok || access.credentialId !== row.id || !access.apiKey) throw new DirectAuthError('Selected account unavailable');
    return { apiKey: access.apiKey, credentialId: row.id, oauthAccess: access.accessToken, oauthIdentity: access };
  };
}
