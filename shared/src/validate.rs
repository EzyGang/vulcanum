use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::api::wire::AgentBackend;
use crate::config::{IsolationBackend, WorkerConfig};
#[cfg(target_os = "macos")]
use crate::constants::MACOS_DOCKER_DESKTOP_CLI_PATH;

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

    match isolation_backend.parse::<IsolationBackend>() {
        Ok(IsolationBackend::Kata) => {
            if cfg!(target_os = "macos") {
                issues.push(ValidationIssue {
                    severity: Severity::Critical,
                    message:
                        "Kata isolation is not supported on macOS because Kata requires Linux KVM"
                            .to_owned(),
                });
                return issues;
            }
            check_docker(&mut issues);
            check_kvm(&mut issues);
            check_binary("kata-runtime", &mut issues, Severity::Critical);
        }
        Ok(IsolationBackend::Docker) => {
            check_docker(&mut issues);
        }
        Ok(IsolationBackend::Host) => {
            check_binary(agent_backend.binary_name(), &mut issues, Severity::Critical);
        }
        Err(err) => {
            issues.push(ValidationIssue {
                severity: Severity::Critical,
                message: err.to_string(),
            });
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
    let kvm_path = Path::new("/dev/kvm");
    if !kvm_path.exists() {
        issues.push(ValidationIssue {
            severity: Severity::Critical,
            message: "/dev/kvm not found — KVM acceleration unavailable".to_owned(),
        });
        return;
    }

    if !can_access_kvm_device(kvm_path) {
        issues.push(ValidationIssue {
            severity: Severity::Critical,
            message: "/dev/kvm exists but is not accessible by the current user".to_owned(),
        });
    }
}

#[must_use]
pub fn is_kvm_available() -> bool {
    can_access_kvm_device(Path::new("/dev/kvm"))
}

#[must_use]
pub fn can_access_kvm_device(path: &Path) -> bool {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .is_ok()
}

fn check_docker(issues: &mut Vec<ValidationIssue>) {
    let Some(docker_path) = find_in_path("docker") else {
        issues.push(ValidationIssue {
            severity: Severity::Critical,
            message: "docker not found in PATH".to_owned(),
        });
        return;
    };

    if docker_info_succeeds(&docker_path) || sudo_docker_info_succeeds(&docker_path) {
        return;
    }

    issues.push(ValidationIssue {
        severity: Severity::Critical,
        message: "docker daemon is not reachable by the current user or passwordless sudo"
            .to_owned(),
    });
}

fn docker_info_succeeds(docker_path: &Path) -> bool {
    Command::new(docker_path)
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn sudo_docker_info_succeeds(docker_path: &Path) -> bool {
    Command::new("sudo")
        .arg("-n")
        .arg(docker_path)
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
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
        if is_executable_file(&candidate) {
            Some(candidate)
        } else {
            #[cfg(windows)]
            {
                let with_exe = dir.join(format!("{name}.exe"));
                if is_executable_file(&with_exe) {
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

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn macos_app_binary(name: &str) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        if name == "docker" {
            let path = PathBuf::from(MACOS_DOCKER_DESKTOP_CLI_PATH);
            if is_executable_file(&path) {
                return Some(path);
            }
        }
    }

    let _ = name;
    None
}
