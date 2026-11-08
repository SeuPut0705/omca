<p align="center">
  <img src="assets/app-icon/craft-anything-1024.png" width="128" height="128" alt="OMCA icon">
</p>

<h1 align="center">OMCA</h1>

<p align="center">
  See how any website, app or game works, then have a coding agent build your own version.<br>
  A modified version of <a href="https://github.com/storytold/craft-anything">Craft Anything</a>.
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
- No GitHub Actions CI; see [CI (local)](#ci-local).
- Internal crate and binary names (`craft_anything_*`, `craft-anything`, `craft-anything-cli`)
  and `CRAFT_ANYTHING_*` environment variables are unchanged, to keep upstream merges simple.

## Platforms and languages

Releases are built for **Apple silicon Macs** (`aarch64`) and **Windows x64** only. The
interface is in **Korean** and **English**; release builds embed craft-fonts' Korean and symbol
fonts (pinned in `packaging/env.sh`).

## Build

Requires Rust 1.97 or newer.

```sh
cargo run --release -p craft-anything                # the desktop app
cargo install --path apps/craft-anything-cli         # the CLI, as `craft-anything-cli`
craft-anything-cli doctor
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

```sh
git fetch upstream
git merge upstream/main
```

## License

MIT OR Apache-2.0, as upstream. See [NOTICE](NOTICE), [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE) and [ATTRIBUTION.md](ATTRIBUTION.md). OMCA is not made,
sponsored or endorsed by the ArtCraft Team.
