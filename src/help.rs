//! Hjälpen: uppdelad i avsnitt med en meny till vänster.

use crate::model::NodeKind;
use crate::theme::{self, cr};
use crate::ui_kit;
use egui::{vec2, Color32, Context, CursorIcon, Margin, RichText, Sense, Ui};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Topic {
    Start,
    Cld,
    Sfd,
    Formulas,
    Results,
    Files,
    Keys,
}

const TOPICS: [(Topic, &str); 7] = [
    (Topic::Start, "Kom igång"),
    (Topic::Cld, "Orsaksdiagram"),
    (Topic::Sfd, "Lager och flöden"),
    (Topic::Formulas, "Formler"),
    (Topic::Results, "Graf och reglage"),
    (Topic::Files, "Spara och dela"),
    (Topic::Keys, "Kortkommandon"),
];

/// Visar hjälpfönstret. Ger `true` om användaren vill öppna presentationen.
pub fn window(ctx: &Context, open: &mut bool) -> bool {
    let topic_id = egui::Id::new("help_topic");
    let mut topic = ctx.data(|d| d.get_temp::<Topic>(topic_id)).unwrap_or(Topic::Start);
    let mut open_presentation = false;
    // Fast storlek, så att fönstret inte hoppar när man byter avsnitt.
    let screen = ctx.screen_rect();
    let size = vec2((screen.width() - 64.0).clamp(420.0, 820.0), (screen.height() - 140.0).clamp(300.0, 580.0));
    egui::Window::new(theme::title("Hjälp", 19.0))
        .open(open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(size)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 24.0))
        .frame(
            egui::Frame::window(&ctx.style()).inner_margin(Margin::same(0)).fill(theme::SURFACE),
        )
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let h = size.y;
                ui.allocate_ui_with_layout(vec2(200.0, h), egui::Layout::top_down(egui::Align::Min), |ui| {
                    egui::Frame::default().fill(theme::SURFACE_2).inner_margin(Margin::same(10)).show(ui, |ui| {
                        ui.set_min_size(vec2(180.0, h - 20.0));
                        for (t, label) in TOPICS {
                            if nav_item(ui, label, topic == t).clicked() {
                                topic = t;
                            }
                        }
                        ui.add_space(16.0);
                        if ui_kit::ghost(ui, true, "🎓  Öppna presentationen").clicked() {
                            open_presentation = true;
                        }
                    });
                });
                ui.allocate_ui_with_layout(vec2(ui.available_width(), h), egui::Layout::top_down(egui::Align::Min), |ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        egui::Frame::default().inner_margin(Margin { left: 24, right: 24, top: 16, bottom: 24 }).show(ui, |ui| {
                            ui.set_max_width(620.0);
                            match topic {
                                Topic::Start => start(ui, &mut open_presentation),
                                Topic::Cld => cld(ui),
                                Topic::Sfd => sfd(ui),
                                Topic::Formulas => formulas(ui),
                                Topic::Results => results(ui),
                                Topic::Files => files(ui),
                                Topic::Keys => keys(ui),
                            }
                        });
                    });
                });
            });
        });
    ctx.data_mut(|d| d.insert_temp(topic_id, topic));
    open_presentation
}

fn nav_item(ui: &mut Ui, label: &str, selected: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, cr(8.0), theme::SURFACE);
        painter.rect_filled(egui::Rect::from_min_size(rect.min + vec2(0.0, 8.0), vec2(3.0, 20.0)), cr(2.0), theme::SELECT);
    } else if resp.hovered() {
        painter.rect_filled(rect, cr(8.0), theme::HOVER);
    }
    let font = if selected { theme::semibold(15.0) } else { theme::regular(15.0) };
    let color = if selected { theme::INK } else { theme::INK_2 };
    painter.text(rect.left_center() + vec2(14.0, 0.0), egui::Align2::LEFT_CENTER, label, font, color);
    resp.on_hover_cursor(CursorIcon::PointingHand)
}

// ---------------------------------------------------------------- byggstenar

