# Attribution

Third-party material bundled with Craft Anything, file by file. Rust crates are pulled from
crates.io under their own licences (see `Cargo.lock`) and are not listed here.

## Fonts (`assets/fonts/`)

All fonts are under the SIL Open Font License 1.1. The fonts are embedded unmodified; the
licence requires that the font software is not sold by itself and that reserved font names are
not used for modified versions.

| File | Font | Copyright | Source | Licence |
|---|---|---|---|---|
| `Pretendard-Regular.ttf`, `Pretendard-Medium.ttf`, `Pretendard-SemiBold.ttf` | Pretendard 1.3.9 (the interface: body text, labels and buttons, headings; Hangul and Latin) | © 2021 Kil Hyung-jin, with Reserved Font Name Pretendard | https://github.com/orioncactus/pretendard (release v1.3.9, `public/static/alternative/`) | `OFL-Pretendard.txt` |
| `DMSans-Regular.ttf`, `DMSans-Medium.ttf`, `DMSans-Bold.ttf` | DM Sans 1.200 (the macOS installer's background art only) | © 2014-2017 Indian Type Foundry, © 2019 Google LLC | https://github.com/googlefonts/dm-fonts (`Sans/Exports/`, commit 23aafb3) | `OFL-DMSans.txt` |
| `JetBrainsMono-Regular.ttf` | JetBrains Mono (code views) | © 2020 The JetBrains Mono Project Authors | https://github.com/JetBrains/JetBrainsMono | `OFL-JetBrainsMono.txt` |
| `GeistMono-Regular.ttf`, `GeistMono-Medium.ttf` | Geist Mono (labels and buttons) | © 2024 The Geist Project Authors | https://github.com/vercel/geist-font (`fonts/GeistMono/ttf/`) | `OFL-GeistMono.txt` |

## egui default fonts (`epaint_default_fonts` crate)

egui embeds these fonts through the `epaint_default_fonts` crate (crate licence:
`(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`). They are embedded unmodified; the licence
texts are in the crate's `fonts/` folder.

| File | Font | Copyright | Licence |
|---|---|---|---|
| `Ubuntu-Light.ttf` | Ubuntu Light (egui's default proportional font) | © 2011 Canonical Ltd. | Ubuntu Font Licence 1.0, `UFL.txt` |
| `NotoEmoji-Regular.ttf` | Noto Emoji | © 2013 Google Inc. | SIL OFL 1.1, `OFL.txt` |
| `Hack-Regular.ttf` | Hack (egui's default monospace font) | © 2018 Source Foundry Authors; Bitstream Vera © 2003 Bitstream, Inc.; DejaVu changes public domain | MIT and Bitstream Vera License, `Hack-Regular.txt` |
| `emoji-icon-font.ttf` | emoji-icon-font | © 2014 John Slegers | MIT, `emoji-icon-font-mit-license.txt` |

## Icons (`assets/icons/`)

| Files | Set | Copyright | Source | Licence |
|---|---|---|---|---|
| `*.svg` | Lucide | © Lucide Icons and Contributors | https://lucide.dev | ISC, `LICENSE-lucide.txt` |

## App icon (`assets/app-icon/`)

Made for OMCA; MIT OR Apache-2.0. It replaces the upstream raven icon, which is not licensed to
forks. See `assets/app-icon/LICENSE.txt`.

## README artwork (`assets/omca-hero.webp`)

Generated for OMCA with OpenAI's image model, using OMCA's app icon as a brand reference.
Distributed under MIT OR Apache-2.0 with the project.

## Provider account runtime

The bundled `omca-provider-runtime` depends on the public `@oh-my-pi/pi-ai` and
`@oh-my-pi/pi-catalog` packages, pinned to 18.20.0, under MIT.
Source: https://github.com/can1357/oh-my-pi.
Each release carries `PROVIDER-RUNTIME-LICENSES.txt` for Bun and the bundled dependency notices.
OMCA's bridge implementation is original; no desktop UI code or assets are reused.

## Optional build-time fonts

Builds made with `CRAFT_FONTS_DIR` (craft-fonts), releases among them, also embed the fonts in
that folder (all OFL-1.1). Their packages ship each one's licence as `OFL-<font>.txt`; craft-fonts'
own `ATTRIBUTION.md` names their authors and pinned sources.
