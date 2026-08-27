//! Crate-internal label/classification helpers for Chronon self-telemetry.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Coarse classification of a Chronon executor error message, used as the
/// `error_class` label on `chronon_runs_failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Parameter validation failure (bad/missing field, wrong type).
    Param,
    /// Referenced record was not found.
    NotFound,
    /// Failed to build a Valence session for the run.
    ValenceBuild,
    /// Unclassified failure.
    Internal,
}

impl ErrorClass {
    /// Classify an error message by keyword sniffing.
    pub fn from_message(message: &str) -> Self {
        let lower = message.to_ascii_lowercase();
        if lower.contains("parameter error")
            || lower.contains("missing field")
            || lower.contains("invalid type")
        {
            Self::Param
        } else if lower.contains("not found") {
            Self::NotFound
        } else if lower.contains("valence") && lower.contains("build") {
            Self::ValenceBuild
        } else {
            Self::Internal
        }
    }

    /// Stable label value for Spectra dimensions.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Param => "param",
            Self::NotFound => "not_found",
            Self::ValenceBuild => "valence_build",
            Self::Internal => "internal",
        }
    }
}

const ALLOWED_ERROR_CLASSES: &[&str] = &["param", "not_found", "valence_build", "internal"];

fn hash_label_value(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("h{:016x}", hasher.finish())
}

/// Bound free-form `job_name` metric labels to stable hashed values.
#[must_use]
pub fn bound_job_name_label(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "unknown" {
        return "unknown".to_owned();
    }
    hash_label_value(trimmed)
}

/// Resolve `error_class` for `chronon_runs_failed` from labels or error text.
#[must_use]
pub fn error_class_from_labels(labels: &[(&str, &str)]) -> &'static str {
    for (key, value) in labels {
        if *key == "error_class" {
            let normalized = value.trim().to_ascii_lowercase();
            if ALLOWED_ERROR_CLASSES.contains(&normalized.as_str()) {
                return match normalized.as_str() {
                    "param" => "param",
                    "not_found" => "not_found",
                    "valence_build" => "valence_build",
                    _ => "internal",
                };
            }
            return "internal";
        }
    }
    for (key, value) in labels {
        if *key == "error" || *key == "message" {
            return ErrorClass::from_message(value).as_str();
        }
    }
    "internal"
}

/// Allowlisted `deployment_shape` label values.
const ALLOWED_DEPLOYMENT_SHAPES: &[&str] = &[
    "embedded",
    "remote_client",
    "worker",
    "coordinator",
    "scheduler",
    "distributed",
    "standalone",
];

fn sanitize_deployment_shape(raw: &str) -> &'static str {
    let trimmed = raw.trim().to_ascii_lowercase();
    ALLOWED_DEPLOYMENT_SHAPES
        .iter()
        .find(|&&allowed| allowed == trimmed)
        .copied()
        .unwrap_or("unknown")
}

