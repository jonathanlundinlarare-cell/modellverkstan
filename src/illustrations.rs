//! Illustrationer till presentationen, ritade direkt i appen.
//!
//! Varje bild ritas i ett tänkt koordinatsystem på 480 × 340 som skalas
//! och centreras i den yta som finns.

use crate::theme::{self, cr};
use crate::ui_kit;
use egui::{pos2, vec2, Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Illustration {
    Title,
    MapVsWorld,
    WhyModel,
    System,
    CldArrows,
    LoopR,
    LoopB,
    Bathtub,
    Flows,
    ConstAux,
    TimeSteps,
    Behaviours,
    Riksbank,
    Spaghetti,
    Limits,
    Summary,
}

const W: f32 = 480.0;
const H: f32 = 340.0;

/// Ritar i det tänkta koordinatsystemet.
struct Pen<'a> {
    p: &'a Painter,
    origin: Pos2,
    s: f32,
}

impl Pen<'_> {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.s
    }

    fn rect(&self, cx: f32, cy: f32, w: f32, h: f32) -> Rect {
        Rect::from_center_size(self.at(cx, cy), vec2(w, h) * self.s)
    }

    fn l(&self, v: f32) -> f32 {
        v * self.s
    }

    fn text(&self, x: f32, y: f32, text: &str, size: f32, color: Color32, align: Align2) -> Rect {
        self.p.text(self.at(x, y), align, text, theme::semibold(size * self.s), color)
    }

    fn text_regular(&self, x: f32, y: f32, text: &str, size: f32, color: Color32, align: Align2) -> Rect {
        self.p.text(self.at(x, y), align, text, theme::regular(size * self.s), color)
    }

    fn line(&self, pts: &[(f32, f32)], width: f32, color: Color32) {
        let pts: Vec<Pos2> = pts.iter().map(|&(x, y)| self.at(x, y)).collect();
        self.p.add(Shape::line(pts, Stroke::new(self.l(width), color)));
    }

    fn dashed(&self, pts: &[(f32, f32)], width: f32, color: Color32) {
        let pts: Vec<Pos2> = pts.iter().map(|&(x, y)| self.at(x, y)).collect();
        self.p.extend(Shape::dashed_line(&pts, Stroke::new(self.l(width), color), self.l(7.0), self.l(5.0)));
    }

    fn arrow_head(&self, tip: Pos2, dir: Vec2, size: f32, color: Color32) {
        let d = if dir.length() < 1e-3 { Vec2::X } else { dir.normalized() };
        let n = vec2(-d.y, d.x);
        let back = tip - d * self.l(size);
        self.p.add(Shape::convex_polygon(
            vec![tip, back + n * self.l(size) * 0.55, back - n * self.l(size) * 0.55],
            color,
            Stroke::NONE,
        ));
    }

    /// Ett lager: rundad låda med nivå, namn och värde.
    fn stock(&self, cx: f32, cy: f32, w: f32, h: f32, level: f32, name: &str) -> Rect {
        let r = self.rect(cx, cy, w, h);
        let radius = cr(self.l(10.0));
        self.p.add(ui_kit::soft_shadow(r, self.l(10.0), 1));
        self.p.rect_filled(r, radius, theme::STOCK_FILL);
        let lh = r.height() * level.clamp(0.0, 1.0);
        let clip = Rect::from_min_max(pos2(r.left(), r.bottom() - lh), r.right_bottom());
        self.p.with_clip_rect(clip).rect_filled(r, radius, theme::STOCK_LEVEL);
        self.p.rect_stroke(r, radius, Stroke::new(self.l(2.4), theme::STOCK), StrokeKind::Inside);
        self.p.text(r.center() - vec2(0.0, self.l(6.0)), Align2::CENTER_CENTER, name, theme::semibold(self.l(15.0)), theme::INK);
        r
    }

    fn cloud(&self, x: f32, y: f32) {
        let c = self.at(x, y);
        let blobs = [(vec2(-11.0, 3.0), 9.0), (vec2(0.0, -5.0), 11.0), (vec2(11.0, 3.0), 9.0), (vec2(0.0, 5.0), 8.0)];
        for (o, r) in blobs {
            self.p.circle_stroke(c + o * self.s, r * self.s, Stroke::new(self.l(1.5), theme::MUTED));
        }
        for (o, r) in blobs {
            self.p.circle_filled(c + o * self.s, r * self.s - 1.0, theme::SURFACE);
        }
    }

    /// Ett rör med ventil från x1 till x2 på höjden y; pilen pekar mot x2.
    fn pipe(&self, x1: f32, x2: f32, y: f32, label: Option<&str>) {
        let dir = if x2 > x1 { 1.0 } else { -1.0 };
        let end = x2 - dir * 12.0;
        self.line(&[(x1, y), (end, y)], 12.0, theme::FLOW);
        self.line(&[(x1, y), (end, y)], 6.5, theme::FLOW_LIGHT);
        self.arrow_head(self.at(x2, y), vec2(dir, 0.0), 18.0, theme::FLOW);
        let vx = (x1 + x2) / 2.0;
        let c = self.at(vx, y);
        self.p.circle(c, self.l(12.0), theme::SURFACE, Stroke::new(self.l(2.2), theme::FLOW));
        let k = self.l(6.5);
        self.p.add(Shape::convex_polygon(vec![c + vec2(-k, -k), c + vec2(k, -k), c], theme::FLOW, Stroke::NONE));
        self.p.add(Shape::convex_polygon(vec![c, c + vec2(k, k), c + vec2(-k, k)], theme::FLOW, Stroke::NONE));
        if let Some(label) = label {
            self.text(vx, y + 20.0, label, 13.5, theme::INK, Align2::CENTER_TOP);
        }
    }

    /// En variabel i ett orsaksdiagram.
    fn var(&self, cx: f32, cy: f32, text: &str) -> Rect {
        let galley = self.p.layout_no_wrap(text.to_string(), theme::semibold(self.l(15.0)), theme::INK);
        let r = Rect::from_center_size(self.at(cx, cy), galley.size() + vec2(self.l(26.0), self.l(18.0)));
        self.p.add(ui_kit::soft_shadow(r, self.l(10.0), 1));
        self.p.rect(r, cr(self.l(10.0)), theme::SURFACE, Stroke::new(self.l(1.3), theme::LINE_STRONG), StrokeKind::Inside);
        self.p.galley(r.center() - galley.size() / 2.0, galley, theme::INK);
        r
    }

    /// En pill för en konstant eller hjälpvariabel.
    fn pill(&self, cx: f32, cy: f32, text: &str, constant: bool) -> Rect {
        let (fill, stroke) = if constant { (theme::CONST_FILL, theme::CONST) } else { (theme::AUX_FILL, theme::AUX) };
        let galley = self.p.layout_no_wrap(text.to_string(), theme::semibold(self.l(14.5)), theme::INK);
        let extra = if constant { 22.0 } else { 0.0 };
        let r = Rect::from_center_size(self.at(cx, cy), galley.size() + vec2(self.l(28.0 + extra), self.l(16.0)));
        let radius = if constant { self.l(8.0) } else { r.height() / 2.0 };
        self.p.add(ui_kit::soft_shadow(r, radius, 0));
        self.p.rect(r, cr(radius), fill, Stroke::new(self.l(1.8), stroke), StrokeKind::Inside);
        let mut tx = r.center().x - galley.size().x / 2.0;
        if constant {
            let gx = r.left() + self.l(16.0);
            let y = r.center().y;
            self.p.line_segment([pos2(gx - self.l(6.0), y), pos2(gx + self.l(6.0), y)], Stroke::new(self.l(2.0), stroke.gamma_multiply(0.5)));
            self.p.circle_filled(pos2(gx + self.l(1.5), y), self.l(3.5), stroke);
            tx += self.l(extra / 2.0);
        }
        self.p.galley(pos2(tx, r.center().y - galley.size().y / 2.0), galley, theme::INK);
        r
    }

    /// Böjd pil mellan två rutor, valfritt med + eller −.
    fn arrow(&self, a: Rect, b: Rect, curve: f32, sign: Option<bool>, color: Color32) {
        let (p0, p2) = (a.center(), b.center());
        let d = p2 - p0;
        let len = d.length().max(1.0);
        let perp = vec2(-d.y, d.x) / len;
        let c = p0 + d * 0.5 + perp * curve * len;
        let bez = |t: f32| {
            let u = 1.0 - t;
            pos2(u * u * p0.x + 2.0 * u * t * c.x + t * t * p2.x, u * u * p0.y + 2.0 * u * t * c.y + t * t * p2.y)
        };
        let (a2, b2) = (a.expand(self.l(5.0)), b.expand(self.l(5.0)));
        let n = 80;
        let t0 = (0..=n).map(|i| i as f32 / n as f32).find(|&t| !a2.contains(bez(t))).unwrap_or(0.0);
        let t1 = (0..=n).rev().map(|i| i as f32 / n as f32).find(|&t| !b2.contains(bez(t))).unwrap_or(1.0);
        let pts: Vec<Pos2> = (0..=24).map(|i| bez(t0 + (t1 - t0) * i as f32 / 24.0)).collect();
        let m = pts.len();
        let dir = pts[m - 1] - pts[m - 3];
        let mut shaft = pts.clone();
        shaft[m - 1] = pts[m - 1] - dir.normalized() * self.l(8.0);
        self.p.add(Shape::line(shaft, Stroke::new(self.l(2.2), color)));
        self.arrow_head(pts[m - 1], dir, 12.0, color);
        if let Some(positive) = sign {
            let at = pts[m - 6];
            let dd = (pts[m - 1] - pts[m - 7]).normalized();
            let side = if curve >= 0.0 { 1.0 } else { -1.0 };
            let sp = at + vec2(-dd.y, dd.x) * self.l(14.0) * side;
            let (txt, col) = if positive { ("+", theme::POSITIVE) } else { ("−", theme::NEGATIVE) };
            self.p.circle(sp, self.l(11.0), theme::SURFACE, Stroke::new(self.l(1.6), col));
            self.p.text(sp, Align2::CENTER_CENTER, txt, theme::bold(self.l(18.0)), col);
        }
    }

    fn loop_mark(&self, x: f32, y: f32, letter: &str, color: Color32) {
        let c = self.at(x, y);
        let r = self.l(18.0);
        let pts: Vec<Pos2> = (0..=24)
            .map(|k| {
                let a = (40.0 + 290.0 * k as f32 / 24.0).to_radians();
                c + vec2(a.cos(), -a.sin()) * r
            })
            .collect();
        let n = pts.len();
        self.p.add(Shape::line(pts.clone(), Stroke::new(self.l(2.6), color)));
        self.arrow_head(pts[n - 1], pts[n - 1] - pts[n - 2], 9.0, color);
        self.p.text(c, Align2::CENTER_CENTER, letter, theme::bold(self.l(15.0)), color);
    }

    /// Ett litet diagram med axlar. Punkterna anges i 0..1 för båda axlarna.
    fn graph(&self, x: f32, y: f32, w: f32, h: f32, series: &[(&[(f32, f32)], Color32, bool)], title: &str) {
        let frame = Rect::from_min_size(self.at(x, y), vec2(w, h) * self.s);
        self.p.rect(frame, cr(self.l(10.0)), theme::SURFACE, Stroke::new(self.l(1.0), theme::LINE), StrokeKind::Inside);
        let plot = Rect::from_min_max(frame.min + vec2(self.l(18.0), self.l(30.0)), frame.max - vec2(self.l(12.0), self.l(16.0)));
        self.p.line_segment([plot.left_top(), plot.left_bottom()], Stroke::new(self.l(1.5), theme::LINE_STRONG));
        self.p.line_segment([plot.left_bottom(), plot.right_bottom()], Stroke::new(self.l(1.5), theme::LINE_STRONG));
        self.p.text(frame.left_top() + vec2(self.l(12.0), self.l(9.0)), Align2::LEFT_TOP, title, theme::semibold(self.l(13.0)), theme::INK_2);
        for (pts, color, dashed) in series {
            let pts: Vec<Pos2> = pts
                .iter()
                .map(|&(u, v)| pos2(plot.left() + u * plot.width(), plot.bottom() - v * plot.height()))
                .collect();
            if *dashed {
                self.p.extend(Shape::dashed_line(&pts, Stroke::new(self.l(2.0), *color), self.l(6.0), self.l(4.0)));
            } else {
                self.p.add(Shape::line(pts, Stroke::new(self.l(3.0), *color)));
            }
        }
    }
}

