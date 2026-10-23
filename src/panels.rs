//! Panelerna runt ritytan: verktyg, inspektör, träning, reglage och resultat.

use crate::app::{App, Drag, ResultTab, SavedRun, Selection, Tool};
use crate::canvas;
use crate::export::{self, ExportAction};
use crate::lessons;
use crate::model::{ModelKind, NodeKind};
use crate::theme::{self, cr, fmt_num};
use crate::ui_kit;
use egui::{
    pos2, vec2, Align2, Color32, Context, CursorIcon, FontId, Margin, Painter, Pos2, RichText, Sense, Shape, Stroke,
    StrokeKind, TextEdit, Ui,
};
use egui_plot::{Corner, Legend, Line, LineStyle, Plot, PlotPoints, VLine};

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
        .exact_width(96.0)
        .frame(
            egui::Frame::default()
                .fill(theme::SURFACE_2)
                .inner_margin(Margin::symmetric(7, 14))
                .stroke(Stroke::new(1.0_f32, theme::LINE)),
        )
        .show(ctx, |ui| {
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
                    (Tool::Place(NodeKind::Constant), "Konstant", Icon::Constant),
                    (Tool::Place(NodeKind::Aux), "Hjälpvariabel", Icon::Aux),
                ],
            };
            let hint = lesson_tool_hint(app);
            ui.vertical_centered(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                for (i, &(tool, label, icon)) in tools.iter().enumerate() {
                    if i == 1 {
                        // Markera för sig, byggverktygen för sig.
                        let (rect, _) = ui.allocate_exact_size(vec2(56.0, 9.0), Sense::hover());
                        ui.painter().hline(rect.x_range(), rect.center().y, Stroke::new(1.0_f32, theme::LINE_STRONG));
                    }
                    let tip = tool_tip(tool);
                    if tool_button(ui, app.tool == tool, hint == Some(tool), label, icon).on_hover_text(tip).clicked() {
                        app.tool = tool;
                        app.drag = Drag::None;
                    }
                }
            });
            if app.selection != Selection::None {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    if ui_kit::ghost_colored(ui, true, "🗑  Ta bort", theme::ERROR).on_hover_text("Ta bort det markerade (Delete)").clicked()
                    {
                        app.delete_selection();
                    }
                });
            }
        });
}

fn tool_tip(tool: Tool) -> &'static str {
    match tool {
        Tool::Select => "Markera och flytta saker. Dra på tom yta för att flytta hela ytan.",
        Tool::Place(NodeKind::Stock) => "Lager: något som samlas, t.ex. vatten, pengar eller energi.",
        Tool::Place(NodeKind::Constant) => "Konstant: ett värde du ändrar med ett reglage.",
        Tool::Place(NodeKind::Aux) => "Hjälpvariabel: räknar ut något med en formel.",
        Tool::Place(NodeKind::Variable) => "Variabel: något som kan öka eller minska.",
        Tool::Place(NodeKind::Flow) | Tool::Flow => "Flöde: fyller på eller tömmer ett lager. Dra från ett lager till ett annat.",
        Tool::Link => "Pil: dra från en variabel till en annan.",
    }
}

fn tool_button(ui: &mut Ui, selected: bool, hinted: bool, label: &str, icon: Icon) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(82.0, 64.0), Sense::click());
    let painter = ui.painter();
    if selected {
        painter.add(ui_kit::soft_shadow(rect, 10.0, 1));
        painter.rect(rect, cr(10.0), theme::SURFACE, Stroke::new(1.0_f32, theme::LINE_STRONG), StrokeKind::Inside);
    } else if resp.hovered() {
        painter.rect_filled(rect, cr(10.0), theme::HOVER);
    }
    if hinted && !selected {
        // Träningen pekar ut verktyget som behövs för uppgiften.
        painter.rect_stroke(rect, cr(10.0), Stroke::new(2.5_f32, theme::SELECT), StrokeKind::Inside);
    }
    draw_icon(painter, icon, rect.center_top() + vec2(0.0, 24.0));
    painter.text(
        rect.center_bottom() - vec2(0.0, 8.0),
        Align2::CENTER_BOTTOM,
        label,
        theme::semibold(12.5),
        if selected { theme::INK } else { theme::INK_2 },
    );
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

// ---------------------------------------------------------------- sidopanel

/// Liten etikett ovanför ett fält.
fn field_label(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).font(theme::semibold(13.5)).color(theme::INK_2));
}

