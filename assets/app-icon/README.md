# OMCA app icon

A glossy cream enamel ring around a glowing amber four-point spark, on deep indigo. MIT OR
Apache-2.0: see `LICENSE.txt`.

## Files

- `craft-anything-master.png`: the canonical artwork, a generated 1024 px full-bleed square.
- `craft-anything.svg`: its flat vector twin (indigo `#272b6e`, cream `#f2f1ee`, amber
  `#ffb547`), for the in-app icon and the hicolor scalable icon.
- `craft-anything-1024.png`, `craft-anything.icns`, `craft-anything.ico`, `hicolor/`: renders of
  the master clipped to a 512-unit tile with 110-unit corners; the macOS icons pad the tile onto
  Apple's 824/1024 icon grid.

Regenerate the renders with `packaging/icons.sh` (needs `resvg`), and the DMG window with
`packaging/macos/dmg/generate.py`.