fn curve(f: impl Fn(f32) -> f32) -> Vec<(f32, f32)> {
    (0..=40).map(|i| i as f32 / 40.0).map(|u| (u, f(u))).collect()
}

/// Ritar en illustration centrerad i `rect`.
pub fn draw(painter: &Painter, rect: Rect, which: Illustration) {
    let s = (rect.width() / W).min(rect.height() / H);
    let origin = rect.center() - vec2(W, H) * s / 2.0;
    let pen = Pen { p: painter, origin, s };
    match which {
        Illustration::Title => title(&pen),
        Illustration::MapVsWorld => map_vs_world(&pen),
        Illustration::WhyModel => why_model(&pen),
        Illustration::System => system(&pen),
        Illustration::CldArrows => cld_arrows(&pen),
        Illustration::LoopR => loop_r(&pen),
        Illustration::LoopB => loop_b(&pen),
        Illustration::Bathtub => bathtub(&pen),
        Illustration::Flows => flows(&pen),
        Illustration::ConstAux => const_aux(&pen),
        Illustration::TimeSteps => time_steps(&pen),
        Illustration::Behaviours => behaviours(&pen),
        Illustration::Riksbank => riksbank(&pen),
        Illustration::Spaghetti => spaghetti(&pen),
        Illustration::Limits => limits(&pen),
        Illustration::Summary => summary(&pen),
    }
}

