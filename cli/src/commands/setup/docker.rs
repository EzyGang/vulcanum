use std::process::Command;

pub fn install_docker() -> anyhow::Result<()> {
    if is_docker_installed() {
        tracing::info!("Docker is already installed");
        return Ok(());
    }

    tracing::info!("installing Docker...");

    install_prerequisites()?;
    add_docker_repo()?;
    install_docker_packages()?;
    enable_docker_service()?;

    tracing::info!("Docker installed successfully");
    Ok(())
}

fn is_docker_installed() -> bool {
    which("docker")
}

fn install_prerequisites() -> anyhow::Result<()> {
    run_apt("update")?;
    run_apt("install -y apt-transport-https ca-certificates curl gnupg")
}

fn add_docker_repo() -> anyhow::Result<()> {
    let keyring_path = "/etc/apt/keyrings/docker.gpg";

    run_shell(&format!(
        "install -m 0755 -d /etc/apt/keyrings && \
         curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
         gpg --dearmor -o {keyring_path} && \
         chmod a+r {keyring_path}"
    ))?;

    let codename = lsb_codename()?;
    let arch = std::env::consts::ARCH;

    let list_path = "/etc/apt/sources.list.d/docker.list";
    let repo_line = format!(
        "deb [arch={arch} signed-by={keyring_path}] \
         https://download.docker.com/linux/ubuntu {codename} stable"
    );

    std::fs::write(list_path, format!("{repo_line}\n"))?;
    run_apt("update")
}

fn install_docker_packages() -> anyhow::Result<()> {
    run_apt("install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin")
}

fn enable_docker_service() -> anyhow::Result<()> {
    run_systemctl("enable --now docker")
}

fn lsb_codename() -> anyhow::Result<String> {
    let output = Command::new("lsb_release")
        .args(["-cs"])
        .output()
        .map_err(|e| anyhow::anyhow!("lsb_release not found — are you on Ubuntu? ({e})"))?;

    if !output.status.success() {
        anyhow::bail!("lsb_release failed — are you on Ubuntu?");
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn which(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_apt(args: &str) -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .arg("apt-get")
        .args(args.split_whitespace())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run apt-get: {e}"))?;

    if !status.success() {
        anyhow::bail!("apt-get {} failed", args);
    }
    Ok(())
}

fn run_shell(cmd: &str) -> anyhow::Result<()> {
    let status = Command::new("sh")
        .args(["-c", cmd])
        .status()
        .map_err(|e| anyhow::anyhow!("shell command failed: {e}"))?;

    if !status.success() {
        anyhow::bail!("shell command failed: {cmd}");
    }
    Ok(())
}

fn run_systemctl(args: &str) -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .arg("systemctl")
        .args(args.split_whitespace())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run systemctl: {e}"))?;

    if !status.success() {
        anyhow::bail!("systemctl {} failed", args);
    }
    Ok(())
}
