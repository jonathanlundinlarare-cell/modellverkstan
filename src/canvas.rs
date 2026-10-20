//! Ritytan: visar modellen och hanterar musen (flytta, rita pilar och flöden, zooma).

use crate::app::{App, Drag, Selection, Tool};
use crate::model::{FeedbackLoop, Link, ModelKind, Node, NodeKind};
use crate::theme::{self, cr};
use egui::text::LayoutJob;
use egui::{
    pos2, vec2, Align, Align2, Color32, CursorIcon, FontId, Galley, Painter, PointerButton, Pos2, Rect, Response,
    Sense, Shape, Stroke, StrokeKind, Ui, Vec2,
};
use std::sync::Arc;

/// Omvandling mellan modellens koordinater och skärmen.
#[derive(Clone, Copy)]
struct Xf {
    origin: Pos2,
    offset: Vec2,
    zoom: f32,
}

impl Xf {
    fn new(app: &App, rect: Rect) -> Self {
        Self { origin: rect.min, offset: app.view.offset, zoom: app.view.zoom }
    }

    fn screen(&self, p: [f32; 2]) -> Pos2 {
        self.origin + self.offset + vec2(p[0], p[1]) * self.zoom
    }

    fn world(&self, s: Pos2) -> [f32; 2] {
        let w = (s - self.origin - self.offset) / self.zoom;
        [w.x, w.y]
    }
}

/// Var en nod hamnar på skärmen.
struct Geo {
    id: u32,
    kind: NodeKind,
    /// Själva formen (lådan, ventilen, ovalen).
    rect: Rect,
    /// Yta som går att klicka på (inklusive etikett).
    hit: Rect,
    galley: Arc<Galley>,
}

fn layout(ui: &Ui, nodes: &[Node], xf: Xf) -> Vec<Geo> {
    let z = xf.zoom;
    nodes
        .iter()
        .map(|n| {
            let mut job = LayoutJob::simple(n.name.clone(), FontId::proportional(15.0 * z), theme::INK, 170.0 * z);
            job.halign = Align::Center;
            let galley = ui.fonts(|f| f.layout_job(job));
            let t = galley.size();
            let c = xf.screen(n.pos);
            let (rect, hit) = match n.kind {
                NodeKind::Stock => {
                    let size = vec2((t.x + 32.0 * z).max(120.0 * z), (t.y + 44.0 * z).max(66.0 * z));
                    let r = Rect::from_center_size(c, size);
                    (r, r)
                }
                NodeKind::Flow => {
                    let r = Rect::from_center_size(c, Vec2::splat(24.0 * z));
                    let label = Rect::from_center_size(c + vec2(0.0, 14.0 * z + t.y / 2.0), t);
                    (r, r.union(label))
                }
                NodeKind::Aux | NodeKind::Constant => {
                    let r = Rect::from_center_size(c, vec2(t.x + 28.0 * z, (t.y + 16.0 * z).max(34.0 * z)));
                    (r, r)
                }
                NodeKind::Variable => {
                    let r = Rect::from_center_size(c, t + vec2(22.0, 14.0) * z);
                    (r, r)
                }
            };
            Geo { id: n.id, kind: n.kind, rect, hit, galley }
        })
        .collect()
}

fn hit_node(geos: &[Geo], p: Pos2, only: Option<NodeKind>) -> Option<u32> {
    geos.iter()
        .rev()
        .filter(|g| only.is_none_or(|k| g.kind == k))
        .find(|g| g.hit.expand(3.0).contains(p))
        .map(|g| g.id)
}

fn geo<'a>(geos: &'a [Geo], id: u32) -> Option<&'a Geo> {
    geos.iter().find(|g| g.id == id)
}

/// Punkt på kanten av `r` i riktning mot `p`.
fn rect_edge_toward(r: Rect, p: Pos2) -> Pos2 {
    let c = r.center();
    let d = p - c;
    if d.length() < 1e-3 {
        return c;
    }
    let tx = if d.x.abs() > 1e-6 { (r.width() / 2.0) / d.x.abs() } else { f32::INFINITY };
    let ty = if d.y.abs() > 1e-6 { (r.height() / 2.0) / d.y.abs() } else { f32::INFINITY };
    c + d * tx.min(ty).min(1.0)
}

fn safe_normalized(v: Vec2, fallback: Vec2) -> Vec2 {
    if v.length() < 1e-3 {
        fallback
    } else {
        v.normalized()
    }
}

