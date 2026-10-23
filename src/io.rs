//! Spara och öppna modellfiler (.mvm), både som program och i webbläsaren.

use crate::model::Model;
use std::sync::{Arc, Mutex};

const EXTENSIONS: &[&str] = &["mvm", "json"];
const FILTER_NAME: &str = "Modellverkstan-modell";

type Pending = Arc<Mutex<Option<Result<Model, String>>>>;

#[derive(Default)]
pub struct FileIo {
    pending: Pending,
}

fn parse(text: &str) -> Result<Model, String> {
    serde_json::from_str(text).map_err(|e| format!("Filen verkar inte vara en modell från Modellverkstan ({e})."))
}

/// Ett filnamn byggt på modellens titel, t.ex. "Ladda en elbil.mvm".
pub fn file_name(model: &Model, extension: &str) -> String {
    let base: String = model
        .title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
        .collect();
    let base = base.trim();
    format!("{}.{extension}", if base.is_empty() { "modell" } else { base })
}

impl FileIo {
    /// Hämtar en fil som har öppnats i bakgrunden.
    pub fn poll(&mut self) -> Option<Result<Model, String>> {
        self.pending.lock().ok()?.take()
    }

    /// Sparar modellen. `Ok(false)` betyder att användaren avbröt.
    pub fn save(&mut self, model: &Model) -> Result<bool, String> {
        let json = serde_json::to_string_pretty(model).map_err(|e| e.to_string())?;
        save_bytes(&file_name(model, "mvm"), json.as_bytes(), &MODEL_FILE)
    }

    pub fn open(&mut self, ctx: &egui::Context) {
        let pending = self.pending.clone();
        let ctx = ctx.clone();
        let task = async move {
            let Some(file) = rfd::AsyncFileDialog::new().add_filter(FILTER_NAME, EXTENSIONS).pick_file().await else {
                return;
            };
            let bytes = file.read().await;
            let result = String::from_utf8(bytes).map_err(|_| "Filen kunde inte läsas.".to_string()).and_then(|t| parse(&t));
            if let Ok(mut slot) = pending.lock() {
                *slot = Some(result);
            }
            ctx.request_repaint();
        };
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(task);
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || pollster_block_on(task));
    }
}

/// Vilken sorts fil som sparas: namn i filväljaren, filändelser och MIME-typ.
// Programmet använder namn och ändelser, webben bara MIME-typen.
#[allow(dead_code)]
pub struct FileKind {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    /// Används bara i webbläsaren, där filen laddas ner.
    pub mime: &'static str,
}

const MODEL_FILE: FileKind = FileKind { name: FILTER_NAME, extensions: EXTENSIONS, mime: "application/json" };
pub const PNG_FILE: FileKind = FileKind { name: "PNG-bild", extensions: &["png"], mime: "image/png" };

/// Sparar en fil: filväljare i programmet, nedladdning i webbläsaren.
/// `Ok(false)` betyder att användaren avbröt.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_bytes(name: &str, bytes: &[u8], kind: &FileKind) -> Result<bool, String> {
    let Some(path) = rfd::FileDialog::new().add_filter(kind.name, kind.extensions).set_file_name(name).save_file() else {
        return Ok(false);
    };
    std::fs::write(&path, bytes).map_err(|e| format!("Kunde inte spara filen: {e}"))?;
    Ok(true)
}

#[cfg(target_arch = "wasm32")]
pub fn save_bytes(name: &str, bytes: &[u8], kind: &FileKind) -> Result<bool, String> {
    use eframe::wasm_bindgen::JsCast;
    let err = |_| "Kunde inte spara filen i webbläsaren.".to_string();
    let window = web_sys::window().ok_or("Inget webbläsarfönster")?;
    let document = window.document().ok_or("Inget dokument")?;
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(kind.mime);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts).map_err(err)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(err)?;
    let a: web_sys::HtmlAnchorElement = document.create_element("a").map_err(err)?.dyn_into().map_err(|_| "Fel".to_string())?;
    a.set_href(&url);
    a.set_download(name);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&url);
    Ok(true)
}

/// Kör en framtid klart på den här tråden (för filväljaren i programversionen).
#[cfg(not(target_arch = "wasm32"))]
fn pollster_block_on<F: std::future::Future>(fut: F) -> F::Output {
    use std::task::{Context, Poll, Wake, Waker};

    struct ThreadWaker(std::thread::Thread);
    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWaker(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::park(),
        }
    }
}
