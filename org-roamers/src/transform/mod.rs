//! # transform module
//! This module contains all tranformation and filtering the library supports
//! on org. Each operation is it's own module:
//! - [`export`]: Export an org string/file to html.
//! - [`org`]: Transform an org string into a
//!   [`NodeFromOrg`](crate::transform::org::NodeFromOrg).
//! - [`subtree`]: Get a subtree of an org file.
//! - [`title`]: Strip all syntax from the org input and return a string that
//!   can be displayed in contexts without org support.
//!
//! All of these parsers use the [`orgize`] parsers.
pub mod export;
pub mod org;
pub mod subtree;
pub mod title;
