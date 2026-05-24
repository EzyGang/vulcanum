use super::utils::{run_shell, which};

pub(super) const KATA_MANAGER_URL: &str =
    "https://raw.githubusercontent.com/kata-containers/kata-containers/main/utils/kata-manager.sh";

const KATA_MANAGER_TMP: &str = "/tmp/kata-manager.sh";

pub fn install_kata() -> anyhow::Result<()> {
    if which("kata-runtime") {
        tracing::debug!("kata-runtime already installed");
        return Ok(());
    }

    run_shell(&format!(
        "curl -fsSL {KATA_MANAGER_URL} -o {KATA_MANAGER_TMP} && bash {KATA_MANAGER_TMP} -D"
    ))?;

    let _ = std::fs::remove_file(KATA_MANAGER_TMP);
    Ok(())
}
