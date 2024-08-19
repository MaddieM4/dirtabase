pub use crate::context::Context;
pub use crate::logger::Logger;
pub use arkive::*;
pub use std::io::Result;

pub fn ctx<'a>(db: &'a DB, log: &'a mut Logger) -> Context<'a> {
    Context::new(db, log)
}
