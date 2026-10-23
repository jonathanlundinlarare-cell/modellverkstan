//! Färger, typsnitt och talformat. Se DESIGN.md för hela designsystemet.

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, RichText, Shadow, Stroke, TextStyle};
use std::sync::Arc;

// Gränssnittet: svagt kalla neutraler, så att modellens färger får bära betydelsen.
pub const INK: Color32 = Color32::from_rgb(26, 34, 51);
pub const INK_2: Color32 = Color32::from_rgb(74, 84, 104);
pub const MUTED: Color32 = Color32::from_rgb(104, 113, 134);
pub const SURFACE: Color32 = Color32::from_rgb(252, 252, 253);
pub const SURFACE_2: Color32 = Color32::from_rgb(242, 244, 248);
pub const HOVER: Color32 = Color32::from_rgb(233, 237, 243);
pub const PRESSED: Color32 = Color32::from_rgb(222, 227, 236);
pub const LINE: Color32 = Color32::from_rgb(227, 231, 238);
pub const LINE_STRONG: Color32 = Color32::from_rgb(203, 210, 221);
pub const CANVAS_BG: Color32 = Color32::from_rgb(247, 248, 251);
pub const GRID: Color32 = Color32::from_rgb(213, 219, 229);
/// Markering och tips: som en överstrykningspenna.
pub const SELECT: Color32 = Color32::from_rgb(245, 165, 36);
pub const SELECT_SOFT: Color32 = Color32::from_rgb(254, 243, 220);
pub const ACCENT: Color32 = INK;
pub const SUCCESS: Color32 = Color32::from_rgb(30, 142, 90);
pub const SUCCESS_SOFT: Color32 = Color32::from_rgb(226, 244, 234);
pub const ERROR: Color32 = Color32::from_rgb(208, 58, 58);
pub const ERROR_SOFT: Color32 = Color32::from_rgb(252, 234, 234);

// Modellnotationen.
pub const STOCK: Color32 = Color32::from_rgb(47, 111, 219);
pub const STOCK_FILL: Color32 = Color32::from_rgb(234, 242, 254);
pub const STOCK_LEVEL: Color32 = Color32::from_rgb(188, 213, 250);
pub const FLOW: Color32 = Color32::from_rgb(31, 157, 92);
pub const FLOW_LIGHT: Color32 = Color32::from_rgb(213, 240, 226);
pub const AUX: Color32 = Color32::from_rgb(115, 83, 209);
pub const AUX_FILL: Color32 = Color32::from_rgb(239, 234, 252);
pub const CONST: Color32 = Color32::from_rgb(217, 116, 28);
pub const CONST_FILL: Color32 = Color32::from_rgb(253, 239, 223);
pub const DEP: Color32 = Color32::from_rgb(150, 158, 175);

pub const POSITIVE: Color32 = SUCCESS;
pub const NEGATIVE: Color32 = ERROR;
pub const REINFORCING: Color32 = Color32::from_rgb(216, 100, 42);
pub const BALANCING: Color32 = STOCK;

pub const SERIES: [Color32; 8] = [
    Color32::from_rgb(47, 111, 219),
    Color32::from_rgb(222, 104, 36),
    Color32::from_rgb(31, 150, 88),
    Color32::from_rgb(160, 72, 190),
    Color32::from_rgb(205, 52, 82),
    Color32::from_rgb(18, 145, 160),
    Color32::from_rgb(160, 125, 20),
    Color32::from_rgb(90, 98, 115),
];

pub fn kind_color(kind: crate::model::NodeKind) -> Color32 {
    use crate::model::NodeKind::*;
    match kind {
        Stock => STOCK,
        Flow => FLOW,
        Aux => AUX,
        Constant => CONST,
        Variable => INK,
    }
}

pub fn cr(r: f32) -> CornerRadius {
    CornerRadius::same(r.round().clamp(0.0, 255.0) as u8)
}

// Typsnitt: Figtree i tre vikter, med egui:s typsnitt som reserv för symboler.
const SEMIBOLD: &str = "semibold";
const BOLD: &str = "bold";

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}

/// Rubrik i fet stil.
pub fn title(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text).font(bold(size)).color(INK)
}

/// Halvfet text, t.ex. etiketter och namn.
pub fn strong(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text).font(semibold(size)).color(INK)
}

