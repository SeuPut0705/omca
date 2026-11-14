import { createHash, randomUUID } from 'node:crypto';
import { readFile, rename, unlink, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import type { AccountUsage } from './usage';

export class Observations {
  constructor(private readonly directory: string) {}

  private path(kind: string, provider: string, id?: number): string {
    const key = createHash('sha256').update(JSON.stringify([kind, provider, id ?? null])).digest('hex');
    return join(this.directory, `observation-${key}.json`);
  }

  private async read(path: string): Promise<unknown> {
    try { return JSON.parse(await readFile(path, 'utf8')); }
    catch (error) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT') return undefined;
      throw error;
    }
  }

  private async write(path: string, data: object): Promise<void> {
    const temporary = join(this.directory, `observation-${process.pid}-${randomUUID()}.tmp`);
    await writeFile(temporary, JSON.stringify(data), { mode: 0o600, flag: 'wx' });
    try { await rename(temporary, path); }
    catch (error) { await unlink(temporary).catch(() => {}); throw error; }
  }

  async usage(provider: string, id: number): Promise<AccountUsage | undefined> {
    const data = await this.read(this.path('usage', provider, id));
    if (!data || typeof data !== 'object' || !('provider' in data) || data.provider !== provider || !('id' in data) || data.id !== id || !('usage' in data)) return undefined;
    const usage = data.usage;
    if (!usage || typeof usage !== 'object' || !('limits' in usage) || !Array.isArray(usage.limits) || !('fetched_at' in usage) || typeof usage.fetched_at !== 'number') return undefined;
    return usage as AccountUsage;
  }

  async saveUsage(provider: string, id: number, usage: AccountUsage): Promise<void> {
    await this.write(this.path('usage', provider, id), { provider, id, usage });
  }

  async lastUsed(provider: string): Promise<number | undefined> {
    const data = await this.read(this.path('last-used', provider));
    if (!data || typeof data !== 'object' || !('provider' in data) || data.provider !== provider || !('id' in data) || !Number.isSafeInteger(data.id) || Number(data.id) <= 0) return undefined;
    return Number(data.id);
  }

  async saveLastUsed(provider: string, id: number): Promise<void> {
    await this.write(this.path('last-used', provider), { provider, id });
  }

  async removeUsage(provider: string, id: number): Promise<void> {
    try { await unlink(this.path('usage', provider, id)); }
    catch (error) { if ((error as NodeJS.ErrnoException).code !== 'ENOENT') throw error; }
  }
}
