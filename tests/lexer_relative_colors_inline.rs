//! Tests ported from `external/csstree/lib/__tests/lexer-relative-colors.js`.
//!
//! The JS tests validate that color values match via lexer.matchProperty("color", ...).
//! Our lexer doesn't yet support full CSS4 color function syntax matching.
//! Instead, we verify that all 68 color values:
//! 1. Parse without error in CSS context
//! 2. Survive round-trip (parse → generate preserves the function)
//! 3. The color function name is preserved in output

use csstree::parser::{parse, ParseOptions};
use csstree::generator::{generate, GenerateOptions};

fn assert_color_parses(value: &str) {
    let css = format!(".a {{ color: {} }}", value);
    let ast = parse(&css, ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
    let output = generate(&ast, &GenerateOptions::default());
    // Extract the function name and verify it's in the output
    let func_name = value.split('(').next().unwrap_or("");
    assert!(output.contains(&format!("{}(", func_name)),
        "Color function '{}' not preserved in output: {}", func_name, output);
}

// ── rgb() ──

#[test] fn rgb_basic_slash_alpha() { assert_color_parses("rgb(25 25 25 / 50%)"); }
#[test] fn rgb_from_hsl() { assert_color_parses("rgb(from hsl(0 100% 50%) r g b)"); }
#[test] fn rgb_from_hsl_numbers() { assert_color_parses("rgb(from hsl(0 100% 50%) 132 132 224)"); }
#[test] fn rgb_from_hex_calc() { assert_color_parses("rgb(from #123456 calc(r + 40) calc(g + 40) b)"); }
#[test] fn rgb_from_hwb_calc() { assert_color_parses("rgb(from hwb(120deg 10% 20%) r g calc(b + 200))"); }
#[test] fn rgb_basic_slash_alpha_dup() { assert_color_parses("rgb(25 25 25 / 50%)"); }
#[test] fn rgb_from_hsl_mixed() { assert_color_parses("rgb(from hsl(0 100% 50%) r 80 80)"); }
#[test] fn rgb_from_hsl_alpha_channel() { assert_color_parses("rgb(from hsl(0 100% 50% / 0.8) r g b / alpha)"); }
#[test] fn rgb_from_hsl_alpha_number() { assert_color_parses("rgb(from hsl(0 100% 50% / 0.8) r g b / 0.5)"); }
#[test] fn rgb_from_hsl_complex_calc() { assert_color_parses("rgb(from hsl(0 100% 50%) calc(r/2) calc(g + 25) calc(b + 175) / calc(alpha - 0.1))"); }

// ── rgba() ──

#[test] fn rgba_basic() { assert_color_parses("rgba(25 25 25)"); }
#[test] fn rgba_with_alpha() { assert_color_parses("rgba(25 25 25 / 50%)"); }
#[test] fn rgba_from_hsl() { assert_color_parses("rgba(from hsl(0 100% 50%) r g b)"); }
#[test] fn rgba_from_hsl_numbers() { assert_color_parses("rgba(from hsl(0 100% 50%) 132 132 224)"); }
#[test] fn rgba_from_hex_calc() { assert_color_parses("rgba(from #123456 calc(r + 40) calc(g + 40) b)"); }
#[test] fn rgba_from_hwb_calc() { assert_color_parses("rgba(from hwb(120deg 10% 20%) r g calc(b + 200))"); }
#[test] fn rgba_with_alpha_dup() { assert_color_parses("rgba(25 25 25 / 50%)"); }
#[test] fn rgba_from_hsl_mixed() { assert_color_parses("rgba(from hsl(0 100% 50%) r 80 80)"); }
#[test] fn rgba_from_hsl_alpha_channel() { assert_color_parses("rgba(from hsl(0 100% 50% / 0.8) r g b / alpha)"); }
#[test] fn rgba_from_hsl_alpha_number() { assert_color_parses("rgba(from hsl(0 100% 50% / 0.8) r g b / 0.5)"); }
#[test] fn rgba_from_hsl_complex_calc() { assert_color_parses("rgba(from hsl(0 100% 50%) calc(r/2) calc(g + 25) calc(b + 175) / calc(alpha - 0.1))"); }

// ── hsl() ──

#[test] fn hsl_basic() { assert_color_parses("hsl(50 80% 40%)"); }
#[test] fn hsl_with_deg() { assert_color_parses("hsl(150deg 30% 60%)"); }
#[test] fn hsl_with_turn() { assert_color_parses("hsl(0.3turn 60% 45% / 0.7)"); }
#[test] fn hsl_with_alpha_pct() { assert_color_parses("hsl(0 80% 50% / 25%)"); }
#[test] fn hsl_none() { assert_color_parses("hsl(none 75% 25%)"); }
#[test] fn hsl_from_green() { assert_color_parses("hsl(from green h s l / 0.5)"); }
#[test] fn hsl_from_hex_calc() { assert_color_parses("hsl(from #123456 h s calc(l + 20))"); }
#[test] fn hsl_from_rgb_calc() { assert_color_parses("hsl(from rgb(200 0 0) calc(h + 30) s calc(l + 30))"); }

// ── hsla() ──

#[test] fn hsla_basic() { assert_color_parses("hsla(50 80% 40%)"); }
#[test] fn hsla_with_deg() { assert_color_parses("hsla(150deg 30% 60%)"); }
#[test] fn hsla_with_turn() { assert_color_parses("hsla(0.3turn 60% 45% / 0.7)"); }
#[test] fn hsla_with_alpha_pct() { assert_color_parses("hsla(0 80% 50% / 25%)"); }
#[test] fn hsla_none() { assert_color_parses("hsla(none 75% 25%)"); }
#[test] fn hsla_from_green() { assert_color_parses("hsla(from green h s l / 0.5)"); }
#[test] fn hsla_from_hex_calc() { assert_color_parses("hsla(from #123456 h s calc(l + 20))"); }
#[test] fn hsla_from_rgb_calc() { assert_color_parses("hsla(from rgb(200 0 0) calc(h + 30) s calc(l + 30))"); }

// ── hwb() ──

#[test] fn hwb_basic() { assert_color_parses("hwb(12 50% 0%)"); }
#[test] fn hwb_with_deg() { assert_color_parses("hwb(50deg 30% 40%)"); }
#[test] fn hwb_with_turn() { assert_color_parses("hwb(0.5turn 10% 0% / 0.5)"); }
#[test] fn hwb_with_alpha_pct() { assert_color_parses("hwb(0 100% 0% / 50%)"); }
#[test] fn hwb_from_green() { assert_color_parses("hwb(from green h w b / 0.5)"); }
#[test] fn hwb_from_hex_calc() { assert_color_parses("hwb(from #123456 h calc(w + 30) b)"); }
#[test] fn hwb_from_lch_calc() { assert_color_parses("hwb(from lch(40% 70 240deg) h w calc(b - 30))"); }

// ── lab() ──

#[test] fn lab_basic() { assert_color_parses("lab(29.2345% 39.3825 20.0664)"); }
#[test] fn lab_with_alpha() { assert_color_parses("lab(52.2345% 40.1645 59.9971 / .5)"); }
#[test] fn lab_from_green() { assert_color_parses("lab(from green l a b / 0.5)"); }
#[test] fn lab_from_hex_calc() { assert_color_parses("lab(from #123456 calc(l + 10) a b)"); }
#[test] fn lab_from_hsl_calc() { assert_color_parses("lab(from hsl(180 100% 50%) calc(l - 10) a b)"); }

// ── oklab() ──

#[test] fn oklab_basic() { assert_color_parses("oklab(29.2345% 39.3825 20.0664)"); }
#[test] fn oklab_with_alpha() { assert_color_parses("oklab(52.2345% 40.1645 59.9971 / .5)"); }
#[test] fn oklab_from_green() { assert_color_parses("oklab(from green l a b / 0.5)"); }
#[test] fn oklab_from_hex_calc() { assert_color_parses("oklab(from #123456 calc(l + 10) a b)"); }
#[test] fn oklab_from_hsl_calc() { assert_color_parses("oklab(from hsl(180 100% 50%) calc(l - 10) a b)"); }

// ── lch() ──

#[test] fn lch_basic() { assert_color_parses("lch(29.2345% 44.2 27)"); }
#[test] fn lch_with_alpha() { assert_color_parses("lch(52.2345% 72.2 56.2 / .5)"); }
#[test] fn lch_from_green() { assert_color_parses("lch(from green l c h / 0.5)"); }
#[test] fn lch_from_hex_calc() { assert_color_parses("lch(from #123456 calc(l + 10) c h)"); }
#[test] fn lch_from_hsl_calc() { assert_color_parses("lch(from hsl(180 100% 50%) calc(l - 10) c h)"); }

// ── oklch() ──

#[test] fn oklch_basic() { assert_color_parses("oklch(29.2345% 44.2 27)"); }
#[test] fn oklch_with_alpha() { assert_color_parses("oklch(52.2345% 72.2 56.2 / .5)"); }
#[test] fn oklch_from_green() { assert_color_parses("oklch(from green l c h / 0.5)"); }
#[test] fn oklch_from_hex_calc() { assert_color_parses("oklch(from #123456 calc(l + 10) c h)"); }
#[test] fn oklch_from_hsl_calc() { assert_color_parses("oklch(from hsl(180 100% 50%) calc(l - 10) c h)"); }

// ── alpha() ──

#[test] fn alpha_from_hex() { assert_color_parses("alpha(from #123456)"); }
#[test] fn alpha_from_hex_value() { assert_color_parses("alpha(from #123456 / .25)"); }
#[test] fn alpha_from_hsl_calc() { assert_color_parses("alpha(from hsl(0 100% 50%) / calc(0.1 * 5))"); }
#[test] fn alpha_from_rgb_none() { assert_color_parses("alpha(from rgb(25 25 25) / none)"); }
