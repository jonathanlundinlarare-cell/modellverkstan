//! Färger, typsnitt och talformat.

use egui::{Color32, CornerRadius, FontId, Stroke, TextStyle};

pub const INK: Color32 = Color32::from_rgb(38, 42, 52);
pub const MUTED: Color32 = Color32::from_rgb(110, 116, 128);
pub const CANVAS_BG: Color32 = Color32::from_rgb(251, 250, 246);
pub const GRID: Color32 = Color32::from_rgb(222, 220, 212);
pub const PANEL_BG: Color32 = Color32::from_rgb(243, 242, 237);
pub const ACCENT: Color32 = Color32::from_rgb(37, 99, 170);

pub const STOCK: Color32 = Color32::from_rgb(45, 110, 190);
pub const STOCK_FILL: Color32 = Color32::from_rgb(226, 237, 251);
pub const STOCK_LEVEL: Color32 = Color32::from_rgb(160, 200, 242);
pub const FLOW: Color32 = Color32::from_rgb(38, 140, 86);
pub const FLOW_LIGHT: Color32 = Color32::from_rgb(205, 236, 215);
pub const AUX: Color32 = Color32::from_rgb(118, 92, 176);
pub const AUX_FILL: Color32 = Color32::from_rgb(240, 235, 252);
pub const CONST: Color32 = Color32::from_rgb(212, 112, 24);
pub const CONST_FILL: Color32 = Color32::from_rgb(255, 238, 218);
pub const VAR_FILL: Color32 = Color32::from_rgb(255, 255, 255);
pub const DEP: Color32 = Color32::from_rgb(150, 154, 165);

pub const POSITIVE: Color32 = Color32::from_rgb(30, 120, 70);
pub const NEGATIVE: Color32 = Color32::from_rgb(200, 45, 45);
pub const REINFORCING: Color32 = Color32::from_rgb(205, 95, 30);
pub const BALANCING: Color32 = Color32::from_rgb(40, 110, 185);
pub const ERROR: Color32 = Color32::from_rgb(205, 38, 38);
pub const SELECT: Color32 = Color32::from_rgb(255, 184, 28);

pub const SERIES: [Color32; 8] = [
    Color32::from_rgb(37, 99, 200),
    Color32::from_rgb(220, 90, 30),
    Color32::from_rgb(40, 150, 80),
    Color32::from_rgb(170, 60, 170),
    Color32::from_rgb(200, 40, 60),
    Color32::from_rgb(20, 150, 160),
    Color32::from_rgb(150, 120, 20),
    Color32::from_rgb(90, 90, 90),
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

pub fn apply(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());
    ctx.style_mut(|s| {
        s.text_styles.insert(TextStyle::Heading, FontId::proportional(24.0));
        s.text_styles.insert(TextStyle::Body, FontId::proportional(16.0));
        s.text_styles.insert(TextStyle::Button, FontId::proportional(16.0));
        s.text_styles.insert(TextStyle::Small, FontId::proportional(13.0));
        s.text_styles.insert(TextStyle::Monospace, FontId::monospace(15.0));
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.interact_size.y = 28.0;
        s.spacing.slider_width = 200.0;
        let v = &mut s.visuals;
        v.panel_fill = PANEL_BG;
        v.window_fill = Color32::WHITE;
        v.extreme_bg_color = Color32::WHITE;
        v.selection.bg_fill = Color32::from_rgb(190, 215, 245);
        v.selection.stroke = Stroke::new(1.0_f32, ACCENT);
        v.hyperlink_color = ACCENT;
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = CornerRadius::same(6);
        }
        v.widgets.noninteractive.fg_stroke.color = INK;
        v.widgets.inactive.fg_stroke.color = INK;
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(232, 231, 225);
        v.widgets.inactive.bg_fill = Color32::from_rgb(225, 224, 217);
        v.window_corner_radius = CornerRadius::same(10);
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
