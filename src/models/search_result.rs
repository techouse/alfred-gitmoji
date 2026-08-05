use serde::Deserialize;

/// A Gitmoji result returned by the Algolia search index.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SearchResult {
    /// Algolia object identifier.
    #[serde(rename = "objectID")]
    pub object_id: String,
    /// Unicode emoji value.
    pub emoji: String,
    /// HTML entity representation.
    pub entity: String,
    /// Gitmoji code such as :bug:.
    pub code: String,
    /// Human-readable result description.
    pub description: String,
    /// Gitmoji name.
    pub name: String,
    /// Optional Gitmoji semantic version.
    pub semver: Option<String>,
}

/// Minimal subset of the Algolia single-index search response.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    /// Search results in provider-defined ranking order.
    pub hits: Vec<SearchResult>,
}
