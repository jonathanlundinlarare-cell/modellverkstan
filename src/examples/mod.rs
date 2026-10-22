//! Färdiga exempelmodeller, ordnade efter ämne.

mod samhalle;
mod teknik;

pub use samhalle::*;
pub use teknik::*;

use crate::model::{Link, Model, ModelKind, Node, NodeKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    Teknik,
    Samhallskunskap,
}

impl Subject {
    pub const ALL: [Subject; 2] = [Subject::Teknik, Subject::Samhallskunskap];

    pub fn label(self) -> &'static str {
        match self {
            Subject::Teknik => "Teknik",
            Subject::Samhallskunskap => "Samhällskunskap",
        }
    }

    /// Om ämnet ska finnas med i webbversionen. Programversionen visar alltid alla ämnen.
    /// Ändra till `true` när ett ämne är klart att publiceras på webben.
    fn on_web(self) -> bool {
        match self {
            Subject::Teknik => true,
            Subject::Samhallskunskap => false,
        }
    }

    /// Om ämnet finns med i den här versionen av appen (program eller webb).
    /// Med `--features alla-amnen` visar även webbversionen allt (för förhandsgranskning).
    pub fn available(self) -> bool {
        !cfg!(target_arch = "wasm32") || cfg!(feature = "alla-amnen") || self.on_web()
    }
}

pub struct Example {
    pub title: &'static str,
    pub summary: &'static str,
    pub kind: ModelKind,
    pub subject: Subject,
    pub build: fn() -> Model,
}

/// Alla exempel som finns med i den här versionen av appen.
pub fn all() -> Vec<Example> {
    use ModelKind::{Cld, Sfd};
    use Subject::{Samhallskunskap, Teknik};
    let ex = |title, summary, kind, subject, build| Example { title, summary, kind, subject, build };
    [
        ex(
            "Fylla ett badkar",
            "Det enklaste systemet: en kran fyller på, ett avlopp tömmer. Svämmar det över?",
            Sfd,
            Teknik,
            bathtub,
        ),
        ex(
            "Värma ett hus med värmepump",
            "En luft-luftvärmepump och en termostat håller huset varmt. Vad händer när det blir riktigt kallt ute?",
            Sfd,
            Teknik,
            heat_pump,
        ),
        ex(
            "Ladda en elbil",
            "Hur lång tid tar det att ladda – och vad kostar det? Laddningen går långsammare när batteriet är nästan fullt.",
            Sfd,
            Teknik,
            ev_charging,
        ),
        ex("Termostaten (orsaksdiagram)", "Varför blir det lagom varmt inne? Två balanserande loopar.", Cld, Teknik, thermostat_cld),
        ex(
            "Elbilar och laddstolpar (orsaksdiagram)",
            "Fler elbilar ger fler laddstolpar som ger fler elbilar… men köerna bromsar.",
            Cld,
            Teknik,
            ev_cld,
        ),
        ex(
            "Sparande och ränta på ränta",
            "Spara en slant varje månad. Hur mycket är räntan värd efter 10, 20 och 40 år?",
            Sfd,
            Samhallskunskap,
            savings,
        ),
        ex(
            "Hushållets budget",
            "Lön in, hyra och mat ut. Vad händer med sparbufferten om man blir arbetslös?",
            Sfd,
            Samhallskunskap,
            household_budget,
        ),
        ex(
            "Lån och skuld",
            "Vad kostar ett lån egentligen? Jämför ett vanligt lån med ett dyrt SMS-lån.",
            Sfd,
            Samhallskunskap,
            loan,
        ),
        ex(
            "Det ekonomiska kretsloppet (orsaksdiagram)",
            "Hushåll, företag och staten hänger ihop: konsumtion blir jobb, jobb blir löner och skatter.",
            Cld,
            Samhallskunskap,
            economic_cycle,
        ),
    ]
    .into_iter()
    .filter(|e: &Example| e.subject.available())
    .collect()
}

/// Ämnen som har minst ett exempel i den här versionen av appen.
pub fn subjects() -> Vec<Subject> {
    let examples = all();
    Subject::ALL.into_iter().filter(|s| examples.iter().any(|e| e.subject == *s)).collect()
}

struct Builder {
    model: Model,
}

impl Builder {
    fn new(kind: ModelKind, title: &str, description: &str, tasks: &[&str]) -> Self {
        let mut model = Model::new(kind);
        model.title = title.into();
        model.description = description.into();
        model.tasks = tasks.iter().map(|s| s.to_string()).collect();
        Self { model }
    }

    fn push(&mut self, node: Node) -> u32 {
        let id = self.model.next_id();
        self.model.nodes.push(Node { id, ..node });
        id
    }

    fn constant(&mut self, name: &str, pos: [f32; 2], value: f64, range: (f64, f64), unit: &str, note: &str) -> u32 {
        self.push(Node {
            kind: NodeKind::Constant,
            name: name.into(),
            pos,
            value,
            min: range.0,
            max: range.1,
            unit: unit.into(),
            note: note.into(),
            ..Default::default()
        })
    }

    fn calc(&mut self, kind: NodeKind, name: &str, pos: [f32; 2], formula: &str, unit: &str, note: &str) -> u32 {
        self.push(Node {
            kind,
            name: name.into(),
            pos,
            formula: formula.into(),
            unit: unit.into(),
            note: note.into(),
            in_graph: kind == NodeKind::Stock,
            ..Default::default()
        })
    }

    fn flow(&mut self, name: &str, pos: [f32; 2], from: Option<u32>, to: Option<u32>, formula: &str, unit: &str, note: &str) -> u32 {
        let id = self.calc(NodeKind::Flow, name, pos, formula, unit, note);
        let n = self.model.node_mut(id).unwrap();
        n.from = from;
        n.to = to;
        id
    }

    fn var(&mut self, name: &str, pos: [f32; 2]) -> u32 {
        self.push(Node { kind: NodeKind::Variable, name: name.into(), pos, ..Default::default() })
    }

    fn link(&mut self, from: u32, to: u32, positive: bool, curve: f32) {
        self.model.links.push(Link { from, to, positive, curve });
    }

    fn graph(&mut self, ids: &[u32]) {
        for n in &mut self.model.nodes {
            n.in_graph = ids.contains(&n.id);
        }
    }
}