/// Överrubrik: liten, fet och i versaler.
pub fn overline(text: &str) -> RichText {
    RichText::new(text.to_uppercase()).font(bold(12.5)).color(MUTED)
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (name, bytes) in [
        ("figtree", &include_bytes!("../assets/fonts/Figtree-400.ttf")[..]),
        ("figtree-semibold", &include_bytes!("../assets/fonts/Figtree-600.ttf")[..]),
        ("figtree-bold", &include_bytes!("../assets/fonts/Figtree-700.ttf")[..]),
    ] {
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_static(bytes)));
    }
    let family = |first: &str| std::iter::once(first.to_string()).chain(fallback.iter().cloned()).collect::<Vec<_>>();
    fonts.families.insert(FontFamily::Proportional, family("figtree"));
    fonts.families.insert(FontFamily::Name(SEMIBOLD.into()), family("figtree-semibold"));
    fonts.families.insert(FontFamily::Name(BOLD.into()), family("figtree-bold"));
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_visuals(egui::Visuals::light());
    ctx.style_mut(|s| {
        s.text_styles.insert(TextStyle::Heading, bold(21.0));
        s.text_styles.insert(TextStyle::Body, regular(16.0));
        s.text_styles.insert(TextStyle::Button, semibold(15.0));
        s.text_styles.insert(TextStyle::Small, regular(13.5));
        s.text_styles.insert(TextStyle::Monospace, FontId::monospace(15.0));

        let sp = &mut s.spacing;
        sp.item_spacing = egui::vec2(8.0, 8.0);
        sp.button_padding = egui::vec2(12.0, 6.0);
        sp.interact_size.y = 32.0;
        sp.slider_width = 200.0;
        sp.combo_height = 300.0;
        sp.menu_margin = egui::Margin::same(8);

        let v = &mut s.visuals;
        v.panel_fill = SURFACE;
        v.window_fill = SURFACE;
        v.faint_bg_color = SURFACE_2;
        v.extreme_bg_color = Color32::from_rgb(255, 255, 254);
        v.code_bg_color = SURFACE_2;
        v.window_stroke = Stroke::new(1.0_f32, LINE);
        v.window_corner_radius = cr(12.0);
        v.menu_corner_radius = cr(10.0);
        v.window_shadow = Shadow { offset: [0, 8], blur: 28, spread: 0, color: Color32::from_black_alpha(28) };
        v.popup_shadow = Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(24) };
        v.selection.bg_fill = Color32::from_rgb(201, 216, 245);
        v.selection.stroke = Stroke::new(1.0_f32, INK);
        v.hyperlink_color = STOCK;
        v.text_cursor.stroke = Stroke::new(2.0_f32, INK);
        v.slider_trailing_fill = true;
        v.handle_shape = egui::style::HandleShape::Circle;
        v.striped = true;

        let w = &mut v.widgets;
        for state in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
            state.corner_radius = cr(8.0);
            state.expansion = 0.0;
        }
        w.noninteractive.bg_stroke = Stroke::new(1.0_f32, LINE);
        w.noninteractive.fg_stroke = Stroke::new(1.0_f32, INK);
        w.noninteractive.bg_fill = SURFACE;
        w.noninteractive.weak_bg_fill = SURFACE;

        w.inactive.weak_bg_fill = SURFACE;
        w.inactive.bg_fill = SURFACE_2;
        w.inactive.bg_stroke = Stroke::new(1.0_f32, LINE_STRONG);
        w.inactive.fg_stroke = Stroke::new(1.5_f32, INK);

        w.hovered.weak_bg_fill = HOVER;
        w.hovered.bg_fill = HOVER;
        w.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(170, 180, 196));
        w.hovered.fg_stroke = Stroke::new(1.5_f32, INK);

        w.active.weak_bg_fill = PRESSED;
        w.active.bg_fill = PRESSED;
        w.active.bg_stroke = Stroke::new(1.0_f32, INK_2);
        w.active.fg_stroke = Stroke::new(2.0_f32, INK);

        w.open.weak_bg_fill = HOVER;
        w.open.bg_fill = SURFACE;
        w.open.bg_stroke = Stroke::new(1.0_f32, LINE_STRONG);
        w.open.fg_stroke = Stroke::new(1.5_f32, INK);
    });
}

/// Formaterar ett tal på svenskt vis (decimalkomma, lagom många decimaler).
pub fn fmt_num(v: f64) -> String {
    if !v.is_finite() {
        return "–".into();
    }
    let a = v.abs();
    let s = if a >= 1000.0 || a == 0.0 || (a - a.round()).abs() < 1e-9 && a >= 1.0 {
        format!("{:.0}", v)
    } else if a >= 100.0 {
        format!("{:.1}", v)
    } else if a >= 1.0 {
        format!("{:.2}", v)
    } else {
        format!("{:.3}", v)
    };
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    let s = if s == "-0" { "0".to_string() } else { s };
    s.replace('.', ",")
}

pub fn fmt_with_unit(v: f64, unit: &str) -> String {
    if unit.trim().is_empty() {
        fmt_num(v)
    } else {
        format!("{} {}", fmt_num(v), unit.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::fmt_num;

    #[test]
    fn swedish_numbers() {
        assert_eq!(fmt_num(21.0), "21");
        assert_eq!(fmt_num(21.456), "21,46");
        assert_eq!(fmt_num(0.15), "0,15");
        assert_eq!(fmt_num(-5.0), "-5");
        assert_eq!(fmt_num(1234.6), "1235");
        assert_eq!(fmt_num(-0.0001), "0");
    }
}
