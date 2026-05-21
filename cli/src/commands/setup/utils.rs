use std::process::Command;

pub fn which(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn run_shell(cmd: &str) -> anyhow::Result<()> {
    let status = Command::new("sh")
        .args(["-c", cmd])
        .status()
        .map_err(|e| anyhow::anyhow!("shell command failed: {e}"))?;

    if !status.success() {
        anyhow::bail!("shell command failed: {cmd}");
    }
    Ok(())
}

pub fn run_systemctl(args: &str) -> anyhow::Result<()> {
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
