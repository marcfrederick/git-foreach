use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    CommandExecutionFailed { path: PathBuf },
    CommandExecutionFailedWithNonZeroExitCode { path: PathBuf, exit_code: i32 },
    Walk(git_walk::Error),
    InvalidUsage(clap::Error),
}

impl Error {
    /// Get the exit code for the error.
    fn get_exit_code(&self) -> i32 {
        match self {
            Self::CommandExecutionFailedWithNonZeroExitCode { exit_code, .. } => *exit_code,
            Self::InvalidUsage(source) => source.exit_code(),
            _ => 1,
        }
    }

    /// Print the error message and exit with the appropriate exit code.
    pub fn print_and_exit(&self) {
        eprintln!("{self}");
        std::process::exit(self.get_exit_code());
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CommandExecutionFailed { path } => {
                write!(f, "run_command failed for {}", path.display())
            }
            Self::CommandExecutionFailedWithNonZeroExitCode { path, exit_code } => {
                write!(
                    f,
                    "run_command failed with non-zero exit code {} for {}",
                    exit_code,
                    path.display()
                )
            }
            Self::Walk(source) => write!(f, "error walking directory: {source}"),
            Self::InvalidUsage(source) => write!(f, "{source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Walk(source) => Some(source),
            Self::InvalidUsage(source) => Some(source),
            _ => None,
        }
    }
}

impl From<git_walk::Error> for Error {
    fn from(source: git_walk::Error) -> Self {
        Self::Walk(source)
    }
}

impl From<clap::Error> for Error {
    fn from(source: clap::Error) -> Self {
        Self::InvalidUsage(source)
    }
}
