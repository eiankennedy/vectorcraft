//! VectorCraft hosted inside Workbench's side panel: the same app as the desktop build, drawn into a view
//! Workbench owns, with Workbench's look and only the canvas showing (Hands off) until the user asks for the
//! menus. Workbench (and Claude) drive it through its control channel. See `craft-embed`.

use craft_embed::{EmbedApp, egui};
use serde_json::{Value, json};
use vectorcraft_engine::Session;
use vectorcraft_ui_egui::control::{self, ControlRequest, Outcome};
use vectorcraft_ui_egui::{Services, VectorcraftApp};

struct Vector(VectorcraftApp);

impl EmbedApp for Vector {
    // As the desktop wrapper (`apps/vectorcraft/src/app.rs`): `tick` then the IME's logic, the app's input
    // hook, and its `ui`.
    fn logic(&mut self, ctx: &egui::Context) {
        self.0.tick(ctx);
        self.0.finish_ime_logic(ctx);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(ctx, raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.0.ui(ui);
    }
    fn control(&mut self, ctx: &egui::Context, request: &str) -> String {
        let req: Value = serde_json::from_str(request).unwrap_or(Value::Null);
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(json!({}));
        let (req, _reply) = ControlRequest::new(method, params);
        match control::handle(&mut self.0, ctx, &req) {
            Outcome::Done(v) => v.to_string(),
            Outcome::Screenshot { .. } => json!({"ok": false, "error": "Workbench takes the screenshot of the panel itself"}).to_string(),
            // the pointer events are queued; they run over the next frames
            Outcome::AfterToolEdit { .. } => json!({"ok": true, "result": {"pending": true}}).to_string(),
        }
    }
}

fn make(_ctx: &egui::Context) -> Box<dyn EmbedApp> {
    let services = Services {
        read: Some(Box::new(|p: &str| std::fs::read(p).map_err(|e| e.to_string()))),
        write: Some(Box::new(|p: &str, b: &[u8]| std::fs::write(p, b).map_err(|e| e.to_string()))),
        ..Default::default()
    };
    let mut app = VectorcraftApp::new(Session::new(), services);
    // Workbench's look and Hands off from the first frame; Workbench sets the mode's accent right after
    let _ = app.run("view.workbench", json!({"on": true}));
    let _ = app.run("view.handsOff", json!({"on": true}));
    Box::new(Vector(app))
}

craft_embed::export_embed!(make);
