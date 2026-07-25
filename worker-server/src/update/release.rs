use anyhow::Context;
use semver::Version;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct GithubRelease {
    pub(super) tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct ReleasePackage {
    pub(super) archive_name: String,
    pub(super) archive_url: String,
    pub(super) checksum_url: String,
}

pub(super) async fn fetch_latest_release(
    client: &reqwest::Client,
    api_url: &str,
) -> anyhow::Result<GithubRelease> {
    client
        .get(api_url)
        .send()
        .await
        .context("failed to request the latest GitHub release")?
        .error_for_status()
        .context("GitHub returned an error for the latest release request")?
        .json()
        .await
        .context("failed to decode the latest GitHub release")
}

pub(super) fn select_package(
    release: &GithubRelease,
    target: &str,
) -> anyhow::Result<ReleasePackage> {
    let archive_name = format!("vulcanum-{target}.tar.gz");
    let checksum_name = format!("{archive_name}.sha256");
    let archive_url = asset_url(release, &archive_name)?;
    let checksum_url = asset_url(release, &checksum_name)?;

    Ok(ReleasePackage {
        archive_name,
        archive_url,
        checksum_url,
    })
}

pub(super) fn is_newer(current: &str, candidate: &str) -> anyhow::Result<bool> {
    let current = parse_version(current).context("installed release version is invalid")?;
    let candidate = parse_version(candidate).context("published release version is invalid")?;
    Ok(candidate > current)
}

pub(super) fn current_target() -> anyhow::Result<&'static str> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "linux") => Ok("x86_64-unknown-linux-gnu"),
        ("aarch64", "linux") => Ok("aarch64-unknown-linux-gnu"),
        ("x86_64", "macos") => Ok("x86_64-apple-darwin"),
        ("aarch64", "macos") => Ok("aarch64-apple-darwin"),
        (arch, os) => anyhow::bail!("automatic updates are unsupported on {os}/{arch}"),
    }
}

fn parse_version(value: &str) -> anyhow::Result<Version> {
    let value = value.strip_prefix('v').unwrap_or(value);
    Version::parse(value).map_err(Into::into)
}

fn asset_url(release: &GithubRelease, name: &str) -> anyhow::Result<String> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .map(|asset| asset.browser_download_url.clone())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "release {} does not contain compatible asset {name}",
                release.tag_name
            )
        })
}
