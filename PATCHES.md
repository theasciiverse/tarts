# Changes from upstream

This is the Asciiverse packaging of [tarts](https://github.com/oiwn/tarts)
by oiwn and contributors (MIT), built from upstream's v0.1.27
(`f6f5f50`, 2026-10-14). Upstream is the `upstream` remote; the
`asciiverse` branch carries the changes below on top of that tag.

## History of this branch

- First packaged 2026-10-13 from v0.1.26 (tag `v0.1.26-av.1`), with our
  own fix for Life's crash on resize (upstream issue #80).
- Moved to v0.1.27 on 2026-10-18. Upstream rewrote its history between
  the two (its v0.1.26 is `00fe9fa`, the same tree as the `4333c33` we
  had built on, under other commit ids), so our commits were rebased onto
  v0.1.27 rather than merged. v0.1.27 fixes Life on resize itself (#80),
  so our fix was dropped; it also stops the panics and hangs on 0x0 and
  1x1 terminals (#94), updates crossbeam-epoch past RUSTSEC-2026-0204,
  and replaces the terrain effect with pocketworld. The tour never
  included terrain, and the twelve effects it shows look as before, so
  the preview was not re-recorded. `tarts pocketworld` runs by name.
- Known, upstream and not fixed here (seen 2026-10-18): maze stops
  answering keys after the window is resized to 1x1 and back, alone or in
  the tour; the other effects survive it.

## Code

- **`tarts tour`**: every effect in turn, 20 seconds each, round and
  round (matrix, donut, boids, plasma, fire, pipes, life, cube,
  constellation, crab, butterfly, maze). n, Space or Right skip ahead, p or
  Left go back; q, Esc and Ctrl+C quit. av starts tarts this way. Building
  an effect by name moved into `make_effect()`, shared with the
  single-effect run, and `run_loop()` takes a boxed effect. Like
  upstream's single-effect run since #94, the tour clamps a 0x0 terminal
  size to 1x1, at the start of each effect and on a resize.

## Packaging

- Upstream's workflows (tests, Nix, coverage, dependency audit, its
  opencode agent, crates.io and Homebrew publishing, its own release
  assets) are removed: they would run against this fork without
  upstream's secrets.
- `.github/workflows/release.yml`: av archives for Linux (static musl),
  macOS and Windows, each x86-64 and ARM64, plus SHA256SUMS. It runs on
  our `v*-av.*` tags only (it took every `v*` tag, upstream's too).
- `asciiverse/record-preview.py` records the catalog preview (a few seconds
  of ten effects through the tour), and `asciiverse/cast-cap-fps.py`
  re-encodes it at 25 frames a second (tarts draws at 60; the raw
  recording is 58 MB, the preview 15 MB).
