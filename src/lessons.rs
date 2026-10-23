//! Träningar: guidade lektioner där eleven bygger en modell steg för steg.
//!
//! Varje steg har en förklaring och oftast en uppgift. Uppgiften kontrolleras
//! automatiskt (`check`) så att eleven ser när den är klar, och `solve` gör
//! steget åt eleven om hen fastnar ("Visa hur"). Kontrollerna är förlåtande:
//! de letar efter rätt sorts delar och kopplingar, inte exakta namn.

use crate::app::Tool;
use crate::model::{Link, Model, ModelKind, Node, NodeKind};
use crate::sim::Compiled;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LessonState {
    pub lesson: usize,
    pub step: usize,
}

pub struct Step {
    pub title: &'static str,
    pub text: &'static str,
    /// Vad eleven ska göra. `None` = ett steg som bara förklarar.
    pub task: Option<&'static str>,
    /// Verktyget som markeras i verktygslådan under uppgiften.
    pub tool: Option<Tool>,
    pub check: fn(&Model, &Compiled) -> bool,
    pub solve: fn(&mut Model),
    /// Visas när uppgiften är klar.
    pub done_text: &'static str,
}

pub struct Lesson {
    pub title: &'static str,
    pub summary: &'static str,
    pub kind: ModelKind,
    pub start: fn() -> Model,
    pub steps: Vec<Step>,
}

pub fn all() -> Vec<Lesson> {
    vec![rabbits_cld(), rabbits_sfd()]
}

fn info(title: &'static str, text: &'static str) -> Step {
    Step { title, text, task: None, tool: None, check: |_, _| true, solve: |_| {}, done_text: "" }
}

#[allow(clippy::too_many_arguments)]
fn task(
    title: &'static str,
    text: &'static str,
    task: &'static str,
    tool: Tool,
    check: fn(&Model, &Compiled) -> bool,
    solve: fn(&mut Model),
    done_text: &'static str,
) -> Step {
    Step { title, text, task: Some(task), tool: Some(tool), check, solve, done_text }
}

// ------------------------------------------------------------ hjälpfunktioner

/// Första delen av ett visst slag vars namn innehåller `has` men inte `not`.
fn find<'a>(m: &'a Model, kind: NodeKind, has: &str, not: &str) -> Option<&'a Node> {
    m.nodes.iter().find(|n| {
        let name = n.name.to_lowercase();
        n.kind == kind && name.contains(has) && (not.is_empty() || !name.contains(not))
    })
}

fn find_id(m: &Model, kind: NodeKind, has: &str, not: &str) -> Option<u32> {
    find(m, kind, has, not).map(|n| n.id)
}

fn name_of(m: &Model, id: u32) -> String {
    m.node(id).map_or(String::new(), |n| n.name.trim().to_string())
}

/// En position i förhållande till en annan del.
fn near(m: &Model, id: u32, dx: f32, dy: f32) -> [f32; 2] {
    let p = m.node(id).map_or([400.0, 300.0], |n| n.pos);
    [p[0] + dx, p[1] + dy]
}

/// Hittar en del, eller skapar den med det givna namnet.
fn ensure(m: &mut Model, kind: NodeKind, (has, not): (&str, &str), name: &str, pos: [f32; 2]) -> u32 {
    if let Some(id) = find_id(m, kind, has, not) {
        return id;
    }
    let id = m.add_node(kind, pos);
    m.node_mut(id).unwrap().name = name.into();
    id
}

fn has_link(m: &Model, from: Option<u32>, to: Option<u32>, positive: bool) -> bool {
    let (Some(from), Some(to)) = (from, to) else { return false };
    m.links.iter().any(|l| l.from == from && l.to == to && l.positive == positive)
}

fn set_link(m: &mut Model, from: u32, to: u32, positive: bool) {
    match m.links.iter_mut().find(|l| l.from == from && l.to == to) {
        Some(l) => l.positive = positive,
        None => m.links.push(Link { from, to, positive, curve: 0.25 }),
    }
}

