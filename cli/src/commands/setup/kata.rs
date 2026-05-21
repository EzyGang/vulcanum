use std::process::Command;

pub fn install_kata() -> anyhow::Result<()> {
    if is_kata_installed() {
        tracing::info!("kata-runtime is already installed");
        return Ok(());
    }

    tracing::info!("installing Kata Containers...");

    add_kata_repo()?;
    install_kata_packages()?;

    tracing::info!("Kata Containers installed successfully");
    Ok(())
}

fn is_kata_installed() -> bool {
    Command::new("which")
        .arg("kata-runtime")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn add_kata_repo() -> anyhow::Result<()> {
    let codename = lsb_codename()?;
    let arch = std::env::consts::ARCH;

    let list_path = "/etc/apt/sources.list.d/kata-containers.list";
    let repo_line = format!(
        "deb [arch={arch}] https://packages.kata-containers.org/kata-containers/stable/deb {codename} main"
    );

    std::fs::write(list_path, format!("{repo_line}\n"))?;

    let key_url = "https://packages.kata-containers.org/kata-containers/stable/deb/Release.key";
    let keyring_path = "/etc/apt/trusted.gpg.d/kata-containers.asc";

    run_shell(&format!(
        "curl -fsSL {key_url} | gpg --dearmor -o {keyring_path}"
    ))?;

    run_apt("update")
}

fn install_kata_packages() -> anyhow::Result<()> {
    run_apt("install -y kata-runtime")
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
