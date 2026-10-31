//! Happy/sad coverage for `SpectraTelemetrySink` and `sink_forward` (no install).
#![allow(missing_docs)]

use chrono::Utc;
use chronon_spectra_telemetry::{
    sink_forward, SpectraTelemetrySink, CHRONON_EXECUTOR_ERROR_TOPIC,
    CHRONON_PARTITION_ASSIGNMENTS_TOPIC, CHRONON_REMOTE_ERRORS_TOPIC, CHRONON_RUNS_COMPLETED_TOPIC,
    CHRONON_RUNS_FAILED_TOPIC, CHRONON_RUNS_STARTED_TOPIC, CHRONON_RUN_DURATION_MS_TOPIC,
    CHRONON_RUN_LOG_TOPIC, CHRONON_SCHEDULER_LOG_TOPIC, CHRONON_SCHEDULER_TICKS_TOPIC,
};
use chronon_telemetry::TelemetrySink;
use serde_json::json;

#[test]
fn spectra_sink_counter_gauge_event_happy() {
    let sink = SpectraTelemetrySink::new();
    sink.record_counter(
        "chronon_runs_started",
        &[
            ("job_name", "nightly"),
            ("job", "ignored-when-job_name-present"),
        ],
        1,
    );
    // legacy `job` label remaps when `job_name` is absent
    sink.record_counter("chronon_runs_completed", &[("job", "legacy-label")], 1);
    sink.record_counter("chronon_runs_failed", &[("job_name", "nightly")], 1);
    sink.record_counter("chronon_scheduler_ticks", &[], 1);
    sink.record_gauge("chronon_run_duration_ms", &[("job_name", "nightly")], 12.5);
    sink.log_event(
        "chronon_executor_error",
        &[
            ("job_name", "j"),
            ("run_id", "r"),
            ("script_name", "s"),
            ("phase", "script_invoke"),
            ("message", "parameter error: bad field"),
        ],
    );
    sink.log_event(
        "chronon_run_failed",
        &[("run_id", "r"), ("job", "j"), ("error", "boom")],
    );
    sink.log_event(
        "chronon_scheduler_warn",
        &[("component", "scheduler"), ("message", "lag")],
    );
    sink.log_event(
        "chronon_scheduler_info",
        &[("component", "scheduler"), ("message", "ok")],
    );
    sink.log_event(
        "chronon_run_log",
        &[
            ("run_id", "r"),
            ("job_name", "j"),
            ("script_name", "s"),
            ("status", "completed"),
            ("message", "done"),
        ],
    );
}

#[test]
fn spectra_sink_unknown_and_empty_fields_accepted_sad() {
    let sink = SpectraTelemetrySink::new();
    // unknown counter / schema names are dropped — must not panic
    sink.record_counter("chronon_custom_counter", &[("k", "v")], 2);
    sink.log_event("chronon_run_log", &[]);
    sink.log_event("totally_unknown_schema", &[("x", "y")]);
    // missing job labels resolve to "unknown" inside remappers
    sink.record_counter("chronon_runs_started", &[], 1);
    sink.log_event(
        "chronon_executor_error",
        &[("message", "unclassified failure")],
    );
}

#[test]
fn sink_forward_known_metrics_and_events_happy() {
    let ts = Utc::now();
    let labels = json!({"job_name": "j", "deployment_shape": "embedded"});

    sink_forward::forward_counter("chronon_runs_started", labels.clone(), 1, ts);
    sink_forward::forward_counter("chronon_runs_completed", labels.clone(), 1, ts);
    sink_forward::forward_counter("chronon_runs_failed", labels.clone(), 1, ts);
    sink_forward::forward_counter("chronon_scheduler_ticks", labels.clone(), 1, ts);
    sink_forward::forward_counter("chronon_remote_errors", labels.clone(), 1, ts);
    sink_forward::forward_counter("chronon_run_duration_ms", labels.clone(), 5, ts);
    sink_forward::forward_counter("chronon_partition_assignments", labels, 1, ts);

    sink_forward::forward_event(
        "chronon_executor_error",
        &json!({
            "job_name": "j", "run_id": "r", "script_name": "s",
            "operation": "script_invoke", "error": "e"
        }),
        ts,
    );
    sink_forward::forward_event(
        "chronon_run_log",
        &json!({
            "run_id": "r", "job_name": "j", "script_name": "s",
            "status": "failed", "duration_ms": "3", "message": "m"
        }),
        ts,
    );
    sink_forward::forward_event(
        "chronon_scheduler_log",
        &json!({"component": "scheduler", "message": "tick"}),
        ts,
    );
}

#[test]
fn sink_forward_unknown_and_missing_fields_ignored_sad() {
    let ts = Utc::now();

    // unknown metric / table names are no-ops
    sink_forward::forward_counter("not_a_chronon_metric", json!({}), 1, ts);
    sink_forward::forward_event("unknown_table", &json!({}), ts);

    // missing fields coerce to empty string / 0
    sink_forward::forward_event("chronon_run_log", &json!({}), ts);
    sink_forward::forward_event("chronon_executor_error", &json!({"job_name": null}), ts);
    sink_forward::forward_event(
        "chronon_scheduler_log",
        &json!({"component": [], "message": {}}),
        ts,
    );
}

#[test]
fn topic_constants_are_non_empty_happy() {
    for topic in [
        CHRONON_RUNS_STARTED_TOPIC,
        CHRONON_RUNS_COMPLETED_TOPIC,
        CHRONON_RUNS_FAILED_TOPIC,
        CHRONON_SCHEDULER_TICKS_TOPIC,
        CHRONON_RUN_DURATION_MS_TOPIC,
        CHRONON_REMOTE_ERRORS_TOPIC,
        CHRONON_PARTITION_ASSIGNMENTS_TOPIC,
    ] {
        assert_ne!(topic, "");
        assert!(
            topic.starts_with("spectra.metric."),
            "unexpected metric topic: {topic}"
        );
        assert!(
            topic.contains("chronon_"),
            "metric topic missing chronon_ prefix: {topic}"
        );
    }
    for topic in [
        CHRONON_RUN_LOG_TOPIC,
        CHRONON_EXECUTOR_ERROR_TOPIC,
        CHRONON_SCHEDULER_LOG_TOPIC,
    ] {
        assert_ne!(topic, "");
        assert!(
            topic.starts_with("spectra.event."),
            "unexpected event topic: {topic}"
        );
        assert!(
            topic.contains("chronon_"),
            "event topic missing chronon_ prefix: {topic}"
        );
    }
}
