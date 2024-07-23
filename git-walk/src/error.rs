/// Result type for git-walk.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for git-walk.
#[derive(Debug)]
pub enum Error {
    Walk(ignore::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Walk(source) => write!(f, "error walking directory: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Walk(source) => Some(source),
        }
    }
}

impl From<ignore::Error> for Error {
    fn from(source: ignore::Error) -> Self {
        Self::Walk(source)
    }
}
