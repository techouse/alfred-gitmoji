use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use ureq::Agent;
use url::Url;

use crate::models::SearchResult;

const IMAGE_CACHE_URL: &str =
    "https://raw.githubusercontent.com/joypixels/emoji-assets/master/png/32/";
const MAX_WORKERS: usize = 8;
const MAX_IMAGE_BYTES: u64 = 1024 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const IMAGE_TIMEOUT: Duration = Duration::from_secs(3);
const BATCH_BUDGET: Duration = Duration::from_secs(5);
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

type Diagnostic = dyn Fn(String) + Send + Sync;

#[cfg(test)]
type TestFetcher = dyn Fn(&Url) -> Result<Vec<u8>> + Send + Sync;

/// Best-effort concurrent cache for Gitmoji image assets.
pub struct EmojiImageCache {
    directory: PathBuf,
    base_url: Url,
    agent: Agent,
    pool: ThreadPool,
    verbose: bool,
    diagnostic: Arc<Diagnostic>,
    batch_budget: Duration,
    #[cfg(test)]
    test_fetcher: Option<Arc<TestFetcher>>,
}

impl EmojiImageCache {
    /// Creates an image cache that downloads JoyPixels PNGs into directory.
    pub fn new(directory: impl Into<PathBuf>, verbose: bool) -> Result<Self> {
        let agent: Agent = Agent::config_builder()
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(IMAGE_TIMEOUT))
            .build()
            .into();
        let pool = ThreadPoolBuilder::new()
            .num_threads(MAX_WORKERS)
            .thread_name(|index| format!("gitmoji-image-{index}"))
            .build()
            .map_err(|error| anyhow!("failed to create image worker pool: {error}"))?;

        Ok(Self {
            directory: directory.into(),
            base_url: Url::parse(IMAGE_CACHE_URL)?,
            agent,
            pool,
            verbose,
            diagnostic: Arc::new(|message| eprintln!("{message}")),
            batch_budget: BATCH_BUDGET,
            #[cfg(test)]
            test_fetcher: None,
        })
    }

    /// Resolves cached or downloaded image paths in the same order as results.
    ///
    /// A missing, invalid, or failed image is represented by None.
    /// Such failures never prevent an Alfred result from being shown.
    pub fn resolve_many(&self, results: &[SearchResult]) -> Vec<Option<PathBuf>> {
        let deadline = Instant::now() + self.batch_budget;

        self.pool.install(|| {
            results
                .par_iter()
                .map(|result| self.resolve_one(result, deadline))
                .collect()
        })
    }

    fn resolve_one(&self, result: &SearchResult, deadline: Instant) -> Option<PathBuf> {
        let filename = match image_filename(&result.emoji) {
            Ok(filename) => filename,
            Err(error) => {
                self.log(format!(
                    "could not resolve image for {}: {error}",
                    result.code
                ));
                return None;
            }
        };
        let target = self.directory.join(&filename);

        if self.valid_cached_image(&target) {
            return Some(target);
        }
        if Instant::now() >= deadline {
            self.log(format!(
                "skipping {} because the image batch deadline elapsed",
                result.code
            ));
            return None;
        }

        let image_url = match self.base_url.join(&filename) {
            Ok(url) => url,
            Err(error) => {
                self.log(format!(
                    "could not build image URL for {}: {error}",
                    result.code
                ));
                return None;
            }
        };
        let bytes = match self.download(&image_url) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.log(format!(
                    "could not download image for {}: {error}",
                    result.code
                ));
                return None;
            }
        };
        if let Err(error) = self.write_atomically(&target, &bytes) {
            self.log(format!(
                "could not cache image for {}: {error}",
                result.code
            ));
            return None;
        }

        Some(target)
    }

    fn download(&self, url: &Url) -> Result<Vec<u8>> {
        #[cfg(test)]
        if let Some(fetcher) = &self.test_fetcher {
            return fetcher(url);
        }

        let mut response = self
            .agent
            .get(url.as_str())
            .call()
            .map_err(|error| anyhow!("image request failed: {error}"))?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_IMAGE_BYTES)
            .read_to_vec()
            .map_err(|error| anyhow!("failed to read image response: {error}"))?;

        validate_png(&bytes)?;
        Ok(bytes)
    }

    fn write_atomically(&self, target: &Path, bytes: &[u8]) -> Result<()> {
        validate_png(bytes)?;
        fs::create_dir_all(&self.directory)
            .with_context(|| format!("failed to create {}", self.directory.display()))?;

        if target.is_file() {
            return Ok(());
        }

        let file_name = target
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("cache target does not have a UTF-8 file name"))?;
        let temp_id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = self.directory.join(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            temp_id
        ));
        fs::write(&temporary, bytes)
            .with_context(|| format!("failed to write {}", temporary.display()))?;

        match fs::rename(&temporary, target) {
            Ok(()) => Ok(()),
            Err(_error) if target.is_file() => {
                let _ = fs::remove_file(&temporary);
                Ok(())
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                bail!("failed to move image into cache: {error}");
            }
        }
    }

    fn valid_cached_image(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        let cached_image = fs::metadata(path)
            .ok()
            .filter(|metadata| metadata.len() <= MAX_IMAGE_BYTES)
            .and_then(|_| fs::read(path).ok());
        if cached_image
            .as_deref()
            .is_some_and(|bytes| validate_png(bytes).is_ok())
        {
            return true;
        }

        self.log(format!("removing invalid cached image {}", path.display()));
        let _ = fs::remove_file(path);
        false
    }

    fn log(&self, message: String) {
        if self.verbose {
            (self.diagnostic)(message);
        }
    }

    #[cfg(test)]
    fn with_test_fetcher(
        directory: PathBuf,
        verbose: bool,
        workers: usize,
        batch_budget: Duration,
        fetcher: Arc<TestFetcher>,
        diagnostic: Arc<Diagnostic>,
    ) -> Result<Self> {
        let agent: Agent = Agent::config_builder().build().into();
        let pool = ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .map_err(|error| anyhow!("failed to create image worker pool: {error}"))?;

        Ok(Self {
            directory,
            base_url: Url::parse("https://images.example/")?,
            agent,
            pool,
            verbose,
            diagnostic,
            batch_budget,
            test_fetcher: Some(fetcher),
        })
    }
}