fn ok(c: &Compiled, id: u32) -> bool {
    !c.errors.contains_key(&id)
}

/// Om formeln för `node` använder `dep`.
fn uses(c: &Compiled, node: u32, dep: Option<u32>) -> bool {
    dep.is_some_and(|dep| c.deps.contains(&(dep, node)))
}

// ------------------------------------------------------------ orsaksdiagram

const RABBITS: (&str, &str) = ("kanin", "mat");
const BIRTHS: (&str, &str) = ("föd", "");
const DEATHS: (&str, &str) = ("död", "");
const FOOD_PER: (&str, &str) = ("mat", "");

fn var(m: &Model, (has, not): (&str, &str)) -> Option<u32> {
    find_id(m, NodeKind::Variable, has, not)
}

fn cld_vars_done(m: &Model, _: &Compiled) -> bool {
    var(m, RABBITS).is_some() && var(m, BIRTHS).is_some()
}

fn cld_vars_solve(m: &mut Model) {
    ensure(m, NodeKind::Variable, RABBITS, "Antal kaniner", [400.0, 320.0]);
    ensure(m, NodeKind::Variable, BIRTHS, "Födslar", [150.0, 140.0]);
}

fn cld_births_done(m: &Model, _: &Compiled) -> bool {
    let (k, f) = (var(m, RABBITS), var(m, BIRTHS));
    has_link(m, k, f, true) && has_link(m, f, k, true)
}

fn cld_births_solve(m: &mut Model) {
    cld_vars_solve(m);
    let (k, f) = (var(m, RABBITS).unwrap(), var(m, BIRTHS).unwrap());
    set_link(m, k, f, true);
    set_link(m, f, k, true);
}

fn cld_deaths_done(m: &Model, _: &Compiled) -> bool {
    let (k, d) = (var(m, RABBITS), var(m, DEATHS));
    has_link(m, k, d, true) && has_link(m, d, k, false)
}

fn cld_deaths_solve(m: &mut Model) {
    cld_births_solve(m);
    let k = var(m, RABBITS).unwrap();
    let d = ensure(m, NodeKind::Variable, DEATHS, "Dödsfall", [650.0, 140.0]);
    set_link(m, k, d, true);
    set_link(m, d, k, false);
}

fn cld_food_done(m: &Model, _: &Compiled) -> bool {
    let (k, f, mat) = (var(m, RABBITS), var(m, BIRTHS), var(m, FOOD_PER));
    has_link(m, k, mat, false) && has_link(m, mat, f, true)
}

fn cld_food_solve(m: &mut Model) {
    cld_deaths_solve(m);
    let (k, f) = (var(m, RABBITS).unwrap(), var(m, BIRTHS).unwrap());
    let mat = ensure(m, NodeKind::Variable, FOOD_PER, "Mat per kanin", [400.0, 60.0]);
    set_link(m, k, mat, false);
    set_link(m, mat, f, true);
}

