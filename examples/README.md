# chronon-spectra-telemetry examples

| Example | Role |
|---------|------|
| `telemetry_sink_smoke` | `install_from_env` + one sink counter |

## 1. TelemetrySink — `telemetry_sink_smoke`

```bash
CHRONON_TELEMETRY=console CARGO_BUILD_JOBS=1 \
  cargo run -p chronon-spectra-telemetry --example telemetry_sink_smoke
```

Success: stdout prints `telemetry_sink_smoke: OK`.

Hand the returned sink to `ChrononBuilder::telemetry_sink` in a real host.
