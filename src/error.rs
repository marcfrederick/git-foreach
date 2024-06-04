use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    CommandExecutionFailed { path: PathBuf },
    CommandExecutionFailedWithNonZeroExitCode { path: PathBuf, exit_code: i32 },
    InvalidUsage { source: clap::Error },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidUsage { source } => write!(f, "{}", source.render()),
            Error::CommandExecutionFailed { path } => {
                write!(f, "run_command failed for {}", path.display())
            }
            Error::CommandExecutionFailedWithNonZeroExitCode { path, .. } => {
                write!(
                    f,
                    "run_command failed with non-zero exit code for {}",
                    path.display()
                )
            }
        }
    }
}

impl Error {
    /// Get the exit code for the error.
    fn get_exit_code(&self) -> i32 {
        match self {
            Error::CommandExecutionFailedWithNonZeroExitCode { exit_code, .. } => *exit_code,
            Error::InvalidUsage { source } => source.exit_code(),
            Error::CommandExecutionFailed { .. } => 1,
        }
    }

    /// Print the error message and exit with the appropriate exit code.
    pub fn print_and_exit(&self) {
        eprintln!("{self}");
        std::process::exit(self.get_exit_code());
    }
}
