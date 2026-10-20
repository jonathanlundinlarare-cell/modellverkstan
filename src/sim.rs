//! Kontroll av formler och simulering (Eulers metod).

use crate::expr::{self, Expr};
use crate::model::{Model, ModelKind, NodeKind};
use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub struct Compiled {
    /// Tolkad formel per nod (samma ordning som `model.nodes`).
    exprs: Vec<Option<Expr>>,
    /// Fel per nod-id, med förklaring på svenska.
    pub errors: BTreeMap<u32, String>,
    /// Informationspilar (från, till) som följer av formlerna.
    pub deps: Vec<(u32, u32)>,
    /// Beräkningsordning för flöden och hjälpvariabler.
    order: Vec<usize>,
}

#[derive(Default)]
pub struct SimResult {
    pub times: Vec<f64>,
    pub series: HashMap<u32, Vec<f64>>,
    pub error: Option<String>,
}

impl SimResult {
    /// Index i tidsserien som ligger närmast tiden `t`.
    pub fn index_at(&self, t: f64) -> Option<usize> {
        let last = *self.times.last()?;
        if last <= 0.0 {
            return Some(0);
        }
        let i = ((t / last) * (self.times.len() - 1) as f64).round();
        Some((i.max(0.0) as usize).min(self.times.len() - 1))
    }

    pub fn value_at(&self, id: u32, t: f64) -> Option<f64> {
        let i = self.index_at(t)?;
        self.series.get(&id)?.get(i).copied()
    }
}

pub fn compile(model: &Model) -> Compiled {
    let mut c = Compiled { exprs: vec![None; model.nodes.len()], ..Default::default() };
    if model.kind != ModelKind::Sfd {
        return c;
    }
    let names: Vec<String> = model.nodes.iter().map(|n| n.name.clone()).collect();

    let mut seen: HashMap<Vec<char>, u32> = HashMap::new();
    for n in &model.nodes {
        let key = expr::normalize_name(&n.name);
        if key.is_empty() {
            c.errors.insert(n.id, "Ge den här delen ett namn.".into());
        } else if seen.insert(key, n.id).is_some() {
            c.errors.insert(n.id, format!("Det finns redan något som heter \"{}\". Namnen måste vara olika.", n.name.trim()));
        }
    }

    for (i, n) in model.nodes.iter().enumerate() {
        if !matches!(n.kind, NodeKind::Stock | NodeKind::Flow | NodeKind::Aux) {
            continue;
        }
        match expr::parse(&n.formula, &names) {
            Ok(e) => {
                let mut vars = Vec::new();
                e.vars(&mut vars);
                if n.kind == NodeKind::Stock {
                    if let Some(&bad) = vars.iter().find(|&&v| model.nodes[v].kind != NodeKind::Constant) {
                        c.errors.entry(n.id).or_insert(format!(
                            "Startvärdet kan bara använda tal och konstanter, inte \"{}\".",
                            model.nodes[bad].name.trim()
                        ));
                    }
                } else {
                    if vars.contains(&i) {
                        c.errors.entry(n.id).or_insert("En formel kan inte använda sig själv.".into());
                    }
                    for &v in &vars {
                        if v != i {
                            c.deps.push((model.nodes[v].id, n.id));
                        }
                    }
                }
                c.exprs[i] = Some(e);
            }
            Err(msg) => {
                c.errors.entry(n.id).or_insert(msg);
            }
        }
    }

    // Beräkningsordning (Kahns algoritm) för flöden och hjälpvariabler.
    let index: HashMap<u32, usize> = model.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();
    let is_calc = |i: usize| matches!(model.nodes[i].kind, NodeKind::Flow | NodeKind::Aux);
    let mut indeg = vec![0usize; model.nodes.len()];
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); model.nodes.len()];
    for &(from, to) in &c.deps {
        let (f, t) = (index[&from], index[&to]);
        if is_calc(f) && is_calc(t) {
            indeg[t] += 1;
            out[f].push(t);
        }
    }
    let mut ready: Vec<usize> = (0..model.nodes.len()).filter(|&i| is_calc(i) && indeg[i] == 0).collect();
    while let Some(i) = ready.pop() {
        c.order.push(i);
        for &t in &out[i] {
            indeg[t] -= 1;
            if indeg[t] == 0 {
                ready.push(t);
            }
        }
    }
    for i in (0..model.nodes.len()).filter(|&i| is_calc(i) && indeg[i] > 0) {
        c.errors.entry(model.nodes[i].id).or_insert(
            "Cirkelberoende: formeln beror (via andra formler) på sig själv. Någonstans i kedjan behövs ett lager.".into(),
        );
    }
    c
}

