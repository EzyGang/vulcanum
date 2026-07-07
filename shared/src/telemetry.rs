use std::sync::atomic::{AtomicBool, Ordering};

static TELEMETRY_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    let _ = try_init();
}

pub fn try_init() -> anyhow::Result<()> {
    let debug = is_debug_enabled();
    let log_format = std::env::var("LOG_FORMAT").ok();
    init_inner(debug, log_format.as_deref())
}

pub fn init_with_config(debug: bool, log_format: Option<&str>) {
    let _ = try_init_with_config(debug, log_format);
}

pub fn try_init_with_config(debug: bool, log_format: Option<&str>) -> anyhow::Result<()> {
    init_inner(debug, log_format)
}

fn init_inner(debug: bool, log_format: Option<&str>) -> anyhow::Result<()> {
    if TELEMETRY_INITIALIZED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(());
    }

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let default_level = if debug { "debug" } else { "warn" };
        tracing_subscriber::EnvFilter::new(default_level)
    });

    let init_result = match log_format {
        Some("json") => tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(false)
            .try_init(),
        _ => tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .try_init(),
    };

    if let Err(err) = init_result {
        TELEMETRY_INITIALIZED.store(false, Ordering::Release);
        anyhow::bail!("failed to initialize telemetry subscriber: {err}");
    }

    let _ = tracing_log::LogTracer::init();
    Ok(())
}

pub fn is_debug_enabled() -> bool {
    std::env::var("VULCANUM_DEBUG")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}
