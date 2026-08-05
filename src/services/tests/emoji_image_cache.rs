use super::*;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

use crate::app::items_from_results;

fn result(index: usize) -> SearchResult {
    let emoji = char::from_u32(0x1f400 + index as u32)
        .expect("test emoji should be a valid Unicode scalar")
        .to_string();

    SearchResult {
        object_id: format!("id-{index}"),
        emoji,
        entity: String::new(),
        code: format!(":code-{index}:"),
        description: format!("description-{index}"),
        name: format!("name-{index}"),
        semver: None,
    }
}

fn png_bytes() -> Vec<u8> {
    let mut bytes = PNG_SIGNATURE.to_vec();
    append_png_chunk(
        &mut bytes,
        b"IHDR",
        &[0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0],
    );
    append_png_chunk(
        &mut bytes,
        b"IDAT",
        &[
            0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0xf0, 0x1f, 0, 5, 0, 1, 0xff,
        ],
    );
    append_png_chunk(&mut bytes, b"IEND", &[]);
    bytes
}

fn append_png_chunk(bytes: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(chunk_type);
    bytes.extend_from_slice(data);
    bytes.extend_from_slice(&png_crc32(&[chunk_type.as_slice(), data].concat()).to_be_bytes());
}

fn no_op_diagnostic() -> Arc<Diagnostic> {
    Arc::new(|_| {})
}

#[test]
fn image_filename_uses_the_first_unicode_scalar() -> Result<()> {
    assert_eq!(image_filename("🐛")?, "1f41b.png");
    assert_eq!(image_filename("👩‍💻")?, "1f469.png");

    Ok(())
}

#[test]
fn validate_png_rejects_non_png_content() {
    assert!(validate_png(b"not an image").is_err());
}

#[test]
fn resolve_many_limits_concurrency_and_preserves_result_order() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let active = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let fetcher = {
        let active = active.clone();
        let maximum = maximum.clone();
        Arc::new(move |url: &Url| {
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            maximum.fetch_max(current, Ordering::SeqCst);
            let filename = url.path().rsplit('/').next().unwrap_or_default();
            let delay = filename
                .strip_suffix(".png")
                .and_then(|value| u32::from_str_radix(value, 16).ok())
                .map_or(0, |value| value % 8);
            std::thread::sleep(Duration::from_millis(u64::from(8 - delay) * 20));
            active.fetch_sub(1, Ordering::SeqCst);
            Ok(png_bytes())
        }) as Arc<TestFetcher>
    };
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        false,
        MAX_WORKERS,
        BATCH_BUDGET,
        fetcher,
        no_op_diagnostic(),
    )?;
    let results = (0..16).map(result).collect::<Vec<_>>();

    let paths = cache.resolve_many(&results);
    let items = items_from_results(&results, &paths)?;

    assert!(maximum.load(Ordering::SeqCst) <= MAX_WORKERS);
    assert_eq!(maximum.load(Ordering::SeqCst), MAX_WORKERS);
    assert_eq!(
        items.iter().map(|item| item.title()).collect::<Vec<_>>(),
        results
            .iter()
            .map(|result| result.code.as_str())
            .collect::<Vec<_>>()
    );
    assert!(paths.iter().all(Option::is_some));

    Ok(())
}

#[test]
fn resolve_many_uses_a_cached_image_without_a_request() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let search_result = result(1);
    let path = directory.path().join(image_filename(&search_result.emoji)?);
    fs::write(&path, png_bytes())?;
    let requests = Arc::new(AtomicUsize::new(0));
    let fetcher = {
        let requests = requests.clone();
        Arc::new(move |_: &Url| {
            requests.fetch_add(1, Ordering::SeqCst);
            Err(anyhow!("cached image should not be requested"))
        }) as Arc<TestFetcher>
    };
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        false,
        MAX_WORKERS,
        BATCH_BUDGET,
        fetcher,
        no_op_diagnostic(),
    )?;

    let paths = cache.resolve_many(&[search_result]);

    assert_eq!(paths, vec![Some(path)]);
    assert_eq!(requests.load(Ordering::SeqCst), 0);

    Ok(())
}

