use std::sync::{Arc, OnceLock};

use chronon_telemetry::{ConsoleSink, NoOpSink, TelemetrySink};

use crate::metrics;

static INSTALLED_SINK: OnceLock<Arc<dyn TelemetrySink>> = OnceLock::new();

/// Spectra-backed Chronon telemetry sink.
///
/// # Examples
///
/// ```rust,no_run
/// use chronon_spectra_telemetry::SpectraTelemetrySink;
/// use chronon_telemetry::TelemetrySink;
///
/// let sink = SpectraTelemetrySink::new();
/// sink.record_counter(
///     "chronon_runs_started",
///     &[("job_name", "nightly")],
///     1,
/// );
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct SpectraTelemetrySink;

impl SpectraTelemetrySink {
    /// Build a Spectra-backed sink.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use chronon_spectra_telemetry::SpectraTelemetrySink;
    ///
    /// let _sink = SpectraTelemetrySink::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl TelemetrySink for SpectraTelemetrySink {
    fn record_counter(&self, name: &str, labels: &[(&str, &str)], delta: u64) {
        metrics::record_counter(name, labels, delta);
    }

    fn record_gauge(&self, name: &str, labels: &[(&str, &str)], value: f64) {
        metrics::record_gauge(name, labels, value);
    }

    fn log_event(&self, schema: &str, fields: &[(&str, &str)]) {
        metrics::log_event(schema, fields);
    }
}

fn sink_from_env() -> Arc<dyn TelemetrySink> {
    match std::env::var("CHRONON_TELEMETRY")
        .ok()
        .map(|v| v.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("off" | "0" | "false" | "none") => Arc::new(NoOpSink),
        Some("console") => Arc::new(ConsoleSink),
        _ => Arc::new(SpectraTelemetrySink::new()),
    }
}

/// Resolve and cache Chronon telemetry sink from `CHRONON_TELEMETRY`.
///
/// # Examples
///
/// ```rust,no_run
/// use chronon_spectra_telemetry::install_from_env;
///
/// let sink = install_from_env();
/// // ChrononBuilder::new().telemetry_sink(sink)...
/// let _ = sink;
/// ```
pub fn install_from_env() -> Arc<dyn TelemetrySink> {
    Arc::clone(INSTALLED_SINK.get_or_init(sink_from_env))
}

/// Compatibility alias for older call sites; prefer [`install_from_env`].
#[deprecated(since = "0.1.0", note = "renamed to install_from_env")]
pub fn install_ops_log_from_env() -> Arc<dyn TelemetrySink> {
    install_from_env()
}