fn h1(ui: &mut Ui, text: &str) {
    ui.label(theme::title(text, 23.0));
    ui.add_space(6.0);
}

fn h2(ui: &mut Ui, text: &str) {
    ui.add_space(14.0);
    ui.label(theme::strong(text, 17.0));
    ui.add_space(2.0);
}

fn p(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).font(theme::regular(15.5)).color(theme::INK_2));
    ui.add_space(4.0);
}

/// En rad med en markering till vänster och en förklaring.
fn term(ui: &mut Ui, add_left: impl FnOnce(&mut Ui), text: &str) {
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(vec2(150.0, 28.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(150.0);
            add_left(ui);
        });
        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.label(RichText::new(text).font(theme::regular(15.0)).color(theme::INK));
        });
    });
    ui.add_space(6.0);
}

fn label_term(ui: &mut Ui, label: &str, color: Color32, text: &str) {
    term(ui, |ui| { ui.label(RichText::new(label).font(theme::bold(15.0)).color(color)); }, text);
}

fn code(ui: &mut Ui, text: &str) {
    egui::Frame::default().fill(theme::SURFACE_2).corner_radius(cr(8.0)).inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(text).monospace().size(15.0).color(theme::INK));
    });
    ui.add_space(6.0);
}

/// En tangent ritad som en liten tangent.
fn key(ui: &mut Ui, text: &str) {
    let galley = ui.painter().layout_no_wrap(text.to_string(), theme::semibold(13.5), theme::INK);
    let (rect, _) = ui.allocate_exact_size(galley.size() + vec2(16.0, 10.0), Sense::hover());
    ui.painter().rect(rect, cr(6.0), theme::SURFACE, egui::Stroke::new(1.0_f32, theme::LINE_STRONG), egui::StrokeKind::Inside);
    ui.painter().line_segment([rect.left_bottom() + vec2(3.0, -1.0), rect.right_bottom() + vec2(-3.0, -1.0)], egui::Stroke::new(2.0_f32, theme::LINE));
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, theme::INK);
}

fn shortcut(ui: &mut Ui, keys: &[&str], text: &str) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(170.0, 30.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(170.0);
            ui.spacing_mut().item_spacing.x = 4.0;
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new("+").color(theme::MUTED));
                }
                key(ui, k);
            }
        });
        ui.label(RichText::new(text).font(theme::regular(15.0)).color(theme::INK));
    });
}

// ---------------------------------------------------------------- avsnitt

fn start(ui: &mut Ui, open_presentation: &mut bool) {
    h1(ui, "Kom igång");
    p(ui, "Modellverkstan hjälper dig att förstå system: saker som hänger ihop och förändras över tid. Det finns tre sätt att börja:");
    label_term(ui, "1. Lär dig", theme::INK, "En presentation med bilder om vad modeller är, lager, flöden och loopar.");
    label_term(ui, "2. Träna", theme::SUCCESS, "Bygg en modell steg för steg. Programmet säger till när du har gjort rätt.");
    label_term(ui, "3. Utforska", theme::STOCK, "Öppna ett färdigt exempel och dra i reglagen.");
    ui.add_space(6.0);
    if ui_kit::primary(ui, true, "🎓  Öppna presentationen").clicked() {
        *open_presentation = true;
    }
    h2(ui, "Två sorters modeller");
    label_term(ui, "Orsaksdiagram", theme::REINFORCING, "Visar vad som påverkar vad, med pilar och + / −. Bra för att tänka och diskutera.");
    label_term(ui, "Flödesmodell", theme::STOCK, "Lager och flöden som datorn räknar ut. Bra för att se vad som händer över tid.");
    h2(ui, "Så är skärmen uppbyggd");
    p(ui, "Till vänster finns verktygen. I mitten ritar du modellen. Till höger ser du det du har markerat, eller information om modellen. Under en flödesmodell finns reglagen och grafen.");
}

