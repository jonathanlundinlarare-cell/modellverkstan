#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod examples;
mod export;
mod expr;
mod help;
mod illustrations;
mod io;
mod learn;
mod lessons;
mod model;
mod panels;
mod sim;
mod start;
mod theme;
mod ui_kit;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Modellverkstan")
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([820.0, 560.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png")).unwrap_or_default(),
            ),
        ..Default::default()
    };
    eframe::run_native("Modellverkstan", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    eframe::WebLogger::init(log::LevelFilter::Warn).ok();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("inget fönster").document().expect("inget dokument");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("hittar inte canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id är inte en canvas");
        let result = eframe::WebRunner::new()
            .start(canvas, eframe::WebOptions::default(), Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
            .await;
        if let Some(loading) = document.get_element_by_id("loading") {
            match result {
                Ok(_) => loading.remove(),
                Err(e) => loading.set_inner_html(&format!("<p>Modellverkstan kunde inte starta: {e:?}</p>")),
            }
        }
    });
}
