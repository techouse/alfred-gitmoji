use std::path::Path;

use alfred_workflow_rs::{Icon, Item, ItemText, Modifier, ModifierKey};
use anyhow::{Result, anyhow};

use crate::models::SearchResult;

/// Converts Gitmoji results and their optional image paths into Alfred items.
///
/// The item order always matches the input results. A missing image path uses
/// the workflow's bundled question-mark icon.
pub fn items_from_results(
    results: &[SearchResult],
    image_paths: &[Option<impl AsRef<Path>>],
) -> Result<Vec<Item>> {
    if results.len() != image_paths.len() {
        return Err(anyhow!(
            "received {} search results but {} image paths",
            results.len(),
            image_paths.len()
        ));
    }

    results
        .iter()
        .zip(image_paths)
        .map(|(result, image_path)| {
            item_from_result(result, image_path.as_ref().map(AsRef::as_ref))
        })
        .collect()
}

/// Builds the non-selectable result shown for a search without hits.
pub fn no_results_item() -> Item {
    Item::new("No matching gitmoji found").set_icon(Icon::new("question.png"))
}

fn item_from_result(result: &SearchResult, image_path: Option<&Path>) -> Result<Item> {
    let icon = Icon::new(image_path.map_or_else(
        || "question.png".to_owned(),
        |path| path.display().to_string(),
    ));
    let python_source = python_source(&result.emoji)?;
    let html_entity = html_entity(&result.emoji);
    let unicode_notation = unicode_notation(&result.emoji);

    Item::builder(&result.code)
        .uid(&result.object_id)
        .subtitle(&result.description)
        .arg(&result.code)
        .match_text(format!("{} {}", result.name, result.description))
        .text(ItemText::new(&result.code).with_large_type(&result.code))
        .icon(icon.clone())
        .try_modifier(
            [ModifierKey::Alt],
            Modifier::new()
                .with_subtitle(format!("Copy \"{}\" to clipboard", result.emoji))
                .with_arg(&result.emoji)
                .with_icon(icon.clone()),
        )?
        .try_modifier(
            [ModifierKey::Shift],
            Modifier::new()
                .with_subtitle(format!(
                    "Copy Python source of \"{}\" to clipboard",
                    result.emoji
                ))
                .with_arg(python_source)
                .with_icon(icon.clone()),
        )?
        .try_modifier(
            [ModifierKey::Ctrl],
            Modifier::new()
                .with_subtitle(format!(
                    "Copy HTML Entity of \"{}\" to clipboard",
                    result.emoji
                ))
                .with_arg(html_entity)
                .with_icon(icon.clone()),
        )?
        .try_modifier(
            [ModifierKey::Ctrl, ModifierKey::Shift],
            Modifier::new()
                .with_subtitle(format!(
                    "Copy formal Unicode notation of \"{}\" to clipboard",
                    result.emoji
                ))
                .with_arg(unicode_notation)
                .with_icon(icon),
        )?
        .valid(true)
        .build()
        .map_err(Into::into)
}

fn python_source(emoji: &str) -> Result<String> {
    let mut scalars = emoji.chars();
    let first = scalars
        .next()
        .ok_or_else(|| anyhow!("emoji must not be empty"))?;
    let trailing = scalars
        .map(|scalar| format!(r#"\u{:X}"#, u32::from(scalar)))
        .collect::<String>();

    Ok(format!(r#"u"\U000{:X}{trailing}""#, u32::from(first)))
}

fn html_entity(emoji: &str) -> String {
    emoji
        .chars()
        .map(|scalar| format!("&#x{:x};", u32::from(scalar)))
        .collect()
}

fn unicode_notation(emoji: &str) -> String {
    emoji
        .chars()
        .map(|scalar| format!("U+{:X}", u32::from(scalar)))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
