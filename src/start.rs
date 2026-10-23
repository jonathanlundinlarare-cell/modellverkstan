//! Startsidan: kom igång, träna steg för steg och utforska exempel.

use crate::app::App;
use crate::examples::{self, Example, Subject};
use crate::lessons::{self, LessonState};
use crate::model::{Model, ModelKind};
use crate::theme::{self, cr};
use crate::ui_kit;
use egui::{pos2, vec2, Context, CursorIcon, Margin, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui};

pub fn show(app: &mut App, ctx: &Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(theme::CANVAS_BG).inner_margin(Margin::symmetric(24, 0)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let avail = ui.available_width().max(200.0);
                let width = avail.min(980.0);
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
    ui.add_space(48.0);
    ui.label(theme::title("Förstå system genom att bygga dem", 34.0));
    ui.add_space(6.0);
    ui.scope(|ui| {
        ui.set_max_width(640.0);
        ui.label(
            RichText::new(
                "Rita hur saker påverkar varandra, bygg modeller med lager och flöden och se direkt vad som händer när du ändrar något.",
            )
            .font(theme::regular(17.0))
            .color(theme::INK_2),
        );
    });
    ui.add_space(28.0);

    // Kom igång
    let gap = 16.0;
    let two = width > 620.0;
    let tile_w = if two { (width - gap) / 2.0 } else { width };
    let mut start = None;
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = vec2(gap, gap);
        if !two {
            ui.vertical(|ui| tiles(ui, tile_w, &mut start));
        } else {
            tiles(ui, tile_w, &mut start);
        }
    });
    if let Some(kind) = start {
        app.load_model(Model::new(kind));
    }
    if let Some(m) = app.resume.clone() {
        ui.add_space(12.0);
        resume_banner(app, ui, m);
    }

    // Träna
    ui.add_space(52.0);
    ui.label(theme::overline("Ny här? Börja här"));
    ui.label(theme::title("Träna steg för steg", 23.0));
    ui.label(
        RichText::new("Bygg en modell själv med förklaringar och uppgifter. Programmet säger till när du har gjort rätt.")
            .font(theme::regular(15.5))
            .color(theme::MUTED),
    );
    ui.add_space(12.0);
    lesson_path(app, ui, width);

    // Exempel
    ui.add_space(52.0);
    ui.label(theme::overline("Färdiga modeller"));
    ui.label(theme::title("Utforska exempel", 23.0));
    let subjects = examples::subjects();
    if subjects.len() > 1 {
        ui.add_space(8.0);
        let mut options: Vec<(Option<Subject>, &str)> = vec![(None, "Alla ämnen")];
        options.extend(subjects.iter().map(|s| (Some(*s), s.label())));
        ui_kit::segmented(ui, &mut app.subject_filter, &options);
    }
    ui.add_space(12.0);
    let list: Vec<Example> =
        examples::all().into_iter().filter(|ex| app.subject_filter.is_none_or(|s| s == ex.subject)).collect();
    example_list(app, ui, width, list);

    ui.add_space(28.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        if ui_kit::ghost(ui, true, "📂  Öppna en sparad modell…").clicked() {
            app.open_file(ui.ctx());
        }
        if ui_kit::ghost(ui, true, "❓  Så fungerar det").clicked() {
            app.show_help = true;
        }
    });
    ui.add_space(8.0);
    ui.label(
        RichText::new("Modellen du arbetar med sparas automatiskt. Spara som fil för att dela den med andra.")
            .font(theme::regular(13.5))
            .color(theme::MUTED),
    );
    ui.add_space(48.0);
}

fn tiles(ui: &mut Ui, w: f32, start: &mut Option<ModelKind>) {
    if action_tile(ui, w, true, "Nytt orsaksdiagram", "Visa vad som påverkar vad med pilar och + / −. Programmet hittar looparna.")
        .clicked()
    {
        *start = Some(ModelKind::Cld);
    }
    if action_tile(ui, w, false, "Ny flödesmodell", "Bygg med lager och flöden. Modellen räknas ut direkt och visas i en graf.")
        .clicked()
    {
        *start = Some(ModelKind::Sfd);
    }
}

/// Ytan runt något klickbart: upphöjd vid hover.
fn hover_surface(ui: &Ui, rect: Rect, resp: &Response, always_framed: bool) {
    let painter = ui.painter();
    if resp.hovered() {
        painter.add(ui_kit::soft_shadow(rect, 12.0, 2));
        painter.rect(rect, cr(12.0), theme::SURFACE, Stroke::new(1.0_f32, theme::LINE_STRONG), StrokeKind::Inside);
    } else if always_framed {
        painter.add(ui_kit::soft_shadow(rect, 12.0, 0));
        painter.rect(rect, cr(12.0), theme::SURFACE, Stroke::new(1.0_f32, theme::LINE), StrokeKind::Inside);
    }
}

fn glyph_tile(ui: &Ui, center: egui::Pos2, size: f32, cld: bool) {
    let bg = if cld { theme::REINFORCING } else { theme::STOCK }.gamma_multiply(0.1);
    ui.painter().rect_filled(Rect::from_center_size(center, vec2(size, size)), cr(size * 0.24), bg);
    ui_kit::kind_glyph(ui.painter(), center, size * 0.8, cld);
}

