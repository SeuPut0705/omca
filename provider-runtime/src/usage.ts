import { resolveUsedFraction, type AuthStorage, type StoredAuthCredential } from '@oh-my-pi/pi-ai';

export interface AccountUsage {
  limits: { id: string; label: string; used_fraction?: number; used?: number; limit?: number; unit: string; window_id?: string; resets_at?: number }[];
  fetched_at: number;
  credits?: string;
}

export async function accountUsage(storage: AuthStorage, provider: string, row: StoredAuthCredential, signal: AbortSignal) {
  const usageProvider = storage.usage.providerFor(provider);
  if (!usageProvider) throw new Error('Usage not supported');
  const credential = row.credential;
  let usageCredential;
  if (credential.type === 'oauth') {
    const access = await storage.oauth.accessById(provider, row.id, { signal });
    if (!access || !access.ok || access.credentialId !== row.id) throw new Error('Account unavailable');
    usageCredential = { ...credential, ...access, type: 'oauth' as const, refreshToken: credential.refresh, expiresAt: credential.expires };
  } else usageCredential = { type: 'api_key' as const, apiKey: credential.key };
  const params = { provider, credential: usageCredential, signal };
  if (usageProvider.supports && !usageProvider.supports(params)) throw new Error('Usage not supported');
  const report = await usageProvider.fetchUsage(params, { fetch });
  if (!report) throw new Error('Usage unavailable');
  return {
    limits: report.limits.map(l => ({ id: l.id, label: [l.label, l.scope.modelId, l.scope.tier].filter(Boolean).join(' · '), used_fraction: resolveUsedFraction(l), used: l.amount.used, limit: l.amount.limit, unit: l.amount.unit, window_id: l.window?.id ?? l.scope.windowId, resets_at: l.window?.resetsAt === undefined ? undefined : Math.floor(l.window.resetsAt / 1000) })),
    fetched_at: Math.floor(report.fetchedAt / 1000),
    credits: report.resetCredits ? `${report.resetCredits.availableCount} saved resets` : undefined,
  };
}
