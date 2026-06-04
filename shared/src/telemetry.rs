pub fn init() {
    let debug = is_debug_enabled();
    let log_format = std::env::var("LOG_FORMAT").ok();
    init_inner(debug, log_format.as_deref());
}

pub fn init_with_config(debug: bool, log_format: Option<&str>) {
    init_inner(debug, log_format);
}

fn init_inner(debug: bool, log_format: Option<&str>) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let default_level = if debug { "debug" } else { "warn" };
        tracing_subscriber::EnvFilter::new(default_level)
    });

    match log_format {
        Some("json") => {
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
