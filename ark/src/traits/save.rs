use crate::types::*;
use std::io::Result;

pub trait Save: serde::Serialize {
    fn save(&self, db: &DB) -> Result<Digest> {
        let json = serde_json::to_string(self)?;
        let d = Digest::from(&json);
        std::fs::write(db.join("cas").join(d.to_hex()), json)?;
        Ok(d)
    }
}

impl Save for Ark<&str> {}
impl Save for Ark<String> {}
impl Save for Ark<Vec<u8>> {}
impl Save for Ark<Digest> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn save() -> Result<()> {
        let db = DB::new_temp()?;
        let digest = Ark::from_entries([("/hello", Contents::File("world"))]).save(&db)?;
        assert_eq!(
            digest.to_hex(),
            "e1bf5a0db81cf1bdbe24152057618a2038395e792dc27eb5cd045415588c701e"
        );

        Ok(())
    }
}