pub fn side_panel(app: &mut App, ctx: &Context) {
    egui::SidePanel::right("props")
        .resizable(true)
        .default_width(340.0)
        .min_width(290.0)
        .frame(
            egui::Frame::default()
                .fill(theme::SURFACE)
                .inner_margin(Margin { left: 20, right: 16, top: 18, bottom: 12 })
                .stroke(Stroke::new(1.0_f32, theme::LINE)),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // Lämna plats för rullningslisten, som annars lägger sig över innehållet.
                ui.set_width((ui.available_width() - 10.0).max(0.0));
                let mut sections = 0;
                if app.lesson.is_some() {
                    lesson_section(app, ui);
                    sections += 1;
                }
                let selected = match app.selection {
                    Selection::Node(id) => app.model.node(id).is_some(),
                    Selection::Link(i) => i < app.model.links.len(),
                    Selection::None => false,
                };
                if selected {
                    if sections > 0 {
                        ui_kit::divider(ui);
                    }
                    match app.selection {
                        Selection::Node(id) => node_props(app, ui, id),
                        Selection::Link(i) => link_props(app, ui, i),
                        Selection::None => {}
                    }
                    sections += 1;
                } else if app.lesson.is_none() {
                    about_section(app, ui);
                    sections += 1;
                }
                if app.model.kind == ModelKind::Cld {
                    if sections > 0 {
                        ui_kit::divider(ui);
                    }
                    loops_section(app, ui);
                } else {
                    app.highlight_loop = None;
                }
            });
        });
}

/// Verktyget som träningens pågående uppgift behöver, om uppgiften inte är klar än.
fn lesson_tool_hint(app: &App) -> Option<Tool> {
    let state = app.lesson?;
    let lessons = lessons::all();
    let step = lessons.get(state.lesson)?.steps.get(state.step)?;
    if (step.check)(&app.model, &app.compiled) {
        None
    } else {
        step.tool
    }
}

fn lesson_section(app: &mut App, ui: &mut Ui) {
    let Some(state) = app.lesson else { return };
    let all = lessons::all();
    let Some(step) = all.get(state.lesson).and_then(|l| l.steps.get(state.step)) else {
        app.lesson = None;
        return;
    };
    let lesson = &all[state.lesson];
    let count = lesson.steps.len();
    let done = (step.check)(&app.model, &app.compiled);
    let last = state.step + 1 == count;
    let mut go_to = None;
    let mut finish = false;
    let mut next_lesson = false;
    let mut solve = false;

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("TRÄNING · STEG {} AV {count}", state.step + 1)).font(theme::bold(12.5)).color(theme::SUCCESS),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui_kit::ghost_colored(ui, true, "Avsluta", theme::MUTED).on_hover_text("Avsluta träningen men behåll modellen").clicked() {
                finish = true;
            }
        });
    });
    // Framstegsmätare: ett segment per steg.
    let (bar, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
    let seg_gap = 3.0;
    let seg_w = (bar.width() - seg_gap * (count as f32 - 1.0)) / count as f32;
    for k in 0..count {
        let x = bar.left() + k as f32 * (seg_w + seg_gap);
        let r = egui::Rect::from_min_size(pos2(x, bar.top()), vec2(seg_w, bar.height()));
        let color = if k <= state.step { theme::SUCCESS } else { theme::LINE };
        ui.painter().rect_filled(r, cr(3.0), color);
    }
    ui.add_space(10.0);
    ui.label(theme::title(step.title, 21.0));
    ui.label(RichText::new(lesson.title).font(theme::regular(13.5)).color(theme::MUTED));
    ui.add_space(6.0);
    ui.label(RichText::new(step.text).font(theme::regular(15.5)).color(theme::INK));

    if let Some(task) = step.task {
        ui.add_space(10.0);
        let fill = if done { theme::SUCCESS_SOFT } else { theme::SELECT_SOFT };
        egui::Frame::default().fill(fill).corner_radius(cr(10.0)).inner_margin(Margin::same(14)).show(ui, |ui| {
            ui.set_width(ui.available_width().max(0.0));
            if done {
                ui.label(RichText::new("✔  Klart!").font(theme::bold(16.0)).color(theme::SUCCESS));
                ui.label(RichText::new(step.done_text).font(theme::regular(15.0)).color(theme::INK));
            } else {
                ui.label(RichText::new("DIN UPPGIFT").font(theme::bold(12.5)).color(Color32::from_rgb(150, 92, 0)));
                ui.label(RichText::new(task).font(theme::semibold(15.5)).color(theme::INK));
                ui.add_space(4.0);
                if ui_kit::ghost(ui, true, "💡  Visa hur").on_hover_text("Programmet gör steget åt dig").clicked() {
                    solve = true;
                }
            }
        });
    }

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        if state.step > 0 && ui_kit::ghost(ui, true, "‹  Tillbaka").clicked() {
            go_to = Some(state.step - 1);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if last {
                if ui_kit::ghost(ui, true, "Klar").on_hover_text("Avsluta träningen och fortsätt med modellen").clicked() {
                    finish = true;
                }
                if state.lesson + 1 < all.len() && ui_kit::primary(ui, true, "Nästa träning  ›").clicked() {
                    next_lesson = true;
                }
            } else {
                let resp = ui_kit::primary(ui, done, "Nästa  ›");
                if !done {
                    resp.on_disabled_hover_text("Gör uppgiften först, eller tryck på Visa hur.");
                } else if resp.clicked() {
                    go_to = Some(state.step + 1);
                }
            }
        });
    });

    if solve {
        (step.solve)(&mut app.model);
        app.selection = Selection::None;
        canvas::request_fit(app);
    }
    if let Some(step) = go_to {
        app.lesson = Some(lessons::LessonState { step, ..state });
    }
    if next_lesson {
        let index = state.lesson + 1;
        app.load_model((all[index].start)());
        app.lesson = Some(lessons::LessonState { lesson: index, step: 0 });
    } else if finish {
        app.lesson = None;
        app.notify(ui.ctx(), "Bra jobbat! Modellen finns kvar så att du kan fortsätta experimentera.");
    }
}

