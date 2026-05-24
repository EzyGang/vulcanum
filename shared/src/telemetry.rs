pub fn init() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let default_level = if is_debug_enabled() { "debug" } else { "warn" };
        tracing_subscriber::EnvFilter::new(default_level)
    });

    match std::env::var("LOG_FORMAT").as_deref() {
        Ok("json") => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .with_target(false)
                .init();
        }
        _ => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .init();
        }
    }

    tracing_log::LogTracer::init().ok();
}

pub fn is_debug_enabled() -> bool {
    std::env::var("VULCANUM_DEBUG")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}
