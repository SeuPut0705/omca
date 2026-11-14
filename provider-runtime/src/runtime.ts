import { chmod, mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { isAbsolute, join } from 'node:path';
import { AuthStorage, assertProviderEnabled, getOAuthCredentialProvider, getOAuthProviders, loginRiskWarning, providerPauseReason, PROVIDER_REGISTRY, streamSimple } from '@oh-my-pi/pi-ai';
import { getBundledModel, getBundledProviders, type Effort } from '@oh-my-pi/pi-catalog';
import { authPolicyFor } from '@oh-my-pi/pi-catalog/compat/auth';
import { fromAssistant, toContext, type WireRequest } from './messages';
import { accountUsage } from './usage';
import { Observations } from './observations';
import { canActOnAccount, modelList, supportsApiKey } from './catalog';
import { assertDirectTransport, DirectAuthError, directCredentialResolver, supportsDirectLogin } from './direct-auth';

const dir = process.env.OMCA_PROVIDER_DATA_DIR;
if (!dir || !isAbsolute(dir)) process.exit(1);
process.umask(0o077);
await mkdir(dir, { recursive: true, mode: 0o700 });
await chmod(dir, 0o700);
const storage = await AuthStorage.create(join(dir, 'agent.db'));
await storage.credentials.reload();
const observations = new Observations(dir);
const selectionPath = join(dir, 'selection.json');
const abort = new AbortController();
let requestId = '';
let finished = false;
let pending: { id: string; resolve: (value: string) => void; reject: (error: Error) => void; confirm: boolean } | undefined;
let sequence = 0;
const emit = (kind: string, fields: object) => process.stdout.write(JSON.stringify({ id: requestId, kind, ...fields }) + '\n');
const prompt = (event: Record<string, unknown>, confirm = false): Promise<string> => {
  const { promise, resolve, reject } = Promise.withResolvers<string>();
  if (pending || abort.signal.aborted) reject(new Error('Cancelled'));
  else {
    const id = String(++sequence);
    pending = { id, resolve, reject, confirm };
    emit('login', { event: { ...event, request_id: id } });
  }
  return promise;
};
function cancel() { abort.abort(); pending?.reject(new Error('Cancelled')); pending = undefined; }
async function selections(): Promise<Record<string, number>> {
  try {
    const parsed: unknown = JSON.parse(await readFile(selectionPath, 'utf8'));
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('Invalid selection');
    const result: Record<string, number> = {};
    for (const [key, value] of Object.entries(parsed)) if (Number.isSafeInteger(value) && Number(value) > 0) result[key] = Number(value);
    return result;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') return {};
    throw error;
  }
}
async function saveSelections(value: Record<string, number>) {
  const temporary = `${selectionPath}.${process.pid}.${crypto.randomUUID()}.tmp`;
  await writeFile(temporary, JSON.stringify(value), { mode: 0o600, flag: 'wx' });
  await rename(temporary, selectionPath);
}
const logins = getOAuthProviders();
function checkProvider(id: string) {
  if (!getBundledProviders().some(provider => provider === id) && !logins.some(provider => provider.id === id)) throw new Error('Unknown provider');
  assertProviderEnabled(id);
}

async function snapshot(refresh: boolean) {
  const selected = await selections();
  const ids = new Set<string>([...getBundledProviders(), ...logins.filter(p => supportsDirectLogin(p.id)).map(p => p.id)]);
  const providers = [];
  for (const id of ids) {
    const login = logins.find(p => p.id === id);
    const storedProvider = getOAuthCredentialProvider(id);
    const rows = storage.credentials.list(storedProvider);
    const usageProvider = storage.usage.providerFor(storedProvider);
    const lastUsed = await observations.lastUsed(storedProvider);
    const accounts = [];
    for (const row of rows) {
      const credential = row.credential;
      const cached = await observations.usage(storedProvider, row.id);
      let limits: object[] = cached?.limits ?? [];
      let fetched_at: number | undefined = cached?.fetched_at;
      let credits: string | undefined = cached?.credits;
      let usage_error: string | undefined;
      if (refresh && usageProvider && !row.disabledCause) {
        try {
          const usage = await accountUsage(storage, storedProvider, row, abort.signal);
          await observations.saveUsage(storedProvider, row.id, usage);
          ({ limits, fetched_at, credits } = usage);
        } catch { usage_error = abort.signal.aborted ? 'Cancelled' : cached ? 'Refresh failed; showing cached usage' : 'Usage unavailable'; }
      } else usage_error = cached ? 'Showing cached usage' : usageProvider ? 'Usage not refreshed' : 'Usage not supported';
      accounts.push({ id: row.id, label: credential.type === 'oauth' ? (credential.email || credential.orgName || credential.accountId || `Account ${row.id}`) : `API key ${row.id}`, detail: credential.type === 'oauth' ? (credential.orgName || credential.orgId) : undefined, kind: credential.type, selected: selected[storedProvider] === row.id, in_use: lastUsed === row.id, state: row.disabledCause ? 'disabled' : 'available', limits, fetched_at, credits, usage_error });
    }
    const definition = PROVIDER_REGISTRY.find(p => p.id === id);
    providers.push({ id, name: login?.name ?? definition?.name ?? id, login_available: !!login?.available && supportsDirectLogin(id), authenticated: accounts.some(a => a.state === 'available'), disabled_reason: providerPauseReason(id), risk_warning: loginRiskWarning(id), auth_kind: supportsApiKey(id) ? 'api_key' : supportsDirectLogin(id) ? 'oauth' : undefined, accounts });
  }
  return { providers, models: [] };
}

async function execute(req: Record<string, unknown>) {
  const command = req.command;
  const id = typeof req.provider_id === 'string' ? req.provider_id : '';
  if (command === 'snapshot') return snapshot(req.refresh === true);
  if (typeof command !== 'string' || !id) throw new Error('Invalid request');
  checkProvider(id);
  const provider = getOAuthCredentialProvider(id);
  if (command === 'models') return modelList(id);
  if (command === 'login') {
    const warning = loginRiskWarning(id);
    if (!supportsDirectLogin(id) || !logins.some(p => p.id === id && p.available)) throw new Error('Login unavailable');
    await storage.oauth.login(id, {
      signal: abort.signal,
      onAuth: info => emit('login', { event: { type: 'open_url', url: info.url, instructions: info.instructions } }),
      onProgress: message => emit('login', { event: { type: 'progress', message } }),
      onPrompt: async p => warning && p.message.includes(warning)
        ? prompt({ type: 'confirm', title: 'Account risk', message: warning }, true)
        : prompt({ type: 'input', title: p.message, placeholder: p.placeholder, secret: !!p.secret }),
    });
    return snapshot(false);
  }
  if (command === 'set_key') {
    if (typeof req.key !== 'string' || !req.key.trim() || !supportsApiKey(id)) throw new Error('Invalid API key request');
    const key = req.key;
    req.key = undefined;
    const warning = loginRiskWarning(id);
    if (authPolicyFor(id)?.login?.kind === 'api-key') {
      await storage.oauth.login(id, {
        signal: abort.signal,
        onAuth: () => {},
        onPrompt: async p => warning && p.message.includes(warning)
          ? prompt({ type: 'confirm', title: 'Account risk', message: warning }, true)
          : key,
      });
    } else {
      if (warning && await prompt({ type: 'confirm', title: 'Account risk', message: warning }, true) !== 'yes') throw new Error('Cancelled');
      await storage.credentials.upsert(provider, { type: 'api_key', key, source: 'login' });
    }
    return snapshot(false);
  }
  if (command === 'select_account' || command === 'remove_account') {
    const selected = await selections();
    const credentialId = req.credential_id;
    if (command === 'select_account' && credentialId === null) delete selected[provider];
    else {
      if (!canActOnAccount(storage.credentials.list(provider), credentialId, command)) throw new Error('Account not found');
      if (command === 'select_account') selected[provider] = Number(credentialId);
      else {
        if (!await storage.credentials.removeById(provider, Number(credentialId))) throw new Error('Account not found');
        await observations.removeUsage(provider, Number(credentialId));
        if (selected[provider] === credentialId) delete selected[provider];
      }
    }
    await saveSelections(selected);
    return snapshot(false);
  }
  if (command === 'turn') {
    if (typeof req.model_id !== 'string' || typeof req.session_id !== 'string' || !req.request || typeof req.request !== 'object') throw new Error('Invalid turn');
    const model = getBundledModel(provider, req.model_id);
    if (!model) throw new Error('Unknown model');
    assertDirectTransport(id, model.api);
    assertDirectTransport(provider, model.api);
    const selection = (await selections())[provider];
    const apiKey = directCredentialResolver(storage, provider, selection);
    const request = req.request as WireRequest;
    const effort = request.effort;
    if (!['none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max'].includes(effort)) throw new Error('Invalid effort');
    const stream = streamSimple(model, toContext(request, model), { apiKey, signal: abort.signal, maxTokens: request.max_tokens, reasoning: effort === 'none' ? undefined : effort as Effort, disableReasoning: effort === 'none' });
    for await (const event of stream) {
      if (event.type === 'text_delta') emit('delta', { delta: { type: 'text', text: event.delta } });
      else if (event.type === 'thinking_delta') emit('delta', { delta: { type: 'thinking', text: event.delta } });
    }
    const result = await stream.result();
    const converted = fromAssistant(result);
    if (result.credentialId !== undefined && storage.credentials.list(provider).some(item => item.id === result.credentialId)) await observations.saveLastUsed(provider, result.credentialId);
    return converted;
  }
  throw new Error('Unknown command');
}

let started = false;
let buffered = Buffer.alloc(0);
function handleLine(line: Buffer) {
  let data: Record<string, unknown>;
  try {
    const value: unknown = JSON.parse(line.toString('utf8'));
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid request');
    data = value as Record<string, unknown>;
  } catch { cancel(); return; }
  if (!started) {
    if (typeof data.id !== 'string' || data.version !== 1) { process.stdin.pause(); return; }
    started = true; requestId = data.id;
    void execute(data).then(result => { if (!finished) { finished = true; emit('result', { data: result }); } }, error => { if (!finished) { finished = true; emit('error', { error: abort.signal.aborted ? 'Cancelled' : error instanceof DirectAuthError ? error.message : 'Provider operation failed' }); } }).finally(() => { storage.close(); process.stdin.pause(); });
  } else if (data.command === 'cancel') cancel();
  else if (data.command === 'respond' && pending && data.request_id === pending.id) {
    const active = pending; pending = undefined;
    if (data.cancelled === true) active.reject(new Error('Cancelled'));
    else active.resolve(active.confirm ? (data.accepted === true ? 'yes' : 'no') : typeof data.value === 'string' ? data.value : '');
  }
}
process.stdin.on('data', (chunk: Buffer) => {
  let offset = 0;
  while (offset < chunk.length) {
    const end = chunk.indexOf(10, offset);
    const part = chunk.subarray(offset, end < 0 ? chunk.length : end);
    if (buffered.length + part.length > 16 * 1024 * 1024) {
      cancel();
      if (started && !finished) { finished = true; emit('error', { error: 'Request too large' }); }
      process.stdin.pause();
      return;
    }
    buffered = Buffer.concat([buffered, part]);
    if (end < 0) break;
    handleLine(buffered);
    buffered = Buffer.alloc(0);
    offset = end + 1;
  }
});
process.stdin.on('end', () => { if (!finished) cancel(); });
