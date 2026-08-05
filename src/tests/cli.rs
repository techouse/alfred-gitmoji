use super::*;

#[test]
fn parse_normalizes_the_query_without_affecting_flags() -> Result<()> {
    let cli = Cli::parse([
        "-q".to_owned(),
        "  Fix   BUG ".to_owned(),
        "--verbose".to_owned(),
    ])?;

    assert_eq!(cli.normalized_query(), "fix bug");
    assert!(cli.verbose);
    assert!(!cli.update);

    Ok(())
}