fn cld(ui: &mut Ui) {
    h1(ui, "Orsaksdiagram");
    p(ui, "Ett orsaksdiagram visar hur saker påverkar varandra.");
    h2(ui, "Rita");
    label_term(ui, "Variabel", theme::INK, "Dubbelklicka på ytan, eller välj Variabel och klicka. Skriv sedan namnet.");
    label_term(ui, "Pil", theme::INK, "Välj Pil och dra från en variabel till en annan.");
    label_term(ui, "Byt + / −", theme::INK, "Klicka på pilens tecken, eller markera pilen och välj till höger.");
    label_term(ui, "Böj pilen", theme::INK, "Markera pilen och dra i den lilla ringen mitt på den.");
    h2(ui, "Plus och minus");
    label_term(ui, "+", theme::POSITIVE, "Samma riktning: ökar A så ökar B, och minskar A så minskar B.");
    label_term(ui, "−", theme::NEGATIVE, "Motsatt riktning: ökar A så minskar B.");
    h2(ui, "Loopar");
    p(ui, "När pilarna leder runt och tillbaka har du en loop. Programmet hittar dem och visar dem till höger. Håll musen över en loop för att se var den går.");
    label_term(ui, "R  förstärkande", theme::REINFORCING, "Förändringen växer sig större. Inga eller ett jämnt antal minuspilar.");
    label_term(ui, "B  balanserande", theme::BALANCING, "Systemet strävar mot jämvikt. Ett udda antal minuspilar.");
}

fn sfd(ui: &mut Ui) {
    h1(ui, "Lager och flöden");
    p(ui, "En flödesmodell byggs av fyra byggstenar. Datorn räknar ut den direkt när du ändrar något.");
    h2(ui, "Byggstenarna");
    term(ui, |ui| ui_kit::kind_tag(ui, NodeKind::Stock), "Något som samlas: vatten i ett badkar, pengar, energi. Skriv ett startvärde.");
    term(ui, |ui| ui_kit::kind_tag(ui, NodeKind::Flow), "Fyller på eller tömmer ett lager, per tidsenhet. Dra från ett lager till ett annat.");
    term(ui, |ui| ui_kit::kind_tag(ui, NodeKind::Constant), "Ett värde du bestämmer. Det får ett reglage under Reglage.");
    term(ui, |ui| ui_kit::kind_tag(ui, NodeKind::Aux), "Räknar ut något med en formel, så att andra formler blir enklare.");
    h2(ui, "Rita flöden");
    p(ui, "Välj Flöde och dra från ett lager till ett annat. Börjar eller slutar du på tom yta kommer flödet från, eller går till, ett moln. Ett moln betyder något utanför modellen.");
    h2(ui, "Grå pilar");
    p(ui, "De grå pilarna ritas automatiskt och visar vilka delar en formel använder. Markera en del för att se dess pilar tydligare.");
    h2(ui, "Hur datorn räknar");
    p(ui, "Datorn tar ett litet tidssteg i taget: varje lager ändras med (inflöden − utflöden) × tidssteget. Tidssteget ändrar du under Tid ovanför grafen.");
}

