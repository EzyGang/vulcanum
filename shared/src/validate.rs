use std::path::PathBuf;

use crate::api_types::AgentBackend;
use crate::config::WorkerConfig;

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Warning,
}

/// A single issue discovered during environment validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub message: String,
}

/// Runs all environment checks for a specific isolation backend and agent backend.
pub fn validate_environment(
    isolation_backend: &str,
    agent_backend: AgentBackend,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    match isolation_backend {
        "kata" => {
            if cfg!(target_os = "macos") {
                issues.push(ValidationIssue {
                    severity: Severity::Critical,
                    message:
                        "Kata isolation is not supported on macOS because Kata requires Linux KVM"
                            .to_owned(),
                });
                return issues;
            }
            check_binary("docker", &mut issues, Severity::Critical);
            check_kvm(&mut issues);
            check_binary("kata-runtime", &mut issues, Severity::Critical);
        }
        "docker" => {
            check_binary("docker", &mut issues, Severity::Critical);
        }
        _ => {
            check_binary(agent_backend.binary_name(), &mut issues, Severity::Critical);
        }
    }

    issues
}

/// Runs all environment checks for a worker configuration.
pub fn validate_environment_for_config(config: &WorkerConfig) -> Vec<ValidationIssue> {
    validate_environment(&config.harness, config.agent_backend)
}

/// Runs all environment checks for a specific backend and returns a list of issues.
pub fn validate_environment_for_backend(backend: &str) -> Vec<ValidationIssue> {
    validate_environment(backend, AgentBackend::OpenCode)
}

/// Validates the environment for a worker configuration and returns true if no critical issues exist.
pub fn is_environment_ready_for_config(config: &WorkerConfig) -> bool {
    validate_environment_for_config(config)
        .iter()
        .all(|i| i.severity != Severity::Critical)
}

/// Validates the environment for a specific backend and returns true if no critical issues exist.
pub fn is_environment_ready_for_backend(backend: &str) -> bool {
    validate_environment_for_backend(backend)
        .iter()
        .all(|i| i.severity != Severity::Critical)
}

fn check_kvm(issues: &mut Vec<ValidationIssue>) {
    let kvm_path = PathBuf::from("/dev/kvm");
    if !kvm_path.exists() {
        issues.push(ValidationIssue {
            severity: Severity::Critical,
            message: "/dev/kvm not found — KVM acceleration unavailable".to_owned(),
        });
        return;
    }

    match std::fs::metadata(&kvm_path) {
        #[allow(unused_variables)]
        Ok(meta) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let mode = meta.mode() & 0o777;
                if mode & 0o666 == 0 {
                    issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        message: format!(
                            "/dev/kvm exists but permissions ({mode:03o}) may prevent access"
                        ),
                    });
                }
            }
        }
        Err(e) => {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: format!("cannot read /dev/kvm metadata: {e}"),
            });
        }
    }
}

fn check_binary(name: &str, issues: &mut Vec<ValidationIssue>, severity: Severity) {
    match find_in_path(name) {
        Some(path) => {
            tracing::debug!("found {} at {}", name, path.display());
        }
        None => {
            issues.push(ValidationIssue {
                severity,
                message: format!("{name} not found in PATH"),
            });
        }
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let Some(path_var) = std::env::var_os("PATH") else {
        return macos_app_binary(name);
    };
    let path_match = std::env::split_paths(&path_var).find_map(|dir| {
        let candidate = dir.join(name);
        if candidate.is_file() {
            Some(candidate)
        } else {
            #[cfg(windows)]
            {
                let with_exe = dir.join(format!("{name}.exe"));
                if with_exe.is_file() {
                    return Some(with_exe);
                }
            }
            None
        }
    });

    match path_match {
        Some(path) => Some(path),
        None => macos_app_binary(name),
    }
}

fn macos_app_binary(name: &str) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        if name == "docker" {
            let path = PathBuf::from("/Applications/Docker.app/Contents/Resources/bin/docker");
            if path.is_file() {
                return Some(path);
            }
        }
    }

    let _ = name;
    None
}
