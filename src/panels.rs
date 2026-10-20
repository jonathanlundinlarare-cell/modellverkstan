//! Panelerna runt ritytan: verktygslåda, egenskaper, reglage och resultat.

use crate::app::{App, Drag, ResultTab, Selection, Tool};
use crate::canvas;
use crate::model::{ModelKind, NodeKind};
use crate::theme::{self, cr, fmt_num};
use egui::{
    pos2, vec2, Align2, Color32, Context, FontId, Margin, Painter, Pos2, RichText, Sense, Shape, Stroke, StrokeKind,
    TextEdit, Ui,
};
use egui_plot::{Corner, Legend, Line, Plot, PlotPoints, VLine};

// ---------------------------------------------------------------- verktyg

#[derive(Clone, Copy)]
enum Icon {
    Pointer,
    Stock,
    Flow,
    Aux,
    Constant,
    Variable,
    Arrow,
}

pub fn tool_panel(app: &mut App, ctx: &Context) {
    egui::SidePanel::left("tools")
        .resizable(false)
        .exact_width(112.0)
        .frame(egui::Frame::default().fill(theme::PANEL_BG).inner_margin(Margin::same(8)))
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let tools: &[(Tool, &str, Icon)] = match app.model.kind {
                ModelKind::Cld => &[
                    (Tool::Select, "Markera", Icon::Pointer),
                    (Tool::Place(NodeKind::Variable), "Variabel", Icon::Variable),
                    (Tool::Link, "Pil", Icon::Arrow),
                ],
                ModelKind::Sfd => &[
                    (Tool::Select, "Markera", Icon::Pointer),
                    (Tool::Place(NodeKind::Stock), "Lager", Icon::Stock),
                    (Tool::Flow, "Flöde", Icon::Flow),
                    (Tool::Place(NodeKind::Aux), "Hjälpvariabel", Icon::Aux),
                    (Tool::Place(NodeKind::Constant), "Konstant", Icon::Constant),
                ],
            };
            ui.label(RichText::new("Verktyg").small().color(theme::MUTED));
            for &(tool, label, icon) in tools {
                if tool_button(ui, app.tool == tool, label, icon).clicked() {
                    app.tool = tool;
                    app.drag = Drag::None;
                }
            }
            ui.add_space(8.0);
            ui.separator();
            ui.label(RichText::new("Visa").small().color(theme::MUTED));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if ui.add_sized([44.0, 30.0], egui::Button::new("−")).on_hover_text("Zooma ut").clicked() {
                    canvas::zoom_by(app, 1.0 / 1.2, None);
                }
                if ui.add_sized([44.0, 30.0], egui::Button::new("+")).on_hover_text("Zooma in").clicked() {
                    canvas::zoom_by(app, 1.2, None);
                }
            });
            if ui.add_sized([92.0, 30.0], egui::Button::new("Anpassa")).on_hover_text("Visa hela modellen").clicked() {
                canvas::request_fit(app);
            }
            if app.selection != Selection::None {
                ui.add_space(8.0);
                ui.separator();
                let btn = egui::Button::new(RichText::new("🗑 Ta bort").color(theme::ERROR));
                if ui.add_sized([92.0, 30.0], btn).on_hover_text("Delete-tangenten").clicked() {
                    app.delete_selection();
                }
            }
        });
}

