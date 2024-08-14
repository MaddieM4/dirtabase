use std::io::Result;
use std::path::{Path, PathBuf};

pub enum DB {
    Persistent(PathBuf),
    Temp(tempfile::TempDir),
}

fn init_sections(p: &Path) -> Result<()> {
    for section in ["tmp", "cas", "labels"] {
        std::fs::create_dir_all(p.join(section))?;
    }
    Ok(())
}

impl DB {
    pub fn new(p: impl AsRef<Path>) -> Result<Self> {
        let p: PathBuf = p.as_ref().into();
        init_sections(p.as_ref())?;
        Ok(Self::Persistent(p))
    }

    pub fn new_temp() -> Result<Self> {
        let t = tempfile::tempdir()?;
        init_sections(t.as_ref())?;
        Ok(Self::Temp(t))
    }

    pub fn join(&self, p: impl AsRef<Path>) -> PathBuf {
        self.as_ref().join(p)
    }
}

impl AsRef<Path> for DB {
    fn as_ref(&self) -> &Path {
        match self {
            Self::Persistent(path) => path,
            Self::Temp(td) => td.as_ref(),
        }
    }
}
