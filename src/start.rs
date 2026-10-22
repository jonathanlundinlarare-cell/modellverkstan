//! Startsidan med nya modeller och exempel.

use crate::app::App;
use crate::examples::{self, Subject};
use crate::model::{Model, ModelKind};
use crate::theme::{self, cr};
use egui::{Color32, Context, Margin, RichText, Sense, Stroke, StrokeKind, Ui};

fn card(ui: &mut Ui, width: f32, badge: (&str, Color32), title: &str, body: &str) -> egui::Response {
    let resp = egui::Frame::default()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(224, 222, 214)))
        .corner_radius(cr(12.0))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.style_mut().interaction.selectable_labels = false;
            ui.set_width((width - 34.0).max(40.0));
            ui.set_min_height(110.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.label(RichText::new(badge.0).small().strong().color(badge.1));
                ui.label(RichText::new(title).size(19.0).strong().color(theme::INK));
                ui.label(RichText::new(body).color(theme::MUTED));
            });
        })
        .response
        .interact(Sense::click());
    if resp.hovered() {
        ui.painter().rect_stroke(resp.rect, cr(12.0), Stroke::new(2.0_f32, badge.1), StrokeKind::Inside);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

struct Card {
    badge: (&'static str, Color32),
    title: String,
    body: String,
    model: Model,
}

fn card_grid(app: &mut App, ui: &mut Ui, width: f32, cols: usize, cards: Vec<Card>) {
    let gap = 12.0;
    let card_w = ((width - gap * (cols as f32 - 1.0)) / cols as f32).floor();
    let mut cards = cards.into_iter().peekable();
    while cards.peek().is_some() {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for c in cards.by_ref().take(cols) {
                if card(ui, card_w, c.badge, &c.title, &c.body).clicked() {
                    app.load_model(c.model);
                }
            }
        });
        ui.add_space(4.0);
    }
}

/// En flik för ett ämne: tydligt markerad när den är vald.
fn subject_tab(ui: &mut Ui, current: &mut Option<Subject>, value: Option<Subject>, label: &str) {
    let selected = *current == value;
    let text = RichText::new(label).size(16.0).color(if selected { Color32::WHITE } else { theme::INK });
    let button = egui::Button::new(text)
        .fill(if selected { theme::ACCENT } else { Color32::WHITE })
        .stroke(Stroke::new(1.0_f32, if selected { theme::ACCENT } else { Color32::from_rgb(214, 212, 204) }))
        .corner_radius(cr(16.0))
        .min_size(egui::vec2(0.0, 32.0));
    if ui.add(button).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
        *current = value;
    }
}

fn badge(kind: ModelKind) -> (&'static str, Color32) {
    match kind {
        ModelKind::Cld => ("ORSAKSDIAGRAM", theme::REINFORCING),
        ModelKind::Sfd => ("FLÖDESMODELL", theme::STOCK),
    }
}

pub fn show(app: &mut App, ctx: &Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(theme::CANVAS_BG).inner_margin(Margin::same(24)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let avail = ui.available_width().max(100.0);
                let width = avail.min(1060.0);
                ui.horizontal(|ui| {
                    ui.add_space((avail - width) / 2.0);
                    ui.vertical(|ui| {
                        ui.set_width(width);
                        content(app, ui, width);
                    });
                });
            });
        });
}

fn content(app: &mut App, ui: &mut Ui, width: f32) {
    ui.add_space(12.0);
    ui.label(RichText::new("Bygg modeller av system").size(34.0).strong().color(theme::INK));
    ui.label(
        RichText::new(
            "Rita hur saker påverkar varandra, bygg modeller med lager och flöden och se hur systemet förändras över tid. \
             Dra i reglagen – grafen ändras direkt.",
        )
        .size(17.0)
        .color(theme::MUTED),
    );
    ui.add_space(20.0);

    let cols = if width > 760.0 { 3 } else if width > 480.0 { 2 } else { 1 };

    ui.label(RichText::new("Börja nytt").size(20.0).strong());
    let mut new_cards = vec![
        Card {
            badge: badge(ModelKind::Cld),
            title: "Nytt orsaksdiagram".into(),
            body: "Visa hur saker påverkar varandra med pilar och + / −. Programmet hittar looparna åt dig.".into(),
            model: Model::new(ModelKind::Cld),
        },
        Card {
            badge: badge(ModelKind::Sfd),
            title: "Ny flödesmodell".into(),
            body: "Bygg med lager, flöden och konstanter. Modellen räknas ut direkt och visas i en graf.".into(),
            model: Model::new(ModelKind::Sfd),
        },
    ];
    if let Some(m) = &app.resume {
        new_cards.push(Card {
            badge: ("FORTSÄTT DÄR DU VAR", theme::FLOW),
            title: m.title.clone(),
            body: "Öppna modellen du arbetade med senast.".into(),
            model: m.clone(),
        });
    }
    card_grid(app, ui, width, cols, new_cards);

    ui.add_space(20.0);
    ui.label(RichText::new("Exempel att utforska").size(20.0).strong());
    let subjects = examples::subjects();
    if subjects.len() > 1 {
        ui.horizontal_wrapped(|ui| {
            subject_tab(ui, &mut app.subject_filter, None, "Alla ämnen");
            for s in subjects {
                subject_tab(ui, &mut app.subject_filter, Some(s), s.label());
            }
        });
        ui.add_space(6.0);
    }
    let examples = examples::all()
        .into_iter()
        .filter(|ex| app.subject_filter.is_none_or(|s| s == ex.subject))
        .map(|ex| Card { badge: badge(ex.kind), title: ex.title.into(), body: ex.summary.into(), model: (ex.build)() })
        .collect();
    card_grid(app, ui, width, cols, examples);

    ui.add_space(20.0);
    ui.horizontal_wrapped(|ui| {
        if ui.button("📂 Öppna en sparad modell…").clicked() {
            app.open_file(ui.ctx());
        }
        if ui.button("❓ Så fungerar det").clicked() {
            app.show_help = true;
        }
    });
    ui.add_space(8.0);
    ui.label(
        RichText::new("Modellverkstan sparar automatiskt den modell du arbetar med. Spara som fil för att dela med andra.")
            .small()
            .color(theme::MUTED),
    );
}
