//! Små återanvändbara komponenter enligt DESIGN.md, så att samma sak ser likadan ut överallt.

use crate::model::NodeKind;
use crate::theme::{self, cr};
use egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Galley, Painter, Pos2, Rect, Response, RichText, Sense, Shadow, Stroke,
    StrokeKind, Ui, Vec2,
};
use std::sync::Arc;

fn text_galley(ui: &Ui, text: &str, font: egui::FontId, color: Color32) -> Arc<Galley> {
    ui.painter().layout_no_wrap(text.to_string(), font, color)
}

/// Primärknapp: fylld Ink. Högst en per vy.
pub fn primary(ui: &mut Ui, enabled: bool, text: &str) -> Response {
    let fill = if enabled { theme::INK } else { theme::LINE_STRONG };
    let button = egui::Button::new(RichText::new(text).font(theme::semibold(15.0)).color(theme::SURFACE))
        .fill(fill)
        .stroke(Stroke::NONE)
        .corner_radius(cr(8.0))
        .min_size(vec2(0.0, 34.0));
    let resp = ui.add_enabled(enabled, button);
    if enabled {
        resp.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// Diskret knapp utan ram; bakgrund bara när man håller musen över den.
pub fn ghost(ui: &mut Ui, enabled: bool, text: &str) -> Response {
    ghost_colored(ui, enabled, text, theme::INK)
}

pub fn ghost_colored(ui: &mut Ui, enabled: bool, text: &str, color: Color32) -> Response {
    let galley = text_galley(ui, text, theme::semibold(14.5), color);
    let size = vec2(galley.size().x + 20.0, 34.0);
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    if ui.is_rect_visible(rect) {
        let fill = if !enabled {
            Color32::TRANSPARENT
        } else if resp.is_pointer_button_down_on() {
            theme::PRESSED
        } else if resp.hovered() {
            theme::HOVER
        } else {
            Color32::TRANSPARENT
        };
        let painter = ui.painter();
        painter.rect_filled(rect, cr(8.0), fill);
        let tint = if enabled { color } else { theme::LINE_STRONG };
        painter.galley(rect.center() - galley.size() / 2.0, galley, tint);
    }
    if enabled {
        resp.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        resp
    }
}

/// Diskret knapp som öppnar en liten meny ovanför sig.
pub fn ghost_menu(ui: &mut Ui, enabled: bool, text: &str, tooltip: &str, add_contents: impl FnOnce(&mut Ui)) {
    let resp = ghost(ui, enabled, text).on_hover_text(tooltip);
    let id = resp.id.with("menu");
    if resp.clicked() {
        ui.memory_mut(|m| m.toggle_popup(id));
    }
    egui::popup_above_or_below_widget(
        ui,
        id,
        &resp,
        egui::AboveOrBelow::Above,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_min_width(240.0);
            add_contents(ui);
        },
    );
}

/// Segmentkontroll: ett spår med upphöjt valt segment. Ger `true` om valet ändrades.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let font = theme::semibold(14.5);
    let galleys: Vec<_> = options.iter().map(|(_, label)| text_galley(ui, label, font.clone(), theme::INK)).collect();
    let pad = 3.0;
    let widths: Vec<f32> = galleys.iter().map(|g| g.size().x + 26.0).collect();
    let size = vec2(widths.iter().sum::<f32>() + pad * 2.0, 36.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, cr(10.0), theme::SURFACE_2);
    let mut x = rect.left() + pad;
    let mut changed = false;
    for (i, ((option, _), galley)) in options.iter().zip(galleys).enumerate() {
        let seg = Rect::from_min_size(pos2(x, rect.top() + pad), vec2(widths[i], size.y - pad * 2.0));
        x += widths[i];
        let resp = ui.interact(seg, ui.id().with(("segment", i)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let selected = *value == *option;
        let painter = ui.painter();
        if selected {
            painter.add(soft_shadow(seg, 8.0, 1));
            painter.rect(seg, cr(8.0), theme::SURFACE, Stroke::new(1.0_f32, theme::LINE), StrokeKind::Inside);
        } else if resp.hovered() {
            painter.rect_filled(seg, cr(8.0), theme::HOVER);
        }
        let color = if selected { theme::INK } else { theme::INK_2 };
        painter.galley(seg.center() - galley.size() / 2.0, galley, color);
        if resp.clicked() && !selected {
            *value = *option;
            changed = true;
        }
    }
    changed
}

/// Ett chip: rundad etikett, valfritt med färgprick. Används för serier, filter och jämförelser.
pub fn chip(ui: &mut Ui, text: &str, dot: Option<Color32>, selected: bool) -> Response {
    let color = if selected { theme::INK } else { theme::MUTED };
    let galley = text_galley(ui, text, theme::semibold(14.0), color);
    let dot_w = if dot.is_some() { 16.0 } else { 0.0 };
    let size = vec2(galley.size().x + 24.0 + dot_w, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let (fill, stroke) = if selected {
            (theme::SURFACE, Stroke::new(1.0_f32, theme::LINE_STRONG))
        } else if resp.hovered() {
            (theme::HOVER, Stroke::new(1.0_f32, theme::LINE))
        } else {
            (Color32::TRANSPARENT, Stroke::new(1.0_f32, theme::LINE))
        };
        painter.rect(rect, cr(15.0), fill, stroke, StrokeKind::Inside);
        let mut x = rect.left() + 12.0;
        if let Some(c) = dot {
            let center = pos2(x + 5.0, rect.center().y);
            if selected {
                painter.circle_filled(center, 5.0, c);
            } else {
                painter.circle_stroke(center, 4.5, Stroke::new(1.5_f32, c));
            }
            x += dot_w;
        }
        painter.galley(pos2(x, rect.center().y - galley.size().y / 2.0), galley, color);
    }
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

/// Liten etikett för en del i modellen: färgprick + namnet på delen (t.ex. "Lager").
pub fn kind_tag(ui: &mut Ui, kind: NodeKind) {
    let color = theme::kind_color(kind);
    let galley = text_galley(ui, kind.label(), theme::bold(13.5), color);
    let size = vec2(galley.size().x + 30.0, 26.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, cr(13.0), color.gamma_multiply(0.12));
    painter.circle_filled(pos2(rect.left() + 12.0, rect.center().y), 4.5, color);
    painter.galley(pos2(rect.left() + 22.0, rect.center().y - galley.size().y / 2.0), galley, color);
}

/// Överrubrik för ett avsnitt.
pub fn section_title(ui: &mut Ui, text: &str) {
    ui.label(theme::overline(text));
}

/// Tunn avdelare med luft runt.
pub fn divider(ui: &mut Ui) {
    ui.add_space(14.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0_f32, theme::LINE));
    ui.add_space(14.0);
}

/// Mjuk skugga för det som ligger ovanpå något annat.
pub fn soft_shadow(rect: Rect, radius: f32, strength: u8) -> egui::Shape {
    let (offset, blur, alpha) = match strength {
        0 => (1, 3, 14),
        1 => (2, 6, 18),
        _ => (6, 18, 26),
    };
    Shadow { offset: [0, offset], blur, spread: 0, color: Color32::from_black_alpha(alpha) }.as_shape(rect, cr(radius)).into()
}

/// Appens logotyp: ett lager som fylls av ett flöde (samma motiv som ikonen).
pub fn logo(painter: &Painter, rect: Rect) {
    let s = rect.width();
    let at = |x: f32, y: f32| rect.min + vec2(x, y) * s;
    painter.rect_filled(rect, cr(s * 0.24), theme::STOCK);
    let stock = Rect::from_min_max(at(0.48, 0.26), at(0.86, 0.76));
    painter.rect_filled(stock, cr(s * 0.08), theme::SURFACE);
    let level = Rect::from_min_max(at(0.52, 0.52), at(0.82, 0.72));
    painter.rect_filled(level, cr(s * 0.04), theme::STOCK_LEVEL);
    painter.line_segment([at(0.12, 0.51), at(0.38, 0.51)], Stroke::new(s * 0.1, Color32::from_rgb(60, 205, 130)));
    painter.add(egui::Shape::convex_polygon(
        vec![at(0.36, 0.39), at(0.36, 0.63), at(0.47, 0.51)],
        Color32::from_rgb(60, 205, 130),
        Stroke::NONE,
    ));
}

/// Liten symbol för en modelltyp (lager med flöde, eller en loop), t.ex. i listor.
pub fn kind_glyph(painter: &Painter, center: Pos2, size: f32, cld: bool) {
    let s = size / 40.0;
    if cld {
        let r = 11.0 * s;
        let pts: Vec<Pos2> = (0..=20)
            .map(|k| {
                let a = (40.0 + 290.0 * k as f32 / 20.0).to_radians();
                center + vec2(a.cos(), -a.sin()) * r
            })
            .collect();
        let n = pts.len();
        painter.add(egui::Shape::line(pts.clone(), Stroke::new(2.5 * s, theme::REINFORCING)));
        let dir = (pts[n - 1] - pts[n - 2]).normalized();
        let side = vec2(-dir.y, dir.x);
        let tip = pts[n - 1] + dir * 4.0 * s;
        painter.add(egui::Shape::convex_polygon(
            vec![tip, pts[n - 1] - dir * 3.0 * s + side * 4.0 * s, pts[n - 1] - dir * 3.0 * s - side * 4.0 * s],
            theme::REINFORCING,
            Stroke::NONE,
        ));
        painter.text(center, Align2::CENTER_CENTER, "R", theme::bold(11.0 * s), theme::REINFORCING);
    } else {
        let stock = Rect::from_center_size(center + vec2(5.0, 0.0) * s, vec2(16.0, 18.0) * s);
        painter.rect(stock, cr(3.0 * s), theme::STOCK_FILL, Stroke::new(2.0 * s, theme::STOCK), StrokeKind::Inside);
        let level = Rect::from_min_max(pos2(stock.left() + 2.0 * s, stock.center().y), stock.right_bottom() - Vec2::splat(2.0 * s));
        painter.rect_filled(level, cr(1.5 * s), theme::STOCK_LEVEL);
        let y = center.y;
        painter.line_segment([pos2(center.x - 15.0 * s, y), pos2(stock.left() - 4.0 * s, y)], Stroke::new(4.0 * s, theme::FLOW));
        painter.add(egui::Shape::convex_polygon(
            vec![
                pos2(stock.left() - 1.0 * s, y),
                pos2(stock.left() - 6.0 * s, y - 4.5 * s),
                pos2(stock.left() - 6.0 * s, y + 4.5 * s),
            ],
            theme::FLOW,
            Stroke::NONE,
        ));
    }
}