fn title(pen: &Pen) {
    pen.cloud(40.0, 200.0);
    pen.pipe(56.0, 176.0, 200.0, Some("Inflöde"));
    let stock = pen.stock(250.0, 200.0, 140.0, 96.0, 0.55, "Lager");
    pen.pipe(320.0, 438.0, 200.0, Some("Utflöde"));
    pen.cloud(456.0, 200.0);
    let a = pen.var(150.0, 64.0, "Orsak");
    let b = pen.var(350.0, 64.0, "Verkan");
    pen.arrow(a, b, 0.25, Some(true), theme::INK);
    pen.arrow(b, a, 0.25, Some(true), theme::INK);
    pen.loop_mark(250.0, 64.0, "R", theme::REINFORCING);
    let _ = stock;
}

fn map_vs_world(pen: &Pen) {
    // Verkligheten: mycket detaljer.
    let left = pen.rect(118.0, 175.0, 210.0, 230.0);
    pen.p.rect(left, cr(pen.l(12.0)), Color32::from_rgb(232, 242, 228), Stroke::new(pen.l(1.0), theme::LINE), StrokeKind::Inside);
    let mut seed = 7u32;
    let mut rnd = || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (seed >> 8) as f32 / (1u32 << 24) as f32
    };
    pen.line(&[(20.0, 120.0), (90.0, 140.0), (130.0, 200.0), (215.0, 230.0)], 7.0, Color32::from_rgb(190, 190, 180));
    pen.line(&[(60.0, 280.0), (100.0, 190.0), (140.0, 80.0)], 6.0, Color32::from_rgb(190, 190, 180));
    for _ in 0..55 {
        let (x, y) = (22.0 + rnd() * 190.0, 70.0 + rnd() * 215.0);
        if rnd() > 0.35 {
            pen.p.circle_filled(pen.at(x, y), pen.l(4.0 + rnd() * 4.0), Color32::from_rgb(110, 165, 100));
        } else {
            let r = pen.rect(x, y, 10.0, 9.0);
            pen.p.rect_filled(r, cr(pen.l(1.5)), Color32::from_rgb(200, 120, 90));
        }
    }
    pen.text(118.0, 52.0, "Verkligheten", 15.0, theme::INK, Align2::CENTER_BOTTOM);
    // Modellen: bara det viktiga.
    let right = pen.rect(362.0, 175.0, 210.0, 230.0);
    pen.p.rect(right, cr(pen.l(12.0)), theme::SURFACE, Stroke::new(pen.l(1.0), theme::LINE_STRONG), StrokeKind::Inside);
    let pts = [(290.0, 120.0, "Hemma"), (420.0, 160.0, "Skolan"), (330.0, 250.0, "Affären")];
    pen.line(&[(290.0, 120.0), (420.0, 160.0)], 3.0, theme::INK_2);
    pen.line(&[(290.0, 120.0), (330.0, 250.0)], 3.0, theme::INK_2);
    for (x, y, name) in pts {
        pen.p.circle(pen.at(x, y), pen.l(8.0), theme::STOCK, Stroke::new(pen.l(2.0), theme::SURFACE));
        // Etiketten ovanför eller under punkten, där den inte krockar med vägarna.
        if y < 200.0 {
            pen.text(x, y - 13.0, name, 13.0, theme::INK, Align2::CENTER_BOTTOM);
        } else {
            pen.text(x, y + 13.0, name, 13.0, theme::INK, Align2::CENTER_TOP);
        }
    }
    pen.text(362.0, 52.0, "Modellen", 15.0, theme::INK, Align2::CENTER_BOTTOM);
    pen.line(&[(228.0, 175.0), (246.0, 175.0)], 3.0, theme::SELECT);
    pen.arrow_head(pen.at(256.0, 175.0), Vec2::X, 12.0, theme::SELECT);
    pen.text(241.0, 188.0, "förenkla", 13.0, theme::MUTED, Align2::CENTER_TOP);
}

