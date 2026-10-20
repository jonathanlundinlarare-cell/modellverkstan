//! Ritar appens ikon: ett lager som fylls av ett flöde.
//!
//! Kör med `cargo run --example make_icon` för att skapa om
//! `assets/icon-256.png` och `assets/icon.ico`.

use image::codecs::ico::{IcoEncoder, IcoFrame};
use image::{ExtendedColorType, Rgba, RgbaImage};

type Rgb = [f32; 3];

const BLUE_TOP: Rgb = [52.0, 120.0, 200.0];
const BLUE_BOTTOM: Rgb = [30.0, 84.0, 155.0];
const WHITE: Rgb = [255.0, 255.0, 255.0];
const LEVEL: Rgb = [150.0, 196.0, 242.0];
const GREEN: Rgb = [46.0, 180.0, 106.0];

fn in_rrect(x: f32, y: f32, (x0, y0, x1, y1): (f32, f32, f32, f32), r: f32) -> bool {
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return false;
    }
    let dx = (x0 + r - x).max(x - (x1 - r)).max(0.0);
    let dy = (y0 + r - y).max(y - (y1 - r)).max(0.0);
    dx * dx + dy * dy <= r * r
}

fn in_triangle(x: f32, y: f32, a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let side = |p: (f32, f32), q: (f32, f32)| (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0);
    let (d1, d2, d3) = (side(a, b), side(b, c), side(c, a));
    !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
}

/// Färgen i punkten (x, y), där båda går från 0 till 1. `None` = genomskinligt.
fn paint(x: f32, y: f32) -> Option<Rgb> {
    if !in_rrect(x, y, (0.03, 0.03, 0.97, 0.97), 0.22) {
        return None;
    }
    let mut c: Rgb = std::array::from_fn(|i| BLUE_TOP[i] + (BLUE_BOTTOM[i] - BLUE_TOP[i]) * y);

    // Lagret: en vit låda som är halvfull.
    let stock = (0.47, 0.27, 0.88, 0.75);
    if in_rrect(x, y, stock, 0.07) {
        c = if y > 0.52 && in_rrect(x, y, (0.51, 0.31, 0.84, 0.71), 0.04) { LEVEL } else { WHITE };
    }

    // Flödet: ett grönt rör med pilspets in i lagret och en ventil.
    let (cy, half) = (0.51, 0.05);
    if (0.09..=0.38).contains(&x) && (y - cy).abs() <= half {
        c = GREEN;
    }
    if in_triangle(x, y, (0.35, cy - 0.12), (0.35, cy + 0.12), (0.47, cy)) {
        c = GREEN;
    }
    let d = ((x - 0.22).powi(2) + (y - cy).powi(2)).sqrt();
    if d <= 0.085 {
        c = if d <= 0.05 { GREEN } else { WHITE };
    }
    Some(c)
}

fn render(size: u32) -> RgbaImage {
    const SS: u32 = 4; // 4×4 punkter per pixel ger mjuka kanter
    RgbaImage::from_fn(size, size, |px, py| {
        let (mut sum, mut hits) = ([0.0f32; 3], 0u32);
        for sy in 0..SS {
            for sx in 0..SS {
                let x = (px as f32 + (sx as f32 + 0.5) / SS as f32) / size as f32;
                let y = (py as f32 + (sy as f32 + 0.5) / SS as f32) / size as f32;
                if let Some(c) = paint(x, y) {
                    (0..3).for_each(|i| sum[i] += c[i]);
                    hits += 1;
                }
            }
        }
        if hits == 0 {
            return Rgba([0, 0, 0, 0]);
        }
        let avg = |i: usize| (sum[i] / hits as f32).round() as u8;
        Rgba([avg(0), avg(1), avg(2), (255 * hits / (SS * SS)) as u8])
    })
}

fn main() {
    std::fs::create_dir_all("assets").unwrap();
    render(256).save("assets/icon-256.png").unwrap();

    let frames: Vec<IcoFrame> = [16, 24, 32, 48, 64, 128, 256]
        .iter()
        .map(|&s| {
            let img = render(s);
            IcoFrame::as_png(img.as_raw(), s, s, ExtendedColorType::Rgba8).unwrap()
        })
        .collect();
    let file = std::fs::File::create("assets/icon.ico").unwrap();
    IcoEncoder::new(file).encode_images(&frames).unwrap();
    println!("Skapade assets/icon-256.png och assets/icon.ico");
}