/// En numrerad rad, t.ex. i listan "Prova detta".
fn numbered(ui: &mut Ui, n: usize, text: &str, color: Color32) {
    ui.horizontal_top(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 11.0, color.gamma_multiply(0.15));
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, n.to_string(), theme::bold(12.5), color);
        ui.vertical(|ui| {
            ui.label(RichText::new(text).font(theme::regular(15.0)).color(theme::INK));
        });
    });
}

fn about_section(app: &mut App, ui: &mut Ui) {
    let edit_id = ui.id().with("edit_about");
    let mut editing = ui.data(|d| d.get_temp::<bool>(edit_id).unwrap_or(false));
    ui.horizontal(|ui| {
        ui_kit::section_title(ui, "Om modellen");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let label = if editing { "✔  Klar" } else { "✏  Ändra" };
            if ui_kit::ghost_colored(ui, true, label, theme::INK_2).clicked() {
                editing = !editing;
            }
        });
    });
    if editing {
        field_label(ui, "Titel");
        ui.add(TextEdit::singleline(&mut app.model.title).desired_width(f32::INFINITY));
        field_label(ui, "Beskrivning");
        ui.add(TextEdit::multiline(&mut app.model.description).hint_text("Vad visar modellen?").desired_rows(4).desired_width(f32::INFINITY));
        field_label(ui, "Uppgifter (en per rad)");
        let mut tasks = app.model.tasks.join("\n");
        if ui.add(TextEdit::multiline(&mut tasks).hint_text("Prova att …").desired_rows(3).desired_width(f32::INFINITY)).changed() {
            app.model.tasks = tasks.lines().map(str::to_string).collect();
        }
    } else {
        ui.label(theme::title(&app.model.title, 21.0));
        ui.add_space(4.0);
        if app.model.description.trim().is_empty() {
            ui.label(
                RichText::new("Klicka på något i modellen för att se och ändra det. Med Ändra kan du skriva en beskrivning.")
                    .font(theme::regular(15.0))
                    .color(theme::MUTED),
            );
        } else {
            ui.label(RichText::new(&app.model.description).font(theme::regular(15.0)).color(theme::INK_2));
        }
        let tasks: Vec<&String> = app.model.tasks.iter().filter(|t| !t.trim().is_empty()).collect();
        if !tasks.is_empty() {
            ui.add_space(16.0);
            ui.label(RichText::new("PROVA DETTA").font(theme::bold(12.5)).color(theme::CONST));
            ui.add_space(2.0);
            for (i, t) in tasks.iter().enumerate() {
                numbered(ui, i + 1, t, theme::CONST);
            }
        }
    }
    ui.data_mut(|d| d.insert_temp(edit_id, editing));
}