fn tool_button(ui: &mut Ui, selected: bool, label: &str, icon: Icon) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(96.0, 64.0), Sense::click());
    let painter = ui.painter();
    let (fill, stroke) = if selected {
        (Color32::from_rgb(214, 230, 250), Stroke::new(1.5_f32, theme::ACCENT))
    } else if resp.hovered() {
        (Color32::from_rgb(230, 229, 222), Stroke::NONE)
    } else {
        (Color32::TRANSPARENT, Stroke::NONE)
    };
    painter.rect(rect, cr(8.0), fill, stroke, StrokeKind::Inside);
    draw_icon(painter, icon, rect.center_top() + vec2(0.0, 23.0));
    painter.text(
        rect.center_bottom() - vec2(0.0, 7.0),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(13.0),
        if selected { theme::ACCENT } else { theme::INK },
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn draw_icon(p: &Painter, icon: Icon, c: Pos2) {
    match icon {
        Icon::Pointer => {
            let pts = [(0.0, 0.0), (0.0, 18.0), (4.5, 13.5), (8.0, 20.0), (11.0, 18.5), (7.5, 12.5), (13.0, 12.0)];
            let pts: Vec<Pos2> = pts.iter().map(|(x, y)| c + vec2(x - 5.0, y - 10.0)).collect();
            p.add(Shape::closed_line(pts, Stroke::new(1.8_f32, theme::INK)));
        }
        Icon::Stock => {
            let r = egui::Rect::from_center_size(c, vec2(34.0, 22.0));
            p.rect(r, cr(4.0), theme::STOCK_FILL, Stroke::new(2.0_f32, theme::STOCK), StrokeKind::Inside);
            let lvl = egui::Rect::from_min_max(pos2(r.left() + 2.0, r.center().y + 2.0), r.right_bottom() - vec2(2.0, 2.0));
            p.rect_filled(lvl, cr(2.0), theme::STOCK_LEVEL);
        }
        Icon::Flow => {
            p.line_segment([c - vec2(18.0, 0.0), c + vec2(12.0, 0.0)], Stroke::new(7.0_f32, theme::FLOW));
            p.line_segment([c - vec2(18.0, 0.0), c + vec2(12.0, 0.0)], Stroke::new(3.0_f32, theme::FLOW_LIGHT));
            p.add(Shape::convex_polygon(
                vec![c + vec2(20.0, 0.0), c + vec2(11.0, -6.0), c + vec2(11.0, 6.0)],
                theme::FLOW,
                Stroke::NONE,
            ));
            p.circle(c - vec2(3.0, 0.0), 6.0, Color32::WHITE, Stroke::new(1.5_f32, theme::FLOW));
        }
        Icon::Aux => {
            let r = egui::Rect::from_center_size(c, vec2(34.0, 18.0));
            p.rect(r, cr(9.0), theme::AUX_FILL, Stroke::new(1.6_f32, theme::AUX), StrokeKind::Inside);
        }
        Icon::Constant => {
            let r = egui::Rect::from_center_size(c, vec2(32.0, 18.0));
            p.rect(r, cr(4.0), theme::CONST_FILL, Stroke::new(1.6_f32, theme::CONST), StrokeKind::Inside);
            p.line_segment([c - vec2(9.0, 0.0), c + vec2(9.0, 0.0)], Stroke::new(1.5_f32, theme::CONST));
            p.circle_filled(c + vec2(3.0, 0.0), 3.0, theme::CONST);
        }
        Icon::Variable => {
            let r = egui::Rect::from_center_size(c, vec2(36.0, 20.0));
            p.rect(r, cr(5.0), Color32::WHITE, Stroke::new(1.0_f32, theme::MUTED), StrokeKind::Inside);
            p.text(c, Align2::CENTER_CENTER, "Abc", FontId::proportional(12.0), theme::INK);
        }
        Icon::Arrow => {
            let pts: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let t = i as f32 / 12.0;
                    c + vec2(-16.0 + 30.0 * t, 6.0 - 16.0 * t * (1.0 - t) * 2.0)
                })
                .collect();
            p.add(Shape::line(pts, Stroke::new(2.0_f32, theme::INK)));
            p.add(Shape::convex_polygon(
                vec![c + vec2(18.0, 6.0), c + vec2(9.0, 1.0), c + vec2(10.0, 10.0)],
                theme::INK,
                Stroke::NONE,
            ));
            p.text(c + vec2(10.0, -10.0), Align2::CENTER_CENTER, "+", FontId::proportional(15.0), theme::POSITIVE);
        }
    }
}

// ---------------------------------------------------------------- sidopanel

fn section<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::default()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(226, 224, 216)))
        .corner_radius(cr(10.0))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().max(0.0));
            add(ui)
        })
        .inner
}

fn heading(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).size(17.0).strong());
}