fn why_model(pen: &Pen) {
    let cards = [(85.0, "Förstå"), (240.0, "Förutsäga"), (395.0, "Testa \"tänk om\"")];
    for (x, label) in cards {
        let r = pen.rect(x, 170.0, 140.0, 190.0);
        pen.p.add(ui_kit::soft_shadow(r, pen.l(14.0), 1));
        pen.p.rect(r, cr(pen.l(14.0)), theme::SURFACE, Stroke::new(pen.l(1.0), theme::LINE), StrokeKind::Inside);
        pen.text(x, 245.0, label, 15.0, theme::INK, Align2::CENTER_CENTER);
    }
    // Förstå: ett förstoringsglas över en loop.
    pen.loop_mark(78.0, 150.0, "?", theme::BALANCING);
    pen.p.circle_stroke(pen.at(78.0, 150.0), pen.l(32.0), Stroke::new(pen.l(4.0), theme::INK_2));
    pen.line(&[(101.0, 173.0), (122.0, 196.0)], 7.0, theme::INK_2);
    // Förutsäga: känd kurva + streckad fortsättning.
    pen.line(&[(185.0, 200.0), (205.0, 190.0), (225.0, 170.0), (240.0, 160.0)], 3.0, theme::STOCK);
    pen.dashed(&[(240.0, 160.0), (260.0, 140.0), (290.0, 110.0)], 3.0, theme::STOCK);
    pen.dashed(&[(240.0, 160.0), (265.0, 160.0), (290.0, 150.0)], 2.0, theme::MUTED);
    pen.text(292.0, 100.0, "?", 18.0, theme::MUTED, Align2::CENTER_CENTER);
    // Testa: ett reglage och två kurvor.
    pen.line(&[(345.0, 112.0), (445.0, 112.0)], 4.0, theme::LINE_STRONG);
    pen.line(&[(345.0, 112.0), (410.0, 112.0)], 4.0, theme::CONST);
    pen.p.circle(pen.at(410.0, 112.0), pen.l(8.0), theme::SURFACE, Stroke::new(pen.l(2.5), theme::CONST));
    pen.line(&[(345.0, 200.0), (380.0, 185.0), (410.0, 160.0), (445.0, 140.0)], 3.0, theme::FLOW);
    pen.dashed(&[(345.0, 200.0), (385.0, 195.0), (445.0, 180.0)], 2.0, theme::MUTED);
}