fn rabbits_cld() -> Lesson {
    Lesson {
        title: "1. Kaninerna som orsaksdiagram",
        summary: "Lär dig variabler, pilar med + och −, och hitta förstärkande och balanserande loopar. Sex korta steg.",
        kind: ModelKind::Cld,
        start: || {
            let mut m = Model::new(ModelKind::Cld);
            m.title = "Kaninerna på ön (orsaksdiagram)".into();
            m
        },
        steps: vec![
            info(
                "Vad är ett orsaksdiagram?",
                "Ett orsaksdiagram visar hur saker påverkar varandra. Vi ska undersöka kaninerna på en ö: \
                 varför blir de fler – och varför blir de inte hur många som helst?\n\n\
                 I varje steg får du en uppgift. Programmet bockar av när du har gjort rätt. \
                 Fastnar du kan du trycka på \"Visa hur\".",
            ),
            task(
                "Två variabler",
                "En VARIABEL är något som kan öka eller minska, till exempel antalet kaniner \
                 eller hur många kaniner som föds varje månad.",
                "Dubbelklicka på ytan för att skapa en variabel och döp den till Antal kaniner. \
                 Skapa en till som heter Födslar.",
                Tool::Place(NodeKind::Variable),
                cld_vars_done,
                cld_vars_solve,
                "Bra! Nu ska vi visa hur de hänger ihop.",
            ),
            task(
                "Pilar med plus",
                "En pil betyder \"påverkar\". Ett PLUS (+) betyder att båda ändras åt samma håll: \
                 fler kaniner ger fler födslar – och fler födslar ger fler kaniner.",
                "Välj verktyget Pil. Dra en pil från Antal kaniner till Födslar, \
                 och en pil tillbaka från Födslar till Antal kaniner. Nya pilar får +.",
                Tool::Link,
                cld_births_done,
                cld_births_solve,
                "Titta – programmet hittade en loop och märkte den R1! R betyder FÖRSTÄRKANDE: \
                 fler kaniner → fler födslar → ännu fler kaniner. En förändring växer sig större och större. \
                 Det är därför kaniner kan bli väldigt många väldigt fort.",
            ),
            task(
                "Kaniner dör också",
                "Kaniner lever inte för evigt. Ju fler kaniner, desto fler dör – det är ett plus. \
                 Men fler dödsfall ger FÄRRE kaniner. De ändras åt olika håll, och då ska pilen ha ett MINUS (−).",
                "Skapa variabeln Dödsfall. Rita en pil från Antal kaniner till Dödsfall och en från \
                 Dödsfall till Antal kaniner. Klicka på den sista pilens + så att den blir −.",
                Tool::Link,
                cld_deaths_done,
                cld_deaths_solve,
                "Nu finns loop B1. B betyder BALANSERANDE: fler kaniner → fler dödsfall → färre kaniner. \
                 Loopen motverkar förändringen och försöker hålla antalet stabilt.",
            ),
            task(
                "Maten räcker inte till alla",
                "På ön finns bara en viss mängd mat. Fler kaniner betyder MINDRE mat per kanin (−). \
                 Och mer mat per kanin ger fler ungar (+).",
                "Skapa variabeln Mat per kanin. Rita en pil från Antal kaniner till Mat per kanin och gör den −. \
                 Rita sedan en pil från Mat per kanin till Födslar (+).",
                Tool::Link,
                cld_food_done,
                cld_food_solve,
                "En balanserande loop till, B2: fler kaniner → mindre mat per kanin → färre födslar → färre kaniner. \
                 Håll musen över looparna i listan till höger för att se var de går.",
            ),
            info(
                "Vilken loop vinner?",
                "Du har hittat tre loopar: R1 får kaninerna att öka, B1 och B2 bromsar. \
                 Men vilken loop är starkast? Det kan man inte se i ett orsaksdiagram – \
                 det visar HUR saker hänger ihop, men inte HUR MYCKET.\n\n\
                 För att ta reda på det behöver vi räkna. Det gör du i nästa träning, där du bygger \
                 samma kaniner med lager och flöden.",
            ),
        ],
    }
}

// ------------------------------------------------------------ lager och flöden

fn stock(m: &Model) -> Option<&Node> {
    find(m, NodeKind::Stock, "kanin", "").or_else(|| m.nodes.iter().find(|n| n.kind == NodeKind::Stock))
}

fn stock_id(m: &Model) -> Option<u32> {
    stock(m).map(|n| n.id)
}

fn inflow(m: &Model) -> Option<u32> {
    let s = stock_id(m)?;
    m.nodes.iter().find(|n| n.kind == NodeKind::Flow && n.to == Some(s)).map(|n| n.id)
}

fn outflow(m: &Model) -> Option<u32> {
    let s = stock_id(m)?;
    m.nodes.iter().find(|n| n.kind == NodeKind::Flow && n.from == Some(s)).map(|n| n.id)
}

