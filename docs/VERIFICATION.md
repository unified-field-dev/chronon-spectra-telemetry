# chronon-spectra-telemetry verification

Re-run after code or doc changes. Covered by unit + integration tests below.

## Environment

Match [`.github/workflows/ci.yml`](../.github/workflows/ci.yml):

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-chronon-spectra-telemetry
export RUSTFLAGS="-D warnings"
```

Toolchain: stable (same as CI).

## PR CI parity

Required PR job `quality`:

| CI step | Local command |
|--------|----------------|
| Formatting | `cargo fmt --all -- --check` |
| Clippy | `cargo clippy --all-targets --all-features -- -D warnings` |
| Test | `cargo test --all-features` |
| cargo doc | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` |
| cargo deny | `cargo deny check` |
| cargo audit | `cargo audit` |

## Unit + integration (CI)

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo deny check
cargo audit
```

### TEST_MAP

| Behavior | Level | Happy | Sad | Notes |
|----------|-------|-------|-----|-------|
| `truncate_message` / event field builders | unit | short message preserved; run/executor/scheduler JSON shape | oversize message clipped to 512 with `...` | `events::tests` |
| `ErrorClass::from_message` | unit | Param / NotFound / ValenceBuild + `as_str` | unclassified / partial keywords → `Internal` | `labels::tests` |
| `deployment_shape_from_env` | unit | shape / role / remote URL → labels | blank env values → `embedded` | under `ENV_LOCK` |
| `label_value` / `counter_delta` | unit | key lookup; u64→i64 | missing key; `u64::MAX` → `i64::MAX` | `metrics::tests` |
| `sink_forward::field_str` / `field_i64` | unit | string/bool/number coercions | missing / null / array / bad parse → `""` / `0` | private helpers |
| `install_from_env` | integ | `off` aliases, `console`, default Spectra | OnceLock ignores later env flips | process-isolated binaries |
| `SpectraTelemetrySink` TelemetrySink methods | integ | known counters/gauges/events + remaps | unknown names / empty fields accepted | via `try_*` gate |
| `sink_forward` | integ | known counters + event tables | unknown name ignored; missing fields default | consumer / sink_forward |
| Topic constants | integ | all `*_TOPIC` non-empty `spectra.{metric,event}.*` + `chronon_` | — | Photon wire names |

## Notes

- Install paths use process-isolated integration binaries because `OnceLock`
  caches the first `CHRONON_TELEMETRY` resolution for the process.
- Label unit tests serialize env mutations with `ENV_LOCK`.
- Under Spectra `try_*` gates, sink emit helpers may no-op when Spectra is
  unconfigured; assertions focus on contracts, remaps, and non-panic forward
  paths rather than captured sink rows.
- Sad-path tests are named with `_sad` / `happy_and_sad` so audits detect them;
  they assert concrete defaults and shapes, beyond smoke-only checks.
