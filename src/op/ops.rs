//! The actual implementations for individual Operations.
//!
//! These have a canonical display order that is used for help printing and
//! consistent ordering in enums. We can't rely on import order, because the
//! rustfmt hook seems to not respect a macro to skip this file >_> .
//!
//! The order is:
//!
//!  - empty
//!  - import
//!  - export
//!  - merge
//!  - filter
//!  - replace
//!  - prefix
//!  - download

pub mod download;
pub mod empty;
pub mod export;
pub mod filter;
pub mod import;
pub mod merge;
pub mod prefix;
pub mod replace;
// pub mod download_impure;
// pub mod cmd_impure;

mod prelude;
