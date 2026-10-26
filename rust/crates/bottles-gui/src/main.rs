mod catalog;
mod state;
mod theme;
mod ui;

use bottles_core::{APP_ID, APP_NAME};
use eframe::egui;
use state::AppState;
use ui::BottlesApp;

fn main() -> eframe::Result {
    env_logger::init();
    let state = match AppState::load() {
        Ok(state) => state,
        Err(error) => {
            eprintln!("Could not open the Bottles data directory: {error}");
            return Err(eframe::Error::AppCreation(Box::new(error)));
        }
    };
    let width = state.context.settings.window_width as f32;
    let height = state.context.settings.window_height as f32;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_inner_size([width.max(960.0), height.max(640.0)])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |_cc| Ok(Box::<BottlesApp>::new(BottlesApp::new(state)))),
    )
}
