use anyhow::{Result, bail};

/// Command-line options accepted by the Gitmoji workflow executable.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Cli {
    /// Search query supplied by Alfred.
    pub query: String,
    /// Enables diagnostic output on stderr.
    pub verbose: bool,
    /// Downloads and opens a newer workflow release.
    pub update: bool,
}

impl Cli {
    /// Parses command-line arguments without the executable name.
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut cli = Self::default();
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "-q" | "--query" => {
                    cli.query = arguments
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("{argument} requires a value"))?;
                }
                "-v" | "--verbose" => cli.verbose = true,
                "-u" | "--update" => cli.update = true,
                _ if argument.starts_with("--query=") => {
                    cli.query = argument["--query=".len()..].to_owned();
                }
                _ => bail!("unknown argument: {argument}"),
            }
        }

        Ok(cli)
    }

    /// Normalizes a query before searching.
    pub fn normalized_query(&self) -> String {
        self.query
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }
}

#[cfg(test)]
#[path = "tests/cli.rs"]
mod tests;
