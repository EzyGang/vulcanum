const REDACTED_PROVIDER_OUTPUT: &str = "[redacted provider output]";

const SENSITIVE_MARKERS: &[&str] = &[
    "api_key",
    "apikey",
    "authorization",
    "bearer",
    "credential",
    "password",
    "secret",
    "token",
];

#[must_use]
pub(crate) fn redact_provider_output(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if SENSITIVE_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return REDACTED_PROVIDER_OUTPUT.to_owned();
    }

    line.to_owned()
}
