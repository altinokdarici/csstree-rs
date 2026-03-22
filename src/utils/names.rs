//! CSS property and keyword name parsing.
//!
//! Extracts vendor prefixes, hacks, and custom property detection.

/// Descriptor for a parsed CSS keyword name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordDescriptor {
    /// The lowercase base name without vendor prefix.
    pub basename: String,
    /// The full lowercase name.
    pub name: String,
    /// The vendor prefix (e.g., `-webkit-`).
    pub vendor: String,
    /// Whether this is a custom property (`--*`).
    pub custom: bool,
}

/// Descriptor for a parsed CSS property name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDescriptor {
    /// The base name without hack prefix or vendor prefix.
    pub basename: String,
    /// The name without hack prefix.
    pub name: String,
    /// The hack prefix (e.g., `*`, `_`, `$`).
    pub hack: String,
    /// The vendor prefix (e.g., `-webkit-`).
    pub vendor: String,
    /// Combined hack + vendor prefix.
    pub prefix: String,
    /// Whether this is a custom property (`--*`).
    pub custom: bool,
}

/// Check if a string is a custom property (starts with `--`).
pub fn is_custom_property(s: &str) -> bool {
    s.len() >= 2 && s.starts_with("--")
}

/// Extract the vendor prefix from a CSS name (e.g., `-webkit-` from `-webkit-transform`).
pub fn vendor_prefix(name: &str) -> &str {
    let bytes = name.as_bytes();
    if bytes.len() >= 3 && bytes[0] == b'-' && bytes[1] != b'-' {
        // Find the second hyphen
        if let Some(pos) = name[2..].find('-') {
            return &name[..pos + 3]; // Include the trailing hyphen
        }
    }
    ""
}

/// Parse a CSS keyword name into a descriptor.
pub fn keyword_descriptor(keyword: &str) -> KeywordDescriptor {
    let name = keyword.to_ascii_lowercase();
    let custom = is_custom_property(&name);
    let vendor = if custom {
        String::new()
    } else {
        vendor_prefix(&name).to_string()
    };
    let basename = name[vendor.len()..].to_string();

    KeywordDescriptor {
        basename,
        name,
        vendor,
        custom,
    }
}

/// Parse a CSS property name into a descriptor.
pub fn property_descriptor(property: &str) -> PropertyDescriptor {
    let bytes = property.as_bytes();

    // Detect hack prefix
    let hack = if bytes.is_empty() {
        ""
    } else {
        match bytes[0] {
            b'/' => {
                if bytes.len() > 1 && bytes[1] == b'/' {
                    "//"
                } else {
                    "/"
                }
            }
            b'_' | b'*' | b'$' | b'#' | b'+' | b'&' => {
                &property[..1]
            }
            _ => "",
        }
    };

    let after_hack = &property[hack.len()..];
    let custom = is_custom_property(after_hack);

    let name = if custom {
        after_hack.to_string()
    } else {
        after_hack.to_ascii_lowercase()
    };

    let vendor = if custom {
        String::new()
    } else {
        vendor_prefix(&name).to_string()
    };

    let prefix = format!("{hack}{vendor}");
    let basename = name[vendor.len()..].to_string();

    PropertyDescriptor {
        basename,
        name,
        hack: hack.to_string(),
        vendor,
        prefix,
        custom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_custom_property_works() {
        assert!(is_custom_property("--my-var"));
        assert!(is_custom_property("--"));
        assert!(!is_custom_property("-my-var"));
        assert!(!is_custom_property("color"));
    }

    #[test]
    fn vendor_prefix_extraction() {
        assert_eq!(vendor_prefix("-webkit-transform"), "-webkit-");
        assert_eq!(vendor_prefix("-moz-appearance"), "-moz-");
        assert_eq!(vendor_prefix("-ms-flex"), "-ms-");
        assert_eq!(vendor_prefix("color"), "");
        assert_eq!(vendor_prefix("--custom"), "");
    }

    #[test]
    fn keyword_descriptor_simple() {
        let desc = keyword_descriptor("color");
        assert_eq!(desc.name, "color");
        assert_eq!(desc.basename, "color");
        assert_eq!(desc.vendor, "");
        assert!(!desc.custom);
    }

    #[test]
    fn keyword_descriptor_vendor() {
        let desc = keyword_descriptor("-webkit-transform");
        assert_eq!(desc.name, "-webkit-transform");
        assert_eq!(desc.basename, "transform");
        assert_eq!(desc.vendor, "-webkit-");
    }

    #[test]
    fn keyword_descriptor_custom() {
        let desc = keyword_descriptor("--my-var");
        assert_eq!(desc.name, "--my-var");
        assert_eq!(desc.basename, "--my-var");
        assert!(desc.custom);
    }

    #[test]
    fn keyword_descriptor_case_insensitive() {
        let desc = keyword_descriptor("Color");
        assert_eq!(desc.name, "color");
    }

    #[test]
    fn property_descriptor_simple() {
        let desc = property_descriptor("color");
        assert_eq!(desc.name, "color");
        assert_eq!(desc.basename, "color");
        assert_eq!(desc.hack, "");
        assert_eq!(desc.vendor, "");
        assert!(!desc.custom);
    }

    #[test]
    fn property_descriptor_vendor() {
        let desc = property_descriptor("-webkit-transform");
        assert_eq!(desc.name, "-webkit-transform");
        assert_eq!(desc.basename, "transform");
        assert_eq!(desc.vendor, "-webkit-");
    }

    #[test]
    fn property_descriptor_hack() {
        let desc = property_descriptor("*color");
        assert_eq!(desc.hack, "*");
        assert_eq!(desc.name, "color");
        assert_eq!(desc.basename, "color");
    }

    #[test]
    fn property_descriptor_hack_underscore() {
        let desc = property_descriptor("_color");
        assert_eq!(desc.hack, "_");
        assert_eq!(desc.name, "color");
    }

    #[test]
    fn property_descriptor_custom() {
        let desc = property_descriptor("--my-var");
        assert_eq!(desc.name, "--my-var");
        assert!(desc.custom);
        assert_eq!(desc.hack, "");
    }

    #[test]
    fn property_descriptor_hack_plus_vendor() {
        let desc = property_descriptor("*-webkit-transform");
        assert_eq!(desc.hack, "*");
        assert_eq!(desc.vendor, "-webkit-");
        assert_eq!(desc.basename, "transform");
    }
}
