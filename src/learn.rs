//! "Lär dig modellering": en presentation med bilder som förklarar begreppen.
//! Fungerar både för läraren på projektor och för eleven på egen hand.

use crate::app::{App, Screen};
use crate::illustrations::{self, Illustration};
use crate::lessons::{self, LessonState};
use crate::theme::{self, cr};
use crate::ui_kit;
use egui::{vec2, Context, Key, Margin, RichText, Sense, Stroke, Ui};

pub struct Slide {
    pub chapter: usize,
    pub title: &'static str,
    pub points: &'static [&'static str],
    pub illustration: Illustration,
    /// Källa eller förtydligande, visas med liten text.
    pub note: Option<&'static str>,
}

/// Illustrationernas proportioner (höjd delat med bredd).
const H_PER_W: f32 = 340.0 / 480.0;

pub const CHAPTERS: [&str; 4] = ["Vad är modellering?", "Orsaksdiagram", "Lager och flöden", "Modeller i verkligheten"];

pub fn slides() -> Vec<Slide> {
    use Illustration::*;
    let s = |chapter, title, points, illustration, note| Slide { chapter, title, points, illustration, note };
    vec![
        s(
            0,
            "Modellering – att förstå system",
            &[
                "En kort genomgång av vad modeller är, hur man bygger dem och varför de ibland slår fel.",
                "Bläddra med pilarna nedanför, eller med piltangenterna.",
            ],
            Title,
            None,
        ),
        s(
            0,
            "Vad är en modell?",
            &[
                "En modell är en förenklad bild av verkligheten.",
                "En karta är en modell: den visar vägarna, men inte varje träd.",
                "Man tar med det som behövs för att svara på en fråga, och lämnar resten utanför.",
            ],
            MapVsWorld,
            Some("Statistikern George Box brukar citeras: \"Alla modeller är fel, men vissa är användbara.\""),
        ),
        s(
            0,
            "Varför modellerar vi?",
            &[
                "För att förstå: varför händer det här?",
                "För att förutsäga: vad händer sedan? Väderprognoser och klimatmodeller är exempel.",
                "För att testa \"tänk om …\" utan att göra det på riktigt. Det är billigare, snabbare och ofarligt.",
                "Ingenjörer, ekonomer, läkare och forskare använder modeller varje dag.",
            ],
            WhyModel,
            None,
        ),
        s(
            0,
            "Ett system är delar som påverkar varandra",
            &[
                "Ett system är flera delar som hänger ihop, till exempel ett hus med värmepump och termostat.",
                "Ändrar man en del påverkas de andra, ibland på överraskande sätt.",
                "Systemtänkande betyder att se helheten, inte bara en sak i taget.",
            ],
            System,
            None,
        ),
        s(
            1,
            "Pilar visar vad som påverkar vad",
            &[
                "En variabel är något som kan öka eller minska.",
                "Pil med plus (+): de ändras åt samma håll. Fler bilar ger mer avgaser.",
                "Pil med minus (−): de ändras åt motsatt håll. Högre pris gör att färre köper.",
            ],
            CldArrows,
            None,
        ),
        s(
            1,
            "Förstärkande loop (R)",
            &[
                "När pilarna leder runt och tillbaka till start har vi en loop.",
                "Fler kaniner ger fler födslar, som ger ännu fler kaniner.",
                "En förstärkande loop får förändringar att växa sig större. Ränta på ränta och smitta som sprids fungerar likadant.",
                "Räkna minuspilarna: inga eller ett jämnt antal ger R.",
            ],
            LoopR,
            None,
        ),
        s(
            1,
            "Balanserande loop (B)",
            &[
                "En balanserande loop motverkar förändringen och strävar mot ett jämviktsläge.",
                "Blir det kallt ger elementet mer värme. Blir det varmt ger det mindre.",
                "Termostater, hunger och mättnad, och tillgång och efterfrågan är balanserande loopar.",
                "Ett udda antal minuspilar ger B.",
            ],
            LoopB,
            None,
        ),
        s(
            2,
            "Lager – det som samlas",
            &[
                "Ett lager är något som finns i en viss mängd just nu: vatten i ett badkar, pengar på ett konto, koldioxid i luften.",
                "Om man kunde frysa tiden skulle man kunna räkna lagret.",
                "Ett lager ändras bara genom flöden.",
            ],
            Bathtub,
            None,
        ),
        s(
            2,
            "Flöden – det som ändrar lagret",
            &[
                "Ett flöde fyller på (inflöde) eller tömmer (utflöde) ett lager.",
                "Flöden mäts per tidsenhet: liter per minut, kronor per månad, kilowatt.",
                "Molnen betyder att något kommer från, eller går till, något utanför modellen.",
            ],
            Flows,
            None,
        ),
        s(
            2,
            "Konstanter och hjälpvariabler",
            &[
                "En konstant är ett värde du bestämmer, till exempel hur mycket mat som finns. Den får ett reglage.",
                "En hjälpvariabel räknar ut något med en formel, till exempel mat per kanin.",
                "De grå pilarna visar vilka delar en formel använder.",
            ],
            ConstAux,
            None,
        ),
        s(
            2,
            "Hur datorn räknar",
            &[
                "Datorn räknar ett litet tidssteg (dt) i taget.",
                "För varje steg: räkna ut flödena, ändra lagren lite, och gör om.",
                "Små steg ger en jämn och noggrann kurva. Stora steg går fortare men blir hackiga och fel.",
            ],
            TimeSteps,
            None,
        ),
        s(
            2,
            "Kurvor berättar vilka loopar som styr",
            &[
                "Växer allt snabbare: en förstärkande loop vinner.",
                "Närmar sig ett mål: en balanserande loop vinner.",
                "S-kurva: först växer det snabbt, sedan tar en balanserande loop över, till exempel när maten tar slut.",
            ],
            Behaviours,
            None,
        ),
        s(
            3,
            "När prognosen slog fel: Riksbanken",
            &[
                "Riksbanken använder modeller för att förutsäga inflationen och bestämma styrräntan.",
                "I september 2021 räknade Riksbanken med att räntan skulle ligga kvar på 0 % ända till hösten 2024.",
                "Men priserna steg snabbt efter pandemin och efter Rysslands invasion av Ukraina. I december 2022 var inflationen (KPIF) 10,2 %.",
                "Riksbanken höjde räntan redan i april 2022, och i september 2023 låg den på 4 %.",
                "Lärdom: en modell bygger på antaganden. När något oväntat händer utanför modellen slår prognosen fel.",
            ],
            Riksbank,
            Some("Källor: Riksbankens protokoll och pressmeddelanden 2021–2023, SCB (KPIF december 2022). Grafen är förenklad."),
        ),
        s(
            3,
            "När modellen blev för stor: Afghanistan",
            &[
                "Under kriget i Afghanistan tog den amerikanska militärledningen fram ett orsaksdiagram över läget i landet, enligt uppgifter med hjälp av ett konsultföretag.",
                "Diagrammet hade dussintals variabler och pilar i åtta färger, och kallades snart \"spagettidiagrammet\".",
                "När general Stanley McChrystal fick se det skämtade han om att kriget skulle vara vunnet den dag någon förstod bilden.",
                "Lärdom: en modell som försöker ta med allt blir lika svår att förstå som verkligheten. En bra modell är så enkel som möjligt, men inte enklare.",
            ],
            Spaghetti,
            Some("Källa: New York Times, 26 april 2010. Bilden är en förenklad efterbildning, inte originalet."),
        ),
        s(
            3,
            "Modellers gränser",
            &[
                "Antaganden: varje modell bygger på gissningar om hur världen fungerar.",
                "Systemgränsen: det som ligger i molnen räknas inte, men kan ändå påverka.",
                "Människor: vi ändrar beteende när vi får veta en prognos.",
                "Osäkra data: fel in ger fel ut.",
                "Använd därför modeller för att förstå och jämföra möjligheter, inte som sanningar.",
            ],
            Limits,
            None,
        ),
        s(
            3,
            "Nu är det din tur!",
            &[
                "Du har lärt dig om modeller, orsaksdiagram, lager, flöden och loopar.",
                "Det bästa sättet att förstå är att bygga själv. Börja med en träning, eller öppna ett exempel och dra i reglagen.",
            ],
            Summary,
            None,
        ),
    ]
}