/// En böjd pil mellan två rutor. Ger punkterna längs pilen och böjens mittpunkt.
fn curve_points(a: Rect, b: Rect, curve: f32) -> Option<(Vec<Pos2>, Pos2)> {
    let p0 = a.center();
    let p2 = b.center();
    let d = p2 - p0;
    let len = d.length();
    if len < 1.0 {
        return None;
    }
    let perp = vec2(-d.y, d.x) / len;
    let c = p0 + d * 0.5 + perp * curve * len;
    let bez = |t: f32| {
        let u = 1.0 - t;
        pos2(
            u * u * p0.x + 2.0 * u * t * c.x + t * t * p2.x,
            u * u * p0.y + 2.0 * u * t * c.y + t * t * p2.y,
        )
    };
    let (a2, b2) = (a.expand(5.0), b.expand(5.0));
    let n = 80;
    let t0 = (0..=n).map(|i| i as f32 / n as f32).find(|&t| !a2.contains(bez(t)))?;
    let t1 = (0..=n).rev().map(|i| i as f32 / n as f32).find(|&t| !b2.contains(bez(t)))?;
    if t1 <= t0 {
        return None;
    }
    let pts = (0..=28).map(|i| bez(t0 + (t1 - t0) * i as f32 / 28.0)).collect();
    Some((pts, bez(0.5)))
}

fn dist_to_polyline(p: Pos2, pts: &[Pos2]) -> f32 {
    pts.windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let ab = b - a;
            let t = ((p - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
            (a + ab * t).distance(p)
        })
        .fold(f32::INFINITY, f32::min)
}

fn arrow_head(painter: &Painter, tip: Pos2, dir: Vec2, size: f32, color: Color32) {
    let d = safe_normalized(dir, Vec2::X);
    let n = vec2(-d.y, d.x);
    let back = tip - d * size;
    painter.add(Shape::convex_polygon(vec![tip, back + n * size * 0.5, back - n * size * 0.5], color, Stroke::NONE));
}

fn link_geometry(geos: &[Geo], l: &Link) -> Option<(Vec<Pos2>, Pos2)> {
    let a = geo(geos, l.from)?;
    let b = geo(geos, l.to)?;
    curve_points(a.rect, b.rect, l.curve)
}

/// Var +/−-märket sitter på en pil.
fn sign_position(pts: &[Pos2], curve: f32, z: f32) -> Pos2 {
    let n = pts.len();
    let d = safe_normalized(pts[n - 1] - pts[n - 7], Vec2::X);
    let side = if curve >= 0.0 { 1.0 } else { -1.0 };
    pts[n - 6] + vec2(-d.y, d.x) * 13.0 * z * side
}

fn hit_link(geos: &[Geo], links: &[Link], p: Pos2, z: f32) -> Option<usize> {
    links
        .iter()
        .enumerate()
        .filter_map(|(i, l)| {
            link_geometry(geos, l).map(|(pts, _)| {
                let on_sign = sign_position(&pts, l.curve, z).distance(p) - 6.0;
                (i, dist_to_polyline(p, &pts).min(on_sign))
            })
        })
        .filter(|(_, d)| *d < 14.0)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Etiketter som "R1", "B1", "B2" för looparna.
pub fn loop_labels(loops: &[FeedbackLoop]) -> Vec<String> {
    let (mut r, mut b) = (0, 0);
    loops
        .iter()
        .map(|l| {
            if l.reinforcing {
                r += 1;
                format!("R{r}")
            } else {
                b += 1;
                format!("B{b}")
            }
        })
        .collect()
}

/// Anpassar zoom och läge så att hela modellen syns.
fn fit_view(app: &mut App, size: Vec2) {
    if app.model.nodes.is_empty() {
        app.view.zoom = 1.0;
        app.view.offset = vec2(40.0, 40.0);
        return;
    }
    let mut bb = Rect::NOTHING;
    for n in &app.model.nodes {
        bb.extend_with(pos2(n.pos[0], n.pos[1]));
    }
    let bb = bb.expand2(vec2(110.0, 70.0));
    let zoom = (size.x / bb.width()).min(size.y / bb.height()).clamp(0.4, 1.25);
    app.view.zoom = zoom;
    app.view.offset = size / 2.0 - bb.center().to_vec2() * zoom;
}

pub fn zoom_by(app: &mut App, factor: f32, around: Option<Pos2>) {
    let rect = app.canvas_rect;
    let p = around.unwrap_or(rect.center());
    let xf = Xf::new(app, rect);
    let w = xf.world(p);
    app.view.zoom = (app.view.zoom * factor).clamp(0.3, 3.0);
    app.view.offset = p - rect.min - vec2(w[0], w[1]) * app.view.zoom;
}

pub fn request_fit(app: &mut App) {
    app.fit_pending = true;
}

pub fn show(app: &mut App, ui: &mut Ui) {
    let (resp, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let rect = resp.rect;
    app.canvas_rect = rect;
    if app.fit_pending && rect.width() > 50.0 {
        fit_view(app, rect.size());
        app.fit_pending = false;
    }

    if resp.hovered() {
        let (zoom, scroll) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta));
        if zoom != 1.0 {
            zoom_by(app, zoom, resp.hover_pos());
        }
        if scroll != Vec2::ZERO {
            app.view.offset += scroll;
        }
    }

    let geos = layout(ui, &app.model.nodes, Xf::new(app, rect));
    interact(app, ui, &resp, &geos, Xf::new(app, rect));

    let xf = Xf::new(app, rect);
    let geos = layout(ui, &app.model.nodes, xf);
    painter.rect_filled(rect, 0.0, theme::CANVAS_BG);
    draw_grid(&painter, rect, xf);
    draw(app, ui, &painter, &geos, xf, resp.hover_pos());
    draw_hints(app, &painter, rect);
}

