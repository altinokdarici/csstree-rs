//! CSS unit groups for dimension type validation.

use std::collections::HashMap;

/// Predefined CSS unit groups.
///
/// Each group maps to a set of valid unit strings.
#[allow(clippy::too_many_lines)]
pub fn default_units() -> HashMap<&'static str, Vec<&'static str>> {
    let mut units = HashMap::new();

    units.insert("length", vec![
        "cm", "mm", "q", "in", "pt", "pc", "px",
        "em", "rem", "ex", "rex", "cap", "rcap", "ch", "rch",
        "ic", "ric", "lh", "rlh",
        "vw", "svw", "lvw", "dvw",
        "vh", "svh", "lvh", "dvh",
        "vi", "svi", "lvi", "dvi",
        "vb", "svb", "lvb", "dvb",
        "vmin", "svmin", "lvmin", "dvmin",
        "vmax", "svmax", "lvmax", "dvmax",
        "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ]);

    units.insert("angle", vec!["deg", "grad", "rad", "turn"]);
    units.insert("time", vec!["s", "ms"]);
    units.insert("frequency", vec!["hz", "khz"]);
    units.insert("resolution", vec!["dpi", "dpcm", "dppx", "x"]);
    units.insert("flex", vec!["fr"]);
    units.insert("decibel", vec!["db"]);
    units.insert("semitones", vec!["st"]);

    units
}

/// Static reference to the default unit groups.
pub static UNITS: std::sync::LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    std::sync::LazyLock::new(default_units);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_has_length() {
        let units = default_units();
        assert!(units.contains_key("length"));
        assert!(units["length"].contains(&"px"));
        assert!(units["length"].contains(&"em"));
    }

    #[test]
    fn units_has_angle() {
        let units = default_units();
        assert!(units["angle"].contains(&"deg"));
    }

    #[test]
    fn units_has_time() {
        let units = default_units();
        assert!(units["time"].contains(&"s"));
        assert!(units["time"].contains(&"ms"));
    }

    #[test]
    fn static_units_accessible() {
        assert!(UNITS.contains_key("length"));
    }
}
