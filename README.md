# chronon-spectra-telemetry

[![CI](https://github.com/unified-field-dev/chronon-spectra-telemetry/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/chronon-spectra-telemetry/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/chronon-spectra-telemetry) · `cargo doc -p chronon-spectra-telemetry --open`

Spectra-backed telemetry for [Chronon](https://github.com/unified-field-dev/chronon): DSL schemas, Photon topic helpers (`*Recorder` / `*Logger`), and a `TelemetrySink` you pass into `ChrononBuilder`.

```toml
chronon-spectra-telemetry = { git = "https://github.com/unified-field-dev/chronon-spectra-telemetry" }
```

```rust
use chronon_spectra_telemetry::install_from_env;

let sink = install_from_env();
// ChrononBuilder::telemetry_sink(sink)
```

Import codegen’d helpers from the crate root, e.g. `chronon_spectra_telemetry::ChrononRunsStartedRecorder`.

## About

- Spectra DSL schemas under `schemas/` (inventory-registered when linked)
- Photon topic helpers for Chronon self-telemetry
- `install_from_env` / `SpectraTelemetrySink` for host wiring
- `sink_forward` onto typed Spectra recorders

## Examples

Runnable smoke: [examples/README.md](examples/README.md).

## Verify

```bash
export CARGO_BUILD_JOBS=1
cargo test
```

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
