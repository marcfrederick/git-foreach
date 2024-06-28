#![deny(clippy::pedantic)]

use std::env;
use std::path::PathBuf;

use clap::Parser;
use ignore::{DirEntry, Walk, WalkBuilder};
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
    if cfg!(debug_assertions) {
        dbg!(&options);
    }

    walk_from_options(&options)
        .flatten()
        .par_bridge()
        .map(DirEntry::into_path)
        .filter(|path| path.is_dir() && path.join(".git").exists())
        .try_for_each(|repository| run_command_in_directory(&options, &repository))
}

/// Parse the command line options.
fn parse_options<T: Iterator<Item = String>>(args: T) -> Result<Options, Error> {
    Options::try_parse_from(args).map_err(Error::from)
}

/// Initialize the directory walker from the options.
fn walk_from_options(options: &Options) -> Walk {
    WalkBuilder::new(&options.directory)
        .hidden(!options.hidden)
        .ignore(!options.no_ignore)
        .git_ignore(!options.no_ignore)
        .build()
}

/// Run a command in a directory.
fn run_command_in_directory(options: &Options, path: &PathBuf) -> Result<(), Error> {
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
            path: path.clone(),
            exit_code: exit_status.code().unwrap_or(1),
        }),
        Err(_) => Err(Error::CommandExecutionFailed { path: path.clone() }),
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
            assert_eq!(options.dry_run, true);
            assert_eq!(options.command, vec!["echo".to_string(), "hello".to_string()]);
        },
        parse_options_quiet: ["git-foreach", "--quiet", "echo", "hello"] => |result: Result<Options, _>| {
            let options = result.expect("Expected Ok(_)");
            assert_eq!(options.quiet, true);
            assert_eq!(options.command, vec!["echo".to_string(), "hello".to_string()]);
        },
    );
}