fn draw_grid(painter: &Painter, rect: Rect, xf: Xf) {
    let step = 40.0 * xf.zoom;
    if step < 12.0 {
        return;
    }
    let start = xf.origin + xf.offset;
    let x0 = rect.left() + (start.x - rect.left()).rem_euclid(step);
    let y0 = rect.top() + (start.y - rect.top()).rem_euclid(step);
    let mut x = x0;
    while x < rect.right() {
        let mut y = y0;
        while y < rect.bottom() {
            painter.circle_filled(pos2(x, y), 1.2, theme::GRID);
            y += step;
        }
        x += step;
    }
}

fn interact(app: &mut App, ui: &Ui, resp: &Response, geos: &[Geo], xf: Xf) {
    if resp.dragged_by(PointerButton::Middle) || resp.dragged_by(PointerButton::Secondary) {
        app.view.offset += resp.drag_delta();
        return;
    }
    let is_cld = app.model.kind == ModelKind::Cld;
    let press = ui.input(|i| i.pointer.press_origin());
    let pointer = resp.interact_pointer_pos().or(resp.hover_pos());

    // Muspekare
    if let Some(p) = resp.hover_pos() {
        let icon = match (app.tool, app.drag) {
            (_, Drag::Move { .. } | Drag::Pan | Drag::Curve(_)) => CursorIcon::Grabbing,
            (Tool::Select, _) if hit_node(geos, p, None).is_some() => CursorIcon::Grab,
            (Tool::Select, _) if is_cld && hit_link(geos, &app.model.links, p, xf.zoom).is_some() => CursorIcon::PointingHand,
            (Tool::Place(_), _) => CursorIcon::Crosshair,
            (Tool::Flow | Tool::Link, _) => CursorIcon::Crosshair,
            _ => CursorIcon::Default,
        };
        ui.ctx().set_cursor_icon(icon);
    }

    match app.tool {
        Tool::Select => {
            if resp.drag_started_by(PointerButton::Primary) {
                let p = press.or(pointer).unwrap_or_default();
                app.drag = if let Some(li) = curve_handle_hit(app, geos, p) {
                    Drag::Curve(li)
                } else if let Some(id) = hit_node(geos, p, None) {
                    app.selection = Selection::Node(id);
                    let pos = app.model.node(id).map_or([0.0; 2], |n| n.pos);
                    let w = xf.world(p);
                    Drag::Move { id, grab: vec2(w[0] - pos[0], w[1] - pos[1]) }
                } else {
                    Drag::Pan
                };
            }
            if resp.dragged_by(PointerButton::Primary) {
                if let Some(p) = pointer {
                    match app.drag {
                        Drag::Move { id, grab } => {
                            let w = xf.world(p);
                            if let Some(n) = app.model.node_mut(id) {
                                n.pos = [w[0] - grab.x, w[1] - grab.y];
                            }
                        }
                        Drag::Pan => app.view.offset += resp.drag_delta(),
                        Drag::Curve(li) => {
                            if let Some(l) = app.model.links.get(li) {
                                if let (Some(a), Some(b)) = (geo(geos, l.from), geo(geos, l.to)) {
                                    let (p0, p2) = (a.rect.center(), b.rect.center());
                                    let d = p2 - p0;
                                    let len = d.length().max(1.0);
                                    let perp = vec2(-d.y, d.x) / len;
                                    let mid = p0 + d * 0.5;
                                    let curve = (2.0 * (p - mid).dot(perp) / len).clamp(-1.2, 1.2);
                                    app.model.links[li].curve = if curve.abs() < 0.03 { 0.0 } else { curve };
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            if resp.clicked() {
                if let Some(p) = pointer {
                    app.selection = if let Some(id) = hit_node(geos, p, None) {
                        Selection::Node(id)
                    } else if let Some(li) = is_cld.then(|| hit_link(geos, &app.model.links, p, xf.zoom)).flatten() {
                        Selection::Link(li)
                    } else {
                        Selection::None
                    };
                }
            }
            if resp.double_clicked() {
                if let Some(p) = pointer {
                    if let Some(id) = hit_node(geos, p, None) {
                        app.selection = Selection::Node(id);
                        app.focus_name = true;
                    } else if let Some(li) = is_cld.then(|| hit_link(geos, &app.model.links, p, xf.zoom)).flatten() {
                        app.model.links[li].positive = !app.model.links[li].positive;
                        app.selection = Selection::Link(li);
                    } else if is_cld {
                        let id = app.model.add_node(NodeKind::Variable, xf.world(p));
                        app.selection = Selection::Node(id);
                        app.focus_name = true;
                    }
                }
            }
        }
        Tool::Place(kind) => {
            if resp.clicked() {
                if let Some(p) = pointer {
                    if let Some(id) = hit_node(geos, p, None) {
                        app.selection = Selection::Node(id);
                    } else {
                        let id = app.model.add_node(kind, xf.world(p));
                        app.selection = Selection::Node(id);
                        app.focus_name = true;
                        app.tool = Tool::Select;
                    }
                }
            }
        }
        Tool::Flow => {
            if resp.drag_started_by(PointerButton::Primary) {
                let p = press.or(pointer).unwrap_or_default();
                app.drag = Drag::NewFlow { start: xf.world(p), from: hit_node(geos, p, Some(NodeKind::Stock)) };
            }
            if resp.drag_stopped() {
                if let (Drag::NewFlow { start, from }, Some(p)) = (app.drag, pointer) {
                    let to = hit_node(geos, p, Some(NodeKind::Stock));
                    if from.is_some() && from == to {
                        app.notify(ui.ctx(), "Ett flöde måste gå från ett lager till ett annat (eller till/från ett moln).");
                    } else {
                        let a = from.and_then(|id| app.model.node(id)).map_or(start, |n| n.pos);
                        let end = xf.world(p);
                        let b = to.and_then(|id| app.model.node(id)).map_or(end, |n| n.pos);
                        let pos = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
                        add_flow(app, pos, from, to);
                    }
                }
                app.drag = Drag::None;
            } else if resp.clicked() {
                if let Some(p) = pointer {
                    match hit_node(geos, p, Some(NodeKind::Stock)) {
                        Some(stock) => {
                            let sp = app.model.node(stock).map_or([0.0; 2], |n| n.pos);
                            add_flow(app, [sp[0] - 190.0, sp[1]], None, Some(stock));
                        }
                        None => {
                            add_flow(app, xf.world(p), None, None);
                        }
                    }
                    app.notify(ui.ctx(), "Tips: dra från ett lager till ett annat för att koppla flödet direkt.");
                }
            }
        }
        Tool::Link => {
            if resp.drag_started_by(PointerButton::Primary) {
                let p = press.or(pointer).unwrap_or_default();
                app.drag = match hit_node(geos, p, None) {
                    Some(from) => Drag::NewLink { from },
                    None => Drag::Pan,
                };
            }
            if resp.dragged_by(PointerButton::Primary) && app.drag == Drag::Pan {
                app.view.offset += resp.drag_delta();
            }
            if resp.drag_stopped() {
                if let (Drag::NewLink { from }, Some(p)) = (app.drag, pointer) {
                    if let Some(to) = hit_node(geos, p, None).filter(|&to| to != from) {
                        if let Some(i) = app.model.links.iter().position(|l| l.from == from && l.to == to) {
                            app.selection = Selection::Link(i);
                        } else {
                            app.model.links.push(Link { from, to, positive: true, curve: 0.2 });
                            app.selection = Selection::Link(app.model.links.len() - 1);
                        }
                    }
                }
                app.drag = Drag::None;
            } else if resp.clicked() {
                if let Some(p) = pointer {
                    if let Some(li) = hit_link(geos, &app.model.links, p, xf.zoom) {
                        app.model.links[li].positive = !app.model.links[li].positive;
                        app.selection = Selection::Link(li);
                    } else if let Some(id) = hit_node(geos, p, None) {
                        app.selection = Selection::Node(id);
                    }
                }
            }
        }
    }
    if resp.drag_stopped() {
        app.drag = Drag::None;
    }
}

fn add_flow(app: &mut App, pos: [f32; 2], from: Option<u32>, to: Option<u32>) {
    let id = app.model.add_node(NodeKind::Flow, pos);
    if let Some(n) = app.model.node_mut(id) {
        n.from = from;
        n.to = to;
    }
    app.selection = Selection::Node(id);
    app.focus_name = true;
    app.tool = Tool::Select;
}

fn curve_handle_hit(app: &App, geos: &[Geo], p: Pos2) -> Option<usize> {
    let Selection::Link(li) = app.selection else { return None };
    let l = app.model.links.get(li)?;
    let (_, mid) = link_geometry(geos, l)?;
    (mid.distance(p) < 12.0).then_some(li)
}

fn draw(app: &App, ui: &Ui, painter: &Painter, geos: &[Geo], xf: Xf, hover: Option<Pos2>) {
    let z = xf.zoom;
    let t = app.shown_time();

    match app.model.kind {
        ModelKind::Cld => draw_links(app, painter, geos, z),
        ModelKind::Sfd => {
            draw_pipes(app, painter, geos, z);
            draw_deps(app, painter, geos, z);
        }
    }

    for (g, n) in geos.iter().zip(&app.model.nodes) {
        let selected = app.selection == Selection::Node(n.id);
        let hovered = hover.is_some_and(|p| g.hit.contains(p));
        let error = app.compiled.errors.contains_key(&n.id);
        let value = match n.kind {
            NodeKind::Constant => Some(n.value),
            NodeKind::Variable => None,
            _ => app.result.value_at(n.id, t),
        };
        let level = (n.kind == NodeKind::Stock)
            .then(|| app.result.series.get(&n.id))
            .flatten()
            .and_then(|s| {
                let lo = s.iter().copied().fold(0.0, f64::min);
                let hi = s.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let v = value?;
                (hi - lo > 1e-9).then(|| ((v - lo) / (hi - lo)) as f32)
            });
        draw_node(painter, g, n, selected, hovered, error, value, level, z);
    }

    if app.model.kind == ModelKind::Cld {
        draw_loop_markers(app, painter, geos, z);
    }

    // Förhandsvisning medan man drar ut ett nytt flöde eller en ny pil.
    if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
        match app.drag {
            Drag::NewFlow { start, from } => {
                let a = from.and_then(|id| geo(geos, id)).map_or(xf.screen(start), |g| g.rect.center());
                painter.line_segment([a, p], Stroke::new(8.0 * z, theme::FLOW_LIGHT));
                arrow_head(painter, p, p - a, 14.0 * z, theme::FLOW);
            }
            Drag::NewLink { from } => {
                if let Some(g) = geo(geos, from) {
                    let a = rect_edge_toward(g.rect, p);
                    painter.extend(Shape::dashed_line(&[a, p], Stroke::new(2.0_f32, theme::INK), 6.0, 4.0));
                    arrow_head(painter, p, p - a, 12.0 * z, theme::INK);
                }
            }
            _ => {}
        }
    }
}

fn loop_edges(lp: &FeedbackLoop) -> Vec<(u32, u32)> {
    (0..lp.nodes.len()).map(|i| (lp.nodes[i], lp.nodes[(i + 1) % lp.nodes.len()])).collect()
}

fn draw_links(app: &App, painter: &Painter, geos: &[Geo], z: f32) {
    let highlight = app.highlight_loop.and_then(|i| app.loops.get(i));
    let edges = highlight.map(loop_edges).unwrap_or_default();
    let hl_color = highlight.map(|l| if l.reinforcing { theme::REINFORCING } else { theme::BALANCING });
    for (i, l) in app.model.links.iter().enumerate() {
        let Some((pts, mid)) = link_geometry(geos, l) else { continue };
        let selected = app.selection == Selection::Link(i);
        let in_loop = edges.contains(&(l.from, l.to));
        let color = match (in_loop, hl_color) {
            (true, Some(c)) => c,
            _ if highlight.is_some() => Color32::from_rgb(190, 192, 198),
            _ => theme::INK,
        };
        let width = if in_loop { 3.5 } else { 2.0 } * z.max(0.6);
        if selected {
            painter.add(Shape::line(pts.clone(), Stroke::new(width + 6.0, theme::SELECT)));
        }
        let n = pts.len();
        let tip = pts[n - 1];
        let dir = tip - pts[n - 3];
        let head = 13.0 * z.max(0.6);
        let mut line = pts.clone();
        line[n - 1] = tip - safe_normalized(dir, Vec2::X) * head * 0.7;
        painter.add(Shape::line(line, Stroke::new(width, color)));
        arrow_head(painter, tip, dir, head, color);

        // Polaritet (+ eller −) nära pilspetsen.
        let sign_pos = sign_position(&pts, l.curve, z);
        let (txt, col) = if l.positive { ("+", theme::POSITIVE) } else { ("−", theme::NEGATIVE) };
        painter.circle(sign_pos, 10.0 * z, Color32::WHITE, Stroke::new(1.5_f32, col));
        painter.text(sign_pos, Align2::CENTER_CENTER, txt, FontId::proportional(19.0 * z), col);

        if selected {
            painter.circle(mid, 6.0, Color32::WHITE, Stroke::new(2.0_f32, theme::ACCENT));
        }
    }
}

fn draw_loop_markers(app: &App, painter: &Painter, geos: &[Geo], z: f32) {
    let labels = loop_labels(&app.loops);
    let mut placed: Vec<Pos2> = Vec::new();
    for (i, lp) in app.loops.iter().enumerate() {
        let centers: Vec<Pos2> = lp.nodes.iter().filter_map(|id| geo(geos, *id)).map(|g| g.rect.center()).collect();
        if centers.len() < 2 {
            continue;
        }
        let center = centers.iter().fold(Pos2::ZERO, |acc, p| acc + p.to_vec2()) / centers.len() as f32;
        // Mittpunkten kan hamna på en variabel eller en annan markering: prova punkter i en spiral runt den.
        let free = |p: Pos2| {
            !placed.iter().any(|q| q.distance(p) < 46.0 * z)
                && !geos.iter().any(|g| g.rect.expand(24.0 * z).contains(p))
        };
        let c = (0..60)
            .map(|k| {
                let a = k as f32 * 2.4;
                center + vec2(a.cos(), a.sin()) * (k as f32 * 7.0 * z)
            })
            .find(|&p| free(p))
            .unwrap_or(center);
        placed.push(c);
        let color = if lp.reinforcing { theme::REINFORCING } else { theme::BALANCING };
        let strong = app.highlight_loop == Some(i);
        let r = 19.0 * z;
        painter.circle_filled(c, r + 3.0, Color32::from_white_alpha(230));
        let pts: Vec<Pos2> = (0..=24)
            .map(|k| {
                let a = (40.0 + 290.0 * k as f32 / 24.0).to_radians();
                c + vec2(a.cos(), -a.sin()) * r
            })
            .collect();
        let w = if strong { 3.5 } else { 2.2_f32 };
        painter.add(Shape::line(pts.clone(), Stroke::new(w, color)));
        let n = pts.len();
        arrow_head(painter, pts[n - 1], pts[n - 1] - pts[n - 2], 8.0 * z.max(0.7), color);
        painter.text(c, Align2::CENTER_CENTER, &labels[i], FontId::proportional(14.0 * z.max(0.7)), color);
    }
}

/// Punkten där ett flöde börjar/slutar: kanten på ett lager, eller ett moln.
fn flow_ends(n: &Node, valve: Pos2, geos: &[Geo], z: f32) -> ((Pos2, bool), (Pos2, bool)) {
    let stock = |id: Option<u32>| id.and_then(|id| geo(geos, id)).filter(|g| g.kind == NodeKind::Stock).map(|g| g.rect);
    let (fr, tr) = (stock(n.from), stock(n.to));
    let away = 95.0 * z;
    let dst = match tr {
        Some(r) => (rect_edge_toward(r, valve), false),
        None => {
            let dir = fr.map_or(Vec2::X, |r| safe_normalized(valve - r.center(), Vec2::X));
            (valve + dir * away, true)
        }
    };
    let src = match fr {
        Some(r) => (rect_edge_toward(r, valve), false),
        None => {
            let dir = safe_normalized(valve - dst.0, -Vec2::X);
            (valve + dir * away, true)
        }
    };
    (src, dst)
}

fn draw_cloud(painter: &Painter, c: Pos2, z: f32) {
    let blobs = [(vec2(-11.0, 3.0), 9.0), (vec2(0.0, -5.0), 11.0), (vec2(11.0, 3.0), 9.0), (vec2(0.0, 5.0), 8.0)];
    for (o, r) in blobs {
        painter.circle_stroke(c + o * z, r * z, Stroke::new(1.5_f32, theme::MUTED));
    }
    for (o, r) in blobs {
        painter.circle_filled(c + o * z, r * z - 1.0, Color32::WHITE);
    }
}

fn draw_pipes(app: &App, painter: &Painter, geos: &[Geo], z: f32) {
    for (g, n) in geos.iter().zip(&app.model.nodes) {
        if n.kind != NodeKind::Flow {
            continue;
        }
        let valve = g.rect.center();
        let ((src, src_cloud), (dst, dst_cloud)) = flow_ends(n, valve, geos, z);
        let dir = safe_normalized(dst - valve, Vec2::X);
        let head = 18.0 * z;
        let end = dst - dir * head * 0.8;
        let pts = vec![src, valve, end];
        painter.add(Shape::line(pts.clone(), Stroke::new(11.0 * z, theme::FLOW)));
        painter.add(Shape::line(pts, Stroke::new(6.0 * z, theme::FLOW_LIGHT)));
        arrow_head(painter, dst, dir, head, theme::FLOW);
        if src_cloud {
            draw_cloud(painter, src, z);
        }
        if dst_cloud {
            draw_cloud(painter, dst + dir * 14.0 * z, z);
        }
    }
}

fn draw_deps(app: &App, painter: &Painter, geos: &[Geo], z: f32) {
    let sel = match app.selection {
        Selection::Node(id) => Some(id),
        _ => None,
    };
    for &(from, to) in &app.compiled.deps {
        let (Some(a), Some(b)) = (geo(geos, from), geo(geos, to)) else { continue };
        let Some((pts, _)) = curve_points(a.rect, b.rect, 0.12) else { continue };
        let active = sel == Some(from) || sel == Some(to);
        let color = if active { theme::ACCENT } else { theme::DEP };
        let width = if active { 2.2 } else { 1.3_f32 };
        let n = pts.len();
        let dir = pts[n - 1] - pts[n - 3];
        painter.add(Shape::line(pts.clone(), Stroke::new(width, color)));
        arrow_head(painter, pts[n - 1], dir, 9.0 * z.max(0.7), color);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_node(
    painter: &Painter,
    g: &Geo,
    n: &Node,
    selected: bool,
    hovered: bool,
    error: bool,
    value: Option<f64>,
    level: Option<f32>,
    z: f32,
) {
    let r = g.rect;
    let c = r.center();
    let small = FontId::proportional(13.0 * z);
    let text_top = |h: f32| pos2(c.x, h);
    match n.kind {
        NodeKind::Stock => {
            painter.rect_filled(r, cr(8.0 * z), theme::STOCK_FILL);
            if let Some(f) = level {
                let h = r.height() * f.clamp(0.0, 1.0);
                let clip = Rect::from_min_max(pos2(r.left(), r.bottom() - h), r.right_bottom());
                painter.with_clip_rect(clip).rect_filled(r, cr(8.0 * z), theme::STOCK_LEVEL);
            }
            painter.rect_stroke(r, cr(8.0 * z), Stroke::new(2.2 * z.max(0.6), theme::STOCK), StrokeKind::Inside);
            painter.galley(text_top(r.top() + 10.0 * z), g.galley.clone(), theme::INK);
            if let Some(v) = value {
                painter.text(
                    pos2(c.x, r.bottom() - 9.0 * z),
                    Align2::CENTER_BOTTOM,
                    theme::fmt_with_unit(v, &n.unit),
                    FontId::proportional(14.0 * z),
                    theme::STOCK,
                );
            }
        }
        NodeKind::Flow => {
            painter.circle(c, 11.0 * z, Color32::WHITE, Stroke::new(2.0 * z.max(0.6), theme::FLOW));
            let k = 6.5 * z;
            painter.add(Shape::convex_polygon(
                vec![c + vec2(-k, -k), c + vec2(k, -k), c],
                theme::FLOW,
                Stroke::NONE,
            ));
            painter.add(Shape::convex_polygon(vec![c, c + vec2(k, k), c + vec2(-k, k)], theme::FLOW, Stroke::NONE));
            let label_top = r.bottom() + 2.0 * z;
            painter.galley(text_top(label_top), g.galley.clone(), theme::INK);
            if let Some(v) = value {
                painter.text(
                    pos2(c.x, label_top + g.galley.size().y + 1.0),
                    Align2::CENTER_TOP,
                    theme::fmt_with_unit(v, &n.unit),
                    small,
                    theme::FLOW,
                );
            }
        }
        NodeKind::Aux | NodeKind::Constant => {
            let (fill, stroke, radius) = if n.kind == NodeKind::Aux {
                (theme::AUX_FILL, theme::AUX, r.height() / 2.0)
            } else {
                (theme::CONST_FILL, theme::CONST, 6.0 * z)
            };
            painter.rect(r, cr(radius), fill, Stroke::new(1.6 * z.max(0.6), stroke), StrokeKind::Inside);
            painter.galley(text_top(c.y - g.galley.size().y / 2.0), g.galley.clone(), theme::INK);
            if let Some(v) = value {
                painter.text(
                    pos2(c.x, r.bottom() + 2.0 * z),
                    Align2::CENTER_TOP,
                    theme::fmt_with_unit(v, &n.unit),
                    small,
                    stroke,
                );
            }
        }
        NodeKind::Variable => {
            let border = if hovered { theme::MUTED } else { Color32::from_rgb(214, 212, 204) };
            painter.rect(r, cr(8.0 * z), theme::VAR_FILL, Stroke::new(1.0_f32, border), StrokeKind::Inside);
            painter.galley(text_top(c.y - g.galley.size().y / 2.0), g.galley.clone(), theme::INK);
        }
    }
    if hovered && !selected && n.kind != NodeKind::Variable {
        painter.rect_stroke(g.hit.expand(3.0), cr(8.0 * z), Stroke::new(1.0_f32, theme::MUTED), StrokeKind::Outside);
    }
    if selected {
        painter.rect_stroke(g.hit.expand(4.0), cr(10.0 * z), Stroke::new(2.5_f32, theme::SELECT), StrokeKind::Outside);
    }
    if error {
        painter.rect_stroke(g.rect.expand(2.0), cr(8.0 * z), Stroke::new(2.0_f32, theme::ERROR), StrokeKind::Outside);
        let b = g.rect.right_top() + vec2(0.0, 0.0);
        painter.circle_filled(b, 9.0, theme::ERROR);
        painter.text(b, Align2::CENTER_CENTER, "!", FontId::proportional(14.0), Color32::WHITE);
    }
}

fn draw_hints(app: &App, painter: &Painter, rect: Rect) {
    let is_cld = app.model.kind == ModelKind::Cld;
    if app.model.nodes.is_empty() {
        let msg = if is_cld {
            "Dubbelklicka på ytan för att lägga till en variabel.\nRita sedan pilar med verktyget Pil till vänster."
        } else {
            "Börja med att välja Lager i verktygslådan till vänster\noch klicka här på ytan."
        };
        painter.text(rect.center(), Align2::CENTER_CENTER, msg, FontId::proportional(20.0), theme::MUTED);
    }
    let hint = match app.tool {
        // Markera-verktyget behöver ingen ruta som skymmer modellen, utom i en tom modell.
        Tool::Select if !app.model.nodes.is_empty() => return,
        Tool::Select if is_cld => {
            "Dra för att flytta. Dubbelklicka på tom yta för ny variabel, på en pil för att byta + / −."
        }
        Tool::Select => "Dra för att flytta saker eller hela ytan. Ctrl + scrollhjul zoomar.",
        Tool::Place(NodeKind::Stock) => "Klicka på ytan där lagret ska ligga. Ett lager är något som samlas, t.ex. vatten, energi eller värme.",
        Tool::Place(NodeKind::Constant) => "Klicka på ytan där konstanten ska ligga. Konstanter kan du ändra med reglage.",
        Tool::Place(NodeKind::Aux) => "Klicka på ytan där hjälpvariabeln ska ligga. Den räknar ut något med en formel.",
        Tool::Place(_) => "Klicka på ytan för att placera.",
        Tool::Flow => "Dra från ett lager till ett annat. Börjar eller slutar du på tom yta kommer flödet från/till ett moln.",
        Tool::Link => "Dra från en variabel till en annan för att rita en pil. Klicka på en pil för att byta mellan + och −.",
    };
    let galley = painter.layout(hint.to_string(), FontId::proportional(14.0), theme::INK, rect.width() - 40.0);
    let pos = pos2(rect.left() + 14.0, rect.bottom() - 14.0 - galley.size().y);
    let bg = Rect::from_min_size(pos, galley.size()).expand2(vec2(10.0, 6.0));
    painter.rect(bg, cr(8.0), Color32::from_white_alpha(235), Stroke::new(1.0_f32, theme::GRID), StrokeKind::Inside);
    painter.galley(pos, galley, theme::INK);
}