fn system(pen: &Pen) {
    let wanted = pen.var(240.0, 40.0, "Önskad temperatur");
    let gap = pen.var(110.0, 150.0, "Skillnad");
    let pump = pen.var(370.0, 150.0, "Värmepumpens effekt");
    let inside = pen.var(240.0, 280.0, "Inomhustemperatur");
    pen.arrow(wanted, gap, 0.15, Some(true), theme::INK);
    pen.arrow(gap, pump, -0.1, Some(true), theme::INK);
    pen.arrow(pump, inside, -0.25, Some(true), theme::INK);
    pen.arrow(inside, gap, -0.25, Some(false), theme::INK);
    pen.loop_mark(240.0, 200.0, "B", theme::BALANCING);
}

fn cld_arrows(pen: &Pen) {
    pen.text(240.0, 40.0, "Samma håll", 15.0, theme::POSITIVE, Align2::CENTER_CENTER);
    let a = pen.var(110.0, 100.0, "Fler bilar");
    let b = pen.var(370.0, 100.0, "Mer avgaser");
    pen.arrow(a, b, 0.0, Some(true), theme::INK);
    pen.text(240.0, 195.0, "Motsatt håll", 15.0, theme::NEGATIVE, Align2::CENTER_CENTER);
    let c = pen.var(110.0, 255.0, "Högre pris");
    let d = pen.var(370.0, 255.0, "Färre köper");
    pen.arrow(c, d, 0.0, Some(false), theme::INK);
}

fn loop_r(pen: &Pen) {
    let a = pen.var(90.0, 95.0, "Kaniner");
    let b = pen.var(290.0, 95.0, "Födslar");
    pen.arrow(a, b, 0.35, Some(true), theme::INK);
    pen.arrow(b, a, 0.35, Some(true), theme::INK);
    pen.loop_mark(190.0, 95.0, "R", theme::REINFORCING);
    let c = curve(|u| ((u * 3.0).exp() - 1.0) / (3f32.exp() - 1.0));
    pen.graph(250.0, 175.0, 220.0, 150.0, &[(&c, theme::REINFORCING, false)], "Kaniner över tid");
}

fn loop_b(pen: &Pen) {
    let a = pen.var(100.0, 60.0, "Temperatur");
    let b = pen.var(330.0, 60.0, "Värme från elementet");
    pen.arrow(a, b, 0.35, Some(false), theme::INK);
    pen.arrow(b, a, 0.35, Some(true), theme::INK);
    pen.loop_mark(205.0, 60.0, "B", theme::BALANCING);
    let c = curve(|u| 0.75 - 0.6 * (-4.5 * u).exp());
    let target = [(0.0, 0.75), (1.0, 0.75)];
    pen.graph(250.0, 175.0, 220.0, 150.0, &[(&target, theme::MUTED, true), (&c, theme::BALANCING, false)], "Temperatur (streckat: termostaten)");
}