fn birth_rate(m: &Model) -> Option<u32> {
    find_id(m, NodeKind::Constant, "föd", "")
}

fn death_rate(m: &Model) -> Option<u32> {
    find_id(m, NodeKind::Constant, "död", "")
}

fn food(m: &Model) -> Option<u32> {
    find_id(m, NodeKind::Constant, "mat", "")
}

fn food_per(m: &Model) -> Option<u32> {
    find_id(m, NodeKind::Aux, "mat", "").or_else(|| m.nodes.iter().find(|n| n.kind == NodeKind::Aux).map(|n| n.id))
}

fn set_formula(m: &mut Model, id: u32, formula: String) {
    if let Some(n) = m.node_mut(id) {
        n.formula = formula;
    }
}

fn constant(m: &mut Model, id: u32, value: f64, max: f64, unit: &str) {
    if let Some(n) = m.node_mut(id) {
        n.value = value;
        n.min = 0.0;
        n.max = max;
        n.unit = unit.into();
    }
}

fn sfd_stock_done(m: &Model, c: &Compiled) -> bool {
    stock(m).is_some_and(|s| s.name.to_lowercase().contains("kanin") && ok(c, s.id) && s.formula.trim() != "0")
}

fn sfd_stock_solve(m: &mut Model) {
    let id = match stock_id(m) {
        Some(id) => id,
        None => m.add_node(NodeKind::Stock, [400.0, 280.0]),
    };
    let n = m.node_mut(id).unwrap();
    if !n.name.to_lowercase().contains("kanin") {
        n.name = "Kaniner".into();
    }
    if n.formula.trim() == "0" {
        n.formula = "10".into();
    }
    n.in_graph = true;
}

fn sfd_inflow_done(m: &Model, _: &Compiled) -> bool {
    inflow(m).is_some()
}

fn sfd_inflow_solve(m: &mut Model) {
    sfd_stock_solve(m);
    if inflow(m).is_none() {
        let s = stock_id(m).unwrap();
        let id = m.add_node(NodeKind::Flow, near(m, s, -230.0, 0.0));
        let n = m.node_mut(id).unwrap();
        n.name = "Födslar".into();
        n.to = Some(s);
        n.formula = "1".into();
        n.unit = "kaniner/månad".into();
    }
}

fn sfd_rate_done(m: &Model, _: &Compiled) -> bool {
    birth_rate(m).and_then(|id| m.node(id)).is_some_and(|n| n.value > 0.0)
}

fn sfd_rate_solve(m: &mut Model) {
    sfd_inflow_solve(m);
    let s = stock_id(m).unwrap();
    let pos = near(m, s, -230.0, -160.0);
    if birth_rate(m).is_none() {
        let id = ensure(m, NodeKind::Constant, ("föd", ""), "Födelsetal", pos);
        constant(m, id, 0.2, 1.0, "per månad");
    }
}

fn sfd_births_done(m: &Model, c: &Compiled) -> bool {
    inflow(m).is_some_and(|f| ok(c, f) && uses(c, f, stock_id(m)) && uses(c, f, birth_rate(m)))
}

fn sfd_births_solve(m: &mut Model) {
    sfd_rate_solve(m);
    let (s, f, r) = (stock_id(m).unwrap(), inflow(m).unwrap(), birth_rate(m).unwrap());
    let formula = format!("{} * {}", name_of(m, s), name_of(m, r));
    set_formula(m, f, formula);
}

fn sfd_deaths_done(m: &Model, c: &Compiled) -> bool {
    outflow(m).is_some_and(|f| ok(c, f) && uses(c, f, stock_id(m)) && uses(c, f, death_rate(m)))
}

