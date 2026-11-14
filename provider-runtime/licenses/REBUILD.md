# Bundled runtime source and rebuilding

OMCA's account helper is built with Bun 1.4.4. Its unmodified upstream source is available at:
https://github.com/oven-sh/bun/tree/bun-v1.4.4

Bun's JavaScriptCore/WebKit sources at that version are pinned to:
https://github.com/oven-sh/WebKit/tree/d6375c72affe0f70222cdcb45012b10f0bf3a9d5

The LGPL-2.1 license text is included in LGPL-2.1.txt. Bun's upstream license overview and its linked-library notices are reproduced in BUN-LICENSE.md.

To replace or modify the linked JavaScriptCore, follow Bun's upstream build instructions and the relinking steps in BUN-LICENSE.md. OMCA's complete helper source, dependency lockfile, and build script are in this release's `provider-runtime/` directory at https://github.com/seuput-inc/omca. Use the rebuilt Bun to run `bun install --frozen-lockfile` and `bun build.ts --target bun-darwin-arm64 --out /absolute/path/omca-provider-runtime` (or `bun-windows-x64` on Windows). Replace the helper and its adjacent `provider-runtime-assets` folder together. No signing key or private credential is required to rebuild the helper. A replacement macOS executable must be signed locally before use.
