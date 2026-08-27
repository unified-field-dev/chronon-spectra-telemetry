//! Spectra-backed self-telemetry for [Chronon]: typed event/metric schemas, Photon topic
//! helpers, and a [`TelemetrySink`](chronon_telemetry::TelemetrySink) adapter that forwards
//! Chronon's own runtime signals (runs, scheduler ticks, executor errors) into [Spectra].
//!
//! [Chronon]'s [`TelemetrySink`](chronon_telemetry::TelemetrySink) trait is deliberately
//! backend-agnostic: Chronon calls `record_counter` / `record_gauge` / `log_event` on whatever
//! implementation the host installs. This crate is that implementation for hosts that already
//! emit their own telemetry through [Spectra]: [`SpectraTelemetrySink`] forwards each call into
//! `spectra-core`, and [`install_from_env`] wires it up (or opts out) based on `CHRONON_TELEMETRY`.
//!
//! [Chronon]: https://github.com/unified-field-dev/chronon
//! [Spectra]: https://github.com/unified-field-dev/spectra
//!
//! ## Features
//!
//! - **Env-resolved telemetry install** — Reads `CHRONON_TELEMETRY` at host boot and caches the matching
//!   process-wide `TelemetrySink` before the Chronon runtime starts.
//!   [Get started](#env-driven-install)
//! - **Spectra `TelemetrySink` adapter** — [`SpectraTelemetrySink`] implements
//!   [`chronon_telemetry::TelemetrySink`] when you wire the Spectra adapter yourself instead of
//!   using the env helper. [Get started](#direct-telemetry-sink)
//! - **Consumer-side forwarding** — [`sink_forward`] re-dispatches raw metric and event emits
//!   onto the matching typed Spectra recorder for sink consumers that re-emit Chronon signals
//!   downstream. [Get started](#sink-forwarding)
//! - **Topic + codegen helpers** — Generated `*Recorder` / `*Logger` / `*Payload` / `*_TOPIC`
//!   symbols for explicit Chronon telemetry emits from host or test code.
//!   [Get started](#typed-recorders)
//! - **Typed schemas** — Spectra DSL schemas for Chronon's run/scheduler tables and counters,
//!   registered via `inventory` when linked into a host.
//!
//! # Getting started
//!
//! Most hosts install the telemetry sink once at startup, then hand it to `ChrononBuilder` so
//! run and scheduler signals flow through Spectra automatically. Pick the env helper for
//! production hosts or wire [`SpectraTelemetrySink`] directly when tests need a fixed backend.
//!
//! ## Env-driven install
//!
//! [`install_from_env`] is the default host path: it resolves `CHRONON_TELEMETRY` once at
//! process boot and caches the matching `TelemetrySink` before you build the Chronon runtime, so
//! run, scheduler, and executor signals flow through Spectra for the process lifetime.
//!
//! Prerequisites: Spectra must already be booted in the host process when `CHRONON_TELEMETRY` is
//! unset or set to `spectra`. Set `off` or `console` to disable or print locally.
//!
//! ```rust,no_run
//! // Call before constructing the Chronon runtime.
//! use chronon_spectra_telemetry::install_from_env;
//!
//! let sink = install_from_env();
//! let telemetry = std::env::var("CHRONON_TELEMETRY").unwrap_or_else(|_| "spectra".into());
//! assert!(!telemetry.trim().is_empty());
//! // ChrononBuilder::new().telemetry_sink(sink)...
//! let _ = sink;
//! ```
//!
//! Runnable: `cargo run -p chronon-spectra-telemetry --example telemetry_sink_smoke`.
//!
//! Next: [Direct telemetry sink](#direct-telemetry-sink) when you need explicit wiring in tests.
//!
//! ## Direct telemetry sink
//!
//! [`SpectraTelemetrySink`] is for hosts or tests that install `TelemetrySink` without reading
//! `CHRONON_TELEMETRY`. Construct the adapter and pass it to `ChrononBuilder::telemetry_sink`
//! before Chronon starts emitting counters and events.
//!
//! Prerequisites: Spectra booted when using the default Spectra backend. Labels for counters and
//! gauges come from Chronon callers via `TelemetrySink::record_counter` / `record_gauge` label
//! slices; `job_name`, `deployment_shape`, and `error_class` are attached automatically by the
//! installed sink (see `CHRONON_DEPLOYMENT_SHAPE` below).
//!
//! ```rust,no_run
//! use chronon_spectra_telemetry::SpectraTelemetrySink;
//! use chronon_telemetry::TelemetrySink;
//!
//! let sink = SpectraTelemetrySink::new();
//! sink.record_counter(
//!     "chronon_runs_started",
//!     &[("job_name", "nightly"), ("deployment_shape", "embedded")],
//!     1,
//! );
//! let metric = "chronon_runs_started";
//! assert_eq!(metric, "chronon_runs_started");
//! ```
//!
//! Next: [Sink forwarding](#sink-forwarding) when a Spectra sink re-emits raw Chronon
//! metric names.
//!
//! ## Sink forwarding
//!
//! [`sink_forward`] maps raw Chronon metric and event names onto this crate's typed
//! `*Recorder` / `*Logger` helpers. Use it from Spectra sink consumers that receive generic
//! emits and need to re-emit onto the Chronon schema surface downstream.
//!
//! Prerequisites: the incoming metric or table name must match a Chronon schema this crate
//! registers (`chronon_runs_started`, `chronon_executor_error`, and the other Chronon topics).
//!
//! ```rust,no_run
//! use chronon_spectra_telemetry::sink_forward;
//! use chrono::Utc;
//! use serde_json::json;
//!
//! sink_forward::forward_counter(
//!     "chronon_runs_started",
//!     json!({"job_name": "nightly", "deployment_shape": "embedded"}),
//!     1,
//!     Utc::now(),
//! );
//! let forwarded = "chronon_runs_started";
//! assert_eq!(forwarded, "chronon_runs_started");
//! ```
//!
//! API reference: [`sink_forward`] module. Next: [Typed recorders](#typed-recorders) when you
//! emit Chronon telemetry directly without a sink hop.
//!
//! ## Typed recorders
//!
//! Generated `*Recorder` and `*Logger` types under [`helpers`] emit Chronon counters and events
//! with typed labels and topic constants from [`topics`]. Call them from host code or tests when
//! you need an explicit emit instead of relying on Chronon's runtime `TelemetrySink` path.
//!
//! Prerequisites: Spectra booted in the process. Import recorders from the crate root or
//! [`helpers`]; transport DTOs and `*_TOPIC` constants live in [`topics`].
//!
//! ```rust,no_run
//! use chronon_spectra_telemetry::{
//!     ChrononRunsStartedPayload, ChrononRunsStartedRecorder, CHRONON_RUNS_STARTED_TOPIC,
//! };
//!
//! ChrononRunsStartedRecorder::record(
//!     1,
//!     serde_json::json!({"job_name": "nightly", "deployment_shape": "embedded"}),
//! );
//! assert_eq!(ChrononRunsStartedPayload::topic(), CHRONON_RUNS_STARTED_TOPIC);
//! ```
//!
//! See [`helpers`] for the full recorder/logger set and [`topics`] for transport DTOs.
//!
//! ## Environment
//!
//! | Variable | Values | Default |
//! |----------|--------|---------|
//! | `CHRONON_TELEMETRY` | `off`, `console`, `spectra` | `spectra` (when Spectra is configured) |
//! | `CHRONON_DEPLOYMENT_SHAPE` | any string | derived from `CHRONON_INSTANCE_ROLE` / `CHRONON_REMOTE_BASE_URL`, else `embedded` |
//!
//! # Feature flags
//!
//! This crate has no Cargo feature flags.

#![allow(clippy::too_long_first_doc_paragraph)]

mod events;
/// Typed emit helpers from Chronon Spectra schemas.
pub mod helpers;
mod install;
mod labels;
mod metrics;
mod sanitize;
// macro-generated Spectra schema types; documented via each schema's `description`
#[allow(missing_docs)]
mod schemas;
/// Forwarders for sink consumers that re-dispatch raw metric/event emits onto the matching
/// typed Spectra recorder generated from this crate's schemas.
///
/// # Examples
///
/// ```rust,no_run
/// use chronon_spectra_telemetry::sink_forward;
/// use chrono::Utc;
/// use serde_json::json;
///
/// let ts = Utc::now();
/// sink_forward::forward_counter(
///     "chronon_runs_started",
///     json!({"job_name": "nightly", "deployment_shape": "embedded"}),
///     1,
///     ts,
/// );
/// ```
pub mod sink_forward;
/// Transport `*Payload` / `*_TOPIC` DTOs from Chronon Spectra schemas.
pub mod topics;

pub use helpers::*;
pub use topics::*;

#[allow(deprecated)]
pub use install::install_ops_log_from_env;
pub use install::{install_from_env, SpectraTelemetrySink};