fn kind_explanation(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Stock => "Något som samlas och kan mätas vid en viss tidpunkt, t.ex. liter vatten eller kWh i ett batteri.",
        NodeKind::Flow => "Fyller på eller tömmer ett lager. Anger hur mycket som flödar per tidsenhet.",
        NodeKind::Aux => "Räknar ut något med en formel, så att andra formler blir enklare.",
        NodeKind::Constant => "Ett värde som inte ändras medan modellen räknar. Du ändrar det med ett reglage.",
        NodeKind::Variable => "Något som kan öka eller minska.",
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

/// Rubrikrad i inspektören: vad som är markerat, och en knapp för att stänga.
fn inspector_header(ui: &mut Ui, add_tag: impl FnOnce(&mut Ui)) -> bool {
    let mut close = false;
    ui.horizontal(|ui| {
        add_tag(ui);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close = ui_kit::ghost_colored(ui, true, "×", theme::MUTED).on_hover_text("Avmarkera (Esc)").clicked();
        });
    });
    close
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
    let mut close = false;

    let Some(node) = app.model.node_mut(id) else { return };
    let kind = node.kind;
    if inspector_header(ui, |ui| ui_kit::kind_tag(ui, kind)) {
        close = true;
    }
    ui.add_space(6.0);
    let out = TextEdit::singleline(&mut node.name)
        .desired_width(f32::INFINITY)
        .font(theme::semibold(19.0))
        .margin(Margin::symmetric(10, 8))
        .show(ui);
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
    ui.label(RichText::new(kind_explanation(kind)).font(theme::regular(14.0)).color(theme::MUTED));

    match kind {
        NodeKind::Stock | NodeKind::Flow | NodeKind::Aux => {
            ui.add_space(8.0);
            let label = match kind {
                NodeKind::Stock => "Startvärde – hur mycket som finns från början".to_string(),
                NodeKind::Flow => format!("Formel – hur mycket som flödar per {}", singular(&time_unit)),
                _ => "Formel".to_string(),
            };
            field_label(ui, &label);
            ui.add(
                TextEdit::multiline(&mut node.formula)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .font(FontId::monospace(15.0))
                    .margin(Margin::symmetric(10, 8)),
            );
            match &error {
                Some(e) => {
                    ui.label(RichText::new(format!("⚠  {e}")).font(theme::semibold(14.5)).color(theme::ERROR));
                }
                None => {
                    ui.label(RichText::new("✔  Formeln fungerar").font(theme::semibold(13.5)).color(theme::SUCCESS));
                }
            }
            let usable: Vec<&(String, NodeKind)> =
                chips.iter().filter(|(_, k)| kind != NodeKind::Stock || *k == NodeKind::Constant).collect();
            if !usable.is_empty() {
                field_label(ui, "Klicka för att lägga till i formeln");
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                    for (name, k) in usable {
                        if ui_kit::chip(ui, name, Some(theme::kind_color(*k)), true).clicked() {
                            append_to_formula(&mut node.formula, name);
                        }
                    }
                });
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                for piece in ["+", "-", "*", "/", "(", ")"] {
                    if ui.add_sized([32.0, 30.0], egui::Button::new(piece)).clicked() {
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
            ui.add_space(8.0);
            field_label(ui, "Värde");
            let (lo, hi) = range(node.min, node.max);
            ui.spacing_mut().slider_width = (ui.available_width() - 80.0).max(80.0);
            ui.add(number_slider(&mut node.value, lo, hi));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Reglaget går från").font(theme::regular(14.0)).color(theme::INK_2));
                ui.add(egui::DragValue::new(&mut node.min).speed(0.1));
                ui.label(RichText::new("till").font(theme::regular(14.0)).color(theme::INK_2));
                ui.add(egui::DragValue::new(&mut node.max).speed(0.1));
            });
        }
        NodeKind::Variable => {}
    }

    if kind == NodeKind::Flow {
        ui.add_space(8.0);
        egui::Grid::new("flow_ends").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label(RichText::new("Från").font(theme::semibold(14.0)).color(theme::INK_2));
            stock_combo(ui, "flow_from", &mut node.from, &stocks);
            ui.end_row();
            ui.label(RichText::new("Till").font(theme::semibold(14.0)).color(theme::INK_2));
            stock_combo(ui, "flow_to", &mut node.to, &stocks);
            ui.end_row();
        });
    }

    if kind != NodeKind::Variable {
        field_label(ui, "Enhet");
        ui.add(TextEdit::singleline(&mut node.unit).hint_text("t.ex. liter, kWh, °C").desired_width(f32::INFINITY));
        ui.add_space(4.0);
        ui.checkbox(&mut node.in_graph, "Visa i grafen");
    }
    field_label(ui, "Anteckning");
    ui.add(TextEdit::multiline(&mut node.note).hint_text("Förklara vad det här är…").desired_rows(2).desired_width(f32::INFINITY));
    ui.add_space(10.0);
    if ui_kit::ghost_colored(ui, true, "🗑  Ta bort", theme::ERROR).clicked() {
        delete = true;
    }
    if delete {
        app.delete_selection();
    } else if close {
        app.selection = Selection::None;
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
    let close = inspector_header(ui, |ui| {
        ui.label(theme::title("Pil", 19.0));
    });
    let link = &mut app.model.links[i];
    ui.label(RichText::new(format!("Från {a} till {b}")).font(theme::regular(15.0)).color(theme::INK_2));
    ui.add_space(10.0);
    for (positive, sign, title, text, color) in [
        (true, "+", "Samma riktning", format!("Ökar {a} så ökar {b}. Minskar {a} så minskar {b}."), theme::POSITIVE),
        (false, "−", "Motsatt riktning", format!("Ökar {a} så minskar {b}. Minskar {a} så ökar {b}."), theme::NEGATIVE),
    ] {
        let selected = link.positive == positive;
        let resp = egui::Frame::default()
            .fill(if selected { color.gamma_multiply(0.08) } else { Color32::TRANSPARENT })
            .stroke(Stroke::new(if selected { 1.5_f32 } else { 1.0 }, if selected { color } else { theme::LINE }))
            .corner_radius(cr(10.0))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width().max(0.0));
                ui.horizontal_top(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::hover());
                    ui.painter().circle(r.center(), 12.0, theme::SURFACE, Stroke::new(1.5_f32, color));
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, sign, theme::bold(18.0), color);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(title).font(theme::semibold(15.5)).color(theme::INK));
                        ui.label(RichText::new(text).font(theme::regular(14.0)).color(theme::INK_2));
                    });
                });
            })
            .response
            .interact(Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if resp.clicked() {
            link.positive = positive;
        }
        ui.add_space(4.0);
    }
    field_label(ui, "Böjning");
    ui.add(egui::Slider::new(&mut link.curve, -1.0..=1.0).show_value(false));
    ui.label(RichText::new("Du kan också dra i den lilla ringen mitt på pilen.").font(theme::regular(13.5)).color(theme::MUTED));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui_kit::ghost(ui, true, "⇄  Vänd pilen").clicked() {
            std::mem::swap(&mut link.from, &mut link.to);
        }
        if ui_kit::ghost_colored(ui, true, "🗑  Ta bort", theme::ERROR).clicked() {
            delete = true;
        }
    });
    if delete {
        app.delete_selection();
    } else if close {
        app.selection = Selection::None;
    }
}

