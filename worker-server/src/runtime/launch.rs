use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

const OPENCODE_DEFAULT_PORT: u16 = 4096;

pub(crate) const HOST_ENV_ALLOWLIST: &[&str] = &[
    "PATH",
    "TMPDIR",
    "HOME",
    "LANG",
    "RUSTUP_HOME",
    "CARGO_HOME",
    "NVM_DIR",
    "NPM_CONFIG_PREFIX",
];

pub(super) async fn launch_host_server(
    workdir: &std::path::Path,
    env_vars: &std::collections::HashMap<String, String>,
    port: u16,
    repo_dir: Option<&std::path::Path>,
) -> Result<tokio::process::Child, HarnessError> {
    let mut cmd = tokio::process::Command::new("opencode");
    cmd.args(["serve", "--port", &port.to_string()])
        .env_clear()
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped());

    for (k, v) in std::env::vars() {
        if HOST_ENV_ALLOWLIST.contains(&k.as_str()) {
            cmd.env(&k, v);
        }
    }

    cmd.env("HOME", workdir.join("home").to_string_lossy().to_string())
        .env(
            "FINISH_ARTIFACT_PATH",
            workdir
                .join("home")
                .join("finish_artifact.json")
                .to_string_lossy()
                .to_string(),
        );

    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    if let Some(repo) = repo_dir {
        cmd.current_dir(repo);
    }

    #[cfg(unix)]
    {
        cmd.process_group(0);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| HarnessError::ServerLaunch(format!("failed to spawn opencode serve: {e}")))?;

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::debug!(target: "opencode::stderr", "{}", line);
            }
        });
    }

    Ok(child)
}

pub(super) async fn launch_container_server(
    env: &IsolatedEnvironment,
    repo_dir: &str,
) -> Result<(u16, String), HarnessError> {
    let container_name = env
        .container_name
        .as_deref()
        .ok_or_else(|| HarnessError::ServerLaunch("container_name missing".to_owned()))?;
    let image = env
        .image
        .as_deref()
        .ok_or_else(|| HarnessError::ServerLaunch("image missing".to_owned()))?;

    let config_env = "OPENCODE_CONFIG=/workdir/home/.config/opencode/opencode.json".to_owned();
    let home_env = "HOME=/workdir/home".to_owned();
    let artifact_env = "FINISH_ARTIFACT_PATH=/workdir/home/finish_artifact.json".to_owned();
    let workdir_str = env.workdir.to_string_lossy().to_string();
    let volume_mount = format!("{workdir_str}:/workdir");
    let port_str = OPENCODE_DEFAULT_PORT.to_string();

    let mut docker_args: Vec<String> = vec![
        "run".to_owned(),
        "-d".to_owned(),
        "--name".to_owned(),
        container_name.to_owned(),
        "-p".to_owned(),
        format!("127.0.0.1::{OPENCODE_DEFAULT_PORT}"),
        "--security-opt=no-new-privileges".to_owned(),
        "-e".to_owned(),
        config_env,
        "-e".to_owned(),
        home_env,
        "-e".to_owned(),
        artifact_env,
    ];

    if let Some(runtime) = env.runtime {
        docker_args.push("--runtime".to_owned());
        docker_args.push(runtime.to_owned());
    }

    for (k, v) in &env.env_vars {
        docker_args.push("-e".to_owned());
        docker_args.push(format!("{k}={v}"));
    }

    if !repo_dir.is_empty() {
        docker_args.extend(["--workdir".to_owned(), repo_dir.to_owned()]);
    }

    docker_args.extend([
        "-v".to_owned(),
        volume_mount,
        image.to_owned(),
        "opencode".to_owned(),
        "serve".to_owned(),
        "--hostname".to_owned(),
        "0.0.0.0".to_owned(),
        "--port".to_owned(),
        port_str,
    ]);

    let output = tokio::process::Command::new("docker")
        .args(&docker_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| HarnessError::ServerLaunch(format!("docker run failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(HarnessError::ServerLaunch(format!(
            "docker run failed: {stderr}"
        )));
    }

    let container_id = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let host_port = read_container_port(container_name).await?;

    let log_container_name = container_name.to_owned();
    tokio::spawn(async move {
        let mut cmd = tokio::process::Command::new("docker");
        cmd.args(["logs", "-f", &log_container_name])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let Ok(mut child) = cmd.spawn() else {
            return;
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(pipe) = stdout {
            tokio::spawn(pipe_lines_to_tracing(pipe));
        }
        if let Some(pipe) = stderr {
            tokio::spawn(pipe_lines_to_tracing(pipe));
        }
    });

    Ok((host_port, container_id))
}

async fn pipe_lines_to_tracing<R>(pipe: R)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut lines = BufReader::new(pipe).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::debug!(target: "opencode::container", "{}", line);
    }
}

pub(crate) async fn read_container_port(name: &str) -> Result<u16, HarnessError> {
    let output = tokio::process::Command::new("docker")
        .args(["port", name, &OPENCODE_DEFAULT_PORT.to_string()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .map_err(|e| HarnessError::ServerLaunch(format!("failed to read container port: {e}")))?;

    let port_line = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let port = port_line
        .rsplit(':')
        .next()
        .ok_or_else(|| {
            HarnessError::ServerLaunch(format!("unexpected docker port output: {port_line}"))
        })?
        .parse::<u16>()
        .map_err(|e| HarnessError::ServerLaunch(format!("invalid port in docker output: {e}")))?;

    Ok(port)
}