pub fn side_panel(app: &mut App, ctx: &Context) {
    egui::SidePanel::right("props")
        .resizable(true)
        .default_width(350.0)
        .min_width(280.0)
        .frame(egui::Frame::default().fill(theme::PANEL_BG).inner_margin(Margin::same(10)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                about_section(app, ui);
                errors_section(app, ui);
                section(ui, |ui| selection_section(app, ui));
                match app.model.kind {
                    ModelKind::Sfd => {
                        sliders_section(app, ui);
                        settings_section(app, ui);
                    }
                    ModelKind::Cld => loops_section(app, ui),
                }
            });
        });
}

fn about_section(app: &mut App, ui: &mut Ui) {
    let edit_id = ui.id().with("edit_about");
    let mut editing = ui.data(|d| d.get_temp::<bool>(edit_id).unwrap_or(false));
    section(ui, |ui| {
        ui.horizontal(|ui| {
            heading(ui, "📘 Om modellen");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let label = if editing { "✔ Klar" } else { "✏ Ändra" };
                if ui.small_button(label).clicked() {
                    editing = !editing;
                }
            });
        });
        if editing {
            ui.label(RichText::new("Titel").small().color(theme::MUTED));
            ui.add(TextEdit::singleline(&mut app.model.title).desired_width(f32::INFINITY));
            ui.label(RichText::new("Beskrivning").small().color(theme::MUTED));
            ui.add(
                TextEdit::multiline(&mut app.model.description)
                    .hint_text("Vad visar modellen?")
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
            ui.label(RichText::new("Uppgifter (en per rad)").small().color(theme::MUTED));
            let mut tasks = app.model.tasks.join("\n");
            if ui
                .add(TextEdit::multiline(&mut tasks).hint_text("Prova att …").desired_rows(3).desired_width(f32::INFINITY))
                .changed()
            {
                app.model.tasks = tasks.lines().map(str::to_string).collect();
            }
        } else {
            ui.label(RichText::new(&app.model.title).size(16.0).strong().color(theme::ACCENT));
            if app.model.description.trim().is_empty() {
                ui.label(RichText::new("Ingen beskrivning än. Klicka på Ändra för att skriva en.").color(theme::MUTED));
            } else {
                ui.label(&app.model.description);
            }
            let tasks: Vec<&String> = app.model.tasks.iter().filter(|t| !t.trim().is_empty()).collect();
            if !tasks.is_empty() {
                ui.add_space(4.0);
                ui.label(RichText::new("Prova detta!").strong().color(theme::CONST));
                for (i, t) in tasks.iter().enumerate() {
                    ui.label(format!("{}. {}", i + 1, t));
                }
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(edit_id, editing));
}

fn errors_section(app: &mut App, ui: &mut Ui) {
    if app.compiled.errors.is_empty() {
        return;
    }
    let errors: Vec<(u32, String, String)> = app
        .compiled
        .errors
        .iter()
        .map(|(id, e)| (*id, app.model.node(*id).map_or(String::new(), |n| n.name.clone()), e.clone()))
        .collect();
    egui::Frame::default()
        .fill(Color32::from_rgb(253, 236, 236))
        .stroke(Stroke::new(1.0_f32, theme::ERROR))
        .corner_radius(cr(10.0))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().max(0.0));
            let n = errors.len();
            ui.label(
                RichText::new(if n == 1 { "⚠ Ett fel i modellen".to_string() } else { format!("⚠ {n} fel i modellen") })
                    .strong()
                    .color(theme::ERROR),
            );
            for (id, name, err) in errors.iter().take(6) {
                let text = format!("{}: {}", if name.trim().is_empty() { "(namnlös)" } else { name }, err);
                if ui.link(RichText::new(text).color(theme::INK)).on_hover_text("Klicka för att markera").clicked() {
                    app.selection = Selection::Node(*id);
                }
            }
        });
}

fn kind_explanation(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Stock => "Ett lager är något som samlas och kan mätas vid en viss tidpunkt, t.ex. liter vatten eller kWh i ett batteri.",
        NodeKind::Flow => "Ett flöde fyller på eller tömmer ett lager. Det anger hur mycket som flödar per tidsenhet.",
        NodeKind::Aux => "En hjälpvariabel räknar ut något med en formel, så att andra formler blir enklare.",
        NodeKind::Constant => "En konstant är ett värde som inte ändras under simuleringen. Du ändrar det med ett reglage.",
        NodeKind::Variable => "En variabel är något som kan öka eller minska.",
    }
}

