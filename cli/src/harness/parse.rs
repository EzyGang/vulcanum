use regex::Regex;

pub(super) fn parse_pr_url(text: &str) -> Option<String> {
    static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"https://github\.com/[^/\s]+/[^/\s]+/pull/\d+").unwrap()
    });
    RE.find(text).map(|m| m.as_str().to_owned())
}

pub(super) fn parse_token_usage(text: &str) -> u64 {
    static RE: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"Tokens used:\s*(\d+)").unwrap());
    RE.captures(text)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0)
}