fn bathtub(pen: &Pen) {
    // Badkaret.
    let tub = pen.rect(240.0, 170.0, 250.0, 120.0);
    let water = Rect::from_min_max(pos2(tub.left() + pen.l(8.0), tub.top() + pen.l(46.0)), tub.max - vec2(pen.l(8.0), pen.l(8.0)));
    pen.p.rect(tub, cr(pen.l(40.0)), theme::SURFACE, Stroke::new(pen.l(3.0), theme::INK_2), StrokeKind::Inside);
    pen.p.rect_filled(water, cr(pen.l(34.0)), theme::STOCK_LEVEL);
    // Kranen.
    pen.line(&[(145.0, 40.0), (145.0, 80.0), (165.0, 80.0)], 8.0, theme::INK_2);
    for k in 0..4 {
        let x = 160.0 + k as f32 * 4.0;
        pen.line(&[(x, 88.0), (x, 140.0)], 2.0, theme::STOCK);
    }
    // Avloppet.
    pen.line(&[(330.0, 230.0), (330.0, 285.0)], 8.0, theme::INK_2);
    pen.arrow_head(pen.at(330.0, 300.0), Vec2::Y, 14.0, theme::STOCK);
    // Etiketter.
    pen.text(100.0, 60.0, "Inflöde", 15.0, theme::FLOW, Align2::RIGHT_CENTER);
    pen.text_regular(100.0, 80.0, "kranen", 13.0, theme::MUTED, Align2::RIGHT_CENTER);
    pen.text(395.0, 135.0, "Lager", 15.0, theme::STOCK, Align2::LEFT_CENTER);
    pen.text_regular(395.0, 155.0, "vattnet", 13.0, theme::MUTED, Align2::LEFT_CENTER);
    pen.text(350.0, 290.0, "Utflöde", 15.0, theme::FLOW, Align2::LEFT_CENTER);
    pen.text_regular(350.0, 310.0, "avloppet", 13.0, theme::MUTED, Align2::LEFT_CENTER);
}

fn flows(pen: &Pen) {
    pen.cloud(36.0, 120.0);
    pen.pipe(52.0, 168.0, 120.0, Some("Inflöde"));
    pen.stock(240.0, 120.0, 130.0, 90.0, 0.5, "Lager");
    pen.pipe(306.0, 428.0, 120.0, Some("Utflöde"));
    pen.cloud(446.0, 120.0);
    let box_ = pen.rect(240.0, 270.0, 380.0, 64.0);
    pen.p.rect(box_, cr(pen.l(12.0)), theme::SURFACE_2, Stroke::NONE, StrokeKind::Inside);
    pen.text(240.0, 262.0, "Ändring i lagret  =  inflöde  −  utflöde", 16.0, theme::INK, Align2::CENTER_CENTER);
    pen.text_regular(240.0, 285.0, "per tidsenhet, t.ex. liter per minut", 13.5, theme::MUTED, Align2::CENTER_CENTER);
}

fn const_aux(pen: &Pen) {
    let stock = pen.stock(330.0, 250.0, 130.0, 84.0, 0.6, "Kaniner");
    pen.cloud(90.0, 250.0);
    pen.pipe(106.0, 262.0, 250.0, Some("Födslar"));
    let food = pen.pill(120.0, 60.0, "Mat på ön", true);
    let per = pen.pill(330.0, 110.0, "Mat per kanin", false);
    let valve = pen.rect(184.0, 250.0, 26.0, 26.0);
    pen.arrow(food, per, 0.1, None, theme::DEP);
    pen.arrow(stock, per, 0.0, None, theme::DEP);
    pen.arrow(per, valve, -0.15, None, theme::DEP);
    pen.text_regular(330.0, 88.0, "= Mat på ön ÷ Kaniner", 13.5, theme::AUX, Align2::CENTER_BOTTOM);
    pen.text_regular(120.0, 84.0, "konstant: du ändrar den", 13.0, theme::MUTED, Align2::CENTER_TOP);
}

fn time_steps(pen: &Pen) {
    let frame = pen.rect(240.0, 170.0, 420.0, 280.0);
    pen.p.rect(frame, cr(pen.l(12.0)), theme::SURFACE, Stroke::new(pen.l(1.0), theme::LINE), StrokeKind::Inside);
    let (x0, y0, w, h) = (60.0, 290.0, 360.0, 220.0);
    pen.line(&[(x0, y0 - h), (x0, y0), (x0 + w, y0)], 1.5, theme::LINE_STRONG);
    let f = |u: f32| ((u * 2.2).exp() - 1.0) / (2.2f32.exp() - 1.0);
    let smooth: Vec<(f32, f32)> = (0..=40).map(|i| i as f32 / 40.0).map(|u| (x0 + u * w, y0 - f(u) * h)).collect();
    // Stora steg: räkna lutningen i början av varje steg.
    let steps = 5;
    let mut v = 0.0f32;
    let mut pts = vec![(x0, y0)];
    let rate = |v: f32| 2.2 * (v + 1.0 / (2.2f32.exp() - 1.0));
    for k in 0..steps {
        let u0 = k as f32 / steps as f32;
        let u1 = (k + 1) as f32 / steps as f32;
        v += rate(v) * (u1 - u0);
        pts.push((x0 + u1 * w, y0 - v.min(1.05) * h));
        let _ = u0;
    }
    pen.line(&pts, 3.0, theme::CONST);
    for &(x, y) in &pts {
        pen.p.circle_filled(pen.at(x, y), pen.l(4.5), theme::CONST);
    }
    pen.line(&smooth, 3.0, theme::STOCK);
    pen.text(x0 + 6.0, y0 - h - 6.0, "Små steg: jämn kurva", 13.5, theme::STOCK, Align2::LEFT_BOTTOM);
    pen.text(x0 + w * 0.6, y0 - 34.0, "Stora steg: hackig", 13.5, theme::CONST, Align2::LEFT_TOP);
    // dt-markering.
    pen.line(&[(x0, y0 + 14.0), (x0 + w / steps as f32, y0 + 14.0)], 2.0, theme::INK_2);
    pen.text(x0 + w / 10.0, y0 + 18.0, "dt", 13.0, theme::INK_2, Align2::CENTER_TOP);
}