fn image_filename(emoji: &str) -> Result<String> {
    let first = emoji
        .chars()
        .next()
        .ok_or_else(|| anyhow!("emoji must not be empty"))?;
    Ok(format!("{:x}.png", u32::from(first)))
}

fn validate_png(bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_IMAGE_BYTES as usize {
        bail!("image exceeds the 1 MiB limit");
    }
    if !bytes.starts_with(&PNG_SIGNATURE) {
        bail!("image response is not a PNG");
    }

    let mut position = PNG_SIGNATURE.len();
    let mut saw_header = false;
    let mut saw_image_data = false;

    while position < bytes.len() {
        let remaining = &bytes[position..];
        if remaining.len() < 12 {
            bail!("PNG chunk is incomplete");
        }

        let data_length = u32::from_be_bytes(remaining[..4].try_into().expect("length slice"));
        let data_start = position + 8;
        let data_end = data_start
            .checked_add(data_length as usize)
            .ok_or_else(|| anyhow!("PNG chunk length overflows"))?;
        let chunk_end = data_end
            .checked_add(4)
            .ok_or_else(|| anyhow!("PNG chunk length overflows"))?;
        if chunk_end > bytes.len() {
            bail!("PNG chunk is incomplete");
        }

        let chunk_type = &bytes[position + 4..data_start];
        let data = &bytes[data_start..data_end];
        let expected_crc =
            u32::from_be_bytes(bytes[data_end..chunk_end].try_into().expect("CRC slice"));
        let actual_crc = png_crc32(&bytes[position + 4..data_end]);
        if actual_crc != expected_crc {
            bail!("PNG chunk CRC does not match");
        }

        match chunk_type {
            b"IHDR" if !saw_header && data.len() == 13 => saw_header = true,
            b"IHDR" => bail!("PNG header is invalid or duplicated"),
            b"IDAT" if saw_header => saw_image_data = true,
            b"IDAT" => bail!("PNG data appears before its header"),
            b"IEND"
                if saw_header && saw_image_data && data.is_empty() && chunk_end == bytes.len() =>
            {
                return Ok(());
            }
            b"IEND" => bail!("PNG end chunk is invalid"),
            _ if !saw_header => bail!("PNG does not begin with an IHDR chunk"),
            _ => {}
        }

        position = chunk_end;
    }

    bail!("PNG is missing its end chunk")
}

fn png_crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;

    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }

    !crc
}

#[cfg(test)]
#[path = "tests/emoji_image_cache.rs"]
mod tests;
