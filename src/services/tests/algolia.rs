use super::*;

#[test]
fn endpoint_uses_single_index_search_route() -> Result<()> {
    let client = AlgoliaSearch::with_base_url(
        AlgoliaSearchConfig {
            application_id: "app".to_owned(),
            api_key: "key".to_owned(),
            index_name: "gitmoji".to_owned(),
        },
        Url::parse("http://127.0.0.1:8080/api/")?,
    )?;

    assert_eq!(
        client.endpoint()?.as_str(),
        "http://127.0.0.1:8080/api/1/indexes/gitmoji/query"
    );

    Ok(())
}