fn sfd_deaths_solve(m: &mut Model) {
    sfd_births_solve(m);
    let s = stock_id(m).unwrap();
    let pos = near(m, s, 230.0, -160.0);
    let r = match death_rate(m) {
        Some(r) => r,
        None => {
            let r = ensure(m, NodeKind::Constant, ("död", ""), "Dödstal", pos);
            constant(m, r, 0.1, 1.0, "per månad");
            r
        }
    };
    let f = match outflow(m) {
        Some(f) => f,
        None => {
            let id = m.add_node(NodeKind::Flow, near(m, s, 230.0, 0.0));
            let n = m.node_mut(id).unwrap();
            n.name = "Dödsfall".into();
            n.from = Some(s);
            n.unit = "kaniner/månad".into();
            id
        }
    };
    let formula = format!("{} * {}", name_of(m, s), name_of(m, r));
    set_formula(m, f, formula);
}

fn sfd_food_done(m: &Model, c: &Compiled) -> bool {
    food_per(m).is_some_and(|a| ok(c, a) && uses(c, a, stock_id(m)) && uses(c, a, food(m)))
}

fn sfd_food_solve(m: &mut Model) {
    sfd_deaths_solve(m);
    let s = stock_id(m).unwrap();
    let pos = near(m, s, 210.0, -300.0);
    let f = match food(m) {
        Some(f) => f,
        None => {
            let f = ensure(m, NodeKind::Constant, ("mat", ""), "Mat på ön", pos);
            constant(m, f, 500.0, 2000.0, "portioner/månad");
            f
        }
    };
    let pos = near(m, s, 0.0, -160.0);
    let a = match food_per(m) {
        Some(a) => a,
        None => ensure(m, NodeKind::Aux, ("mat", ""), "Mat per kanin", pos),
    };
    let formula = format!("{} / {}", name_of(m, f), name_of(m, s));
    set_formula(m, a, formula);
    if let Some(n) = m.node_mut(a) {
        n.unit = "portioner".into();
    }
}

fn sfd_limit_done(m: &Model, c: &Compiled) -> bool {
    inflow(m).is_some_and(|f| ok(c, f) && uses(c, f, food_per(m)) && uses(c, f, stock_id(m)))
}

fn sfd_limit_solve(m: &mut Model) {
    sfd_food_solve(m);
    let (s, f, r, a) = (stock_id(m).unwrap(), inflow(m).unwrap(), birth_rate(m).unwrap(), food_per(m).unwrap());
    let formula = format!("{} * {} * min(1; {})", name_of(m, s), name_of(m, r), name_of(m, a));
    set_formula(m, f, formula);
}

