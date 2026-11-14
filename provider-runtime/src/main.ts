import { existsSync } from 'node:fs';
import { createRequire } from 'node:module';
import { basename, dirname, join } from 'node:path';

const executableDir = dirname(process.execPath);
const appBundle = process.platform === 'darwin' && basename(executableDir) === 'MacOS'
  && basename(dirname(executableDir)) === 'Contents' && dirname(dirname(executableDir)).endsWith('.app');
const assets = join(appBundle ? join(executableDir, '..', 'Resources') : executableDir, 'provider-runtime-assets');
  const tag = `${process.platform}-${process.arch}`;
  if (tag !== 'darwin-arm64' && tag !== 'win32-x64') throw new Error('Unsupported provider runtime platform');
  const versionDir = join(assets, '18.20.0');
  const filename = tag === 'win32-x64' ? `pi_natives.${tag}-baseline.node` : `pi_natives.${tag}.node`;
  const addon = join(versionDir, filename);
  if (!existsSync(addon)) throw new Error('Provider runtime assets missing');
  const native = createRequire(import.meta.url)(addon) as { __piNativesBuildVersion?: () => string };
  if (native.__piNativesBuildVersion?.() !== '18.20.0') throw new Error('Provider runtime asset version mismatch');
  process.env.PI_NATIVES_DIR = assets;
  process.env.PI_COMPILED = '1';
  process.env.PI_NATIVE_VARIANT = 'baseline';
// The SDK native loader must observe the verified asset path before evaluation.
await import('./runtime');
