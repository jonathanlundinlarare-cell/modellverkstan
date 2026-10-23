//! Appens tillstånd och huvudloop.

use crate::model::{FeedbackLoop, Model, ModelKind, NodeKind};
use crate::sim::{self, Compiled, SimResult};
use crate::{canvas, examples, export, help, io, lessons, panels, start, theme, ui_kit};
use egui::{Context, Key, KeyboardShortcut, Modifiers, Vec2};
use std::collections::HashMap;

const STORAGE_KEY: &str = "senaste_modell";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Place(NodeKind),
    Flow,
    Link,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    None,
    Node(u32),
    Link(usize),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Drag {
    None,
    Move { id: u32, grab: Vec2 },
    Pan,
    Curve(usize),
    NewFlow { start: [f32; 2], from: Option<u32> },
    NewLink { from: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Start,
    Editor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResultTab {
    Graph,
    Table,
}

pub struct View {
    pub offset: Vec2,
    pub zoom: f32,
}

/// En sparad körning som visas som streckad linje i grafen, för att jämföra med.
pub struct SavedRun {
    pub number: usize,
    pub times: Vec<f64>,
    pub series: HashMap<u32, Vec<f64>>,
    /// Konstanternas värden när körningen sparades (id, värde).
    pub constants: Vec<(u32, f64)>,
}

/// Högst så här många jämförelser åt gången, annars blir grafen oläslig.
const MAX_SAVED_RUNS: usize = 4;

pub struct App {
    pub screen: Screen,
    pub model: Model,
    committed: Model,
    undo: Vec<Model>,
    redo: Vec<Model>,
    pub resume: Option<Model>,

    pub tool: Tool,
    pub selection: Selection,
    pub drag: Drag,
    pub view: View,
    pub fit_pending: bool,
    pub canvas_rect: egui::Rect,

    pub compiled: Compiled,
    pub result: SimResult,
    pub loops: Vec<FeedbackLoop>,
    sim_of: Option<Model>,

    pub scrub: Option<f64>,
    pub playing: bool,
    pub result_tab: ResultTab,
    pub highlight_loop: Option<usize>,
    pub saved_runs: Vec<SavedRun>,
    run_counter: usize,
    /// Var grafen ritades senast (för att kunna spara den som bild).
    pub plot_rect: egui::Rect,
    pub export: Option<(export::ExportAction, f64)>,
    /// Pågående träning (guidad lektion), om någon.
    pub lesson: Option<lessons::LessonState>,

    pub show_help: bool,
    /// Ämnet som visas bland exemplen på startsidan (None = alla).
    pub subject_filter: Option<examples::Subject>,
    pub focus_name: bool,
    pub rename_origin: Option<(u32, String)>,
    toast: Option<(String, f64)>,
    file_io: io::FileIo,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);
        let resume = cc
            .storage
            .and_then(|s| s.get_string(STORAGE_KEY))
            .and_then(|json| serde_json::from_str::<Model>(&json).ok())
            .filter(|m| !m.nodes.is_empty());
        let model = Model::default();
        Self {
            screen: Screen::Start,
            committed: model.clone(),
            model,
            undo: Vec::new(),
            redo: Vec::new(),
            resume,
            tool: Tool::Select,
            selection: Selection::None,
            drag: Drag::None,
            view: View { offset: Vec2::ZERO, zoom: 1.0 },
            fit_pending: true,
            canvas_rect: egui::Rect::NOTHING,
            compiled: Compiled::default(),
            result: SimResult::default(),
            loops: Vec::new(),
            sim_of: None,
            scrub: None,
            playing: false,
            result_tab: ResultTab::Graph,
            highlight_loop: None,
            saved_runs: Vec::new(),
            run_counter: 0,
            plot_rect: egui::Rect::NOTHING,
            export: None,
            lesson: None,
            show_help: false,
            subject_filter: None,
            focus_name: false,
            rename_origin: None,
            toast: None,
            file_io: io::FileIo::default(),
        }
    }

    /// Öppnar en modell i redigeraren och börjar om ångra-historiken.
    pub fn load_model(&mut self, model: Model) {
        self.committed = model.clone();
        self.model = model;
        self.undo.clear();
        self.redo.clear();
        self.selection = Selection::None;
        self.drag = Drag::None;
        self.tool = Tool::Select;
        self.scrub = None;
        self.playing = false;
        self.fit_pending = true;
        self.highlight_loop = None;
        self.rename_origin = None;
        self.saved_runs.clear();
        self.run_counter = 0;
        self.lesson = None;
        self.screen = Screen::Editor;
    }

    /// Sparar den nuvarande körningen som jämförelse i grafen.
    pub fn save_run(&mut self) {
        if self.result.times.is_empty() {
            return;
        }
        self.run_counter += 1;
        let constants = self
            .model
            .nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Constant)
            .map(|n| (n.id, n.value))
            .collect();
        self.saved_runs.push(SavedRun {
            number: self.run_counter,
            times: self.result.times.clone(),
            series: self.result.series.clone(),
            constants,
        });
        if self.saved_runs.len() > MAX_SAVED_RUNS {
            self.saved_runs.remove(0);
        }
    }

    pub fn notify(&mut self, ctx: &Context, msg: impl Into<String>) {
        self.toast = Some((msg.into(), ctx.input(|i| i.time)));
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.model != self.committed
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        if self.model != self.committed {
            self.undo.push(self.committed.clone());
            self.committed = self.model.clone();
        }
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.model, prev));
            self.committed = self.model.clone();
            self.fix_selection();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.model, next));
            self.committed = self.model.clone();
            self.fix_selection();
        }
    }

    fn fix_selection(&mut self) {
        match self.selection {
            Selection::Node(id) if self.model.node(id).is_none() => self.selection = Selection::None,
            Selection::Link(i) if i >= self.model.links.len() => self.selection = Selection::None,
            _ => {}
        }
    }

    /// Sparar ett ångra-steg när användaren har gjort klart en ändring
    /// (släppt musknappen och lämnat textrutan).
    fn track_undo(&mut self, ctx: &Context) {
        if self.model == self.committed {
            return;
        }
        let busy = ctx.input(|i| i.pointer.any_down()) || ctx.wants_keyboard_input();
        if !busy {
            self.undo.push(std::mem::replace(&mut self.committed, self.model.clone()));
            if self.undo.len() > 200 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }

    pub fn delete_selection(&mut self) {
        match self.selection {
            Selection::Node(id) => self.model.remove_node(id),
            Selection::Link(i) if i < self.model.links.len() => {
                self.model.links.remove(i);
            }
            _ => {}
        }
        self.selection = Selection::None;
        self.drag = Drag::None;
    }

    /// Tiden som visas i ritytan och grafen.
    pub fn shown_time(&self) -> f64 {
        let end = self.result.times.last().copied().unwrap_or(0.0);
        self.scrub.map_or(end, |t| t.min(end))
    }

    fn refresh_sim(&mut self) {
        if self.sim_of.as_ref() == Some(&self.model) {
            return;
        }
        self.compiled = sim::compile(&self.model);
        self.result = sim::simulate(&self.model, &self.compiled);
        self.loops = match self.model.kind {
            ModelKind::Cld => self.model.find_loops(),
            ModelKind::Sfd => Vec::new(),
        };
        self.sim_of = Some(self.model.clone());
    }

    pub fn save_file(&mut self, ctx: &Context) {
        match self.file_io.save(&self.model) {
            Ok(true) => self.notify(ctx, "Modellen sparades."),
            Ok(false) => {}
            Err(e) => self.notify(ctx, e),
        }
    }

    pub fn open_file(&mut self, ctx: &Context) {
        self.file_io.open(ctx);
    }

    fn shortcuts(&mut self, ctx: &Context) {
        if self.screen != Screen::Editor {
            return;
        }
        let cmd = |key| KeyboardShortcut::new(Modifiers::COMMAND, key);
        let cmd_shift = |key| KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, key);
        if ctx.input_mut(|i| i.consume_shortcut(&cmd(Key::S))) {
            self.save_file(ctx);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&cmd(Key::O))) {
            self.open_file(ctx);
        }
        if ctx.wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&cmd_shift(Key::Z)) || i.consume_shortcut(&cmd(Key::Y))) {
            self.redo();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&cmd(Key::Z))) {
            self.undo();
        }
        if ctx.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            self.delete_selection();
        }
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.tool = Tool::Select;
            self.drag = Drag::None;
            self.selection = Selection::None;
        }
    }

    fn top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("topbar")
            .exact_height(56.0)
            .frame(
                egui::Frame::default()
                    .fill(theme::SURFACE)
                    .inner_margin(egui::Margin::symmetric(16, 0))
                    .stroke(egui::Stroke::new(1.0_f32, theme::LINE)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let (rect, logo) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::click());
                    ui_kit::logo(ui.painter(), rect);
                    let name = ui.add(egui::Label::new(theme::title("Modellverkstan", 17.0)).sense(egui::Sense::click()));
                    if logo.union(name).on_hover_text("Till startsidan").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        self.screen = Screen::Start;
                    }
                    if self.screen == Screen::Editor {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("/").font(theme::regular(18.0)).color(theme::LINE_STRONG));
                        ui.add_space(4.0);
                        let kind = match self.model.kind {
                            ModelKind::Cld => "Orsaksdiagram",
                            ModelKind::Sfd => "Flödesmodell",
                        };
                        ui.label(theme::strong(&self.model.title, 16.0));
                        ui.label(egui::RichText::new(kind).font(theme::regular(14.0)).color(theme::MUTED));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        let help = if self.show_help { "✕  Stäng hjälpen" } else { "❓  Hjälp" };
                        if ui_kit::ghost(ui, true, help).clicked() {
                            self.show_help = !self.show_help;
                        }
                        if self.screen == Screen::Editor {
                            ui.add_space(8.0);
                            if ui_kit::ghost(ui, true, "💾  Spara").on_hover_text("Spara som fil (Ctrl+S)").clicked() {
                                self.save_file(ctx);
                            }
                            if ui_kit::ghost(ui, true, "📂  Öppna").on_hover_text("Öppna en sparad modell (Ctrl+O)").clicked() {
                                self.open_file(ctx);
                            }
                            ui.add_space(8.0);
                            if ui_kit::ghost(ui, self.can_redo(), "⟳").on_hover_text("Gör om (Ctrl+Y)").clicked() {
                                self.redo();
                            }
                            if ui_kit::ghost(ui, self.can_undo(), "⟲  Ångra").on_hover_text("Ångra (Ctrl+Z)").clicked() {
                                self.undo();
                            }
                        }
                    });
                });
            });
    }

    /// Korta meddelanden visas som en mörk bricka nere i mitten och försvinner av sig själva.
    fn toast(&mut self, ctx: &Context) {
        let Some((msg, at)) = &self.toast else { return };
        let age = ctx.input(|i| i.time) - at;
        if age > 4.0 {
            self.toast = None;
            return;
        }
        ctx.request_repaint();
        egui::Area::new(egui::Id::new("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -28.0))
            .order(egui::Order::Tooltip)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(theme::INK)
                    .corner_radius(theme::cr(10.0))
                    .inner_margin(egui::Margin::symmetric(18, 12))
                    .shadow(egui::Shadow { offset: [0, 6], blur: 20, spread: 0, color: egui::Color32::from_black_alpha(40) })
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(msg).font(theme::semibold(15.0)).color(theme::SURFACE));
                    });
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(result) = self.file_io.poll() {
            match result {
                Ok(model) => {
                    self.load_model(model);
                    self.notify(ctx, "Modellen öppnades.");
                }
                Err(e) => self.notify(ctx, e),
            }
        }
        export::poll(self, ctx);
        // När namnrutan lämnas uppdateras formler som använde det gamla namnet.
        if !ctx.wants_keyboard_input() {
            if let Some((id, old)) = self.rename_origin.take() {
                if let Some(new) = self.model.node(id).map(|n| n.name.clone()) {
                    self.model.rename_refs(&old, &new);
                }
            }
        }
        self.refresh_sim();
        let screen_before = self.screen;
        self.shortcuts(ctx);
        self.top_bar(ctx);

        match self.screen {
            Screen::Start => start::show(self, ctx),
            Screen::Editor => {
                panels::tool_panel(self, ctx);
                panels::side_panel(self, ctx);
                if self.model.kind == ModelKind::Sfd {
                    panels::results_panel(self, ctx);
                }
                egui::CentralPanel::default().frame(egui::Frame::default()).show(ctx, |ui| canvas::show(self, ui));
            }
        }
        help::window(ctx, &mut self.show_help);
        self.toast(ctx);

        if self.playing {
            let end = self.result.times.last().copied().unwrap_or(0.0);
            let step = ctx.input(|i| i.stable_dt as f64) * end / 8.0;
            let t = self.scrub.unwrap_or(0.0) + step;
            if t >= end {
                self.scrub = None;
                self.playing = false;
            } else {
                self.scrub = Some(t);
            }
            ctx.request_repaint();
        }

        self.track_undo(ctx);
        if self.screen != screen_before || self.sim_of.as_ref() != Some(&self.model) {
            ctx.request_repaint();
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.screen == Screen::Editor && !self.model.nodes.is_empty() {
            if let Ok(json) = serde_json::to_string(&self.model) {
                storage.set_string(STORAGE_KEY, json);
            }
        }
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }
}