fn behaviours(pen: &Pen) {
    let grow = curve(|u| ((u * 3.2).exp() - 1.0) / (3.2f32.exp() - 1.0));
    let goal = curve(|u| 0.8 * (1.0 - (-4.0 * u).exp()));
    let s = curve(|u| 0.85 / (1.0 + (-11.0 * (u - 0.45)).exp()));
    pen.graph(10.0, 60.0, 148.0, 150.0, &[(&grow, theme::REINFORCING, false)], "Växer allt snabbare");
    pen.graph(166.0, 60.0, 148.0, 150.0, &[(&goal, theme::BALANCING, false)], "Söker ett mål");
    pen.graph(322.0, 60.0, 148.0, 150.0, &[(&s, theme::FLOW, false)], "S-kurva");
    pen.text(84.0, 230.0, "R vinner", 15.0, theme::REINFORCING, Align2::CENTER_TOP);
    pen.text(240.0, 230.0, "B vinner", 15.0, theme::BALANCING, Align2::CENTER_TOP);
    pen.text(396.0, 230.0, "R först, sedan B", 15.0, theme::FLOW, Align2::CENTER_TOP);
}

fn riksbank(pen: &Pen) {
    let frame = pen.rect(240.0, 170.0, 470.0, 320.0);
    pen.p.rect(frame, cr(pen.l(12.0)), theme::SURFACE, Stroke::new(pen.l(1.0), theme::LINE), StrokeKind::Inside);
    let (x0, y0, w, h) = (58.0, 280.0, 390.0, 210.0);
    let (t0, t1, vmax) = (2021.0, 2024.75, 4.5);
    let px = |t: f32| x0 + (t - t0) / (t1 - t0) * w;
    let py = |v: f32| y0 - v / vmax * h;
    for v in [0.0, 1.0, 2.0, 3.0, 4.0] {
        pen.line(&[(x0, py(v)), (x0 + w, py(v))], 1.0, theme::LINE);
        pen.text_regular(x0 - 8.0, py(v), &format!("{} %", v as i32), 12.5, theme::MUTED, Align2::RIGHT_CENTER);
    }
    for year in [2021, 2022, 2023, 2024] {
        pen.text_regular(px(year as f32 + 0.5), y0 + 8.0, &year.to_string(), 12.5, theme::MUTED, Align2::CENTER_TOP);
    }
    // Prognosen i september 2021: noll procent till hösten 2024.
    pen.dashed(&[(px(2021.7), py(0.0)), (px(2024.7), py(0.0))], 3.0, theme::CONST);
    pen.text(px(2023.3), py(0.0) - 8.0, "Prognos 2021: 0 % till 2024", 13.0, theme::CONST, Align2::CENTER_BOTTOM);
    // Det som hände: styrräntan steg för steg.
    let changes = [
        (2021.0, 0.0),
        (2022.32, 0.25),
        (2022.5, 0.75),
        (2022.72, 1.75),
        (2022.9, 2.5),
        (2023.1, 3.0),
        (2023.32, 3.5),
        (2023.49, 3.75),
        (2023.72, 4.0),
    ];
    let mut pts = Vec::new();
    for (i, &(t, v)) in changes.iter().enumerate() {
        if i > 0 {
            pts.push((px(t), py(changes[i - 1].1)));
        }
        pts.push((px(t), py(v)));
    }
    pts.push((px(2024.3), py(4.0)));
    pen.line(&pts, 3.5, theme::STOCK);
    pen.text(px(2022.2), py(3.3), "Det som hände", 13.5, theme::STOCK, Align2::RIGHT_CENTER);
    pen.text(x0, 40.0, "Riksbankens styrränta", 14.0, theme::INK, Align2::LEFT_CENTER);
    pen.text_regular(x0 + w, 40.0, "Förenklad bild", 12.0, theme::MUTED, Align2::RIGHT_CENTER);
}