fn selection_section(app: &mut App, ui: &mut Ui) {
    match app.selection {
        Selection::Node(id) if app.model.node(id).is_some() => node_props(app, ui, id),
        Selection::Link(i) if i < app.model.links.len() => link_props(app, ui, i),
        _ => {
            heading(ui, "Markerat");
            ui.label(RichText::new("Klicka på något i modellen för att se och ändra det.").color(theme::MUTED));
        }
    }
}

fn append_to_formula(formula: &mut String, piece: &str) {
    let trimmed = formula.trim();
    if trimmed.is_empty() || trimmed == "0" || trimmed == "1" {
        *formula = piece.to_string();
    } else {
        formula.push(' ');
        formula.push_str(piece);
    }
}

fn node_props(app: &mut App, ui: &mut Ui, id: u32) {
    let chips: Vec<(String, NodeKind)> = app
        .model
        .nodes
        .iter()
        .filter(|n| n.id != id && n.kind != NodeKind::Variable && !n.name.trim().is_empty())
        .map(|n| (n.name.trim().to_string(), n.kind))
        .collect();
    let stocks: Vec<(u32, String)> =
        app.model.nodes.iter().filter(|n| n.kind == NodeKind::Stock).map(|n| (n.id, n.name.clone())).collect();
    let error = app.compiled.errors.get(&id).cloned();
    let time_unit = app.model.time_unit.clone();
    let focus = std::mem::take(&mut app.focus_name);
    let mut delete = false;

    let Some(node) = app.model.node_mut(id) else { return };
    let kind = node.kind;
    ui.horizontal(|ui| {
        ui.label(RichText::new(kind.label()).strong().color(theme::kind_color(kind)));
    });
    ui.label(RichText::new(kind_explanation(kind)).small().color(theme::MUTED));
    ui.add_space(4.0);

    ui.label(RichText::new("Namn").small().color(theme::MUTED));
    let out = TextEdit::singleline(&mut node.name).desired_width(f32::INFINITY).font(FontId::proportional(18.0)).show(ui);
    if focus {
        out.response.request_focus();
        let mut state = out.state.clone();
        let len = node.name.chars().count();
        state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
            egui::text::CCursor::new(0),
            egui::text::CCursor::new(len),
        )));
        state.store(ui.ctx(), out.response.id);
    }
    if out.response.gained_focus() || (focus && app.rename_origin.is_none()) {
        app.rename_origin = Some((id, node.name.clone()));
    }

    match kind {
        NodeKind::Stock | NodeKind::Flow | NodeKind::Aux => {
            let label = match kind {
                NodeKind::Stock => "Startvärde – hur mycket som finns från början".to_string(),
                NodeKind::Flow => format!("Formel – hur mycket som flödar per {}", singular(&time_unit)),
                _ => "Formel".to_string(),
            };
            ui.label(RichText::new(label).small().color(theme::MUTED));
            ui.add(
                TextEdit::multiline(&mut node.formula)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .font(FontId::monospace(15.0)),
            );
            match &error {
                Some(e) => {
                    ui.label(RichText::new(format!("⚠ {e}")).color(theme::ERROR));
                }
                None => {
                    ui.label(RichText::new("✔ Formeln fungerar").small().color(theme::POSITIVE));
                }
            }
            let usable: Vec<&(String, NodeKind)> =
                chips.iter().filter(|(_, k)| kind != NodeKind::Stock || *k == NodeKind::Constant).collect();
            if !usable.is_empty() {
                ui.label(RichText::new("Klicka för att lägga till i formeln:").small().color(theme::MUTED));
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                    for (name, k) in usable {
                        let b = egui::Button::new(RichText::new(name).color(theme::kind_color(*k)).size(14.0));
                        if ui.add(b).clicked() {
                            append_to_formula(&mut node.formula, name);
                        }
                    }
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                for piece in ["+", "-", "*", "/", "(", ")"] {
                    if ui.add_sized([28.0, 26.0], egui::Button::new(piece)).clicked() {
                        append_to_formula(&mut node.formula, piece);
                    }
                }
                if kind != NodeKind::Stock {
                    if ui.button("om(…)").on_hover_text("om(villkor; värde om sant; värde om falskt)").clicked() {
                        append_to_formula(&mut node.formula, "om( ; ; )");
                    }
                    if ui.button("min").on_hover_text("Det minsta av värdena: min(A; B)").clicked() {
                        append_to_formula(&mut node.formula, "min( ; )");
                    }
                    if ui.button("max").on_hover_text("Det största av värdena: max(A; B)").clicked() {
                        append_to_formula(&mut node.formula, "max( ; )");
                    }
                }
            });
        }
        NodeKind::Constant => {
            ui.label(RichText::new("Värde").small().color(theme::MUTED));
            let (lo, hi) = range(node.min, node.max);
            ui.spacing_mut().slider_width = (ui.available_width() - 80.0).max(80.0);
            ui.add(number_slider(&mut node.value, lo, hi));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Reglaget går från").small());
                ui.add(egui::DragValue::new(&mut node.min).speed(0.1));
                ui.label(RichText::new("till").small());
                ui.add(egui::DragValue::new(&mut node.max).speed(0.1));
            });
        }
        NodeKind::Variable => {}
    }

    if kind == NodeKind::Flow {
        ui.horizontal(|ui| {
            ui.label("Från");
            stock_combo(ui, "flow_from", &mut node.from, &stocks);
        });
        ui.horizontal(|ui| {
            ui.label("Till ");
            stock_combo(ui, "flow_to", &mut node.to, &stocks);
        });
    }

    if kind != NodeKind::Variable {
        ui.horizontal(|ui| {
            ui.label("Enhet");
            ui.add(TextEdit::singleline(&mut node.unit).hint_text("t.ex. liter, kWh, °C").desired_width(f32::INFINITY));
        });
        ui.checkbox(&mut node.in_graph, "Visa i grafen");
    }
    ui.label(RichText::new("Anteckning").small().color(theme::MUTED));
    ui.add(
        TextEdit::multiline(&mut node.note)
            .hint_text("Förklara vad det här är…")
            .desired_rows(2)
            .desired_width(f32::INFINITY),
    );
    ui.add_space(4.0);
    if ui.button(RichText::new("🗑 Ta bort").color(theme::ERROR)).clicked() {
        delete = true;
    }
    if delete {
        app.delete_selection();
    }
}

