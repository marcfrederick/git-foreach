use std::path::{Path, PathBuf};

/// Error type for the walk module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to walk directory")]
    Walk(#[from] ignore::Error),
}

/// Result type for the walk module.
pub type Result<T> = std::result::Result<T, Error>;

/// Walk directory entries.
pub struct Walk {
    walk: ignore::Walk,
}

impl Walk {
    /// Create a new Walk instance.
    pub fn new(path: &Path, skip_hidden: bool, skip_ignored: bool) -> Self {
        let walk = ignore::WalkBuilder::new(path)
            .hidden(skip_hidden)
            .ignore(skip_ignored)
            .build();

        Self { walk }
    }

    /// Get the next directory entry.
    fn next(&mut self) -> Option<Result<PathBuf>> {
        match self.walk.next() {
            Some(Ok(entry)) => Some(Ok(entry.into_path())),
            Some(Err(err)) => Some(Err(err.into())),
            None => None,
        }
    }
}

impl Iterator for Walk {
    type Item = Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
