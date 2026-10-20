//! Hjälpfönstret.

use crate::theme;
use egui::{Context, RichText, Ui};

fn h(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).size(18.0).strong().color(theme::ACCENT));
}

fn p(ui: &mut Ui, text: &str) {
    ui.label(text);
}

fn row(ui: &mut Ui, term: &str, color: egui::Color32, text: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(term).strong().color(color));
        ui.label(text);
    });
}

pub fn window(ctx: &Context, open: &mut bool) {
    egui::Window::new("Hjälp – så fungerar Modellverkstan")
        .open(open)
        .default_width(560.0)
        .default_height(560.0)
        .vscroll(true)
        .show(ctx, |ui| {
            h(ui, "Vad är en modell?");
            p(ui, "En modell är en förenklad bild av verkligheten. Den hjälper oss att förstå hur ett system fungerar \
                   och att testa \"vad händer om…?\" utan att behöva prova på riktigt.");

            h(ui, "Orsaksdiagram");
            p(ui, "Ett orsaksdiagram visar hur saker påverkar varandra.");
            row(ui, "Variabel", theme::INK, "– något som kan öka eller minska. Dubbelklicka på ytan för att skapa en.");
            row(ui, "Pil +", theme::POSITIVE, "– samma riktning: ökar A så ökar B (och minskar A så minskar B).");
            row(ui, "Pil −", theme::NEGATIVE, "– motsatt riktning: ökar A så minskar B.");
            row(ui, "Loop R", theme::REINFORCING, "– förstärkande: en förändring växer sig större (jämnt antal minus).");
            row(ui, "Loop B", theme::BALANCING, "– balanserande: systemet strävar mot jämvikt (udda antal minus).");
            p(ui, "Rita pilar med verktyget Pil: dra från en variabel till en annan. Klicka på en pil för att byta + / −.");

            h(ui, "Flödesmodell (lager och flöden)");
            row(ui, "Lager", theme::STOCK, "– något som samlas: vatten i ett badkar, energi i ett batteri, värme i ett hus.");
            row(ui, "Flöde", theme::FLOW, "– fyller på eller tömmer ett lager, t.ex. liter per minut eller kW. Dra från ett lager till ett annat. \
                 Ett moln betyder att det kommer från eller går till något utanför modellen.");
            row(ui, "Konstant", theme::CONST, "– ett värde du bestämmer och kan ändra med ett reglage.");
            row(ui, "Hjälpvariabel", theme::AUX, "– räknar ut något med en formel.");
            p(ui, "De grå pilarna ritas automatiskt och visar vilka delar som används i varje formel.");
            p(ui, "Modellen räknas ut direkt när du ändrar något. Datorn tar ett litet tidssteg i taget: \
                   lagret ändras med (inflöden − utflöden) × tidssteget.");

            h(ui, "Formler");
            p(ui, "Skriv namnen på andra delar precis som de heter (stora/små bokstäver spelar ingen roll). Exempel:");
            ui.label(RichText::new("Vatten i badkaret * Avloppets storlek").monospace());
            ui.label(RichText::new("(Inomhustemperatur - Utetemperatur) * 0.15").monospace());
            ui.label(RichText::new("om(Laddnivå < 80; Laddeffekt max; 2)").monospace());
            p(ui, "Räknesätt: + − * / ^ (upphöjt till) och parenteser.");
            p(ui, "Jämförelser: < > <= >= = <>  och  och / eller / inte.");
            p(ui, "Funktioner: om(villkor; då; annars), min(A; B), max(A; B), abs(A), rot(A), avrunda(A).");
            p(ui, "Ordet tid ger tiden som har gått i simuleringen.");
            p(ui, "Decimaltal: 0.15 eller 0,15 – men inuti en funktion måste du använda punkt, eftersom komma/semikolon skiljer värdena åt.");

            h(ui, "Kortkommandon");
            p(ui, "Ctrl+Z ångra · Ctrl+Y gör om · Delete ta bort · Esc avmarkera · Ctrl+S spara · Ctrl+O öppna");
            p(ui, "Scrollhjul flyttar ytan, Ctrl + scrollhjul zoomar. Du kan också dra på tom yta för att flytta.");
        });
}