fn singular(unit: &str) -> String {
    match unit.trim() {
        "minuter" => "minut".into(),
        "timmar" => "timme".into(),
        "sekunder" => "sekund".into(),
        "dagar" => "dag".into(),
        "veckor" => "vecka".into(),
        "månader" => "månad".into(),
        "år" => "år".into(),
        other => other.into(),
    }
}

fn range(min: f64, max: f64) -> (f64, f64) {
    if max > min {
        (min, max)
    } else {
        (min, min + 1.0)
    }
}

fn number_slider(value: &mut f64, lo: f64, hi: f64) -> egui::Slider<'_> {
    egui::Slider::new(value, lo..=hi)
        .clamping(egui::SliderClamping::Never)
        .custom_formatter(|v, _| fmt_num(v))
        .custom_parser(|s| s.trim().replace(',', ".").replace('−', "-").parse().ok())
}

fn stock_combo(ui: &mut Ui, id: &str, value: &mut Option<u32>, stocks: &[(u32, String)]) {
    let cloud = "Moln (utanför modellen)";
    let current = value
        .and_then(|v| stocks.iter().find(|(id, _)| *id == v))
        .map_or(cloud.to_string(), |(_, n)| n.clone());
    egui::ComboBox::from_id_salt(id).selected_text(current).width(ui.available_width()).show_ui(ui, |ui| {
        ui.selectable_value(value, None, cloud);
        for (sid, name) in stocks {
            ui.selectable_value(value, Some(*sid), name);
        }
    });
}

