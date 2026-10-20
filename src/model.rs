//! Datamodellen: delar (noder), pilar och inställningar. Sparas som JSON.

use crate::expr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelKind {
    /// Orsaksdiagram (causal loop diagram).
    Cld,
    /// Lager- och flödesmodell som går att simulera.
    Sfd,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeKind {
    Stock,
    Flow,
    #[default]
    Aux,
    Constant,
    Variable,
}

impl NodeKind {
    pub fn label(self) -> &'static str {
        match self {
            NodeKind::Stock => "Lager",
            NodeKind::Flow => "Flöde",
            NodeKind::Aux => "Hjälpvariabel",
            NodeKind::Constant => "Konstant",
            NodeKind::Variable => "Variabel",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Node {
    pub id: u32,
    pub kind: NodeKind,
    pub name: String,
    pub pos: [f32; 2],
    /// Formel för flöden och hjälpvariabler, startvärde för lager.
    pub formula: String,
    /// Värdet för konstanter.
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub unit: String,
    /// Flöden: lagret flödet kommer från (None = utanför modellen, ett "moln").
    pub from: Option<u32>,
    /// Flöden: lagret flödet går till.
    pub to: Option<u32>,
    pub in_graph: bool,
    pub note: String,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            id: 0,
            kind: NodeKind::Aux,
            name: String::new(),
            pos: [0.0, 0.0],
            formula: String::new(),
            value: 0.0,
            min: 0.0,
            max: 100.0,
            unit: String::new(),
            from: None,
            to: None,
            in_graph: false,
            note: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Link {
    pub from: u32,
    pub to: u32,
    /// `true` = samma riktning (+), `false` = motsatt riktning (−).
    pub positive: bool,
    /// Hur mycket pilen böjer sig (0 = rak).
    pub curve: f32,
}

impl Default for Link {
    fn default() -> Self {
        Self { from: 0, to: 0, positive: true, curve: 0.2 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Model {
    pub kind: ModelKind,
    pub title: String,
    pub description: String,
    pub tasks: Vec<String>,
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
    pub dt: f64,
    pub t_end: f64,
    pub time_unit: String,
}

impl Default for Model {
    fn default() -> Self {
        Self::new(ModelKind::Sfd)
    }
}

/// En återkopplingsloop i ett orsaksdiagram.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedbackLoop {
    pub nodes: Vec<u32>,
    pub reinforcing: bool,
}

impl Model {
    pub fn new(kind: ModelKind) -> Self {
        Self {
            kind,
            title: match kind {
                ModelKind::Cld => "Nytt orsaksdiagram".into(),
                ModelKind::Sfd => "Ny flödesmodell".into(),
            },
            description: String::new(),
            tasks: Vec::new(),
            nodes: Vec::new(),
            links: Vec::new(),
            dt: 0.1,
            t_end: 20.0,
            time_unit: "minuter".into(),
        }
    }

    pub fn node(&self, id: u32) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: u32) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn next_id(&self) -> u32 {
        self.nodes.iter().map(|n| n.id).max().map_or(1, |m| m + 1)
    }

    /// Ett namn som inte redan används, t.ex. "Lager 2".
    pub fn unique_name(&self, base: &str) -> String {
        (1..)
            .map(|i| format!("{base} {i}"))
            .find(|name| {
                let n = expr::normalize_name(name);
                !self.nodes.iter().any(|x| expr::normalize_name(&x.name) == n)
            })
            .unwrap()
    }

    pub fn add_node(&mut self, kind: NodeKind, pos: [f32; 2]) -> u32 {
        let id = self.next_id();
        let name = self.unique_name(kind.label());
        let mut node = Node { id, kind, name, pos, ..Default::default() };
        match kind {
            NodeKind::Stock => {
                node.formula = "0".into();
                node.in_graph = true;
            }
            NodeKind::Flow | NodeKind::Aux => node.formula = "1".into(),
            NodeKind::Constant => {
                node.value = 1.0;
                node.min = 0.0;
                node.max = 10.0;
            }
            NodeKind::Variable => {}
        }
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        self.links.retain(|l| l.from != id && l.to != id);
        for n in &mut self.nodes {
            if n.from == Some(id) {
                n.from = None;
            }
            if n.to == Some(id) {
                n.to = None;
            }
        }
    }

    /// Uppdaterar alla formler när en del byter namn.
    pub fn rename_refs(&mut self, old: &str, new: &str) {
        if expr::normalize_name(old) == expr::normalize_name(new) || new.trim().is_empty() {
            return;
        }
        for n in &mut self.nodes {
            n.formula = expr::replace_name(&n.formula, old, new);
        }
    }

    /// Hittar alla återkopplingsloopar bland pilarna (högst 64 st).
    pub fn find_loops(&self) -> Vec<FeedbackLoop> {
        let mut ids: Vec<u32> = self.nodes.iter().map(|n| n.id).collect();
        ids.sort();
        let rank: HashMap<u32, usize> = ids.iter().enumerate().map(|(i, &id)| (id, i)).collect();
        let mut adj: HashMap<u32, Vec<(u32, bool)>> = HashMap::new();
        for l in &self.links {
            if rank.contains_key(&l.from) && rank.contains_key(&l.to) {
                adj.entry(l.from).or_default().push((l.to, l.positive));
            }
        }

        struct Search<'a> {
            adj: &'a HashMap<u32, Vec<(u32, bool)>>,
            rank: &'a HashMap<u32, usize>,
            path: Vec<u32>,
            negatives: usize,
            found: Vec<FeedbackLoop>,
        }

        impl Search<'_> {
            fn dfs(&mut self, start: u32, cur: u32) {
                if self.found.len() >= 64 || self.path.len() > 14 {
                    return;
                }
                let Some(edges) = self.adj.get(&cur) else { return };
                for &(next, positive) in edges {
                    let neg = usize::from(!positive);
                    if next == start {
                        self.found.push(FeedbackLoop {
                            nodes: self.path.clone(),
                            reinforcing: (self.negatives + neg) % 2 == 0,
                        });
                    } else if self.rank[&next] > self.rank[&start] && !self.path.contains(&next) {
                        self.path.push(next);
                        self.negatives += neg;
                        self.dfs(start, next);
                        self.negatives -= neg;
                        self.path.pop();
                    }
                }
            }
        }

        let mut s = Search { adj: &adj, rank: &rank, path: Vec::new(), negatives: 0, found: Vec::new() };
        for &start in &ids {
            s.path = vec![start];
            s.negatives = 0;
            s.dfs(start, start);
        }
        let mut loops = s.found;
        loops.sort_by_key(|l| l.nodes.len());
        loops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cld(links: &[(u32, u32, bool)]) -> Model {
        let mut m = Model::new(ModelKind::Cld);
        for _ in 0..4 {
            m.add_node(NodeKind::Variable, [0.0, 0.0]);
        }
        for &(from, to, positive) in links {
            m.links.push(Link { from, to, positive, curve: 0.2 });
        }
        m
    }

    #[test]
    fn detects_loop_polarity() {
        // 1 →(+) 2 →(−) 3 →(+) 1 : en minus ⇒ balanserande.
        let m = cld(&[(1, 2, true), (2, 3, false), (3, 1, true)]);
        let loops = m.find_loops();
        assert_eq!(loops.len(), 1);
        assert!(!loops[0].reinforcing);
        assert_eq!(loops[0].nodes, vec![1, 2, 3]);

        // Två minus ⇒ förstärkande.
        let m = cld(&[(1, 2, false), (2, 1, false)]);
        let loops = m.find_loops();
        assert_eq!(loops.len(), 1);
        assert!(loops[0].reinforcing);
    }

    #[test]
    fn finds_every_loop_once() {
        let m = cld(&[(1, 2, true), (2, 1, true), (2, 3, true), (3, 4, true), (4, 2, false), (3, 3, true)]);
        let loops = m.find_loops();
        assert_eq!(loops.len(), 3);
        assert_eq!(loops.iter().filter(|l| l.reinforcing).count(), 2);
    }

    #[test]
    fn rename_updates_formulas() {
        let mut m = Model::new(ModelKind::Sfd);
        let a = m.add_node(NodeKind::Constant, [0.0, 0.0]);
        let b = m.add_node(NodeKind::Aux, [0.0, 0.0]);
        m.node_mut(b).unwrap().formula = "Konstant 1 * 2".into();
        m.node_mut(a).unwrap().name = "Pris".into();
        m.rename_refs("Konstant 1", "Pris");
        assert_eq!(m.node(b).unwrap().formula, "Pris * 2");
    }
}
