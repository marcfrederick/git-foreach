use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("run_command failed for {path}")]
    CommandExecutionFailed { path: PathBuf },

    #[error("run_command failed with non-zero exit code for {path}")]
    CommandExecutionFailedWithNonZeroExitCode { path: PathBuf, exit_code: i32 },

    #[error("{0}")]
    InvalidUsage(#[from] clap::Error),
}

impl Error {
    /// Get the exit code for the error.
    fn get_exit_code(&self) -> i32 {
        match self {
            Error::CommandExecutionFailedWithNonZeroExitCode { exit_code, .. } => *exit_code,
            Error::InvalidUsage(source) => source.exit_code(),
            Error::CommandExecutionFailed { .. } => 1,
        }
    }

    /// Print the error message and exit with the appropriate exit code.
    pub fn print_and_exit(&self) {
        eprintln!("{self}");
        std::process::exit(self.get_exit_code());
    }
}
