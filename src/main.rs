#![deny(clippy::pedantic)]

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

use clap::Parser;
use ignore::Walk;

use crate::error::Error;

mod error;

#[derive(Debug, Parser)]
#[clap(version, about)]
struct Options {
    #[arg(
        short,
        long,
        help = "Suppress output. Output from the command will still be displayed."
    )]
    quiet: bool,

    #[arg(
        short,
        long,
        help = "Root directory to search for repositories.",
        default_value = "."
    )]
    directory: PathBuf,

    #[arg(trailing_var_arg = true, required = true)]
    command: Vec<String>,
}

fn main() {
    repository_foreach(env::args()).unwrap_or_else(|err| err.print_and_exit());
}

/// Run a command in each git repository in the current directory and its
/// subdirectories.
fn repository_foreach<T: Iterator<Item = String>>(args: T) -> Result<(), Error> {
    let options = parse_options(args)?;

    for repository in find_repositories(&options.directory)? {
        run_command_in_directory(&options, &repository)?;
    }

    Ok(())
}

/// Parse the command line options.
fn parse_options<T: Iterator<Item = String>>(args: T) -> Result<Options, Error> {
    Options::try_parse_from(args).map_err(|err| Error::InvalidUsage { source: err })
}

/// Find all git repositories in a directory and its subdirectories.
fn find_repositories(path: &PathBuf) -> Result<HashSet<PathBuf>, Error> {
    Walk::new(path)
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                Some(Ok(path.to_path_buf()))
            } else {
                None
            }
        })
        .collect()
}

/// Run a command in a directory.
fn run_command_in_directory(options: &Options, path: &PathBuf) -> Result<(), Error> {
    if !options.quiet {
        println!("Entering '{}'", path.display());
    }

    let shell_binary = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "/bin/sh"
    };

    let shell_arg = if cfg!(target_os = "windows") {
        "/C".to_string()
    } else {
        "-c".to_string()
    };

    let shell_command = options.command.join(" ");

    let status = std::process::Command::new(shell_binary)
        .args([shell_arg, shell_command])
        .current_dir(path)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => Ok(()),
        Ok(exit_status) => Err(Error::CommandExecutionFailedWithNonZeroExitCode {
            path: path.clone(),
            exit_code: exit_status.code().unwrap_or(1),
        }),
        Err(_) => Err(Error::CommandExecutionFailed { path: path.clone() }),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_options() {
        let args = ["repository-foreach", "--quiet", "echo", "foo"]
            .iter()
            .map(ToString::to_string);

        let options = parse_options(args).expect("Failed to parse options");

        assert!(options.quiet);
        assert_eq!(options.command, vec!["echo".to_string(), "foo".to_string()]);
    }

    #[test]
    fn test_parse_options_missing_command() {
        let args = ["repository-foreach"].iter().map(ToString::to_string);

        let error = parse_options(args);

        match error {
            Ok(_) => panic!("Expected an error"),
            Err(Error::InvalidUsage { .. }) => {}
            Err(err) => panic!("Unexpected error: {err}"),
        }
    }
}