/// Bläddrar i presentationen. Ger `true` om något ändrades.
fn go(app: &mut App, delta: i32, count: usize) {
    let next = (app.learn_slide as i32 + delta).clamp(0, count as i32 - 1) as usize;
    app.learn_slide = next;
}

pub fn show(app: &mut App, ctx: &Context) {
    let all = slides();
    let count = all.len();
    app.learn_slide = app.learn_slide.min(count - 1);

    if !ctx.wants_keyboard_input() {
        let (next, prev, exit) = ctx.input(|i| {
            (
                i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::PageDown) || i.key_pressed(Key::Space),
                i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::PageUp),
                i.key_pressed(Key::Escape),
            )
        });
        if next {
            go(app, 1, count);
        }
        if prev {
            go(app, -1, count);
        }
        if exit {
            set_fullscreen(ctx, false);
            app.screen = Screen::Start;
            return;
        }
    }

    let wide = ctx.screen_rect().width() > 1000.0;
    if wide {
        egui::SidePanel::left("learn_nav")
            .resizable(false)
            .exact_width(260.0)
            .frame(
                egui::Frame::default()
                    .fill(theme::SURFACE)
                    .inner_margin(Margin { left: 16, right: 12, top: 20, bottom: 12 })
                    .stroke(Stroke::new(1.0_f32, theme::LINE)),
            )
            .show(ctx, |ui| chapter_nav(app, ui, &all));
    }

    egui::TopBottomPanel::bottom("learn_bar")
        .exact_height(64.0)
        .frame(
            egui::Frame::default()
                .fill(theme::SURFACE)
                .inner_margin(Margin::symmetric(20, 0))
                .stroke(Stroke::new(1.0_f32, theme::LINE)),
        )
        .show(ctx, |ui| nav_bar(app, ui, count));

    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(theme::CANVAS_BG).inner_margin(Margin::symmetric(32, 24)))
        .show(ctx, |ui| slide_view(app, ui, &all[app.learn_slide], count));
}