fn loops_section(app: &mut App, ui: &mut Ui) {
    app.highlight_loop = None;
    let labels = canvas::loop_labels(&app.loops);
    let name = |id: &u32| app.model.node(*id).map_or("?".to_string(), |n| n.name.trim().to_string());
    let mut hovered = None;
    ui_kit::section_title(ui, "Återkopplingsloopar");
    ui.add_space(4.0);
    if app.loops.is_empty() {
        ui.label(
            RichText::new("Inga loopar än. En loop uppstår när pilarna leder runt och tillbaka till samma variabel.")
                .font(theme::regular(15.0))
                .color(theme::MUTED),
        );
    }
    for (i, lp) in app.loops.iter().enumerate() {
        let color = if lp.reinforcing { theme::REINFORCING } else { theme::BALANCING };
        let kind = if lp.reinforcing { "Förstärkande" } else { "Balanserande" };
        let mut chain: Vec<String> = lp.nodes.iter().map(name).collect();
        chain.push(name(&lp.nodes[0]));
        let resp = egui::Frame::default()
            .fill(if app.highlight_loop == Some(i) { color.gamma_multiply(0.1) } else { Color32::TRANSPARENT })
            .corner_radius(cr(10.0))
            .inner_margin(Margin::symmetric(8, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width().max(0.0));
                ui.horizontal_top(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 16.0, color.gamma_multiply(0.14));
                    ui.painter().text(r.center(), Align2::CENTER_CENTER, &labels[i], theme::bold(14.0), color);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(kind).font(theme::semibold(15.5)).color(color));
                        ui.label(RichText::new(chain.join(" ➡ ")).font(theme::regular(13.5)).color(theme::INK_2));
                    });
                });
            })
            .response;
        if ui.rect_contains_pointer(resp.rect) {
            hovered = Some(i);
        }
    }
    ui.add_space(14.0);
    for (letter, title, text, color) in [
        ("R", "Förstärkande loop", "En förändring växer sig större och större, som ränta på ränta. Jämnt antal minuspilar.", theme::REINFORCING),
        ("B", "Balanserande loop", "Systemet strävar mot jämvikt, som en termostat. Udda antal minuspilar.", theme::BALANCING),
    ] {
        ui.horizontal_top(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
            ui.painter().text(r.center(), Align2::CENTER_CENTER, letter, theme::bold(14.0), color);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new(title).font(theme::semibold(14.0)).color(theme::INK));
                ui.label(RichText::new(text).font(theme::regular(13.5)).color(theme::MUTED));
            });
        });
        ui.add_space(4.0);
    }
    app.highlight_loop = hovered;
}

// ---------------------------------------------------------------- resultat