pub fn simulate(model: &Model, c: &Compiled) -> SimResult {
    let mut r = SimResult::default();
    if model.kind != ModelKind::Sfd || model.nodes.is_empty() {
        return r;
    }
    if !c.errors.is_empty() {
        r.error = Some("Modellen har fel som behöver rättas innan den kan köras (se de röda markeringarna).".into());
        return r;
    }
    if !(model.dt > 0.0) || !(model.t_end > 0.0) {
        r.error = Some("Tidssteget och sluttiden måste vara större än noll.".into());
        return r;
    }
    let steps = (model.t_end / model.dt).round() as usize;
    if steps > 200_000 {
        r.error = Some("För många tidssteg. Gör tidssteget (dt) större eller sluttiden kortare.".into());
        return r;
    }
    let stride = (steps / 2000).max(1);
    let n = model.nodes.len();
    let index: HashMap<u32, usize> = model.nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect();

    let mut v = vec![0.0; n];
    for (i, node) in model.nodes.iter().enumerate() {
        if node.kind == NodeKind::Constant {
            v[i] = node.value;
        }
    }
    for (i, node) in model.nodes.iter().enumerate() {
        if node.kind == NodeKind::Stock {
            v[i] = c.exprs[i].as_ref().map_or(0.0, |e| e.eval(&v, 0.0));
        }
    }
    // (flödets index, från-lager, till-lager)
    let flows: Vec<(usize, Option<usize>, Option<usize>)> = model
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == NodeKind::Flow)
        .map(|(i, n)| {
            let stock = |id: Option<u32>| {
                id.and_then(|id| index.get(&id).copied()).filter(|&s| model.nodes[s].kind == NodeKind::Stock)
            };
            (i, stock(n.from), stock(n.to))
        })
        .collect();

    let mut series: Vec<Vec<f64>> = vec![Vec::new(); n];
    let mut change = vec![0.0; n];
    for step in 0..=steps {
        let t = step as f64 * model.dt;
        for &i in &c.order {
            v[i] = c.exprs[i].as_ref().map_or(0.0, |e| e.eval(&v, t));
        }
        if let Some(bad) = (0..n).find(|&i| !v[i].is_finite()) {
            r.error = Some(format!(
                "Vid tiden {} blev \"{}\" ogiltig (t.ex. division med noll eller ett för stort tal).",
                crate::theme::fmt_num(t),
                model.nodes[bad].name.trim()
            ));
            break;
        }
        if step % stride == 0 || step == steps {
            r.times.push(t);
            for i in 0..n {
                series[i].push(v[i]);
            }
        }
        if step == steps {
            break;
        }
        change.iter_mut().for_each(|d| *d = 0.0);
        for &(f, from, to) in &flows {
            if let Some(s) = from {
                change[s] -= v[f];
            }
            if let Some(s) = to {
                change[s] += v[f];
            }
        }
        for (i, node) in model.nodes.iter().enumerate() {
            if node.kind == NodeKind::Stock {
                v[i] += model.dt * change[i];
            }
        }
    }
    r.series = model.nodes.iter().zip(series).map(|(n, s)| (n.id, s)).collect();
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::examples;

    fn run(m: &Model) -> SimResult {
        let c = compile(m);
        assert!(c.errors.is_empty(), "fel: {:?}", c.errors);
        let r = simulate(m, &c);
        assert!(r.error.is_none(), "{:?}", r.error);
        r
    }

    fn id(m: &Model, name: &str) -> u32 {
        m.nodes.iter().find(|n| n.name == name).unwrap().id
    }

    fn set(m: &mut Model, name: &str, value: f64) {
        m.nodes.iter_mut().find(|n| n.name == name).unwrap().value = value;
    }

    #[test]
    fn bathtub_fills_linearly() {
        let mut m = Model::new(ModelKind::Sfd);
        m.dt = 1.0;
        m.t_end = 10.0;
        let s = m.add_node(NodeKind::Stock, [0.0, 0.0]);
        let f = m.add_node(NodeKind::Flow, [0.0, 0.0]);
        m.node_mut(f).unwrap().to = Some(s);
        m.node_mut(f).unwrap().formula = "2".into();
        let r = run(&m);
        assert_eq!(r.times.len(), 11);
        assert_eq!(*r.series[&s].last().unwrap(), 20.0);
    }

    #[test]
    fn heat_pump_holds_thermostat_and_house_cools_without_it() {
        let mut m = examples::heat_pump();
        let temp = id(&m, "Inomhustemperatur");
        let r = run(&m);
        let end = *r.series[&temp].last().unwrap();
        assert!((end - 21.0).abs() < 0.5, "sluttemperatur {end}");

        // Utan värmepump närmar sig temperaturen utetemperaturen.
        set(&mut m, "Max eleffekt", 0.0);
        m.t_end = 400.0;
        let r = run(&m);
        let end = *r.series[&temp].last().unwrap();
        assert!((end - -5.0).abs() < 0.5, "sluttemperatur {end}");
    }

    #[test]
    fn heat_pump_too_weak_in_hard_frost() {
        let mut m = examples::heat_pump();
        set(&mut m, "Utetemperatur", -25.0);
        let temp = id(&m, "Inomhustemperatur");
        let r = run(&m);
        assert!(*r.series[&temp].last().unwrap() < 19.0);
    }

    #[test]
    fn ev_charges_and_slows_down_after_80_percent() {
        let m = examples::ev_charging();
        let level = id(&m, "Laddnivå");
        let power = id(&m, "Laddning");
        let r = run(&m);
        let lv = &r.series[&level];
        let pw = &r.series[&power];
        assert!((lv[0] - 20.0).abs() < 1e-9);
        assert!(*lv.last().unwrap() > 95.0 && *lv.last().unwrap() <= 100.0);
        assert!(lv.windows(2).all(|w| w[1] >= w[0] - 1e-9));
        let i80 = lv.iter().position(|&x| x >= 85.0).unwrap();
        assert!(pw[i80] < pw[0]);
    }

    #[test]
    fn examples_compile() {
        for ex in examples::all() {
            let m = (ex.build)();
            let c = compile(&m);
            assert!(c.errors.is_empty(), "{}: {:?}", m.title, c.errors);
            if m.kind == ModelKind::Sfd {
                assert!(simulate(&m, &c).error.is_none(), "{}", m.title);
            } else {
                assert!(!m.find_loops().is_empty(), "{}", m.title);
            }
        }
    }

    #[test]
    fn reports_cycles_and_unknown_names() {
        let mut m = Model::new(ModelKind::Sfd);
        let a = m.add_node(NodeKind::Aux, [0.0, 0.0]);
        let b = m.add_node(NodeKind::Aux, [0.0, 0.0]);
        m.node_mut(a).unwrap().formula = "Hjälpvariabel 2 + 1".into();
        m.node_mut(b).unwrap().formula = "Hjälpvariabel 1 * 2".into();
        let c = compile(&m);
        assert!(c.errors[&a].contains("Cirkel"));
        m.node_mut(b).unwrap().formula = "Okänd".into();
        let c = compile(&m);
        assert!(c.errors[&b].contains("Okänd"));
        assert!(simulate(&m, &c).error.is_some());
    }
}