fn link_props(app: &mut App, ui: &mut Ui, i: usize) {
    let name = |id: u32| app.model.node(id).map_or("?".to_string(), |n| n.name.trim().to_string());
    let (a, b) = (name(app.model.links[i].from), name(app.model.links[i].to));
    let mut delete = false;
    let link = &mut app.model.links[i];
    ui.label(RichText::new("Pil").strong());
    ui.label(format!("Från \"{a}\" till \"{b}\""));
    ui.add_space(4.0);
    ui.radio_value(&mut link.positive, true, RichText::new("+  Samma riktning").strong().color(theme::POSITIVE));
    ui.label(RichText::new(format!("Ökar {a} så ökar {b}. Minskar {a} så minskar {b}.")).small().color(theme::MUTED));
    ui.radio_value(&mut link.positive, false, RichText::new("−  Motsatt riktning").strong().color(theme::NEGATIVE));
    ui.label(RichText::new(format!("Ökar {a} så minskar {b}. Minskar {a} så ökar {b}.")).small().color(theme::MUTED));
    ui.add_space(4.0);
    ui.add(egui::Slider::new(&mut link.curve, -1.0..=1.0).text("Böjning").show_value(false));
    ui.label(RichText::new("Tips: du kan också dra i den lilla ringen mitt på pilen.").small().color(theme::MUTED));
    ui.horizontal(|ui| {
        if ui.button("Vänd pilen").clicked() {
            std::mem::swap(&mut link.from, &mut link.to);
        }
        if ui.button(RichText::new("🗑 Ta bort").color(theme::ERROR)).clicked() {
            delete = true;
        }
    });
    if delete {
        app.delete_selection();
    }
}

fn sliders_section(app: &mut App, ui: &mut Ui) {
    section(ui, |ui| {
        heading(ui, "Reglage");
        let mut any = false;
        let width = (ui.available_width() - 80.0).max(80.0);
        let mut select = None;
        for n in app.model.nodes.iter_mut().filter(|n| n.kind == NodeKind::Constant) {
            any = true;
            let label = if n.unit.trim().is_empty() { n.name.clone() } else { format!("{} ({})", n.name.trim(), n.unit.trim()) };
            if ui.add(egui::Label::new(RichText::new(label).strong()).sense(Sense::click())).clicked() {
                select = Some(n.id);
            }
            let (lo, hi) = range(n.min, n.max);
            ui.spacing_mut().slider_width = width;
            ui.add(number_slider(&mut n.value, lo, hi));
        }
        if let Some(id) = select {
            app.selection = Selection::Node(id);
        }
        if !any {
            ui.label(RichText::new("Lägg till konstanter så får du reglage här som du kan dra i medan modellen körs.").color(theme::MUTED));
        }
    });
}

fn settings_section(app: &mut App, ui: &mut Ui) {
    section(ui, |ui| {
        egui::CollapsingHeader::new(RichText::new("⏱ Tid och simulering").size(17.0).strong())
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("sim_settings").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
                    ui.label("Tidsenhet");
                    ui.add(TextEdit::singleline(&mut app.model.time_unit).desired_width(140.0));
                    ui.end_row();
                    ui.label("Simulera i");
                    ui.add(
                        egui::DragValue::new(&mut app.model.t_end)
                            .range(0.1..=100_000.0)
                            .speed(0.5)
                            .suffix(format!(" {}", app.model.time_unit)),
                    );
                    ui.end_row();
                    ui.label("Tidssteg (dt)");
                    ui.add(egui::DragValue::new(&mut app.model.dt).range(0.0001..=1000.0).speed(0.005).max_decimals(4));
                    ui.end_row();
                });
                ui.label(
                    RichText::new(
                        "Datorn räknar fram modellen ett litet tidssteg i taget. Mindre tidssteg ger noggrannare svar men tar längre tid.",
                    )
                    .small()
                    .color(theme::MUTED),
                );
            });
    });
}

