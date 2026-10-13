# Changes from upstream

This is the Asciiverse packaging of [tarts](https://github.com/oiwn/tarts)
by oiwn and contributors (MIT), built from upstream's v0.1.26 (master on
2026-10-13). Upstream is the `upstream` remote; the `asciiverse` branch
carries the changes below. Both code changes are worth offering upstream.

## Code

- **Life no longer crashes on a resize.** `ConwayLife::reset()` was
  `todo!()`, and every resize calls it, so Life panicked with "not yet
  implemented" whenever the window changed size, and at start on Windows
  (upstream issue #80). It now rebuilds the effect for the new size, as
  the other effects do.
- **`tarts tour`**: every effect in turn, 20 seconds each, round and
  round (matrix, donut, boids, plasma, fire, pipes, life, cube,
  constellation, crab, butterfly, maze). n, Space or Right skip ahead, p or
  Left go back; q, Esc and Ctrl+C quit. av starts tarts this way. Building
  an effect by name moved into `make_effect()`, shared with the
  single-effect run, and `run_loop()` takes a boxed effect.

## Packaging

- Upstream's workflows (tests, Nix, crates.io and Homebrew publishing, its
  own release assets) are removed: they would run against this fork
  without upstream's secrets.
- `.github/workflows/release.yml`: av archives for Linux (static musl),
  macOS and Windows, each x86-64 and ARM64, plus SHA256SUMS.
- `asciiverse/record-preview.py` records the catalog preview (a few seconds
  of ten effects through the tour), and `asciiverse/cast-cap-fps.py`
  re-encodes it at 25 frames a second (tarts draws at 60; the raw
  recording is 58 MB, the preview 15 MB).
