use types::*;

pub trait Store {
    type Error;
    // type Result<T> = Result<T, Self::Error>;

    // Low-level mandatory API
    fn load(&mut self, d: &Digest) -> Result<Option<Buffer>, Self::Error>;
    fn save(&mut self, d: &Digest, b: &Buffer) -> Result<(), Self::Error>;
    fn read_root(&mut self) -> Result<RootData, Self::Error>;
    fn replace_root(&mut self, previous: RootData, next: RootData) -> Result<bool, Self::Error>;

    // Higher-level ergonomic API
    fn exists(&mut self, d: &Digest) -> Result<bool,Self::Error> {
        match self.load(d) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(x) => Err(x),
        }
    }

    fn store(&mut self, rsc: impl Into<Resource>) -> Result<Resource, Self::Error> {
        let rsc: Resource = rsc.into();
        self.save(&rsc.digest, &rsc.body)?;
        Ok(rsc)
    }

    fn load_archive(&mut self, spec: &Option<Spec>) -> Result<Archive, Self::Error> {
        let archive: Option<Archive> = match &spec {
            None => None,
            Some(s) => match self.load(&s.digest)? {
                None => None,
                Some(buf) => Some(Archive::from_buffer(s.format, s.compression, &buf)),
            }
        };

        Ok(archive.unwrap_or(Archive {
            format: Format::JSON,
            compression: Compression::Plain,
            entries: vec![],
        }))
    }

    fn set_label(&mut self, name: &str, spec: &Spec) -> Result<(), Self::Error> {
        let entry = ArchiveEntry {
            path: format!("@{}", name).into(),
            spec: spec.clone(),
            attrs: vec![],
        };
        for _attempt in 0..10 {
            let original_root_spec = self.read_root()?;
            let mut archive = self.load_archive(&original_root_spec)?;
            archive.set(&entry);

            let rsc = self.store(&archive)?;
            let rd = Some(Spec {
                format: Format::JSON,
                compression: Compression::Plain,
                digest: rsc.digest.clone(),
            });

            if self.replace_root(original_root_spec, rd)? {
                return Ok(())
            }
        }
        panic!("After multiple attempts, could not set label");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::Memory;

    #[test]
    fn load_archive() {
        let mut store = Memory::new();
        let archive = Archive {
            format: Format::JSON,
            compression: Compression::Plain,
            entries: vec![
                ArchiveEntry {
                    path: "foo".into(),
                    spec: Spec {
                        format: Format::File,
                        compression: Compression::Plain,
                        digest: "foo".into(),
                    },
                    attrs: vec![],
                },
            ],
        };
        let rsc: Resource = (&archive).into();
        let spec = Spec {
            format: archive.format,
            compression: archive.compression,
            digest: rsc.digest,
        };

        // No spec provided: default archive
        assert_eq!(store.load_archive(&None).unwrap(), Archive {
            format: Format::JSON,
            compression: Compression::Plain,
            entries: vec![],
        });

        // Spec not found: default archive
        assert_eq!(store.load_archive(&Some(spec.clone())).unwrap(), Archive {
            format: Format::JSON,
            compression: Compression::Plain,
            entries: vec![],
        });

        // Spec found: restore archive
        let _ = store.store(&archive).unwrap();
        assert_eq!(store.load_archive(&Some(spec)).unwrap(), archive);
    }
}