/// Resolve the allowlisted `deployment_shape` label from `CHRONON_DEPLOYMENT_SHAPE` /
/// `CHRONON_INSTANCE_ROLE` / `CHRONON_REMOTE_BASE_URL`, defaulting to `"embedded"`.
///
/// Unknown env values map to `"unknown"` so metric cardinality stays bounded.
pub fn deployment_shape_from_env() -> &'static str {
    if let Ok(v) = std::env::var("CHRONON_DEPLOYMENT_SHAPE") {
        let t = v.trim();
        if !t.is_empty() {
            return sanitize_deployment_shape(t);
        }
    }
    if let Ok(v) = std::env::var("CHRONON_INSTANCE_ROLE") {
        let t = v.trim();
        if !t.is_empty() {
            return sanitize_deployment_shape(t);
        }
    }
    if std::env::var("CHRONON_REMOTE_BASE_URL").is_ok_and(|v| !v.trim().is_empty()) {
        return "remote_client";
    }
    "embedded"
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, MutexGuard};

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_guard() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn clear_deployment_env() {
        std::env::remove_var("CHRONON_DEPLOYMENT_SHAPE");
        std::env::remove_var("CHRONON_INSTANCE_ROLE");
        std::env::remove_var("CHRONON_REMOTE_BASE_URL");
    }

    #[test]
    fn bound_job_name_label_hashes_free_form_values_sad() {
        let a = bound_job_name_label("nightly-backup-prod-west");
        let b = bound_job_name_label("nightly-backup-prod-west");
        let c = bound_job_name_label("other-job");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.starts_with('h'));
        assert_eq!(a.len(), 17);
        assert_eq!(bound_job_name_label(""), "unknown");
        assert_eq!(bound_job_name_label("unknown"), "unknown");
    }

    #[test]
    fn error_class_from_labels_prefers_explicit_label_happy() {
        let labels = [("error_class", "param"), ("message", "other")];
        assert_eq!(error_class_from_labels(&labels), "param");
    }

    #[test]
    fn error_class_from_labels_classifies_message_sad() {
        let labels = [("message", "parameter error: bad field")];
        assert_eq!(error_class_from_labels(&labels), "param");
        assert_eq!(error_class_from_labels(&[]), "internal");
        let bad = [("error_class", "custom")];
        assert_eq!(error_class_from_labels(&bad), "internal");
    }

    #[test]
    fn error_class_maps_known_messages_happy() {
        assert_eq!(
            ErrorClass::from_message("Parameter error: bad"),
            ErrorClass::Param
        );
        assert_eq!(
            ErrorClass::from_message("missing field foo"),
            ErrorClass::Param
        );
        assert_eq!(
            ErrorClass::from_message("invalid type: string"),
            ErrorClass::Param
        );
        assert_eq!(
            ErrorClass::from_message("job not found"),
            ErrorClass::NotFound
        );
        assert_eq!(
            ErrorClass::from_message("failed to valence build session"),
            ErrorClass::ValenceBuild
        );
        assert_eq!(ErrorClass::Param.as_str(), "param");
        assert_eq!(ErrorClass::NotFound.as_str(), "not_found");
        assert_eq!(ErrorClass::ValenceBuild.as_str(), "valence_build");
        assert_eq!(ErrorClass::Internal.as_str(), "internal");
    }

    #[test]
    fn error_class_unknown_maps_to_internal_sad() {
        assert_eq!(
            ErrorClass::from_message("something else"),
            ErrorClass::Internal
        );
        assert_eq!(ErrorClass::from_message(""), ErrorClass::Internal);
        // valence without build → Internal
        assert_eq!(
            ErrorClass::from_message("valence missing"),
            ErrorClass::Internal
        );
        // build without valence → Internal
        assert_eq!(
            ErrorClass::from_message("build failed"),
            ErrorClass::Internal
        );
    }

    #[test]
    fn deployment_shape_reads_env_happy() {
        let _guard = env_guard();
        clear_deployment_env();
        assert_eq!(deployment_shape_from_env(), "embedded");

        std::env::set_var("CHRONON_REMOTE_BASE_URL", "http://127.0.0.1:9");
        assert_eq!(deployment_shape_from_env(), "remote_client");
        std::env::remove_var("CHRONON_REMOTE_BASE_URL");

        std::env::set_var("CHRONON_INSTANCE_ROLE", "Worker");
        assert_eq!(deployment_shape_from_env(), "worker");
        std::env::remove_var("CHRONON_INSTANCE_ROLE");

        std::env::set_var("CHRONON_DEPLOYMENT_SHAPE", "Distributed");
        assert_eq!(deployment_shape_from_env(), "distributed");
        clear_deployment_env();
    }

    #[test]
    fn deployment_shape_unknown_env_maps_to_unknown_sad() {
        let _guard = env_guard();
        clear_deployment_env();

        std::env::set_var("CHRONON_DEPLOYMENT_SHAPE", "tenant-acme-prod-west");
        assert_eq!(deployment_shape_from_env(), "unknown");
        clear_deployment_env();
    }

    #[test]
    fn deployment_shape_blank_env_falls_through_sad() {
        let _guard = env_guard();
        clear_deployment_env();

        std::env::set_var("CHRONON_DEPLOYMENT_SHAPE", "   ");
        std::env::set_var("CHRONON_INSTANCE_ROLE", "");
        std::env::set_var("CHRONON_REMOTE_BASE_URL", "  ");
        assert_eq!(deployment_shape_from_env(), "embedded");
        clear_deployment_env();
    }
}
