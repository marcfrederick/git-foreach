/// Error type for git-walk.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to walk directory")]
    Walk(#[from] ignore::Error),
}

/// Result type for git-walk.
pub type Result<T> = std::result::Result<T, Error>;
