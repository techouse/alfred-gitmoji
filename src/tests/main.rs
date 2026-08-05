use std::cell::Cell;

use alfred_workflow_rs::FileCache;
use anyhow::anyhow;

use super::*;

#[test]
fn file_cache_hit_bypasses_algolia_and_image_resolution() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut cached_workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    cached_workflow.set_cache_key(Some("cached-query"));
    cached_workflow.add_item(Item::new("cached result"))?;

    let mut workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    let algolia_calls = Cell::new(0);
    let image_resolution_calls = Cell::new(0);

    populate_workflow_with(
        &mut workflow,
        "query",
        |workflow, _| {
            workflow.set_cache_key(Some("cached-query"));
            Ok(())
        },
        |_| {
            algolia_calls.set(algolia_calls.get() + 1);
            Err(anyhow!("Algolia must not run for cached items"))
        },
        |_| {
            image_resolution_calls.set(image_resolution_calls.get() + 1);
            Err(anyhow!("image resolution must not run for cached items"))
        },
    )?;

    assert_eq!(algolia_calls.get(), 0);
    assert_eq!(image_resolution_calls.get(), 0);
    assert_eq!(workflow.get_items()?.len(), 1);

    Ok(())
}
