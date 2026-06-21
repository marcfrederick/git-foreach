#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use clap::Parser;
use ignore::WalkBuilder;
use rayon::prelude::*;

use crate::error::Error;

mod error;

#[allow(clippy::struct_excessive_bools)]
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

    #[arg(long, help = "Search hidden files and directories.")]
    hidden: bool,

    #[arg(
        long,
        help = "When set, ignore files such as .gitignore will not be respected."
    )]
    no_ignore: bool,

    #[arg(long, help = "Dry run. Do not execute the command.")]
    dry_run: bool,

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

    repository_walk(&options)
        .par_bridge()
        .try_for_each(|entry| {
            let path = entry?.into_path();
            if path.is_dir() && path.join(".git").exists() {
                run_command_in_directory(&options, &path)?;
            }
            Ok(())
        })
}

fn repository_walk(options: &Options) -> ignore::Walk {
    WalkBuilder::new(&options.directory)
        .standard_filters(!options.no_ignore)
        .hidden(!options.hidden)
        .filter_entry(|entry| entry.file_name() != OsStr::new(".git"))
        .build()
}

/// Parse the command line options.
fn parse_options<T: Iterator<Item = String>>(args: T) -> Result<Options, Error> {
    Options::try_parse_from(args).map_err(Error::from)
}

/// Run a command in a directory.
fn run_command_in_directory(options: &Options, path: &Path) -> Result<(), Error> {
    if !options.quiet {
        println!("Entering '{}'", path.display());
    }

    let (shell_binary, shell_arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("/bin/sh", "-c")
    };

    let shell_command = options.command.join(" ");

    if options.dry_run {
        println!(
            "dry-run: would run '{} {} \"{}\"' in '{}'",
            shell_binary,
            shell_arg,
            shell_command,
            path.display()
        );
        return Ok(());
    }

    let status = std::process::Command::new(shell_binary)
        .args([shell_arg, &shell_command])
        .current_dir(path)
        .status();

    match status {
        Ok(exit_status) if exit_status.success() => Ok(()),
        Ok(exit_status) => Err(Error::CommandExecutionFailedWithNonZeroExitCode {
            path: path.to_path_buf(),
            exit_code: exit_status.code().unwrap_or(1),
        }),
        Err(_) => Err(Error::CommandExecutionFailed {
            path: path.to_path_buf(),
        }),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    macro_rules! parse_options_tests {
        ($($name:ident: $args:expr => $foo:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let args = $args.iter().map(ToString::to_string);
                let result = parse_options(args);
                $foo(result);
            }
        )*
        }
    }

    parse_options_tests!(
        parse_options_empty: [] as [&str; 0] => |result: Result<_, _>| assert!(matches!(result, Err(Error::InvalidUsage { .. }))),
        parse_options_help: ["git-foreach", "--help"] => |result: Result<_, _>| assert!(matches!(result, Err(Error::InvalidUsage { .. }))),
        parse_options_version: ["git-foreach", "--version"] => |result: Result<_, _>| assert!(matches!(result, Err(Error::InvalidUsage { .. }))),
        parse_options_invalid: ["git-foreach", "--invalid"] => |result: Result<_, _>| assert!(matches!(result, Err(Error::InvalidUsage { .. }))),
        parse_options_valid: ["git-foreach", "echo", "hello"] => |result: Result<Options, _>| {
            let options = result.expect("Expected Ok(_)");
            assert_eq!(options.command, vec!["echo".to_string(), "hello".to_string()]);
        },
        parse_options_dry_run: ["git-foreach", "--dry-run", "echo", "hello"] => |result: Result<Options, _>| {
            let options = result.expect("Expected Ok(_)");
            assert!(options.dry_run);
            assert_eq!(options.command, vec!["echo".to_string(), "hello".to_string()]);
        },
        parse_options_quiet: ["git-foreach", "--quiet", "echo", "hello"] => |result: Result<Options, _>| {
            let options = result.expect("Expected Ok(_)");
            assert!(options.quiet);
            assert_eq!(options.command, vec!["echo".to_string(), "hello".to_string()]);
        },
    );

    #[test]
    fn hidden_repositories_are_only_included_when_requested() {
        let directory = tempdir().expect("failed to create temporary directory");
        let visible = create_repository(directory.path(), "visible");
        let hidden = create_repository(directory.path(), ".hidden");

        let repositories: Vec<PathBuf> = discovered_repositories(directory.path(), false, false);
        assert!(repositories.contains(&visible));
        assert!(!repositories.contains(&hidden));

        let repositories = discovered_repositories(directory.path(), true, false);
        assert!(repositories.contains(&visible));
        assert!(repositories.contains(&hidden));
    }

    #[test]
    fn ignored_repositories_are_only_included_when_requested() {
        let directory = tempdir().expect("failed to create temporary directory");
        create_repository(directory.path(), "");
        let ignored = create_repository(directory.path(), "ignored");
        fs::write(directory.path().join(".gitignore"), "ignored/\n")
            .expect("failed to create .gitignore");

        assert!(!discovered_repositories(directory.path(), false, false).contains(&ignored));
        assert!(discovered_repositories(directory.path(), false, true).contains(&ignored));
    }

    fn create_repository(parent: &Path, name: &str) -> PathBuf {
        let path = parent.join(name);
        fs::create_dir_all(path.join(".git")).expect("failed to create test repository");
        path
    }

    fn discovered_repositories(directory: &Path, hidden: bool, no_ignore: bool) -> Vec<PathBuf> {
        let options = Options {
            quiet: false,
            directory: directory.to_owned(),
            hidden,
            no_ignore,
            dry_run: true,
            command: vec!["true".to_owned()],
        };

        repository_walk(&options)
            .map(|entry| entry.expect("failed to walk test directory").into_path())
            .into_iter()
            .filter(|path| path.is_dir() && path.join(".git").exists())
            .collect()
    }
}
