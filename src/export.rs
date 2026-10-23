//! Spara eller kopiera grafen som bild.
//!
//! Appen ber om en skärmbild av hela fönstret och klipper sedan ut grafen.
//! Bilden kommer tillbaka som en händelse en bildruta senare, därför sker
//! exporten i två steg: `request` och sedan `poll` i varje bildruta.

use crate::app::App;
use crate::io;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExportAction {
    Save,
    Copy,
}

pub fn request(app: &mut App, ctx: &egui::Context, action: ExportAction) {
    if app.plot_rect.is_negative() || app.plot_rect.area() < 1.0 {
        return;
    }
    app.export = Some((action, ctx.input(|i| i.time)));
    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
}

pub fn poll(app: &mut App, ctx: &egui::Context) {
    let Some((action, asked_at)) = app.export else { return };
    let shot = ctx.input(|i| {
        i.raw.events.iter().find_map(|e| match e {
            egui::Event::Screenshot { image, .. } => Some(image.clone()),
            _ => None,
        })
    });
    let Some(screen) = shot else {
        if ctx.input(|i| i.time) - asked_at > 3.0 {
            app.export = None;
            app.notify(ctx, "Det gick inte att ta en bild av grafen.");
        } else {
            ctx.request_repaint();
        }
        return;
    };
    app.export = None;

    let ppp = ctx.pixels_per_point();
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(screen.width() as f32, screen.height() as f32) / ppp);
    let region = app.plot_rect.intersect(bounds);
    if region.is_negative() || region.area() < 1.0 {
        app.notify(ctx, "Grafen syns inte – det gick inte att ta en bild.");
        return;
    }
    let image = screen.region(&region, Some(ppp));
    match action {
        ExportAction::Copy => {
            ctx.copy_image(image);
            app.notify(ctx, "Grafen är kopierad – klistra in den i t.ex. Word eller Google Docs.");
        }
        ExportAction::Save => {
            let name = io::file_name(&app.model, "png");
            match png_bytes(&image).and_then(|bytes| io::save_bytes(&name, &bytes, &io::PNG_FILE)) {
                Ok(true) => app.notify(ctx, "Grafen sparades som bild."),
                Ok(false) => {}
                Err(e) => app.notify(ctx, e),
            }
        }
    }
}

fn png_bytes(image: &egui::ColorImage) -> Result<Vec<u8>, String> {
    let raw: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
    let [w, h] = image.size;
    let rgba = image::RgbaImage::from_raw(w as u32, h as u32, raw).ok_or("Bilden blev fel.")?;
    let mut out = std::io::Cursor::new(Vec::new());
    rgba.write_to(&mut out, image::ImageFormat::Png).map_err(|e| format!("Kunde inte skapa bilden: {e}"))?;
    Ok(out.into_inner())
}