fn loops_section(app: &mut App, ui: &mut Ui) {
    app.highlight_loop = None;
    let labels = canvas::loop_labels(&app.loops);
    let name = |id: &u32| app.model.node(*id).map_or("?".to_string(), |n| n.name.trim().to_string());
    let mut hovered = None;
    section(ui, |ui| {
        heading(ui, "🔁 Återkopplingsloopar");
        if app.loops.is_empty() {
            ui.label(
                RichText::new("Inga loopar än. En loop uppstår när pilarna leder runt och tillbaka till samma variabel.")
                    .color(theme::MUTED),
            );
        }
        for (i, lp) in app.loops.iter().enumerate() {
            let color = if lp.reinforcing { theme::REINFORCING } else { theme::BALANCING };
            let kind = if lp.reinforcing { "förstärkande" } else { "balanserande" };
            let mut chain: Vec<String> = lp.nodes.iter().map(name).collect();
            chain.push(name(&lp.nodes[0]));
            let resp = egui::Frame::default()
                .fill(color.gamma_multiply(0.08))
                .corner_radius(cr(8.0))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width().max(0.0));
                    ui.label(RichText::new(format!("{} – {kind}", labels[i])).strong().color(color));
                    ui.label(RichText::new(chain.join(" ➡ ")).small());
                })
                .response;
            if ui.rect_contains_pointer(resp.rect) {
                hovered = Some(i);
            }
        }
        ui.add_space(4.0);
        ui.label(RichText::new("R = förstärkande loop").strong().color(theme::REINFORCING));
        ui.label(
            RichText::new("En förändring växer sig större och större (som ränta på ränta). Antalet minus-pilar är jämnt.")
                .small(),
        );
        ui.label(RichText::new("B = balanserande loop").strong().color(theme::BALANCING));
        ui.label(
            RichText::new("Systemet strävar mot ett jämviktsläge (som en termostat). Antalet minus-pilar är udda.").small(),
        );
    });
    app.highlight_loop = hovered;
}

// ---------------------------------------------------------------- resultat

pub fn results_panel(app: &mut App, ctx: &Context) {
    egui::TopBottomPanel::bottom("results")
        .resizable(true)
        .default_height(300.0)
        .min_height(170.0)
        .frame(egui::Frame::default().fill(Color32::WHITE).inner_margin(Margin::same(10)))
        .show(ctx, |ui| {
            let end = app.result.times.last().copied().unwrap_or(0.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.result_tab, ResultTab::Graph, RichText::new("📈 Graf").size(16.0));
                ui.selectable_value(&mut app.result_tab, ResultTab::Table, RichText::new("🔢 Tabell").size(16.0));
                ui.separator();
                let play = if app.playing { "⏸ Pausa" } else { "▶ Spela upp" };
                if ui.add_enabled(end > 0.0, egui::Button::new(play)).clicked() {
                    if app.playing {
                        app.playing = false;
                    } else {
                        if app.scrub.is_none_or(|t| t >= end) {
                            app.scrub = Some(0.0);
                        }
                        app.playing = true;
                    }
                }
                let mut t = app.shown_time();
                ui.label("Tid:");
                ui.spacing_mut().slider_width = (ui.available_width() - 160.0).clamp(120.0, 500.0);
                let slider = egui::Slider::new(&mut t, 0.0..=end.max(0.001))
                    .custom_formatter(|v, _| fmt_num(v))
                    .text(app.model.time_unit.clone());
                if ui.add_enabled(end > 0.0, slider).changed() {
                    app.scrub = Some(t);
                    app.playing = false;
                }
                if app.scrub.is_some() && ui.button("⏭ Till slutet").clicked() {
                    app.scrub = None;
                    app.playing = false;
                }
            });
            if let Some(e) = &app.result.error {
                ui.label(RichText::new(format!("⚠ {e}")).color(theme::ERROR));
            }
            match app.result_tab {
                ResultTab::Graph => graph(app, ui),
                ResultTab::Table => table(app, ui),
            }
        });
}

fn series_color(i: usize) -> Color32 {
    theme::SERIES[i % theme::SERIES.len()]
}