fn action_tile(ui: &mut Ui, w: f32, cld: bool, title: &str, body: &str) -> Response {
    let text_left = 96.0;
    let body_galley = ui.painter().layout(body.to_string(), theme::regular(15.0), theme::MUTED, w - text_left - 24.0);
    let h = (52.0 + body_galley.size().y + 18.0).max(100.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
    hover_surface(ui, rect, &resp, true);
    glyph_tile(ui, pos2(rect.left() + 48.0, rect.center().y), 56.0, cld);
    let painter = ui.painter();
    painter.text(pos2(rect.left() + text_left, rect.top() + 20.0), egui::Align2::LEFT_TOP, title, theme::bold(18.0), theme::INK);
    painter.galley(pos2(rect.left() + text_left, rect.top() + 48.0), body_galley, theme::MUTED);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

fn resume_banner(app: &mut App, ui: &mut Ui, model: Model) {
    let mut open = false;
    egui::Frame::default()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0_f32, theme::LINE))
        .corner_radius(cr(12.0))
        .inner_margin(Margin::symmetric(18, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().max(0.0));
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(theme::overline("Fortsätt där du var"));
                    ui.label(theme::strong(&model.title, 16.0));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    open = ui_kit::primary(ui, true, "Öppna").clicked();
                });
            });
        });
    if open {
        app.load_model(model);
    }
}

/// Träningarna som en numrerad väg: steg 1 leder till steg 2.
fn lesson_path(app: &mut App, ui: &mut Ui, width: f32) {
    let all = lessons::all();
    let n = all.len();
    let mut chosen = None;
    let mut circles = Vec::new();
    for (i, lesson) in all.iter().enumerate() {
        let left = 64.0;
        let summary = ui.painter().layout(lesson.summary.to_string(), theme::regular(15.0), theme::MUTED, width - left - 140.0);
        let h = 30.0 + 22.0 + summary.size().y + 18.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(width, h), Sense::click());
        hover_surface(ui, rect, &resp, false);
        let painter = ui.painter();
        let c = pos2(rect.left() + 32.0, rect.top() + 32.0);
        circles.push(c);
        painter.circle_filled(c, 17.0, theme::SUCCESS);
        painter.text(c, egui::Align2::CENTER_CENTER, format!("{}", i + 1), theme::bold(16.0), theme::SURFACE);
        let title = lesson.title.trim_start_matches(|ch: char| ch.is_ascii_digit() || ch == '.' || ch == ' ');
        painter.text(pos2(rect.left() + left, rect.top() + 16.0), egui::Align2::LEFT_TOP, title, theme::bold(18.0), theme::INK);
        let kind = if lesson.kind == ModelKind::Cld { "Orsaksdiagram" } else { "Lager och flöden" };
        painter.text(
            pos2(rect.left() + left, rect.top() + 40.0),
            egui::Align2::LEFT_TOP,
            format!("{kind} · {} steg", lesson.steps.len()),
            theme::semibold(13.5),
            theme::SUCCESS,
        );
        painter.galley(pos2(rect.left() + left, rect.top() + 62.0), summary, theme::MUTED);
        let go_color = if resp.hovered() { theme::SUCCESS } else { theme::INK_2 };
        painter.text(rect.right_center() - vec2(20.0, 0.0), egui::Align2::RIGHT_CENTER, "Starta  ›", theme::semibold(15.5), go_color);
        if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            chosen = Some(i);
        }
        if i + 1 < n {
            ui.add_space(4.0);
        }
    }
    // Linje som binder ihop stegen.
    for pair in circles.windows(2) {
        ui.painter().line_segment(
            [pair[0] + vec2(0.0, 20.0), pair[1] - vec2(0.0, 20.0)],
            Stroke::new(2.0_f32, theme::SUCCESS.gamma_multiply(0.35)),
        );
    }
    if let Some(i) = chosen {
        app.load_model((all[i].start)());
        app.lesson = Some(LessonState { lesson: i, step: 0 });
    }
}

/// Exemplen som en lista i två spalter, med symbol, rubrik och kort beskrivning.
fn example_list(app: &mut App, ui: &mut Ui, width: f32, list: Vec<Example>) {
    let gap = 20.0;
    let cols = if width > 640.0 { 2 } else { 1 };
    let col_w = (width - gap * (cols as f32 - 1.0)) / cols as f32;
    let mut chosen: Option<Model> = None;
    let mut items = list.into_iter().peekable();
    while items.peek().is_some() {
        let row: Vec<Example> = items.by_ref().take(cols).collect();
        let summaries: Vec<_> = row
            .iter()
            .map(|ex| ui.painter().layout(ex.summary.to_string(), theme::regular(14.5), theme::MUTED, col_w - 96.0))
            .collect();
        let h = 44.0 + summaries.iter().map(|g| g.size().y).fold(0.0, f32::max) + 16.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (ex, summary) in row.iter().zip(summaries) {
                let (rect, resp) = ui.allocate_exact_size(vec2(col_w, h.max(84.0)), Sense::click());
                hover_surface(ui, rect, &resp, false);
                glyph_tile(ui, pos2(rect.left() + 36.0, rect.top() + 40.0), 44.0, ex.kind == ModelKind::Cld);
                let painter = ui.painter();
                painter.text(pos2(rect.left() + 76.0, rect.top() + 16.0), egui::Align2::LEFT_TOP, ex.title, theme::semibold(16.5), theme::INK);
                painter.galley(pos2(rect.left() + 76.0, rect.top() + 40.0), summary, theme::MUTED);
                if !resp.hovered() {
                    painter.hline(rect.left() + 76.0..=rect.right() - 8.0, rect.bottom() - 0.5, Stroke::new(1.0_f32, theme::LINE));
                }
                if resp.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    chosen = Some((ex.build)());
                }
            }
        });
        ui.add_space(2.0);
    }
    if let Some(m) = chosen {
        app.load_model(m);
    }
}