fn chapter_nav(app: &mut App, ui: &mut Ui, all: &[Slide]) {
    ui.label(theme::overline("Lär dig modellering"));
    ui.add_space(8.0);
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for (c, chapter) in CHAPTERS.iter().enumerate() {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 11.0, theme::INK);
                ui.painter().text(r.center(), egui::Align2::CENTER_CENTER, (c + 1).to_string(), theme::bold(12.5), theme::SURFACE);
                ui.label(theme::strong(*chapter, 15.0));
            });
            ui.add_space(2.0);
            for (i, slide) in all.iter().enumerate().filter(|(_, s)| s.chapter == c) {
                let current = i == app.learn_slide;
                let color = if current { theme::INK } else { theme::INK_2 };
                let font = if current { theme::semibold(14.0) } else { theme::regular(14.0) };
                let width = ui.available_width();
                let galley = ui.painter().layout(slide.title.to_string(), font, color, width - 40.0);
                let (rect, resp) = ui.allocate_exact_size(vec2(width, (galley.size().y + 14.0).max(32.0)), Sense::click());
                let painter = ui.painter();
                if current {
                    painter.rect_filled(rect, cr(8.0), theme::SURFACE_2);
                    let bar = egui::Rect::from_center_size(egui::pos2(rect.left() + 1.5, rect.center().y), vec2(3.0, rect.height() - 12.0));
                    painter.rect_filled(bar, cr(2.0), theme::SELECT);
                } else if resp.hovered() {
                    painter.rect_filled(rect, cr(8.0), theme::HOVER);
                }
                painter.galley(egui::pos2(rect.left() + 32.0, rect.center().y - galley.size().y / 2.0), galley, color);
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    app.learn_slide = i;
                }
            }
        }
    });
}

fn nav_bar(app: &mut App, ui: &mut Ui, count: usize) {
    let current = app.learn_slide;
    ui.horizontal_centered(|ui| {
        if ui_kit::ghost(ui, current > 0, "‹  Föregående").on_hover_text("Vänsterpil").clicked() {
            go(app, -1, count);
        }
        ui.add_space(12.0);
        // Framsteg: en prick per bild.
        let dot_w = 14.0;
        let (rect, _) = ui.allocate_exact_size(vec2(dot_w * count as f32, 20.0), Sense::hover());
        for k in 0..count {
            let c = egui::pos2(rect.left() + dot_w * (k as f32 + 0.5), rect.center().y);
            let (r, color) = if k == current {
                (5.0, theme::INK)
            } else if k < current {
                (3.5, theme::INK_2)
            } else {
                (3.5, theme::LINE_STRONG)
            };
            ui.painter().circle_filled(c, r, color);
        }
        ui.add_space(8.0);
        ui.label(RichText::new(format!("{} / {count}", current + 1)).font(theme::semibold(14.0)).color(theme::MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if current + 1 < count {
                if ui_kit::primary(ui, true, "Nästa  ›").on_hover_text("Högerpil eller mellanslag").clicked() {
                    go(app, 1, count);
                }
            } else if ui_kit::primary(ui, true, "Klar  ✔").clicked() {
                set_fullscreen(ui.ctx(), false);
                app.screen = Screen::Start;
            }
            let full = is_fullscreen(ui.ctx());
            let label = if full { "Lämna helskärm" } else { "Helskärm" };
            if ui_kit::ghost_colored(ui, true, label, theme::INK_2).on_hover_text("Bra när du visar på projektor").clicked() {
                set_fullscreen(ui.ctx(), !full);
            }
        });
    });
}

