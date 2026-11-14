<p align="center">
  <img src="assets/omca-hero.webp" width="960" alt="OMCA — inspect software and build your own">
</p>

<p align="center">
  See how any website, app or game works, then have a coding agent build your own version.<br>
  A modified version of <a href="https://github.com/storytold/craft-anything">Craft Anything</a>.
</p>

<p align="center">
  <a href="https://github.com/seuput-inc/omca/releases/download/v0.4.2/craft-anything-0.4.2-macos-aarch64.dmg"><img src="https://img.shields.io/badge/Install-macOS%20Apple%20silicon-111827?style=for-the-badge&amp;logo=apple&amp;logoColor=white" alt="Install for macOS Apple silicon"></a>
  <a href="https://github.com/seuput-inc/omca/releases/download/v0.4.2/craft-anything-0.4.2-windows-x64.msi"><img src="https://img.shields.io/badge/Install-Windows%20x64-0078D4?style=for-the-badge" alt="Install for Windows x64"></a>
</p>
<p align="center">
  Published release: <a href="https://github.com/seuput-inc/omca/releases/tag/v0.4.2">v0.4.2</a>
  · <a href="https://github.com/seuput-inc/omca/releases/download/v0.4.2/craft-anything-0.4.2-windows-x64-portable.zip">Windows portable ZIP</a>
</p>

## What it is

OMCA studies a target (a website, desktop or phone app, game or file) with real inspection
tools, has a coding agent build a new project, compares the build with the original and refines
it: **Study → Build → Check → Ready**. Every tool is also a CLI command and an MCP tool.

The engine, workflow, Clean room / Deep modes and tool set come from upstream Craft Anything; see
[`docs/`](docs/) for how they work (those pages still use the upstream name).

## Differences from upstream

- Name, bundle identifier (`com.seuput.omca`) and app icon.
- Its own storage, so it runs beside an installed Craft Anything without sharing anything:

  | What | OMCA |
  |---|---|
  | Settings, logs, engine store (macOS) | `~/Library/Application Support/omca` |
  | Portable data folder | `OMCAData` next to the executable |
  | Keychain service | `com.seuput.omca` |
  | Projects | `~/Documents/OMCA` |
  | MCP server / skill name | `omca` (tools are `mcp__omca__*`) |

- Releases for Apple silicon Macs and Windows x64 only; the interface in Korean and English only.
- Its own version numbers, starting at 0.1.0, and its own `vX.Y.Z` tags; upstream's tags are not
  kept.
- No GitHub Actions CI; see [CI (local)](#ci-local).
- Internal crate and binary names (`craft_anything_*`, `craft-anything`, `craft-anything-cli`)
  and `CRAFT_ANYTHING_*` environment variables are unchanged, to keep upstream merges simple.

Settings → Models & accounts groups connected providers and their accounts, with sign-in,
account selection, model selection and provider-reported quota windows. The bundled provider
runtime uses the public omp SDK; its credentials stay in OMCA's private provider store, separate
from other apps. Provider-specific warnings and disabled-provider rules still apply.
API keys use a separate connection form. Legacy command-line agents, local/custom endpoints
and game assets are under Advanced.

## Platforms and languages

Releases are built for **Apple silicon Macs with macOS 13 or newer** (`aarch64`) and **Windows x64** only. The
interface is in **Korean** and **English**; release builds embed craft-fonts' Korean and symbol
fonts (pinned in `packaging/env.sh`).

## Build

Requires Rust 1.97 or newer.
Packaged account connections also need Bun 1.4.4 or newer at build time. Release packages include
the standalone helper, so users do not need Bun or an installed coding-agent runtime.

```sh
cargo run --release -p craft-anything                # the desktop app
cargo install --path apps/craft-anything-cli         # the CLI, as `craft-anything-cli`
craft-anything-cli doctor
```

To build the account helper for a source checkout (macOS):

```sh
export CARGO_TARGET_DIR="$PWD/target"
. packaging/env.sh
build_provider_runtime bun-darwin-arm64
export OMCA_PROVIDER_RUNTIME="$PROVIDER_RUNTIME_DIR/omca-provider-runtime"
cargo run -p craft-anything
```

Building with Claude Code needs `claude` on `PATH` (or in `~/.local/bin`) and signed in
(`claude auth login`).

## CI (local)

There is no GitHub Actions CI: the gate runs on your machine.

```sh
git config core.hooksPath .githooks   # once per clone: secrets check on commit, full gate on push
cargo xtask ci                        # the full gate by hand: fmt, clippy, tests, tool/schema checks
```

## Release

```sh
cargo xtask version set X.Y.Z && git commit -am "Release X.Y.Z"
packaging/release.sh
```

`packaging/release.sh` runs the gate, builds the macOS DMG and CLI zip locally
(`packaging/macos/package.sh`; ad-hoc signed unless `MACOS_SIGN_IDENTITY` is set, notarized with
`APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID`), tags `vX.Y.Z`, publishes the GitHub release and
starts `release-windows.yml`, the only Actions workflow (manual, never on push), which attaches the
Windows MSI and portable zip.

## Upstream

Integrated through upstream `8374ad7` (2026-11-14, Craft Anything 0.6.1), retaining
OMCA's identity, storage, Korean/English interface and Mac/Windows release targets.
OMCA 0.4.1 follows the locally installed 0.4.0 build; upstream version tags remain separate.

The upstream changes include parser and archive safety fixes, builder/session recovery,
settings-file preservation, website traffic recording and application update checks.

Settings → Updates and Help → Check for Updates use the same `seuput-inc/omca` release check
and recognize the existing `craft-anything-*` package names. Settings shows the installed version.
On macOS, use the Releases link to install manually:
the upstream installer only accepts the original app's bundle ID and signing team.
OMCA does not bypass those signature checks.


```sh
git config remote.upstream.tagOpt --no-tags   # once per clone: keep upstream's version tags out
git fetch upstream
git merge upstream/main
```

## License

MIT OR Apache-2.0, as upstream. See [NOTICE](NOTICE), [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE) and [ATTRIBUTION.md](ATTRIBUTION.md). OMCA is not made,
sponsored or endorsed by the ArtCraft Team.