pub fn results_panel(app: &mut App, ctx: &Context) {
    egui::TopBottomPanel::bottom("results")
        .resizable(true)
        .default_height(300.0)
        .min_height(220.0)
        .max_height((ctx.screen_rect().height() * 0.5).max(240.0))
        .frame(
            egui::Frame::default()
                .fill(theme::SURFACE)
                .inner_margin(Margin { left: 16, right: 16, top: 12, bottom: 10 })
                .stroke(Stroke::new(1.0_f32, theme::LINE)),
        )
        .show(ctx, |ui| {
            results_header(app, ui);
            if let Some(e) = &app.result.error {
                ui.label(RichText::new(format!("⚠  {e}")).font(theme::semibold(14.5)).color(theme::ERROR));
            }
            ui.add_space(6.0);
            let h = ui.available_height();
            let has_constants = app.model.nodes.iter().any(|n| n.kind == NodeKind::Constant);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if has_constants || app.result_tab == ResultTab::Graph {
                    ui.allocate_ui_with_layout(vec2(270.0, h), egui::Layout::top_down(egui::Align::Min), |ui| {
                        sliders_column(app, ui);
                    });
                    let (sep, _) = ui.allocate_exact_size(vec2(25.0, h), Sense::hover());
                    ui.painter().vline(sep.center().x, sep.y_range(), Stroke::new(1.0_f32, theme::LINE));
                }
                let w = ui.available_width();
                ui.allocate_ui_with_layout(vec2(w, h), egui::Layout::top_down(egui::Align::Min), |ui| match app.result_tab {
                    ResultTab::Graph => graph(app, ui),
                    ResultTab::Table => table(app, ui),
                });
            });
        });
}

fn results_header(app: &mut App, ui: &mut Ui) {
    let end = app.result.times.last().copied().unwrap_or(0.0);
    let has_result = !app.result.times.is_empty() && app.result.error.is_none();
    // Smalt fönster (t.ex. en Chromebook): kortare knappar så att allt får plats.
    let narrow = ui.available_width() < 980.0;
    ui.horizontal(|ui| {
        ui_kit::segmented(ui, &mut app.result_tab, &[(ResultTab::Graph, "📈  Graf"), (ResultTab::Table, "🔢  Tabell")]);
        ui.add_space(12.0);
        let play = match (app.playing, narrow) {
            (true, true) => "⏸",
            (true, false) => "⏸  Pausa",
            (false, true) => "▶",
            (false, false) => "▶  Spela upp",
        };
        if ui_kit::primary(ui, end > 0.0, play).on_hover_text("Spela upp: se hur modellen ändras över tid").clicked() {
            if app.playing {
                app.playing = false;
            } else {
                if app.scrub.is_none_or(|t| t >= end) {
                    app.scrub = Some(0.0);
                }
                app.playing = true;
            }
        }
        ui.add_space(6.0);
        let mut t = app.shown_time();
        // Det som står till höger om reglaget: tidsetikett, "till slutet" och verktygsknapparna.
        let right = if narrow { 300.0 } else { 440.0 };
        ui.spacing_mut().slider_width = (ui.available_width() - right).clamp(60.0, 420.0);
        let slider = egui::Slider::new(&mut t, 0.0..=end.max(0.001)).show_value(false);
        if ui.add_enabled(end > 0.0, slider).on_hover_text("Dra för att se en viss tidpunkt").changed() {
            app.scrub = Some(t);
            app.playing = false;
        }
        ui.label(
            RichText::new(format!("{} {}", fmt_num(app.shown_time()), app.model.time_unit))
                .font(theme::semibold(14.5))
                .color(theme::INK_2),
        );
        if app.scrub.is_some() && ui_kit::ghost(ui, true, "⏭").on_hover_text("Till slutet").clicked() {
            app.scrub = None;
            app.playing = false;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let label = |icon: &str, text: &str| if narrow { icon.to_string() } else { format!("{icon}  {text}") };
            ui_kit::ghost_menu(ui, true, &label("⏱", "Tid"), "Hur länge och hur noga modellen räknas", |ui| {
                time_settings(app, ui);
            });
            ui_kit::ghost_menu(ui, has_result, &label("🖼", "Bild"), "Spara eller kopiera grafen, t.ex. till en labbrapport", |ui| {
                if ui.add(egui::Button::new("Spara grafen som bild…").min_size(vec2(ui.available_width(), 34.0))).clicked() {
                    export::request(app, ui.ctx(), ExportAction::Save);
                    ui.memory_mut(|m| m.close_popup());
                }
                if ui.add(egui::Button::new("Kopiera grafen").min_size(vec2(ui.available_width(), 34.0))).clicked() {
                    export::request(app, ui.ctx(), ExportAction::Copy);
                    ui.memory_mut(|m| m.close_popup());
                }
            });
            if ui_kit::ghost(ui, has_result, &label("📌", "Jämför"))
                .on_hover_text("Behåll kurvorna som streckade linjer. Ändra sedan något och jämför!")
                .clicked()
            {
                app.save_run();
            }
        });
    });
}

