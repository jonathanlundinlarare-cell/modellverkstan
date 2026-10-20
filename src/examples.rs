//! Färdiga exempelmodeller.

use crate::model::{Link, Model, ModelKind, Node, NodeKind};

pub struct Example {
    pub title: &'static str,
    pub summary: &'static str,
    pub kind: ModelKind,
    pub build: fn() -> Model,
}

pub fn all() -> Vec<Example> {
    vec![
        Example {
            title: "Fylla ett badkar",
            summary: "Det enklaste systemet: en kran fyller på, ett avlopp tömmer. Svämmar det över?",
            kind: ModelKind::Sfd,
            build: bathtub,
        },
        Example {
            title: "Värma ett hus med värmepump",
            summary: "En luft-luftvärmepump och en termostat håller huset varmt. Vad händer när det blir riktigt kallt ute?",
            kind: ModelKind::Sfd,
            build: heat_pump,
        },
        Example {
            title: "Ladda en elbil",
            summary: "Hur lång tid tar det att ladda – och vad kostar det? Laddningen går långsammare när batteriet är nästan fullt.",
            kind: ModelKind::Sfd,
            build: ev_charging,
        },
        Example {
            title: "Termostaten (orsaksdiagram)",
            summary: "Varför blir det lagom varmt inne? Två balanserande loopar.",
            kind: ModelKind::Cld,
            build: thermostat_cld,
        },
        Example {
            title: "Elbilar och laddstolpar (orsaksdiagram)",
            summary: "Fler elbilar ger fler laddstolpar som ger fler elbilar… men köerna bromsar.",
            kind: ModelKind::Cld,
            build: ev_cld,
        },
    ]
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

pub fn bathtub() -> Model {
    let mut b = Builder::new(
        ModelKind::Sfd,
        "Fylla ett badkar",
        "Vattnet i badkaret är ett LAGER – något som samlas. Kranen och avloppet är FLÖDEN som ändrar hur mycket vatten som finns. \
         Avloppet släpper ut mer vatten ju mer vatten det finns (trycket blir högre).",
        &[
            "Kör modellen. Varför slutar vattennivån att stiga fast kranen fortfarande är på?",
            "Hur mycket måste du skruva upp kranen för att badkaret ska svämma över?",
            "Sätt avloppets storlek till 0 (proppen i). Vad händer nu?",
        ],
    );
    let tub = b.calc(NodeKind::Stock, "Vatten i badkaret", [420.0, 220.0], "0", "liter", "Hur många liter vatten som finns i badkaret.");
    let tap = b.constant("Kranflöde", [190.0, 90.0], 12.0, (0.0, 30.0), "liter/minut", "Hur mycket vatten kranen släpper ut varje minut.");
    let drain_size = b.constant("Avloppets storlek", [640.0, 90.0], 0.05, (0.0, 0.3), "per minut", "Hur stor del av vattnet som rinner ut varje minut.");
    let _ = b.flow("Kran", [200.0, 220.0], None, Some(tub), "Kranflöde", "liter/minut", "Vatten in i badkaret.");
    let _ = b.flow("Avlopp", [640.0, 220.0], Some(tub), None, "Vatten i badkaret * Avloppets storlek", "liter/minut", "Vatten ut ur badkaret.");
    let vol = b.constant("Badkarets volym", [420.0, 380.0], 200.0, (100.0, 300.0), "liter", "Hur mycket som får plats.");
    let over = b.calc(NodeKind::Aux, "Svämmar över", [640.0, 380.0], "om(Vatten i badkaret > Badkarets volym; 1; 0)", "1 = ja, 0 = nej", "Blir 1 om det är för mycket vatten.");
    let _ = (tap, drain_size, vol);
    b.graph(&[tub, over]);
    b.model.dt = 0.1;
    b.model.t_end = 60.0;
    b.model.time_unit = "minuter".into();
    b.model
}

pub fn heat_pump() -> Model {
    let mut b = Builder::new(
        ModelKind::Sfd,
        "Värma ett hus med värmepump",
        "Värmen i huset är ett LAGER som mäts som inomhustemperatur. Värmepumpen tillför värme (ett inflöde) och värme läcker ut genom väggar, \
         fönster och tak (ett utflöde). Termostaten slår på värmepumpen när det är kallare än önskat. \
         COP (värmefaktor) säger hur många kWh värme pumpen ger för varje kWh el – den hämtar värme ur uteluften.",
        &[
            "Kör modellen. Hur lång tid tar det innan huset är varmt?",
            "Sänk utetemperaturen till −20 °C. Klarar värmepumpen att hålla 21 °C? Varför/varför inte?",
            "Gör huset bättre isolerat (lägre värmeförlust per grad). Hur påverkas elkostnaden?",
            "Vad händer med kostnaden om COP är 1 (som ett vanligt element)?",
        ],
    );
    let temp = b.calc(NodeKind::Stock, "Inomhustemperatur", [460.0, 260.0], "15", "°C", "Temperaturen inne i huset. Startvärdet är 15 °C.");
    let thermostat = b.constant("Termostat", [460.0, 70.0], 21.0, (15.0, 25.0), "°C", "Den temperatur vi vill ha inne.");
    let max_el = b.constant("Max eleffekt", [80.0, 70.0], 1.5, (0.0, 4.0), "kW", "Hur mycket el värmepumpen som mest kan använda.");
    let cop = b.constant("COP", [260.0, 30.0], 3.0, (1.0, 5.0), "", "Värmefaktor: kWh värme per kWh el.");
    let heat = b.calc(NodeKind::Aux, "Värmeeffekt", [240.0, 140.0], "om(Inomhustemperatur < Termostat; Max eleffekt * COP; 0)", "kW", "Värmen från värmepumpen. Termostaten slår av när det är varmt nog.");
    let outdoor = b.constant("Utetemperatur", [840.0, 70.0], -5.0, (-25.0, 15.0), "°C", "");
    let loss_per = b.constant("Värmeförlust per grad", [660.0, 30.0], 0.15, (0.05, 0.4), "kW/°C", "Hur dåligt huset är isolerat. Lägre = bättre isolering.");
    let loss = b.calc(NodeKind::Aux, "Förlusteffekt", [680.0, 140.0], "(Inomhustemperatur - Utetemperatur) * Värmeförlust per grad", "kW", "Ju större skillnad mellan inne och ute, desto mer värme läcker ut.");
    let capacity = b.constant("Husets värmekapacitet", [460.0, 385.0], 4.0, (1.0, 15.0), "kWh/°C", "Hur mycket energi som behövs för att värma huset en grad.");
    b.flow("Uppvärmning", [240.0, 260.0], None, Some(temp), "Värmeeffekt / Husets värmekapacitet", "°C/timme", "Hur snabbt värmepumpen höjer temperaturen.");
    b.flow("Värmeläckage", [680.0, 260.0], Some(temp), None, "Förlusteffekt / Husets värmekapacitet", "°C/timme", "Hur snabbt temperaturen sjunker av värmeförlusterna.");
    let el = b.calc(NodeKind::Stock, "Använd el", [350.0, 545.0], "0", "kWh", "All el som värmepumpen har använt.");
    b.flow("Elförbrukning", [120.0, 545.0], None, Some(el), "Värmeeffekt / COP", "kW", "El som går åt just nu.");
    let price = b.constant("Elpris", [580.0, 455.0], 2.0, (0.5, 5.0), "kr/kWh", "");
    let cost = b.calc(NodeKind::Aux, "Elkostnad", [580.0, 545.0], "Använd el * Elpris", "kr", "Vad elen har kostat hittills.");
    let _ = (max_el, cop, heat, outdoor, loss_per, loss, capacity, price, cost);
    b.graph(&[temp, thermostat, outdoor]);
    b.model.dt = 0.05;
    b.model.t_end = 48.0;
    b.model.time_unit = "timmar".into();
    b.model
}

pub fn ev_charging() -> Model {
    let mut b = Builder::new(
        ModelKind::Sfd,
        "Ladda en elbil",
        "Energin i batteriet är ett LAGER (kWh). Laddningen är ett FLÖDE (kW = kWh per timme). \
         För att skydda batteriet laddar bilen långsammare när laddnivån är över 80 %. \
         En del av elen blir värme i laddaren, därför köper man lite mer el än det som hamnar i batteriet.",
        &[
            "Hur lång tid tar det att ladda från 20 % till 80 %? Och till 100 %?",
            "Jämför en vanlig laddbox (11 kW) med ett vägguttag (2,3 kW). Hur stor blir skillnaden?",
            "Vad kostar en full laddning? Ändra elpriset och se vad som händer.",
            "Varför tror du att bilen laddar långsammare på slutet?",
        ],
    );
    let energy = b.calc(NodeKind::Stock, "Energi i batteriet", [460.0, 260.0], "Batteristorlek * Startnivå / 100", "kWh", "Startvärdet räknas ut från startnivån.");
    let size = b.constant("Batteristorlek", [700.0, 160.0], 60.0, (20.0, 100.0), "kWh", "Hur mycket energi batteriet rymmer.");
    let start = b.constant("Startnivå", [700.0, 340.0], 20.0, (0.0, 95.0), "%", "Laddnivån när vi börjar ladda.");
    let level = b.calc(NodeKind::Aux, "Laddnivå", [460.0, 110.0], "Energi i batteriet / Batteristorlek * 100", "%", "Hur fullt batteriet är.");
    let max_p = b.constant("Laddeffekt max", [90.0, 110.0], 11.0, (2.0, 22.0), "kW", "Laddboxens effekt. Vanligt vägguttag ≈ 2,3 kW.");
    let charge = b.flow(
        "Laddning",
        [240.0, 260.0],
        None,
        Some(energy),
        "max(0; om(Laddnivå < 80; Laddeffekt max; Laddeffekt max * (100 - Laddnivå) / 20))",
        "kW",
        "Full fart upp till 80 %, sedan allt långsammare.",
    );
    let bought = b.calc(NodeKind::Stock, "Köpt el", [460.0, 520.0], "0", "kWh", "El som har gått genom elmätaren.");
    let eff = b.constant("Verkningsgrad", [90.0, 430.0], 90.0, (70.0, 100.0), "%", "Hur stor del av elen som hamnar i batteriet.");
    b.flow("Elinköp", [240.0, 520.0], None, Some(bought), "Laddning / (Verkningsgrad / 100)", "kW", "");
    let price = b.constant("Elpris", [700.0, 440.0], 2.0, (0.5, 5.0), "kr/kWh", "");
    let cost = b.calc(NodeKind::Aux, "Kostnad", [700.0, 560.0], "Köpt el * Elpris", "kr", "Vad laddningen har kostat hittills.");
    let _ = (size, start, max_p, eff, price, cost);
    b.graph(&[level, charge]);
    b.model.dt = 0.02;
    b.model.t_end = 10.0;
    b.model.time_unit = "timmar".into();
    b.model
}

pub fn thermostat_cld() -> Model {
    let mut b = Builder::new(
        ModelKind::Cld,
        "Termostaten",
        "Ett orsaksdiagram visar hur saker påverkar varandra. En pil med + betyder \"åt samma håll\": ökar det ena så ökar det andra. \
         En pil med − betyder \"åt motsatt håll\": ökar det ena så minskar det andra. \
         Följer man pilarna runt och kommer tillbaka till start har man hittat en LOOP.",
        &[
            "Följ loop B1 med fingret: om inomhustemperaturen ökar – vad händer med värmepumpens effekt?",
            "Varför kallas loopen balanserande?",
            "Lägg till variabeln \"Öppna fönster\". Var ska den kopplas in, och med + eller −?",
        ],
    );
    let inside = b.var("Inomhustemperatur", [420.0, 330.0]);
    let gap = b.var("Skillnad mot önskad temperatur", [180.0, 170.0]);
    let wanted = b.var("Önskad temperatur", [120.0, 40.0]);
    let pump = b.var("Värmepumpens effekt", [420.0, 60.0]);
    let loss = b.var("Värmeförlust", [700.0, 330.0]);
    let outside = b.var("Utetemperatur", [760.0, 160.0]);
    b.link(inside, gap, false, 0.25);
    b.link(wanted, gap, true, 0.0);
    b.link(gap, pump, true, 0.25);
    b.link(pump, inside, true, 0.25);
    b.link(inside, loss, true, -0.3);
    b.link(loss, inside, false, -0.3);
    b.link(outside, loss, false, 0.0);
    b.model
}

pub fn ev_cld() -> Model {
    let mut b = Builder::new(
        ModelKind::Cld,
        "Elbilar och laddstolpar",
        "Fler elbilar gör det lönsamt att bygga laddstolpar, och fler laddstolpar gör det lättare att välja elbil. \
         Det är en FÖRSTÄRKANDE loop (R): förändringen växer. Men fler elbilar ger också köer vid laddstolparna, \
         vilket bromsar – en BALANSERANDE loop (B).",
        &[
            "Hitta de tre looparna i listan till höger. Håll musen över dem för att se var de går.",
            "Vad kan göra att utvecklingen går åt andra hållet – färre elbilar?",
            "Lägg till \"Elpris\" och \"Bensinpris\". Hur påverkar de viljan att köpa elbil?",
        ],
    );
    let cars = b.var("Antal elbilar", [140.0, 230.0]);
    let demand = b.var("Efterfrågan på laddning", [420.0, 60.0]);
    let chargers = b.var("Antal laddstolpar", [720.0, 230.0]);
    let will = b.var("Viljan att köpa elbil", [420.0, 420.0]);
    let queues = b.var("Köer vid laddstolpar", [420.0, 240.0]);
    b.link(cars, demand, true, 0.2);
    b.link(demand, chargers, true, 0.2);
    b.link(chargers, will, true, 0.2);
    b.link(will, cars, true, 0.2);
    b.link(cars, queues, true, 0.0);
    b.link(chargers, queues, false, 0.0);
    b.link(queues, will, false, 0.0);
    b.model
}
