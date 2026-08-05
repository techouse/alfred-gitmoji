use super::*;

fn search_result(code: &str, emoji: &str) -> SearchResult {
    SearchResult {
        object_id: code.to_owned(),
        emoji: emoji.to_owned(),
        entity: String::new(),
        code: code.to_owned(),
        description: format!("{code} description"),
        name: format!("{code} name"),
        semver: None,
    }
}

#[test]
fn items_preserve_search_order_and_use_the_fallback_icon() -> Result<()> {
    let results = vec![
        search_result(":first:", "🐛"),
        search_result(":second:", "✨"),
    ];
    let items = items_from_results(&results, &[None::<&Path>, None])?;

    assert_eq!(
        items.iter().map(|item| item.title()).collect::<Vec<_>>(),
        vec![":first:", ":second:"]
    );
    assert_eq!(items[0].icon().map(Icon::path), Some("question.png"));

    Ok(())
}

#[test]
fn items_include_expected_modifier_arguments() -> Result<()> {
    let result = search_result(":bug:", "🐛");
    let items = items_from_results(&[result], &[None::<&Path>])?;
    let modifiers = items[0].modifiers().expect("item should have modifiers");

    assert_eq!(
        modifiers.get("shift").and_then(Modifier::arg),
        Some(r#"u"\U0001F41B""#)
    );
    assert_eq!(
        modifiers.get("ctrl").and_then(Modifier::arg),
        Some("&#x1f41b;")
    );
    assert_eq!(
        modifiers.get("ctrl+shift").and_then(Modifier::arg),
        Some("U+1F41B")
    );

    Ok(())
}

#[test]
fn rendered_items_write_valid_script_filter_json() -> Result<()> {
    let items = items_from_results(&[search_result(":bug:", "🐛")], &[None::<&Path>])?;
    let mut workflow = alfred_workflow_rs::Workflow::new();
    workflow.add_items(items)?;
    let mut stdout = Vec::new();

    workflow.write_to(&mut stdout)?;

    let json: serde_json::Value = serde_json::from_slice(&stdout)?;
    assert_eq!(json["items"][0]["title"], ":bug:");

    Ok(())
}