fn graph(app: &mut App, ui: &mut Ui) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Visa:").color(theme::MUTED));
        for (i, n) in app.model.nodes.iter_mut().enumerate() {
            if n.kind == NodeKind::Variable {
                continue;
            }
            ui.checkbox(&mut n.in_graph, RichText::new(n.name.trim()).color(series_color(i)));
        }
    });
    let shown: Vec<(usize, &crate::model::Node)> =
        app.model.nodes.iter().enumerate().filter(|(_, n)| n.in_graph && n.kind != NodeKind::Variable).collect();
    if shown.is_empty() {
        ui.label(RichText::new("Kryssa i något ovanför för att se det i grafen.").color(theme::MUTED));
        return;
    }
    let unit = app.model.time_unit.clone();
    let t_now = app.shown_time();
    let show_line = app.scrub.is_some() || app.playing;
    let clicked = Plot::new("graf")
        .legend(Legend::default().position(Corner::LeftTop))
        .x_axis_label(format!("Tid ({unit})"))
        .x_axis_formatter(|m, _| fmt_num(m.value))
        .y_axis_formatter(|m, _| fmt_num(m.value))
        .label_formatter(move |name, v| {
            let name = if name.is_empty() { String::new() } else { format!("{name}\n") };
            format!("{name}tid {} {unit}: {}", fmt_num(v.x), fmt_num(v.y))
        })
        .allow_scroll(false)
        .show(ui, |pu| {
            for (i, n) in &shown {
                let Some(s) = app.result.series.get(&n.id) else { continue };
                let pts: Vec<[f64; 2]> = app.result.times.iter().zip(s).map(|(t, v)| [*t, *v]).collect();
                let name = if n.unit.trim().is_empty() { n.name.trim().to_string() } else { format!("{} ({})", n.name.trim(), n.unit.trim()) };
                pu.line(Line::new(PlotPoints::new(pts)).name(name).color(series_color(*i)).width(2.5_f32));
            }
            if show_line {
                pu.vline(VLine::new(t_now).color(theme::MUTED).width(1.5_f32));
            }
            if pu.response().clicked() {
                pu.pointer_coordinate().map(|p| p.x)
            } else {
                None
            }
        })
        .inner;
    if let Some(t) = clicked {
        let end = app.result.times.last().copied().unwrap_or(0.0);
        app.scrub = Some(t.clamp(0.0, end));
        app.playing = false;
    }
}

fn table(app: &mut App, ui: &mut Ui) {
    let cols: Vec<&crate::model::Node> = app
        .model
        .nodes
        .iter()
        .filter(|n| matches!(n.kind, NodeKind::Stock | NodeKind::Flow | NodeKind::Aux))
        .collect();
    let times = &app.result.times;
    if times.is_empty() || cols.is_empty() {
        ui.label(RichText::new("Ingen data än – bygg en modell utan fel så räknas den ut direkt.").color(theme::MUTED));
        return;
    }
    let header = |n: &crate::model::Node| {
        if n.unit.trim().is_empty() {
            n.name.trim().to_string()
        } else {
            format!("{} ({})", n.name.trim(), n.unit.trim())
        }
    };
    if ui.button("📋 Kopiera tabellen").on_hover_text("Klistra sedan in i t.ex. Excel eller Google Kalkylark").clicked() {
        let mut tsv = format!("Tid ({})", app.model.time_unit);
        for n in &cols {
            tsv.push('\t');
            tsv.push_str(&header(n));
        }
        tsv.push('\n');
        for (row, t) in times.iter().enumerate() {
            tsv.push_str(&format!("{t}").replace('.', ","));
            for n in &cols {
                let v = app.result.series.get(&n.id).and_then(|s| s.get(row)).copied().unwrap_or(f64::NAN);
                tsv.push('\t');
                tsv.push_str(&format!("{:.6}", v).trim_end_matches('0').trim_end_matches('.').replace('.', ","));
            }
            tsv.push('\n');
        }
        ui.ctx().copy_text(tsv);
    }
    let step = (times.len() / 200).max(1);
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui::Grid::new("tabell").striped(true).min_col_width(80.0).spacing([16.0, 4.0]).show(ui, |ui| {
            ui.label(RichText::new(format!("Tid ({})", app.model.time_unit)).strong());
            for n in &cols {
                ui.label(RichText::new(header(n)).strong());
            }
            ui.end_row();
            let mut row = 0;
            while row < times.len() {
                ui.label(fmt_num(times[row]));
                for n in &cols {
                    let v = app.result.series.get(&n.id).and_then(|s| s.get(row)).copied();
                    ui.label(v.map_or("–".into(), fmt_num));
                }
                ui.end_row();
                if row == times.len() - 1 {
                    break;
                }
                row = (row + step).min(times.len() - 1);
            }
        });
    });
}
