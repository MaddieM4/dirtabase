//! API for storing and retrieving potentially large files by digest.

pub mod simple;

use simple::{SimpleCAS, SimpleLabels};
use std::io::Result;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// All supported storage backends.
pub enum Store {
    /// Behaves persistently, does not auto-delete itself when dropped.
    Simple(PathBuf, SimpleCAS, SimpleLabels),

    /// Deletes itself from disk when it goes out of lexical scope.
    SimpleTemp(TempDir, SimpleCAS, SimpleLabels),
}

impl Store {
    pub fn new_simple(path: impl AsRef<Path>) -> Result<Self> {
        let path: PathBuf = path.as_ref().into();
        let cas = SimpleCAS::new(path.join("cas"))?;
        let labels = SimpleLabels::new(path.join("labels"))?;
        Ok(Self::Simple(path, cas, labels))
    }

    pub fn new_simpletemp() -> Result<Self> {
        let dir = tempfile::tempdir()?;
        let cas = SimpleCAS::new(dir.path().join("cas"))?;
        let labels = SimpleLabels::new(dir.path().join("labels"))?;
        Ok(Self::SimpleTemp(dir, cas, labels))
    }

    pub fn cas(&self) -> &SimpleCAS {
        match self {
            Self::Simple(_, cas, _) => &cas,
            Self::SimpleTemp(_, cas, _) => &cas,
        }
    }

    pub fn labels(&self) -> &SimpleLabels {
        match self {
            Self::Simple(_, _, labels) => &labels,
            Self::SimpleTemp(_, _, labels) => &labels,
        }
    }
}
