use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Context;
use flate2::read::GzDecoder;
use futures::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

const CLI_BINARY: &str = "vulcanum";
const WORKER_BINARY: &str = "vulcanum-server";

pub(super) async fn download(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
) -> anyhow::Result<()> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to download {url}"))?
        .error_for_status()
        .with_context(|| format!("download returned an error for {url}"))?;
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(destination)
        .await
        .with_context(|| format!("failed to create {}", destination.display()))?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| format!("failed while downloading {url}"))?;
        file.write_all(&chunk)
            .await
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }
    file.flush()
        .await
        .with_context(|| format!("failed to flush {}", destination.display()))?;
    Ok(())
}

pub(super) fn verify_and_extract(
    archive_path: &Path,
    checksum_path: &Path,
    staging_dir: &Path,
) -> anyhow::Result<()> {
    verify_checksum(archive_path, checksum_path)?;
    extract_pair(archive_path, staging_dir)
}

fn verify_checksum(archive_path: &Path, checksum_path: &Path) -> anyhow::Result<()> {
    let checksum = std::fs::read_to_string(checksum_path)
        .with_context(|| format!("failed to read checksum file {}", checksum_path.display()))?;
    let expected = checksum
        .split_whitespace()
        .next()
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| anyhow::anyhow!("checksum file does not contain a valid SHA-256 digest"))?;

    let mut file = File::open(archive_path)
        .with_context(|| format!("failed to open archive {}", archive_path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read archive {}", archive_path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected) {
        anyhow::bail!("archive checksum verification failed");
    }
    Ok(())
}

fn extract_pair(archive_path: &Path, staging_dir: &Path) -> anyhow::Result<()> {
    let file = File::open(archive_path)
        .with_context(|| format!("failed to open archive {}", archive_path.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut cli_found = false;
    let mut worker_found = false;

    for entry in archive
        .entries()
        .context("failed to read release archive")?
    {
        let mut entry = entry.context("failed to read release archive entry")?;
        if !entry.header().entry_type().is_file() {
            anyhow::bail!("release archive contains a non-file entry");
        }
        let path = entry
            .path()
            .context("release archive contains an invalid path")?;
        let name = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("release archive contains a non-UTF-8 path"))?
            .to_owned();
        let destination = match name.as_str() {
            CLI_BINARY if !cli_found => {
                cli_found = true;
                staging_dir.join(CLI_BINARY)
            }
            WORKER_BINARY if !worker_found => {
                worker_found = true;
                staging_dir.join(WORKER_BINARY)
            }
            CLI_BINARY | WORKER_BINARY => {
                anyhow::bail!("release archive contains duplicate binary {name}")
            }
            _ => anyhow::bail!("release archive contains unexpected file {name}"),
        };
        entry
            .unpack(&destination)
            .with_context(|| format!("failed to stage {name}"))?;
        set_executable(&destination)?;
    }

    if !cli_found || !worker_found {
        anyhow::bail!("release archive must contain vulcanum and vulcanum-server");
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .with_context(|| format!("failed to mark {} executable", path.display()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