fn rabbits_sfd() -> Lesson {
    Lesson {
        title: "2. Kaninerna med lager och flöden",
        summary: "Bygg en modell som går att köra. Lär dig lager, flöden, konstanter och se vilken loop som vinner. Nio steg.",
        kind: ModelKind::Sfd,
        start: || {
            let mut m = Model::new(ModelKind::Sfd);
            m.title = "Kaninerna på ön".into();
            m.time_unit = "månader".into();
            m.t_end = 100.0;
            m.dt = 0.05;
            m
        },
        steps: vec![
            info(
                "Nu ska vi räkna",
                "I den här träningen bygger du kaninerna som en modell som datorn kan räkna på. \
                 Då kan du se hur många kaniner det blir, och vilken loop som vinner.\n\n\
                 Du kommer att lära dig fyra byggstenar: lager, flöden, konstanter och hjälpvariabler.",
            ),
            task(
                "Ett lager – något som samlas",
                "Ett LAGER är något som finns i en viss mängd just nu och som kan öka eller minska: \
                 vatten i ett badkar, pengar på ett konto – eller kaniner på en ö. \
                 Om du kunde frysa tiden skulle du kunna räkna lagret.",
                "Välj Lager i verktygslådan och klicka på ytan. Döp lagret till Kaniner och skriv 10 som startvärde.",
                Tool::Place(NodeKind::Stock),
                sfd_stock_done,
                sfd_stock_solve,
                "Lagret visar hur många kaniner det finns. Titta på grafen: ett rakt streck på 10. \
                 Inget fyller på eller tömmer lagret, så ingenting händer.",
            ),
            task(
                "Ett flöde – det som ändrar lagret",
                "Ett FLÖDE ändrar ett lager, som en kran som fyller ett badkar. Flöden mäts per tidsenhet, här per månad. \
                 När nya kaniner föds fylls lagret på – det är ett inflöde.",
                "Välj Flöde och dra från en tom yta till vänster om lagret och in i lagret. Döp flödet till Födslar.",
                Tool::Flow,
                sfd_inflow_done,
                sfd_inflow_solve,
                "Flödet har formeln 1, alltså föds en kanin per månad – hur många kaniner det än finns. \
                 Grafen blir en rak linje. Är det rimligt?",
            ),
            task(
                "En konstant – något du kan ändra",
                "Ju fler kaniner, desto fler ungar. Hur många ungar varje kanin får i snitt per månad kallar vi födelsetal. \
                 Det är en KONSTANT: ett värde som inte ändras medan modellen räknar, men som du kan ändra med ett reglage.",
                "Lägg till en Konstant som heter Födelsetal och ge den värdet 0,2.",
                Tool::Place(NodeKind::Constant),
                sfd_rate_done,
                sfd_rate_solve,
                "Konstanten har fått ett reglage under \"Reglage\" till höger.",
            ),
            task(
                "Koppla ihop med en formel",
                "Nu ska antalet födslar bero på hur många kaniner som finns. Markera flödet Födslar och skriv en formel. \
                 Du kan klicka på namnen under formelrutan i stället för att skriva dem.",
                "Ge Födslar formeln:  Kaniner * Födelsetal",
                Tool::Select,
                sfd_births_done,
                sfd_births_solve,
                "Titta på grafen! Kurvan blir brantare och brantare: fler kaniner → fler födslar → ännu fler kaniner. \
                 Det är den FÖRSTÄRKANDE loopen (R) från orsaksdiagrammet. Den grå pilen från Kaniner till Födslar visar den. \
                 Prova att dra i reglaget för Födelsetal.",
            ),
            task(
                "Kaniner dör också",
                "Dödsfall är ett UTFLÖDE: det tömmer lagret. Ju fler kaniner, desto fler dör varje månad.",
                "Dra ett flöde FRÅN lagret ut till en tom yta till höger och döp det till Dödsfall. \
                 Lägg till en konstant Dödstal = 0,1 och ge Dödsfall formeln:  Kaniner * Dödstal",
                Tool::Flow,
                sfd_deaths_done,
                sfd_deaths_solve,
                "Fler kaniner → fler dödsfall → färre kaniner. Det är en BALANSERANDE loop (B). \
                 Prova: vad händer om dödstalet är större än födelsetalet? Och om de är lika stora?",
            ),
            task(
                "Hur mycket mat finns det?",
                "På ön växer bara en viss mängd mat. En HJÄLPVARIABEL räknar ut något med en formel – \
                 här hur mycket mat varje kanin får. Ju fler kaniner, desto mindre mat per kanin.",
                "Lägg till en konstant Mat på ön = 500. Lägg sedan till en Hjälpvariabel Mat per kanin med formeln:  \
                 Mat på ön / Kaniner",
                Tool::Place(NodeKind::Aux),
                sfd_food_done,
                sfd_food_solve,
                "Mat per kanin räknas ut hela tiden. Klicka på Mat per kanin under \"Visa\" i grafen för att se hur den sjunker.",
            ),
            task(
                "Maten bromsar födslarna",
                "Har kaninerna för lite mat får de färre ungar. Funktionen min(1; Mat per kanin) blir 1 \
                 när maten räcker, och mindre än 1 när den inte gör det.",
                "Ändra formeln för Födslar till:  Kaniner * Födelsetal * min(1; Mat per kanin)",
                Tool::Select,
                sfd_limit_done,
                sfd_limit_solve,
                "Nu planar kurvan ut – en S-kurva! I början vinner den förstärkande loopen. \
                 När maten inte räcker tar en ny balanserande loop över: fler kaniner → mindre mat per kanin \
                 → färre födslar → färre kaniner.",
            ),
            info(
                "Experimentera!",
                "Du har byggt en riktig systemmodell. Prova:\n\
                 • Tryck på \"Spara som jämförelse\" ovanför grafen. Dubbla sedan maten. Hur många kaniner blir det nu?\n\
                 • Sänk födelsetalet till 0,12. Vad händer – och varför?\n\
                 • Kan du få kaninerna att dö ut?\n\n\
                 Det här har du lärt dig:\n\
                 • LAGER – något som samlas (kaninerna)\n\
                 • FLÖDE – det som fyller på eller tömmer ett lager (födslar, dödsfall)\n\
                 • KONSTANT – ett värde du ändrar med ett reglage (födelsetal, mat)\n\
                 • HJÄLPVARIABEL – en uträkning (mat per kanin)\n\
                 • R-loopar får saker att växa, B-loopar bromsar.",
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim;

    fn check(step: &Step, m: &Model) -> bool {
        (step.check)(m, &sim::compile(m))
    }

    #[test]
    fn every_task_starts_undone_and_show_how_solves_it() {
        for lesson in all() {
            let mut m = (lesson.start)();
            for step in &lesson.steps {
                if step.task.is_none() {
                    continue;
                }
                assert!(!check(step, &m), "{}: \"{}\" var klar från början", lesson.title, step.title);
                (step.solve)(&mut m);
                assert!(check(step, &m), "{}: \"Visa hur\" löste inte \"{}\"", lesson.title, step.title);
                assert!(sim::compile(&m).errors.is_empty(), "{}: fel efter \"{}\"", lesson.title, step.title);
            }
        }
    }

    #[test]
    fn rabbit_diagram_ends_with_one_reinforcing_and_two_balancing_loops() {
        let lesson = rabbits_cld();
        let mut m = (lesson.start)();
        (lesson.steps.iter().rev().find(|s| s.task.is_some()).unwrap().solve)(&mut m);
        let loops = m.find_loops();
        assert_eq!(loops.len(), 3);
        assert_eq!(loops.iter().filter(|l| l.reinforcing).count(), 1);
    }

    #[test]
    fn rabbit_model_levels_off_where_food_runs_out() {
        let lesson = rabbits_sfd();
        let mut m = (lesson.start)();
        (lesson.steps.iter().rev().find(|s| s.task.is_some()).unwrap().solve)(&mut m);
        let c = sim::compile(&m);
        let r = sim::simulate(&m, &c);
        assert!(r.error.is_none(), "{:?}", r.error);
        let end = *r.series[&stock_id(&m).unwrap()].last().unwrap();
        // Jämvikt: födslar 0,2 · 500 = 100 per månad, dödsfall 0,1 · K ⇒ K = 1000.
        assert!((end - 1000.0).abs() < 60.0, "{end}");
    }

    #[test]
    fn checks_accept_the_students_own_names() {
        let lesson = rabbits_sfd();
        let mut m = (lesson.start)();
        let s = m.add_node(NodeKind::Stock, [0.0, 0.0]);
        m.node_mut(s).unwrap().name = "Kaninerna".into();
        m.node_mut(s).unwrap().formula = "25".into();
        assert!(sfd_stock_done(&m, &sim::compile(&m)));
        // "Visa hur" i nästa steg ska bygga vidare på elevens lager i stället för att skapa ett nytt.
        sfd_inflow_solve(&mut m);
        assert_eq!(m.nodes.iter().filter(|n| n.kind == NodeKind::Stock).count(), 1);
        assert_eq!(m.node(s).unwrap().name, "Kaninerna");
    }
}