fn formulas(ui: &mut Ui) {
    h1(ui, "Formler");
    p(ui, "Skriv namnen på andra delar precis som de heter. Stora och små bokstäver spelar ingen roll. Du kan också klicka på namnen under formelrutan.");
    code(ui, "Vatten i badkaret * Avloppets storlek");
    code(ui, "(Inomhustemperatur - Utetemperatur) * 0.15");
    code(ui, "om(Laddnivå < 80; Laddeffekt max; 2)");
    h2(ui, "Räknesätt och jämförelser");
    label_term(ui, "+  −  *  /", theme::INK, "Plus, minus, gånger, delat med.");
    label_term(ui, "^", theme::INK, "Upphöjt till, t.ex. 2^3 = 8.");
    label_term(ui, "< > <= >= = <>", theme::INK, "Jämförelser. De blir 1 (sant) eller 0 (falskt).");
    label_term(ui, "och  eller  inte", theme::INK, "Kombinera villkor.");
    h2(ui, "Funktioner");
    label_term(ui, "om(v; a; b)", theme::AUX, "Blir a om villkoret v är sant, annars b.");
    label_term(ui, "min(a; b)", theme::AUX, "Det minsta av värdena.");
    label_term(ui, "max(a; b)", theme::AUX, "Det största av värdena.");
    label_term(ui, "abs(a)", theme::AUX, "Talet utan minustecken.");
    label_term(ui, "rot(a)", theme::AUX, "Kvadratroten.");
    label_term(ui, "avrunda(a)", theme::AUX, "Avrundar till heltal.");
    label_term(ui, "tid", theme::AUX, "Hur lång tid som har gått i simuleringen.");
    h2(ui, "Decimaltal");
    p(ui, "Skriv 0.15 eller 0,15. Inuti en funktion måste du använda punkt, eftersom semikolon eller komma skiljer värdena åt.");
    h2(ui, "Om något blir rött");
    p(ui, "Ett rött band överst på ytan betyder att något behöver rättas. Klicka på det, så markeras delen och felet förklaras till höger.");
}

fn results(ui: &mut Ui) {
    h1(ui, "Graf och reglage");
    label_term(ui, "Reglage", theme::CONST, "Varje konstant får ett reglage till vänster om grafen. Dra, så ändras grafen direkt.");
    label_term(ui, "Visa i grafen", theme::INK, "Klicka på de färgade knapparna ovanför grafen för att visa eller dölja något.");
    label_term(ui, "▶ Spela upp", theme::INK, "Se hur modellen ändras över tid. Lagren fylls och töms på ytan.");
    label_term(ui, "📌 Jämför", theme::INK, "Sparar kurvorna som streckade linjer. Ändra sedan något och jämför.");
    label_term(ui, "🖼 Bild", theme::INK, "Spara grafen som bild eller kopiera den, t.ex. till en labbrapport.");
    label_term(ui, "⏱ Tid", theme::INK, "Ändra tidsenhet, hur länge modellen ska räknas och tidssteget.");
    label_term(ui, "Tabell", theme::INK, "Visar värdena som siffror. Kopiera tabellen till Excel eller Google Kalkylark.");
}

fn files(ui: &mut Ui) {
    h1(ui, "Spara och dela");
    label_term(ui, "Spara", theme::INK, "Sparar modellen som en fil (.mvm) som du kan skicka till någon annan.");
    label_term(ui, "Öppna", theme::INK, "Öppnar en sparad .mvm-fil.");
    label_term(ui, "Automatiskt", theme::INK, "Modellen du arbetar med sparas av sig själv. Den finns under \"Fortsätt där du var\" på startsidan.");
    h2(ui, "Webben och programmet");
    p(ui, "Modellverkstan finns både som webbsida och som program för Windows. Filerna fungerar i båda. På webben sparas din senaste modell i webbläsaren på just den datorn.");
}

fn keys(ui: &mut Ui) {
    h1(ui, "Kortkommandon");
    h2(ui, "Redigera");
    shortcut(ui, &["Ctrl", "Z"], "Ångra");
    shortcut(ui, &["Ctrl", "Y"], "Gör om");
    shortcut(ui, &["Delete"], "Ta bort det markerade");
    shortcut(ui, &["Esc"], "Avmarkera och gå tillbaka till Markera");
    shortcut(ui, &["Ctrl", "S"], "Spara");
    shortcut(ui, &["Ctrl", "O"], "Öppna");
    h2(ui, "Ytan");
    shortcut(ui, &["Scrollhjul"], "Flytta ytan");
    shortcut(ui, &["Ctrl", "Scrollhjul"], "Zooma");
    p(ui, "Du kan också dra på en tom del av ytan för att flytta den.");
    h2(ui, "Presentationen");
    shortcut(ui, &["→"], "Nästa bild (eller mellanslag)");
    shortcut(ui, &["←"], "Föregående bild");
    shortcut(ui, &["Esc"], "Avsluta presentationen");
}