#[test]
fn resolve_many_replaces_an_invalid_cached_image() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let search_result = result(1);
    let path = directory.path().join(image_filename(&search_result.emoji)?);
    fs::write(&path, b"incomplete image")?;
    let requests = Arc::new(AtomicUsize::new(0));
    let fetcher = {
        let requests = requests.clone();
        Arc::new(move |_: &Url| {
            requests.fetch_add(1, Ordering::SeqCst);
            Ok(png_bytes())
        }) as Arc<TestFetcher>
    };
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        false,
        MAX_WORKERS,
        BATCH_BUDGET,
        fetcher,
        no_op_diagnostic(),
    )?;

    let paths = cache.resolve_many(&[search_result]);

    assert_eq!(paths, vec![Some(path.clone())]);
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(fs::read(path)?, png_bytes());

    Ok(())
}

#[test]
fn resolve_many_uses_fallbacks_for_failed_or_invalid_images() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let diagnostics = Arc::new(Mutex::new(Vec::new()));
    let fetcher = Arc::new(|url: &Url| {
        let filename = url.path().rsplit('/').next().unwrap_or_default();
        match filename {
            "1f400.png" => Err(anyhow!("HTTP 404")),
            "1f401.png" => Err(anyhow!("request timed out")),
            "1f402.png" => {
                let mut bytes = png_bytes();
                bytes.resize(MAX_IMAGE_BYTES as usize + 1, 0);
                Ok(bytes)
            }
            "1f403.png" => Ok(PNG_SIGNATURE.to_vec()),
            _ => Ok(png_bytes()),
        }
    }) as Arc<TestFetcher>;
    let diagnostic = {
        let diagnostics = diagnostics.clone();
        Arc::new(move |message| diagnostics.lock().expect("lock").push(message)) as Arc<Diagnostic>
    };
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        true,
        MAX_WORKERS,
        BATCH_BUDGET,
        fetcher,
        diagnostic,
    )?;

    let paths = cache.resolve_many(&(0..5).map(result).collect::<Vec<_>>());

    assert_eq!(paths[..4], [None, None, None, None]);
    assert!(paths[4].is_some());
    assert_eq!(diagnostics.lock().expect("lock").len(), 4);

    Ok(())
}

#[test]
fn resolve_many_respects_an_elapsed_batch_deadline() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let requests = Arc::new(AtomicUsize::new(0));
    let fetcher = {
        let requests = requests.clone();
        Arc::new(move |_: &Url| {
            requests.fetch_add(1, Ordering::SeqCst);
            Ok(png_bytes())
        }) as Arc<TestFetcher>
    };
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        false,
        1,
        Duration::ZERO,
        fetcher,
        no_op_diagnostic(),
    )?;

    let paths = cache.resolve_many(&[result(1)]);

    assert_eq!(paths, vec![None]);
    assert_eq!(requests.load(Ordering::SeqCst), 0);

    Ok(())
}

#[test]
fn failed_image_diagnostics_require_verbose_mode() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let diagnostics = Arc::new(Mutex::new(Vec::new()));
    let diagnostic = {
        let diagnostics = diagnostics.clone();
        Arc::new(move |message| diagnostics.lock().expect("lock").push(message)) as Arc<Diagnostic>
    };
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        false,
        MAX_WORKERS,
        BATCH_BUDGET,
        Arc::new(|_: &Url| Err(anyhow!("request failed"))) as Arc<TestFetcher>,
        diagnostic,
    )?;

    let paths = cache.resolve_many(&[result(1)]);

    assert_eq!(paths, vec![None]);
    assert!(diagnostics.lock().expect("lock").is_empty());

    Ok(())
}

#[test]
fn concurrent_resolvers_leave_one_complete_cached_image() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let fetcher = Arc::new(|_: &Url| {
        std::thread::sleep(Duration::from_millis(20));
        Ok(png_bytes())
    }) as Arc<TestFetcher>;
    let cache = Arc::new(EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        false,
        MAX_WORKERS,
        BATCH_BUDGET,
        fetcher,
        no_op_diagnostic(),
    )?);
    let search_result = result(1);
    let concurrent_cache = cache.clone();
    let concurrent_result = search_result.clone();
    let handle = std::thread::spawn(move || concurrent_cache.resolve_many(&[concurrent_result]));

    let first_paths = cache.resolve_many(std::slice::from_ref(&search_result));
    let second_paths = handle.join().expect("resolver thread should not panic");
    let image_path = directory.path().join(image_filename(&search_result.emoji)?);

    assert!(first_paths[0].is_some());
    assert!(second_paths[0].is_some());
    assert_eq!(fs::read(&image_path)?, png_bytes());
    assert!(fs::read_dir(directory.path())?.all(|entry| {
        !entry
            .expect("directory entry")
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));

    Ok(())
}
