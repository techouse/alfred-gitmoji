#![forbid(unsafe_code)]

mod cli;
mod config;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use alfred_gitmoji::app::{items_from_results, no_results_item};
use alfred_gitmoji::models::SearchResult;
use alfred_gitmoji::services::{AlgoliaSearch, EmojiImageCache};
use alfred_workflow_rs::{Icon, Item, RenderOptions, Updater, UserConfiguration, Workflow};
use anyhow::Result;

use crate::cli::Cli;

const GITHUB_REPOSITORY_URL: &str = "https://github.com/techouse/alfred-gitmoji";
const UPDATE_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

fn main() -> ExitCode {
    let cli = match Cli::parse(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };

    if cli.update {
        return update_workflow();
    }

    let mut workflow = Workflow::new();
    workflow.set_disable_alfred_smart_result_ordering(true);

    let (options, exit_code) = match populate_workflow(&mut workflow, &cli) {
        Ok(()) => (update_render_options(&cli), ExitCode::SUCCESS),
        Err(error) => {
            let _ = workflow.clear_items();
            if let Err(add_error) = workflow.add_item(Item::new(error.to_string())) {
                eprintln!("failed to render workflow error: {add_error}");
                return ExitCode::from(1);
            }
            (RenderOptions::new(), ExitCode::from(1))
        }
    };

    if let Err(error) = workflow.write_stdout_with(options) {
        eprintln!("failed to write Script Filter JSON: {error}");
        return ExitCode::from(1);
    }

    exit_code
}

fn populate_workflow(workflow: &mut Workflow, cli: &Cli) -> Result<()> {
    let query = cli.normalized_query();
    if cli.verbose {
        eprintln!("Query: \"{query}\"");
    }

    populate_workflow_with(
        workflow,
        &query,
        configure_cache,
        |query| {
            let search = AlgoliaSearch::new(config::algolia_search_config()?)?;
            search.query(query)
        },
        |results| {
            let image_cache = EmojiImageCache::new(image_cache_directory()?, cli.verbose)?;
            Ok(image_cache.resolve_many(results))
        },
    )
}

fn populate_workflow_with<C, S, R>(
    workflow: &mut Workflow,
    query: &str,
    configure: C,
    search: S,
    resolve_images: R,
) -> Result<()>
where
    C: FnOnce(&mut Workflow, &str) -> Result<()>,
    S: FnOnce(&str) -> Result<Vec<SearchResult>>,
    R: FnOnce(&[SearchResult]) -> Result<Vec<Option<PathBuf>>>,
{
    configure(workflow, query)?;
    if !workflow.get_items()?.is_empty() {
        return Ok(());
    }

    let results = search(query)?;
    if results.is_empty() {
        workflow.add_item(no_results_item())?;
        return Ok(());
    }

    let image_paths = resolve_images(&results)?;
    workflow.add_items(items_from_results(&results, &image_paths)?)?;

    Ok(())
}

fn configure_cache(workflow: &mut Workflow, query: &str) -> Result<()> {
    let defaults = workflow.get_user_defaults("info.plist", "prefs.plist")?;
    let use_alfred_cache = checkbox_value(&defaults, "use_alfred_cache").unwrap_or(false);
    let use_file_cache = checkbox_value(&defaults, "use_file_cache").unwrap_or(false);
    let cache_ttl = slider_value(&defaults, "cache_ttl");
    let max_entries = slider_value(&defaults, "file_cache_max_entries");

    if use_alfred_cache {
        workflow.set_use_automatic_cache(true);
    } else if use_file_cache {
        let cache_key = if query.is_empty() {
            "ALL_GITMOJIS"
        } else {
            query
        };
        workflow.set_cache_key(Some(cache_key));
        workflow.set_max_cache_entries(max_entries.and_then(|value| usize::try_from(value).ok()));
    }
    workflow.set_cache_time_to_live(cache_ttl.and_then(|value| u64::try_from(value).ok()));

    Ok(())
}

fn checkbox_value(
    defaults: &std::collections::BTreeMap<String, UserConfiguration>,
    variable: &str,
) -> Option<bool> {
    match defaults.get(variable) {
        Some(UserConfiguration::CheckBox(configuration)) => Some(configuration.config.value),
        _ => None,
    }
}

fn slider_value(
    defaults: &std::collections::BTreeMap<String, UserConfiguration>,
    variable: &str,
) -> Option<i64> {
    match defaults.get(variable) {
        Some(UserConfiguration::NumberSlider(configuration)) => Some(configuration.config.value),
        _ => None,
    }
}

fn image_cache_directory() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("workflow executable has no parent directory"))?;

    Ok(directory.join("image_cache"))
}

fn update_render_options(cli: &Cli) -> RenderOptions {
    let updater = match updater() {
        Ok(updater) => updater,
        Err(error) => {
            if cli.verbose {
                eprintln!("could not create updater: {error}");
            }
            return RenderOptions::new();
        }
    };

    match updater.update_available() {
        Ok(true) => RenderOptions::new().add_to_beginning(update_item()),
        Ok(false) => RenderOptions::new(),
        Err(error) => {
            if cli.verbose {
                eprintln!("could not check for updates: {error}");
            }
            RenderOptions::new()
        }
    }
}

fn update_workflow() -> ExitCode {
    eprintln!("Updating workflow...");
    match updater().and_then(|updater| updater.update().map_err(Into::into)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn updater() -> Result<Updater> {
    Ok(
        Updater::builder(GITHUB_REPOSITORY_URL.parse()?, env!("CARGO_PKG_VERSION"))?
            .update_interval(UPDATE_INTERVAL)
            .build()?,
    )
}

fn update_item() -> Item {
    Item::with_arg("Auto-Update available!", "update:workflow")
        .set_subtitle("Press <enter> to auto-update to a new version of this workflow.")
        .set_match_text(
            "Auto-Update available! Press <enter> to auto-update to a new version of this workflow.",
        )
        .set_icon(Icon::new("alfredhatcog.png"))
        .set_valid(true)
}

#[cfg(test)]
#[path = "tests/main.rs"]
mod tests;
