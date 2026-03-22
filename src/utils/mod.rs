//! Utilities for CSS name parsing, identifier encoding/decoding, and more.
//!
//! Reference: `external/csstree/lib/utils/`
//!
//! ## Modules
//!
//! - `names` — Parse property/keyword names, extract vendor prefixes and hacks
//! - `ident` — CSS identifier encode/decode
//! - `string` — CSS string literal encode/decode
//! - `url` — CSS `url()` value encode/decode

pub mod names;
pub mod ident;

pub use names::{
    keyword_descriptor, property_descriptor, vendor_prefix, is_custom_property,
    KeywordDescriptor, PropertyDescriptor,
};
pub use ident::{decode_ident, encode_ident};
