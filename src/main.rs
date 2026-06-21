#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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

fn main() -> ExitCode {
    let options = Options::parse();

    match repository_foreach(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(errors) => {
            eprintln!("encountered {} error(s):", errors.len());
            for error in errors {
                eprintln!("- {error}");
            }
            ExitCode::FAILURE
        }
    }
}

/// Run a command in each git repository in the given directory and its
/// subdirectories.
fn repository_foreach(options: &Options) -> Result<(), Vec<Error>> {
    let mut errors: Vec<_> = repository_walk(options)
        .par_bridge()
        .filter_map(|entry| {
            let path = match entry {
                Ok(entry) => entry.into_path(),
                Err(source) => return Some(Error::from(source)),
            };

            if path.is_dir() && path.join(".git").exists() {
                run_command_in_directory(options, &path).err()
            } else {
                None
            }
        })
        .collect();

    errors.sort_by_key(ToString::to_string);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn repository_walk(options: &Options) -> ignore::Walk {
    WalkBuilder::new(&options.directory)
        .standard_filters(!options.no_ignore)
        .hidden(!options.hidden)
        .filter_entry(|entry| entry.file_name() != OsStr::new(".git"))
        .build()
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
        .status()
        .map_err(|source| Error::CommandExecutionFailed {
            path: path.to_path_buf(),
            source,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandExitedUnsuccessfully {
            path: path.to_path_buf(),
            status,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::error::ErrorKind;
    use std::fs;
    use tempfile::tempdir;

    macro_rules! parse_error_tests {
        ($($name:ident: $args:expr => $kind:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let args = $args.iter().map(ToString::to_string);
                let error = Options::try_parse_from(args).expect_err("expected parsing to fail");
                assert_eq!(error.kind(), $kind);
            }
        )*
        }
    }

    parse_error_tests!(
        parse_options_empty: [] as [&str; 0] => ErrorKind::MissingRequiredArgument,
        parse_options_help: ["git-foreach", "--help"] => ErrorKind::DisplayHelp,
        parse_options_version: ["git-foreach", "--version"] => ErrorKind::DisplayVersion,
        parse_options_invalid: ["git-foreach", "--invalid"] => ErrorKind::UnknownArgument,
    );

    #[test]
    fn parse_options() {
        let options =
            Options::try_parse_from(["git-foreach", "--dry-run", "--quiet", "echo", "hello"])
                .expect("expected parsing to succeed");

        assert!(options.dry_run);
        assert!(options.quiet);
        assert_eq!(options.command, ["echo", "hello"]);
    }

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
            .filter(|path| path.is_dir() && path.join(".git").exists())
            .collect()
    }
}
