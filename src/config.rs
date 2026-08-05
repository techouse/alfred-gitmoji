use std::env::VarError;

use anyhow::{Result, anyhow};

use alfred_gitmoji::services::AlgoliaSearchConfig;

const EMBEDDED_APPLICATION_ID: Option<&str> = option_env!("ALGOLIA_APPLICATION_ID");
const EMBEDDED_SEARCH_ONLY_API_KEY: Option<&str> = option_env!("ALGOLIA_SEARCH_ONLY_API_KEY");
const EMBEDDED_SEARCH_INDEX: Option<&str> = option_env!("ALGOLIA_SEARCH_INDEX");

/// Loads the Algolia configuration required for a cache miss.
pub fn algolia_search_config() -> Result<AlgoliaSearchConfig> {
    let _ = dotenvy::dotenv();

    Ok(AlgoliaSearchConfig {
        application_id: configuration_value(
            "ALGOLIA_APPLICATION_ID",
            std::env::var("ALGOLIA_APPLICATION_ID"),
            EMBEDDED_APPLICATION_ID,
        )?,
        api_key: configuration_value(
            "ALGOLIA_SEARCH_ONLY_API_KEY",
            std::env::var("ALGOLIA_SEARCH_ONLY_API_KEY"),
            EMBEDDED_SEARCH_ONLY_API_KEY,
        )?,
        index_name: configuration_value(
            "ALGOLIA_SEARCH_INDEX",
            std::env::var("ALGOLIA_SEARCH_INDEX"),
            EMBEDDED_SEARCH_INDEX,
        )?,
    })
}

fn configuration_value(
    name: &str,
    runtime_value: Result<String, VarError>,
    embedded_value: Option<&str>,
) -> Result<String> {
    match runtime_value {
        Ok(value) if value.is_empty() => Err(anyhow!("{name} must not be empty")),
        Ok(value) => Ok(value),
        Err(VarError::NotUnicode(_)) => Err(anyhow!("{name} must contain valid Unicode")),
        Err(VarError::NotPresent) => embedded_value
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| {
                anyhow!(
                    "{name} must be set in the environment, .env file, or embedded at build time"
                )
            }),
    }
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
