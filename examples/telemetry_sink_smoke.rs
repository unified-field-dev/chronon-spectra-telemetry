//! Install Chronon `TelemetrySink` from env and emit one typed counter.
//!
//! ```bash
//! CHRONON_TELEMETRY=console CARGO_BUILD_JOBS=1 \
//!   cargo run -p chronon-spectra-telemetry --example telemetry_sink_smoke
//! ```
//!
//! Success: `telemetry_sink_smoke: OK`.

#![allow(clippy::print_stdout)]

use chronon_spectra_telemetry::{install_from_env, ChrononRunsStartedRecorder};

fn main() {
    std::env::set_var("CHRONON_TELEMETRY", "console");
    let _sink = install_from_env();
    ChrononRunsStartedRecorder::record(
        1,
        serde_json::json!({"job_name": "example", "deployment_shape": "embedded"}),
    );
    println!("telemetry_sink_smoke: OK");
}
