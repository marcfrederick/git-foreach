use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum Error {
    CommandExecutionFailed { path: PathBuf, source: io::Error },
    CommandExitedUnsuccessfully { path: PathBuf, status: ExitStatus },
    Walk(ignore::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CommandExecutionFailed { path, source } => {
                write!(f, "failed to start command in {}: {source}", path.display())
            }
            Self::CommandExitedUnsuccessfully { path, status } => {
                write!(
                    f,
                    "command exited unsuccessfully in {}: {status}",
                    path.display(),
                )
            }
            Self::Walk(source) => write!(f, "failed to walk directory: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommandExecutionFailed { source, .. } => Some(source),
            Self::Walk(source) => Some(source),
            Self::CommandExitedUnsuccessfully { .. } => None,
        }
    }
}

impl From<ignore::Error> for Error {
    fn from(source: ignore::Error) -> Self {
        Self::Walk(source)
    }
}
