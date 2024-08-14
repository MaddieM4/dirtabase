use super::types::Ark;
use crate::db::DB;
use crate::digest::Digest;
use std::io::Result;

pub trait Save: serde::Serialize {
    fn save(&self, db: &DB) -> Result<Digest> {
        let json = serde_json::to_string(self)?;
        let d = Digest::from(&json);
        std::fs::write(db.join("cas").join(d.to_hex()), json)?;
        Ok(d)
    }
}

impl Save for Ark<Digest> {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ark::import::Import;

    #[test]
    fn save() -> Result<()> {
        let db = DB::new_temp()?;
        let ark = Ark::scan("fixture")?;
        let digest = ark.import_files(&db)?.save(&db)?;
        assert_eq!(
            digest.to_hex(),
            "647f1efbfa520cfc16d974d0a1414f5795e58f612bd4928039b7088c347250b8"
        );

        Ok(())
    }
}
