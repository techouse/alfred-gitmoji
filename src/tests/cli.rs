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

#[test]
fn parse_preserves_query_text_and_flags() -> Result<()> {
    let cli = Cli::parse([
        "-q".to_owned(),
        "  Fix   BUG ".to_owned(),
        "--verbose".to_owned(),
    ])?;

    assert_eq!(
        cli,
        Cli {
            query: "  Fix   BUG ".to_owned(),
            verbose: true,
            update: false,
        }
    );
    Ok(())
}

#[test]
fn parse_rejects_missing_query_values() {
    for option in ["-q", "--query"] {
        let error = Cli::parse([option.to_owned()]).expect_err("query value must be required");

        assert_eq!(error.to_string(), format!("{option} requires a value"));
    }
}

#[test]
fn parse_rejects_recognized_options_as_separated_query_values() {
    for option in ["-q", "--query"] {
        for value in ["-q", "--query", "-v", "--verbose", "-u", "--update"] {
            let error = Cli::parse([option.to_owned(), value.to_owned()])
                .expect_err("recognized options must not become query values");

            assert_eq!(error.to_string(), format!("{option} requires a value"));
        }
    }
}

#[test]
fn parse_rejects_option_assignments_as_separated_query_values() {
    for option in ["-q", "--query"] {
        let error = Cli::parse([option.to_owned(), "--query=bug".to_owned()])
            .expect_err("query assignments must not become query values");

        assert_eq!(error.to_string(), format!("{option} requires a value"));
    }
}

#[test]
fn parse_rejects_short_option_clusters_as_query_values() {
    for option in ["-q", "--query"] {
        for value in ["-vu", "-vuqbackground"] {
            let error = Cli::parse([option.to_owned(), value.to_owned()])
                .expect_err("valid short-option clusters must not become query values");

            assert_eq!(error.to_string(), format!("{option} requires a value"));
        }
    }
}

#[test]
fn parse_accepts_unrecognized_dash_prefixed_query() -> Result<()> {
    let cli = Cli::parse(["-q".to_owned(), "--force".to_owned()])?;

    assert_eq!(cli.query, "--force");
    Ok(())
}

#[test]
fn parse_accepts_query_assignment() -> Result<()> {
    let cli = Cli::parse(["--query=-u".to_owned()])?;

    assert_eq!(cli.query, "-u");
    Ok(())
}

#[test]
fn parse_accepts_attached_short_query_value() -> Result<()> {
    let cli = Cli::parse(["-qbackground".to_owned()])?;

    assert_eq!(cli.query, "background");
    Ok(())
}

#[test]
fn parse_accepts_collapsed_short_flags() -> Result<()> {
    let cli = Cli::parse(["-vu".to_owned()])?;

    assert_eq!(
        cli,
        Cli {
            query: String::new(),
            verbose: true,
            update: true,
        }
    );
    Ok(())
}

#[test]
fn parse_accepts_collapsed_flags_with_attached_query() -> Result<()> {
    let cli = Cli::parse(["-vuqbackground".to_owned()])?;

    assert_eq!(
        cli,
        Cli {
            query: "background".to_owned(),
            verbose: true,
            update: true,
        }
    );
    Ok(())
}

#[test]
fn parse_accepts_collapsed_flags_with_separated_query() -> Result<()> {
    let cli = Cli::parse(["-vq".to_owned(), "background".to_owned()])?;

    assert_eq!(
        cli,
        Cli {
            query: "background".to_owned(),
            verbose: true,
            update: false,
        }
    );
    Ok(())
}

#[test]
fn parse_rejects_unknown_flag_in_cluster() {
    let error = Cli::parse(["-vx".to_owned()]).expect_err("unknown cluster flag must fail");

    assert_eq!(error.to_string(), "unknown argument: -vx");
}

#[test]
fn parse_rejects_cluster_query_without_value() {
    let error = Cli::parse(["-vq".to_owned()]).expect_err("cluster query value must be required");

    assert_eq!(error.to_string(), "-q requires a value");
}

#[test]
fn parse_rejects_cluster_after_query_option() {
    for option in ["-q", "--query"] {
        let error = Cli::parse([option.to_owned(), "-vu".to_owned()])
            .expect_err("a following option cluster must not become the query");

        assert_eq!(error.to_string(), format!("{option} requires a value"));
    }
}

#[test]
fn parse_rejects_unknown_arguments() {
    let error = Cli::parse(["--unknown".to_owned()]).expect_err("unknown flag must fail");

    assert_eq!(error.to_string(), "unknown argument: --unknown");
}
