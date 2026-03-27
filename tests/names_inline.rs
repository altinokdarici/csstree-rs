//! Tests ported from `external/csstree/lib/__tests/names.js`.
//!
//! Verifies keyword_descriptor, property_descriptor, vendor_prefix, is_custom_property.

use csstree::utils::{
    keyword_descriptor, property_descriptor, vendor_prefix, is_custom_property,
};

// ── keyword ──

#[test]
fn keyword_base_test() {
    let kw = keyword_descriptor("test");
    assert_eq!(kw.name, "test");
    assert_eq!(kw.basename, "test");
    assert_eq!(kw.vendor, "");
    assert_eq!(kw.vendor, "");
    assert!(!kw.custom);
}

#[test]
fn keyword_normalize_lower_case() {
    let kw = keyword_descriptor("TesT");
    assert_eq!(kw.name, "test");
    assert_eq!(kw.basename, "test");
    assert_eq!(kw.vendor, "");
    assert!(!kw.custom);
}

#[test]
fn keyword_vendor_moz() {
    let kw = keyword_descriptor("-moz-test");
    assert_eq!(kw.name, "-moz-test");
    assert_eq!(kw.basename, "test");
    assert_eq!(kw.vendor, "-moz-");
    assert_eq!(kw.vendor, "-moz-");
    assert!(!kw.custom);
}

#[test]
fn keyword_vendor_webkit() {
    let kw = keyword_descriptor("-webkit-test");
    assert_eq!(kw.name, "-webkit-test");
    assert_eq!(kw.basename, "test");
    assert_eq!(kw.vendor, "-webkit-");
    assert_eq!(kw.vendor, "-webkit-");
}

#[test]
fn keyword_vendor_ms() {
    let kw = keyword_descriptor("-ms-test");
    assert_eq!(kw.basename, "test");
    assert_eq!(kw.vendor, "-ms-");
}

#[test]
fn keyword_vendor_o() {
    let kw = keyword_descriptor("-o-test");
    assert_eq!(kw.basename, "test");
    assert_eq!(kw.vendor, "-o-");
}

#[test]
fn keyword_no_vendor_for_non_dash() {
    let kw = keyword_descriptor("test-vendor-test");
    assert_eq!(kw.name, "test-vendor-test");
    assert_eq!(kw.basename, "test-vendor-test");
    assert_eq!(kw.vendor, "");
    assert_eq!(kw.vendor, "");
}

#[test]
fn keyword_custom_property_not_vendor() {
    let kw = keyword_descriptor("--test");
    assert_eq!(kw.name, "--test");
    assert_eq!(kw.basename, "--test");
    assert_eq!(kw.vendor, "");
    assert_eq!(kw.vendor, "");
    assert!(kw.custom);
}

#[test]
fn keyword_custom_property_with_vendor_like_name() {
    let kw = keyword_descriptor("--vendor-test");
    assert!(kw.custom);
    assert_eq!(kw.vendor, "");
}

// ── property ──

#[test]
fn property_base_test() {
    let p = property_descriptor("test");
    assert_eq!(p.name, "test");
    assert_eq!(p.basename, "test");
    assert!(!p.custom);
    assert_eq!(p.prefix, "");
    assert_eq!(p.hack, "");
    assert_eq!(p.vendor, "");
}

#[test]
fn property_normalize_lower_case() {
    let p = property_descriptor("TesT");
    assert_eq!(p.name, "test");
    assert_eq!(p.basename, "test");
}

#[test]
fn property_vendor_moz() {
    let p = property_descriptor("-moz-test");
    assert_eq!(p.name, "-moz-test");
    assert_eq!(p.basename, "test");
    assert_eq!(p.prefix, "-moz-");
    assert_eq!(p.vendor, "-moz-");
    assert_eq!(p.hack, "");
}

#[test]
fn property_vendor_webkit() {
    let p = property_descriptor("-webkit-test");
    assert_eq!(p.vendor, "-webkit-");
    assert_eq!(p.basename, "test");
}

#[test]
fn property_vendor_with_dashes() {
    let p = property_descriptor("-a-test-test");
    assert_eq!(p.name, "-a-test-test");
    assert_eq!(p.basename, "test-test");
    assert_eq!(p.vendor, "-a-");
}

#[test]
fn property_vendor_normalize_case() {
    let p = property_descriptor("-VenDor-TesT");
    assert_eq!(p.name, "-vendor-test");
    assert_eq!(p.basename, "test");
    assert_eq!(p.vendor, "-vendor-");
}

#[test]
fn property_hack_star() {
    let p = property_descriptor("*test");
    assert_eq!(p.name, "test");
    assert_eq!(p.basename, "test");
    assert_eq!(p.hack, "*");
    assert_eq!(p.prefix, "*");
}

#[test]
fn property_hack_underscore() {
    let p = property_descriptor("_test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "_");
}

#[test]
fn property_hack_dollar() {
    let p = property_descriptor("$test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "$");
}

#[test]
fn property_hack_plus() {
    let p = property_descriptor("+test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "+");
}

#[test]
fn property_hack_ampersand() {
    let p = property_descriptor("&test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "&");
}

#[test]
fn property_hack_hash() {
    let p = property_descriptor("#test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "#");
}

#[test]
fn property_hack_double_slash() {
    let p = property_descriptor("//test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "//");
}

#[test]
fn property_hack_slash() {
    let p = property_descriptor("/test");
    assert_eq!(p.name, "test");
    assert_eq!(p.hack, "/");
}

#[test]
fn property_custom() {
    let p = property_descriptor("--test");
    assert_eq!(p.name, "--test");
    assert_eq!(p.basename, "--test");
    assert!(p.custom);
    assert_eq!(p.hack, "");
    assert_eq!(p.vendor, "");
}

#[test]
fn property_vendor_and_hack() {
    let p = property_descriptor("//-moz-test");
    assert_eq!(p.name, "-moz-test");
    assert_eq!(p.basename, "test");
    assert_eq!(p.hack, "//");
    assert_eq!(p.vendor, "-moz-");
    assert_eq!(p.prefix, "//-moz-");
}

#[test]
fn property_custom_and_hack() {
    let p = property_descriptor("*--test");
    assert_eq!(p.hack, "*");
    assert!(p.custom);
}

// ── vendor_prefix ──

#[test]
fn vendor_prefix_moz() {
    assert_eq!(vendor_prefix("-moz-appearance"), "-moz-");
}

#[test]
fn vendor_prefix_webkit() {
    assert_eq!(vendor_prefix("-webkit-transform"), "-webkit-");
}

#[test]
fn vendor_prefix_none() {
    assert_eq!(vendor_prefix("color"), "");
}

#[test]
fn vendor_prefix_custom_not_vendor() {
    assert_eq!(vendor_prefix("--custom"), "");
}

// ── is_custom_property ──

#[test]
fn is_custom_property_yes() {
    assert!(is_custom_property("--my-var"));
}

#[test]
fn is_custom_property_no() {
    assert!(!is_custom_property("color"));
}

#[test]
fn is_custom_property_vendor_not_custom() {
    assert!(!is_custom_property("-webkit-appearance"));
}
