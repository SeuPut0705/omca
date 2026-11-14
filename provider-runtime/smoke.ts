import { mkdir, rm, stat } from 'node:fs/promises';
import { isAbsolute } from 'node:path';

const [helper, data] = process.argv.slice(2);
function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
if (!helper || !data || !isAbsolute(helper) || !isAbsolute(data)) throw new Error('Usage: bun smoke.ts /absolute/helper /absolute/new-data-dir');
await mkdir(data, { recursive: false, mode: 0o700 });
async function request(command: Record<string, unknown>): Promise<unknown> {
  const child = Bun.spawn([helper], { env: { PATH: process.env.PATH, SystemRoot: process.env.SystemRoot, OMCA_PROVIDER_DATA_DIR: data }, stdin: 'pipe', stdout: 'pipe', stderr: 'pipe' });
  const timer = setTimeout(() => child.kill(), 30_000);
  const errors = new Response(child.stderr).text();
  child.stdin.write(JSON.stringify({ id: 'smoke', version: 1, ...command }) + '\n');
  child.stdin.flush();
  try {
    const reader = child.stdout.getReader();
    let pending = '';
    while (true) {
      const chunk = await reader.read();
      if (chunk.done) throw new Error(`Runtime exited without a result: ${(await errors).slice(0, 4096)}`);
      pending += new TextDecoder().decode(chunk.value, { stream: true });
      let end: number;
      while ((end = pending.indexOf('\n')) !== -1) {
        const event: unknown = JSON.parse(pending.slice(0, end));
        pending = pending.slice(end + 1);
        if (!record(event) || event.id !== 'smoke') throw new Error('Invalid runtime identifier');
        if (event.kind === 'error') throw new Error(`${command.command}: ${typeof event.error === 'string' ? event.error : 'Runtime error'}`);
        if (event.kind === 'result') return event.data;
        throw new Error('Unexpected interactive event');
      }
    }
  } finally {
    clearTimeout(timer);
    child.kill();
    await child.exited;
  }
}
try {
  const snapshot = await request({ command: 'snapshot', refresh: false });
  if (!record(snapshot) || !Array.isArray(snapshot.providers) || !snapshot.providers.some(p => record(p) && p.id === 'openai') || snapshot.providers.some(p => !record(p) || !Array.isArray(p.accounts) || p.accounts.length !== 0)) throw new Error('Invalid empty account catalog');
  const apiOnly = snapshot.providers.find(provider => record(provider) && provider.id === 'abliteration');
  if (!record(apiOnly) || apiOnly.auth_kind !== 'api_key' || apiOnly.login_available !== false) throw new Error('API-key provider advertises a subscription login');
  const models = await request({ command: 'models', provider_id: 'openai' });
  if (!Array.isArray(models) || !models.some(m => record(m) && m.provider_id === 'openai' && typeof m.id === 'string')) throw new Error('Missing model catalog');
  const marker = 'omca-artifact-smoke-not-a-real-api-key';
  const saved = await request({ command: 'set_key', provider_id: 'openai', key: marker });
  if (!record(saved) || !Array.isArray(saved.providers) || JSON.stringify(saved).includes(marker)) throw new Error('Credential leaked in account snapshot');
  const provider = saved.providers.find(p => record(p) && p.id === 'openai');
  if (!record(provider) || !Array.isArray(provider.accounts) || provider.accounts.length !== 1) throw new Error('API key account was not saved');
  const account: unknown = provider.accounts[0];
  if (!record(account) || !Number.isSafeInteger(account.id)) throw new Error('Missing durable account ID');
  const selected = await request({ command: 'select_account', provider_id: 'openai', credential_id: account.id });
  if (!record(selected) || !Array.isArray(selected.providers) || !selected.providers.some(p => record(p) && p.id === 'openai' && Array.isArray(p.accounts) && p.accounts.some(a => record(a) && a.id === account.id && a.selected === true))) throw new Error('Account selection did not persist');
  const removed = await request({ command: 'remove_account', provider_id: 'openai', credential_id: account.id });
  if (!record(removed) || !Array.isArray(removed.providers) || removed.providers.some(p => record(p) && p.id === 'openai' && Array.isArray(p.accounts) && p.accounts.length !== 0)) throw new Error('Account removal did not persist');
  if (process.platform !== 'win32' && ((await stat(data)).mode & 0o077) !== 0) throw new Error('Account directory is not private');
  console.log(`Provider artifact smoke passed: ${snapshot.providers.length} providers; ${models.length} OpenAI models; isolated key save/select/remove; no credential leak or account import.`);
} finally {
  await rm(data, { recursive: true, force: true });
}
