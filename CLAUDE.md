# tarts (Asciiverse packaging)

Terminal screensavers and effects by oiwn and contributors, in Rust on
crossterm; packaged for the Asciiverse catalog as its first demo. Not our
code: keep changes small, list every one in PATCHES.md, and offer fixes
upstream. Upstream's own notes for agents are in AGENTS.md.

- Remotes: `upstream` is github.com/oiwn/tarts, `origin` is
  github.com/theasciiverse/tarts.
- Branch `asciiverse` = upstream master plus our commits.
- Release: tag `v<version>-av.<n>` and push only that tag (upstream's own
  `v*` tags would start the release workflow too).
- av starts it as `tarts tour`.
- Local static build: `docker run --rm -v "$PWD":/src -w /src
  rust:1-alpine sh -c 'apk add musl-dev && cargo build --release'`.
- Preview: `asciiverse/record-preview.py <binary> raw.cast`, then
  `asciiverse/cast-cap-fps.py raw.cast asciiverse/tarts.cast 25`.