fn time_settings(app: &mut App, ui: &mut Ui) {
    egui::Grid::new("sim_settings").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
        ui.label("Tidsenhet");
        ui.add(TextEdit::singleline(&mut app.model.time_unit).desired_width(140.0));
        ui.end_row();
        ui.label("Simulera i");
        ui.add(egui::DragValue::new(&mut app.model.t_end).range(0.1..=100_000.0).speed(0.5).suffix(format!(" {}", app.model.time_unit)));
        ui.end_row();
        ui.label("Tidssteg (dt)");
        ui.add(egui::DragValue::new(&mut app.model.dt).range(0.0001..=1000.0).speed(0.005).max_decimals(4));
        ui.end_row();
    });
    ui.add_space(4.0);
    ui.label(
        RichText::new("Datorn räknar fram modellen ett litet tidssteg i taget. Mindre steg ger noggrannare svar.")
            .font(theme::regular(13.5))
            .color(theme::MUTED),
    );
}

/// Reglagen till vänster om grafen: orsak bredvid verkan.
fn sliders_column(app: &mut App, ui: &mut Ui) {
    ui_kit::section_title(ui, "Reglage");
    ui.add_space(2.0);
    let constants = app.model.nodes.iter().filter(|n| n.kind == NodeKind::Constant).count();
    if constants == 0 {
        ui.label(
            RichText::new("Lägg till en konstant så får du ett reglage här som du kan dra i medan modellen körs.")
                .font(theme::regular(14.0))
                .color(theme::MUTED),
        );
        return;
    }
    let mut select = None;
    let max_h = ui.available_height();
    egui::ScrollArea::vertical().id_salt("sliders").max_height(max_h).auto_shrink([false, false]).show(ui, |ui| {
        ui.set_width(ui.available_width() - 8.0);
        for n in app.model.nodes.iter_mut().filter(|n| n.kind == NodeKind::Constant) {
            ui.horizontal(|ui| {
                let name = ui
                    .add(egui::Label::new(RichText::new(n.name.trim()).font(theme::semibold(14.5)).color(theme::INK)).sense(Sense::click()))
                    .on_hover_text("Klicka för att markera")
                    .on_hover_cursor(CursorIcon::PointingHand);
                if name.clicked() {
                    select = Some(n.id);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(theme::fmt_with_unit(n.value, &n.unit)).font(theme::semibold(14.5)).color(theme::CONST));
                });
            });
            let (lo, hi) = range(n.min, n.max);
            ui.spacing_mut().slider_width = ui.available_width();
            ui.add(number_slider(&mut n.value, lo, hi).show_value(false));
            ui.add_space(6.0);
        }
    });
    if let Some(id) = select {
        app.selection = Selection::Node(id);
    }
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

fn series_color(i: usize) -> Color32 {
    theme::SERIES[i % theme::SERIES.len()]
}

/// Vad som skiljer en sparad körning från modellen nu, t.ex. "Årsränta = 5 %".
fn run_difference(model: &crate::model::Model, run: &SavedRun) -> String {
    let diffs: Vec<String> = run
        .constants
        .iter()
        .filter_map(|(id, v)| {
            let n = model.node(*id)?;
            ((n.value - v).abs() > 1e-9).then(|| format!("{} = {}", n.name.trim(), theme::fmt_with_unit(*v, &n.unit)))
        })
        .collect();
    if diffs.is_empty() {
        "samma värden som nu".into()
    } else {
        diffs.join(", ")
    }
}

/// En liten streckad linje, samma symbol som jämförelserna har i grafen.
fn dashed_icon(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(22.0, 14.0), Sense::hover());
    let y = rect.center().y;
    for k in 0..3 {
        let x = rect.left() + k as f32 * 8.0;
        ui.painter().line_segment([pos2(x, y), pos2(x + 5.0, y)], Stroke::new(2.0_f32, theme::MUTED));
    }
}


