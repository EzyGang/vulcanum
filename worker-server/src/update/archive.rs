use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use anyhow::Context;
use flate2::read::GzDecoder;
use futures::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

const CLI_BINARY: &str = "vulcanum";
const WORKER_BINARY: &str = "vulcanum-server";
pub(super) const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
pub(super) const MAX_CHECKSUM_BYTES: u64 = 4 * 1024;
const MAX_BINARY_BYTES: u64 = 128 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 2 * MAX_BINARY_BYTES;
const MAX_ARCHIVE_METADATA_BYTES: u64 = 1024 * 1024;
const MAX_DECOMPRESSED_BYTES: u64 = MAX_EXTRACTED_BYTES + MAX_ARCHIVE_METADATA_BYTES;

struct LimitedReader<R> {
    inner: R,
    remaining: u64,
    limit: u64,
}

impl<R> Read for LimitedReader<R>
where
    R: Read,
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            let mut probe = [0_u8; 1];
            return match self.inner.read(&mut probe)? {
                0 => Ok(0),
                _ => Err(io::Error::other(format!(
                    "release archive exceeds the {}-byte decompressed limit",
                    self.limit
                ))),
            };
        }

        let allowed = self.remaining.min(buffer.len() as u64) as usize;
        let read = self.inner.read(&mut buffer[..allowed])?;
        self.remaining -= read as u64;
        Ok(read)
    }
}

pub(super) async fn download(
    client: &reqwest::Client,
    url: &str,
    destination: &Path,
    max_bytes: u64,
) -> anyhow::Result<()> {
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("failed to download {url}"))?
        .error_for_status()
        .with_context(|| format!("download returned an error for {url}"))?;
    match response.content_length() {
        Some(length) if length > max_bytes => {
            anyhow::bail!("download from {url} exceeds the {max_bytes}-byte limit");
        }
        _ => (),
    }
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(destination)
        .await
        .with_context(|| format!("failed to create {}", destination.display()))?;
    let mut downloaded = 0_u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| format!("failed while downloading {url}"))?;
        downloaded = downloaded
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| anyhow::anyhow!("download size overflow for {url}"))?;
        if downloaded > max_bytes {
            drop(file);
            let _ = tokio::fs::remove_file(destination).await;
            anyhow::bail!("download from {url} exceeds the {max_bytes}-byte limit");
        }
        file.write_all(&chunk)
            .await
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }
    file.flush()
        .await
        .with_context(|| format!("failed to flush {}", destination.display()))?;
    file.sync_all()
        .await
        .with_context(|| format!("failed to sync {}", destination.display()))?;
    Ok(())
}

pub(super) fn verify_and_extract(
    archive_path: &Path,
    checksum_path: &Path,
    staging_dir: &Path,
) -> anyhow::Result<()> {
    verify_checksum(archive_path, checksum_path)?;
    extract_pair(
        archive_path,
        staging_dir,
        MAX_BINARY_BYTES,
        MAX_EXTRACTED_BYTES,
        MAX_DECOMPRESSED_BYTES,
    )
}
#[cfg(test)]
pub(super) fn verify_and_extract_with_limits(
    archive_path: &Path,
    checksum_path: &Path,
    staging_dir: &Path,
    max_binary_bytes: u64,
    max_extracted_bytes: u64,
    max_decompressed_bytes: u64,
) -> anyhow::Result<()> {
    verify_checksum(archive_path, checksum_path)?;
    extract_pair(
        archive_path,
        staging_dir,
        max_binary_bytes,
        max_extracted_bytes,
        max_decompressed_bytes,
    )
}

fn verify_checksum(archive_path: &Path, checksum_path: &Path) -> anyhow::Result<()> {
    let checksum_size = std::fs::metadata(checksum_path)
        .with_context(|| {
            format!(
                "failed to inspect checksum file {}",
                checksum_path.display()
            )
        })?
        .len();
    if checksum_size > MAX_CHECKSUM_BYTES {
        anyhow::bail!("checksum file exceeds the {MAX_CHECKSUM_BYTES}-byte limit");
    }
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

fn extract_pair(
    archive_path: &Path,
    staging_dir: &Path,
    max_binary_bytes: u64,
    max_extracted_bytes: u64,
    max_decompressed_bytes: u64,
) -> anyhow::Result<()> {
    let file = File::open(archive_path)
        .with_context(|| format!("failed to open archive {}", archive_path.display()))?;
    let decoder = GzDecoder::new(file);
    let limited = LimitedReader {
        inner: decoder,
        remaining: max_decompressed_bytes,
        limit: max_decompressed_bytes,
    };
    let mut archive = tar::Archive::new(limited);
    let mut cli_found = false;
    let mut worker_found = false;
    let mut extracted_bytes = 0_u64;

    for entry in archive
        .entries()
        .context("failed to read release archive")?
    {
        let mut entry = entry.context("failed to read release archive entry")?;
        if !entry.header().entry_type().is_file() {
            anyhow::bail!("release archive contains a non-file entry");
        }
        let entry_size = entry.size();
        if entry_size > max_binary_bytes {
            anyhow::bail!("release archive binary exceeds the {max_binary_bytes}-byte limit");
        }
        extracted_bytes = extracted_bytes
            .checked_add(entry_size)
            .ok_or_else(|| anyhow::anyhow!("release archive size overflow"))?;
        if extracted_bytes > max_extracted_bytes {
            anyhow::bail!(
                "release archive exceeds the {max_extracted_bytes}-byte extraction limit"
            );
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