fn slide_view(app: &mut App, ui: &mut Ui, slide: &Slide, count: usize) {
    let avail = ui.available_size();
    // Större text på stora skärmar, t.ex. en projektor i helskärm.
    let k = (avail.y / 640.0).clamp(0.9, 1.5);
    let wide = avail.x > 760.0;
    ui.label(
        RichText::new(format!("KAPITEL {} · {}", slide.chapter + 1, CHAPTERS[slide.chapter].to_uppercase()))
            .font(theme::bold(13.0 * k))
            .color(theme::MUTED),
    );
    ui.add_space(4.0 * k);
    ui.label(theme::title(slide.title, 32.0 * k));
    ui.add_space(18.0 * k);

    let body_h = ui.available_height();
    if wide {
        let text_w = (avail.x * 0.44).max(320.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(text_w, body_h), egui::Layout::top_down(egui::Align::Min), |ui| {
                points(app, ui, slide, k, count);
            });
            ui.add_space(24.0);
            let w = ui.available_width();
            let (rect, _) = ui.allocate_exact_size(vec2(w, body_h.min(w * H_PER_W).min(520.0 * k)), Sense::hover());
            illustrations::draw(ui.painter(), rect, slide.illustration);
        });
    } else {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), (body_h * 0.48).max(180.0)), Sense::hover());
        illustrations::draw(ui.painter(), rect, slide.illustration);
        ui.add_space(12.0);
        egui::ScrollArea::vertical().show(ui, |ui| points(app, ui, slide, k, count));
    }
}

fn points(app: &mut App, ui: &mut Ui, slide: &Slide, k: f32, count: usize) {
    for point in slide.points {
        ui.horizontal_top(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(18.0 * k, 26.0 * k), Sense::hover());
            ui.painter().circle_filled(egui::pos2(r.left() + 5.0 * k, r.top() + 13.0 * k), 3.5 * k, theme::SELECT);
            ui.vertical(|ui| {
                ui.label(RichText::new(*point).font(theme::regular(19.0 * k)).color(theme::INK));
            });
        });
        ui.add_space(10.0 * k);
    }
    if let Some(note) = slide.note {
        ui.add_space(6.0 * k);
        egui::Frame::default()
            .fill(theme::SURFACE_2)
            .corner_radius(cr(10.0))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.label(RichText::new(note).font(theme::regular(14.5 * k)).color(theme::INK_2));
            });
    }
    // Sista bilden: vägar vidare.
    if app.learn_slide + 1 == count {
        ui.add_space(16.0 * k);
        let all = lessons::all();
        for (i, lesson) in all.iter().enumerate() {
            let title = lesson.title.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' ');
            if ui_kit::primary(ui, true, &format!("Träning {}: {title}  ›", i + 1)).clicked() {
                set_fullscreen(ui.ctx(), false);
                app.load_model((lesson.start)());
                app.lesson = Some(LessonState { lesson: i, step: 0 });
            }
            ui.add_space(6.0);
        }
        if ui_kit::ghost(ui, true, "Till startsidan").clicked() {
            set_fullscreen(ui.ctx(), false);
            app.screen = Screen::Start;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_fullscreen(ctx: &Context) -> bool {
    ctx.input(|i| i.viewport().fullscreen.unwrap_or(false))
}

#[cfg(target_arch = "wasm32")]
fn is_fullscreen(_ctx: &Context) -> bool {
    web_sys::window().and_then(|w| w.document()).is_some_and(|d| d.fullscreen_element().is_some())
}

#[cfg(not(target_arch = "wasm32"))]
fn set_fullscreen(ctx: &Context, on: bool) {
    if is_fullscreen(ctx) != on {
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(on));
    }
}

#[cfg(target_arch = "wasm32")]
fn set_fullscreen(_ctx: &Context, on: bool) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
    if on {
        if let Some(el) = document.document_element() {
            let _ = el.request_fullscreen();
        }
    } else if document.fullscreen_element().is_some() {
        document.exit_fullscreen();
    }
}
