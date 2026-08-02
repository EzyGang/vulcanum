use vulcanum_shared::config::{load_config, save_config};

use crate::console;

/// Sets verified automatic worker-side updates in the local configuration.
pub(crate) fn run(enabled: bool) -> anyhow::Result<()> {
    let mut config = load_config()?;
    config.auto_update_enabled = enabled;
    save_config(&config)?;

    let state = if config.auto_update_enabled {
        "enabled"
    } else {
        "disabled"
    };
    console::info(&format!(
        "Verified automatic worker updates are {state}. The worker daemon will apply this setting on its next startup."
    ));
    Ok(())
}
