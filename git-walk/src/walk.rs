use std::path::{Path, PathBuf};

use crate::Result;

/// Walk directory entries.
pub struct Walk {
    inner: ignore::Walk,
}

impl Walk {
    /// Create a new walk iterator. The iterator will walk the directory tree
    /// starting at the given path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let walk = git_walk::Walk::new(".");
    /// for entry in walk {
    ///     match entry {
    ///        Ok(path) => println!("{}", path.display()),
    ///        Err(err) => eprintln!("error: {}", err),
    ///    }
    /// }
    /// ```
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            inner: ignore::Walk::new(path),
        }
    }

    /// Advances the iterator and returns the next entry. Returns `None` when the
    /// iterator has finished.
    fn next(&mut self) -> Option<Result<PathBuf>> {
        match self.inner.next() {
            Some(Ok(entry)) => Some(Ok(entry.into_path())),
            Some(Err(err)) => Some(Err(err.into())),
            None => None,
        }
    }
}

impl Iterator for Walk {
    type Item = Result<PathBuf>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl std::iter::FusedIterator for Walk {}

/// A builder for configuring a `Walk` iterator.
///
/// # Example
///
/// ```no_run
/// let _walk = git_walk::WalkBuilder::new(".")
///    .hidden(true)
///    .ignore(true)
///    .build();
/// ```
pub struct WalkBuilder {
    inner: ignore::WalkBuilder,
}

impl WalkBuilder {
    /// Create a new `WalkBuilder` for the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            inner: ignore::WalkBuilder::new(path),
        }
    }

    /// Build the `Walk` iterator.
    #[must_use]
    pub fn build(&self) -> Walk {
        Walk {
            inner: self.inner.build(),
        }
    }

    /// Ignore hidden files and directories.
    pub fn hidden(&mut self, yes: bool) -> &mut Self {
        self.inner.hidden(yes);
        self
    }

    /// Ignore files excluded by `.ignore` files.
    pub fn ignore(&mut self, yes: bool) -> &mut Self {
        self.inner.ignore(yes);
        self
    }
}