fn graph(app: &mut App, ui: &mut Ui) {
    // Vad som visas: klickbara chips med samma färg som kurvan, på en rad som går att rulla i sidled.
    // Det som visas står först; ordningen ändras inte medan man klickar.
    let mut order: Vec<usize> = (0..app.model.nodes.len()).filter(|&i| app.model.nodes[i].kind != NodeKind::Variable).collect();
    let order_id = ui.id().with("chip_order");
    let saved: Option<Vec<usize>> = ui.data(|d| d.get_temp(order_id));
    if let Some(prev) = saved.filter(|p| p.len() == order.len() && p.iter().all(|i| order.contains(i))) {
        order = prev;
    } else {
        order.sort_by_key(|&i| !app.model.nodes[i].in_graph);
        ui.data_mut(|d| d.insert_temp(order_id, order.clone()));
    }
    egui::ScrollArea::horizontal().id_salt("series").auto_shrink([false, true]).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for &i in &order {
                let n = &mut app.model.nodes[i];
                let resp = ui_kit::chip(ui, n.name.trim(), Some(series_color(i)), n.in_graph);
                if resp.on_hover_text(if n.in_graph { "Dölj i grafen" } else { "Visa i grafen" }).clicked() {
                    n.in_graph = !n.in_graph;
                }
            }
        });
    });
    if !app.saved_runs.is_empty() {
        let mut remove = None;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Jämför med").font(theme::semibold(13.5)).color(theme::MUTED));
            for (k, run) in app.saved_runs.iter().enumerate() {
                egui::Frame::default()
                    .fill(theme::SURFACE_2)
                    .corner_radius(cr(15.0))
                    .inner_margin(Margin { left: 10, right: 4, top: 2, bottom: 2 })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            dashed_icon(ui);
                            ui.label(
                                RichText::new(format!("Körning {}: {}", run.number, run_difference(&app.model, run)))
                                    .font(theme::regular(14.0))
                                    .color(theme::INK),
                            );
                            if ui_kit::ghost_colored(ui, true, "×", theme::MUTED).on_hover_text("Ta bort jämförelsen").clicked() {
                                remove = Some(k);
                            }
                        });
                    });
            }
            if app.saved_runs.len() > 1 && ui_kit::ghost_colored(ui, true, "Rensa alla", theme::MUTED).clicked() {
                remove = Some(usize::MAX);
            }
        });
        match remove {
            Some(usize::MAX) => app.saved_runs.clear(),
            Some(k) => {
                app.saved_runs.remove(k);
            }
            None => {}
        }
    }
    let shown: Vec<(usize, &crate::model::Node)> =
        app.model.nodes.iter().enumerate().filter(|(_, n)| n.in_graph && n.kind != NodeKind::Variable).collect();
    if shown.is_empty() {
        ui.add_space(24.0);
        ui.label(RichText::new("Klicka på något ovanför för att visa det i grafen.").font(theme::regular(15.0)).color(theme::MUTED));
        return;
    }
    let unit = app.model.time_unit.clone();
    let t_now = app.shown_time();
    let show_line = app.scrub.is_some() || app.playing;
    let label = |n: &crate::model::Node| {
        if n.unit.trim().is_empty() {
            n.name.trim().to_string()
        } else {
            format!("{} ({})", n.name.trim(), n.unit.trim())
        }
    };
    ui.add_space(4.0);
    let plot = Plot::new("graf")
        .legend(Legend::default().position(Corner::LeftTop).background_alpha(0.85))
        .x_axis_label(format!("Tid ({unit})"))
        .x_axis_formatter(|m, _| fmt_num(m.value))
        .y_axis_formatter(|m, _| fmt_num(m.value))
        .label_formatter(move |name, v| {
            let name = if name.is_empty() { String::new() } else { format!("{name}\n") };
            format!("{name}tid {} {unit}: {}", fmt_num(v.x), fmt_num(v.y))
        })
        .allow_scroll(false)
        .show(ui, |pu| {
            // Sparade körningar först, så att den nuvarande hamnar överst.
            for (k, run) in app.saved_runs.iter().enumerate() {
                let fade = 0.85 - 0.12 * k as f32;
                for (i, n) in &shown {
                    let Some(s) = run.series.get(&n.id) else { continue };
                    let pts: Vec<[f64; 2]> = run.times.iter().zip(s).map(|(t, v)| [*t, *v]).collect();
                    pu.line(
                        Line::new(PlotPoints::new(pts))
                            .name(format!("{} – körning {}", label(n), run.number))
                            .color(series_color(*i).gamma_multiply(fade))
                            .style(LineStyle::Dashed { length: (12.0 - 3.0 * k as f32).max(4.0) })
                            .width(2.0_f32),
                    );
                }
            }
            for (i, n) in &shown {
                let Some(s) = app.result.series.get(&n.id) else { continue };
                let pts: Vec<[f64; 2]> = app.result.times.iter().zip(s).map(|(t, v)| [*t, *v]).collect();
                pu.line(Line::new(PlotPoints::new(pts)).name(label(n)).color(series_color(*i)).width(3.0_f32));
            }
            if show_line {
                pu.vline(VLine::new(t_now).color(theme::INK_2).width(1.5_f32));
            }
            if pu.response().clicked() {
                pu.pointer_coordinate().map(|p| p.x)
            } else {
                None
            }
        });
    app.plot_rect = plot.response.rect;
    if let Some(t) = plot.inner {
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
    if ui_kit::ghost(ui, true, "📋  Kopiera tabellen").on_hover_text("Klistra sedan in i t.ex. Excel eller Google Kalkylark").clicked() {
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
