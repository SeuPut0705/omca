#!/usr/bin/env bash
# Release OMCA for the two supported platforms: Apple silicon Macs and Windows x64.
#
#   packaging/release.sh [--skip-ci] [--no-windows]
#
# 1. Runs the local CI gate (`cargo xtask ci`) unless --skip-ci.
# 2. Builds the macOS aarch64 DMG and CLI zip here (packaging/macos/package.sh; signing and
#    notarization follow its MACOS_SIGN_IDENTITY / APPLE_* environment).
# 3. Tags v<version> (from [workspace.package] in Cargo.toml), pushes the tag and publishes a
#    GitHub release with the macOS files and their SHA-256 sums.
# 4. Starts the manual Windows x64 workflow, which attaches the MSI and portable zip to the same
#    release, unless --no-windows. That workflow is the only GitHub Actions run.
set -euo pipefail
# shellcheck source=env.sh
. "$(dirname "${BASH_SOURCE[0]}")/env.sh"

RUN_CI=1
WINDOWS=1
while [ $# -gt 0 ]; do
  case "$1" in
    --skip-ci) RUN_CI=0; shift ;;
    --no-windows) WINDOWS=0; shift ;;
    -h | --help) sed -n '2,13p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

TAG="v$VERSION"
cd "$ROOT"
if [ -n "$(git status --porcelain)" ]; then
  echo "error: the working tree has uncommitted changes; commit them first" >&2
  exit 1
fi
if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  echo "error: $TAG already exists; bump the version with: cargo xtask version set X.Y.Z" >&2
  exit 1
fi

if [ "$RUN_CI" = 1 ]; then
  cargo xtask ci
fi

rm -f "$DIST"/craft-anything-*macos* "$DIST/SHA256SUMS.txt"
"$ROOT/packaging/macos/package.sh"
(cd "$DIST" && shasum -a 256 craft-anything-*macos* >SHA256SUMS.txt)

git tag -a "$TAG" -m "OMCA $VERSION"
git push origin "$TAG"
gh release create "$TAG" --title "OMCA $VERSION" --generate-notes \
  "$DIST"/craft-anything-*macos* "$DIST/SHA256SUMS.txt"

if [ "$WINDOWS" = 1 ]; then
  gh workflow run release-windows.yml -f tag="$TAG"
  echo "==> Windows x64 build started: gh run watch \$(gh run list -w release-windows.yml -L 1 --json databaseId -q '.[0].databaseId')"
fi
echo "==> released $TAG"