fn spaghetti(pen: &Pen) {
    let colors = [
        theme::STOCK,
        theme::FLOW,
        theme::CONST,
        theme::AUX,
        theme::REINFORCING,
        theme::ERROR,
        Color32::from_rgb(18, 145, 160),
        Color32::from_rgb(160, 125, 20),
    ];
    let mut seed = 2010u32;
    let mut rnd = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / (1u32 << 24) as f32
    };
    let centers = [(90.0, 80.0), (240.0, 70.0), (390.0, 90.0), (110.0, 230.0), (250.0, 200.0), (380.0, 250.0), (240.0, 300.0)];
    let mut nodes = Vec::new();
    for (k, &(cx, cy)) in centers.iter().enumerate() {
        for _ in 0..7 {
            nodes.push((cx + (rnd() - 0.5) * 120.0, cy + (rnd() - 0.5) * 80.0, k));
        }
    }
    for _ in 0..110 {
        let a = nodes[(rnd() * nodes.len() as f32) as usize % nodes.len()];
        let b = nodes[(rnd() * nodes.len() as f32) as usize % nodes.len()];
        let mid = ((a.0 + b.0) / 2.0 + (rnd() - 0.5) * 80.0, (a.1 + b.1) / 2.0 + (rnd() - 0.5) * 80.0);
        let pts: Vec<(f32, f32)> = (0..=12)
            .map(|i| {
                let t = i as f32 / 12.0;
                let u = 1.0 - t;
                (u * u * a.0 + 2.0 * u * t * mid.0 + t * t * b.0, u * u * a.1 + 2.0 * u * t * mid.1 + t * t * b.1)
            })
            .collect();
        pen.line(&pts, 1.0, colors[a.2 % colors.len()].gamma_multiply(0.75));
    }
    for (x, y, k) in nodes {
        let r = pen.rect(x, y, 26.0, 10.0);
        pen.p.rect(r, cr(pen.l(3.0)), theme::SURFACE, Stroke::new(pen.l(1.0), colors[k % colors.len()]), StrokeKind::Inside);
    }
    pen.text_regular(470.0, 335.0, "Förenklad efterbildning, inte originalet", 12.0, theme::MUTED, Align2::RIGHT_BOTTOM);
}

fn limits(pen: &Pen) {
    let boundary = pen.rect(240.0, 170.0, 250.0, 150.0);
    pen.p.rect_filled(boundary, cr(pen.l(16.0)), theme::SURFACE);
    let corners = [boundary.left_top(), boundary.right_top(), boundary.right_bottom(), boundary.left_bottom(), boundary.left_top()];
    pen.p.extend(Shape::dashed_line(&corners, Stroke::new(pen.l(2.0), theme::INK_2), pen.l(8.0), pen.l(6.0)));
    pen.text(240.0, 110.0, "Modellen", 14.0, theme::INK_2, Align2::CENTER_CENTER);
    pen.cloud(150.0, 185.0);
    pen.pipe(166.0, 222.0, 185.0, None);
    pen.stock(276.0, 185.0, 90.0, 62.0, 0.5, "Lager");
    let outside = [
        (65.0, 60.0, "Krig"),
        (415.0, 60.0, "Pandemi"),
        (60.0, 300.0, "Nya uppfinningar"),
        (420.0, 300.0, "Människors val"),
        (440.0, 175.0, "Väder"),
    ];
    for (x, y, label) in outside {
        pen.cloud(x, y);
        pen.text(x, y + 18.0, label, 13.0, theme::INK_2, Align2::CENTER_TOP);
    }
}

fn summary(pen: &Pen) {
    let items: [(f32, f32, &str); 6] = [
        (85.0, 95.0, "Lager"),
        (240.0, 95.0, "Flöde"),
        (395.0, 95.0, "Konstant"),
        (85.0, 245.0, "Hjälpvariabel"),
        (240.0, 245.0, "Förstärkande loop"),
        (395.0, 245.0, "Balanserande loop"),
    ];
    for (i, (x, y, label)) in items.iter().enumerate() {
        let card = pen.rect(*x, *y, 140.0, 130.0);
        pen.p.rect(card, cr(pen.l(14.0)), theme::SURFACE, Stroke::new(pen.l(1.0), theme::LINE), StrokeKind::Inside);
        let gy = y - 14.0;
        match i {
            0 => {
                pen.stock(*x, gy, 80.0, 50.0, 0.5, "");
            }
            1 => pen.pipe(x - 50.0, x + 50.0, gy, None),
            2 => {
                pen.pill(*x, gy, "Värde", true);
            }
            3 => {
                pen.pill(*x, gy, "Formel", false);
            }
            4 => pen.loop_mark(*x, gy, "R", theme::REINFORCING),
            _ => pen.loop_mark(*x, gy, "B", theme::BALANCING),
        }
        pen.text(*x, y + 38.0, label, 14.0, theme::INK, Align2::CENTER_CENTER);
    }
}
