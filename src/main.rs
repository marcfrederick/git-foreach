#![deny(clippy::pedantic)]

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use ignore::Walk;

const USAGE: &str = "Usage: repository-foreach <command>";

struct Options {
    command: Vec<String>,
}

#[derive(Debug)]
struct Error(String);

fn main() {
    match repository_foreach(env::args()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err.0);
            std::process::exit(1);
        }
    }
}

/// Run a command in each git repository in the current directory and its
/// subdirectories.
fn repository_foreach<T: Iterator<Item = String>>(args: T) -> Result<(), Error> {
    let options = parse_options(args)?;
    let current_dir =
        env::current_dir().map_err(|_| Error("Error getting the current directory".to_string()))?;

    for repository in find_repositories(&current_dir)? {
        run_command_in_directory(&options.command, &repository)?;
    }

    Ok(())
}

/// Parse the command line options.
fn parse_options<T: Iterator<Item = String>>(args: T) -> Result<Options, Error> {
    let args = args.skip(1);

    // Collect the remaining arguments into a vector. This will be the command
    // to run in each repository.
    let command = args.collect::<Vec<String>>();
    if command.is_empty() {
        return Err(Error(USAGE.to_string()));
    }

    Ok(Options { command })
}

/// Find all git repositories in a directory and its subdirectories.
fn find_repositories(path: &Path) -> Result<HashSet<PathBuf>, Error> {
    let mut repositories = HashSet::new();
    for entry in Walk::new(path) {
        let entry = entry.map_err(|err| Error(format!("Error walking directory: {err}")))?;
        let path = entry.path();
        if path.is_dir() && path.join(".git").exists() {
            repositories.insert(path.to_path_buf());
        }
    }
    Ok(repositories)
}

/// Run a command in a directory.
fn run_command_in_directory(command: &[String], path: &Path) -> Result<(), Error> {
    println!("Entering '{}'", path.display());

    let status = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", &command.join(" ")])
            .current_dir(path)
            .status()
    } else {
        std::process::Command::new("/bin/sh")
            .args(["-c", &command.join(" ")])
            .current_dir(path)
            .status()
    };

    match status {
        Ok(exit_status) if exit_status.success() => Ok(()),
        Ok(_) => Err(Error(format!(
            "run_command returned non-zero status for {}",
            path.display()
        ))),
        Err(_) => Err(Error(format!("run_command failed for {}", path.display()))),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_options() {
        let args = ["repository-foreach", "echo", "foo"]
            .iter()
            .map(ToString::to_string);

        let options = parse_options(args).expect("Failed to parse options");

        assert_eq!(options.command, vec!["echo".to_string(), "foo".to_string()]);
    }

    #[test]
    fn test_parse_options_missing_command() {
        let args = ["repository-foreach"].iter().map(ToString::to_string);

        let error = parse_options(args);

        match error {
            Ok(_) => panic!("Expected an error"),
            Err(err) => assert_eq!(err.0, USAGE),
        }
    }
}
