import { readdir, readFile, mkdir, copyFile, writeFile } from 'node:fs/promises';
import { dirname, isAbsolute, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const option = (name: string) => { const index = args.indexOf(name); return index < 0 ? undefined : args[index + 1]; };
const target = option('--target');
const out = option('--out') ?? join(root, 'dist', target === 'bun-windows-x64' ? 'omca-provider-runtime.exe' : 'omca-provider-runtime');
if ((target !== 'bun-darwin-arm64' && target !== 'bun-windows-x64') || !out || !isAbsolute(out)) throw new Error('Usage: bun build.ts --target bun-darwin-arm64|bun-windows-x64 --out /absolute/path');
const platform = target === 'bun-darwin-arm64' ? 'darwin-arm64' : 'win32-x64';
const nativeDir = join(root, 'node_modules', '@oh-my-pi', `pi-natives-${platform}`);
const nativeFiles = (await readdir(nativeDir)).filter(name => name.endsWith('.node'));
if (!nativeFiles.length) throw new Error(`Missing native addon package for ${platform}; install target optional dependencies`);
await mkdir(dirname(out), { recursive: true });
const compile = Bun.spawn(['bun', 'build', '--compile', `--target=${target}`, join(root, 'src', 'main.ts'), '--outfile', out], { cwd: root, stdout: 'inherit', stderr: 'inherit' });
if (await compile.exited !== 0) throw new Error('Compilation failed');
const assets = join(dirname(out), 'provider-runtime-assets', '18.20.0');
await mkdir(assets, { recursive: true });
for (const file of nativeFiles) await copyFile(join(nativeDir, file), join(assets, file));
const licenses: string[] = [];
const visited = new Set<string>();
async function collect(name: string, parent: string) {
  let base = parent;
  let path: string;
  let source: string;
  while (true) {
    path = join(base, 'node_modules', name);
    try { source = await readFile(join(path, 'package.json'), 'utf8'); break; }
    catch (error) {
      if (!(error instanceof Error) || !('code' in error) || error.code !== 'ENOENT') throw error;
      const next = dirname(base);
      if (next === base) throw new Error(`Missing production dependency ${name}`);
      base = next;
    }
  }
  if (visited.has(path)) return;
  visited.add(path);
  const manifest: unknown = JSON.parse(source);
  if (!manifest || typeof manifest !== 'object' || !('name' in manifest) || !('version' in manifest)) throw new Error(`Invalid package manifest: ${path}`);
  const files = (await readdir(path)).filter(file => /^(licen[sc]e|(?:third[-_ ]party[-_ ])?notices?)(?:\..*)?$/i.test(file));
  if (!files.some(file => /^licen[sc]e/i.test(file))) throw new Error(`Missing license text for ${name}`);
  for (const file of files) licenses.push(`${name}@${manifest.version} ${file}\n${await readFile(join(path, file), 'utf8')}`);
  for (const key of ['dependencies', 'optionalDependencies'] as const) {
    if (!(key in manifest)) continue;
    const dependencies: unknown = Reflect.get(manifest, key);
    if (!dependencies || typeof dependencies !== 'object' || Array.isArray(dependencies)) throw new Error(`Invalid dependencies: ${name}`);
    for (const dependency of Object.keys(dependencies)) {
      if (dependency.startsWith('@oh-my-pi/pi-natives-') && dependency !== `@oh-my-pi/pi-natives-${platform}`) continue;
      await collect(dependency, path);
    }
  }
}
const packageJson: unknown = JSON.parse(await readFile(join(root, 'package.json'), 'utf8'));
if (!packageJson || typeof packageJson !== 'object' || !('dependencies' in packageJson) || !packageJson.dependencies || typeof packageJson.dependencies !== 'object') throw new Error('Missing production dependencies');
for (const name of Object.keys(packageJson.dependencies)) await collect(name, root);
const bunLicense = process.env.BUN_LICENSE_FILE ?? join(root, 'licenses', 'BUN-LICENSE.md');
if (!isAbsolute(bunLicense)) throw new Error('BUN_LICENSE_FILE must be an absolute path');
licenses.unshift(`Bun runtime\n${await readFile(bunLicense, 'utf8')}`);
for (const name of await readdir(join(root, 'licenses'))) {
  const path = join(root, 'licenses', name);
  if (path !== bunLicense) licenses.push(`${name}\n${await readFile(path, 'utf8')}`);
}
await writeFile(join(dirname(out), 'PROVIDER-RUNTIME-LICENSES.txt'), licenses.join('\n\n-----\n\n'));
