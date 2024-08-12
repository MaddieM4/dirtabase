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

pub mod empty;
pub mod export;
pub mod filter;
pub mod import;
pub mod merge;
// replace
// prefix
// download
// download_impure
// cmd_impure

mod prelude;
