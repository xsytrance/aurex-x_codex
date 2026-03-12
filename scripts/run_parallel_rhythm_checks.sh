#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[parallel] starting targeted crate checks"

cargo test -p aurex_music &
PID_MUSIC=$!

cargo test -p aurex_pulse &
PID_PULSE=$!

FAIL=0

wait "$PID_MUSIC" || FAIL=1
wait "$PID_PULSE" || FAIL=1

if [[ "$FAIL" -ne 0 ]]; then
  echo "[parallel] targeted checks failed"
  exit 1
fi

echo "[serial gate] running format + full workspace + runtime example"
cargo fmt --all -- --check
cargo test
cargo run -p aurex_pulse --example run_pulse examples/pulses/infinite_circuit_megacity.pulse.json

echo "[ok] parallel plan execution checks passed"
