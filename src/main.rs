use std::fs;

use app_settings::AppSettings;
use eframe::{egui::{Vec2, ViewportBuilder}, run_native, NativeOptions};
use log::{info, LevelFilter};

use app::App;

mod app;
mod app_data;
mod app_settings;
mod child_windows;

const APP_NAME: &str = "Blades Factions";

fn main() {
    setup_logger().expect("log did not start");
    info!("Starting");

    // TODO: load settings
    let settings = AppSettings::default();

    let app_name = format!("{} (version {})", APP_NAME, env!("CARGO_PKG_VERSION"));
    let initial_window_size = Vec2::new(1200., 720.);

    let win_option = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(app_name.clone())
            .with_resizable(false)
            // .with_icon(None)
            .with_active(true)
            .with_inner_size(initial_window_size)
            .with_min_inner_size(initial_window_size)
            .with_maximize_button(false)
            .with_drag_and_drop(false),
        ..Default::default()
    };

    // todo: pass settings to App
    let _res = run_native(&app_name, win_option, Box::new(|cc| Ok(Box::new(App::new(settings, cc)))));
}


// ========================

fn setup_logger ( ) -> Result<(), fern::InitError> {
    const LOG_FILE: &str = "factions_output.log";
    let _ = fs::remove_file(LOG_FILE);  // !! ignoring possible real errors
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{:.5}][{}]: {}",
                // "[{}][{}] {}",
                    // "[{}]:[{}][{}] {}",
                    // humantime::format_rfc3339_seconds(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(LevelFilter::Error)
        .level_for(module_path!(), LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file(LOG_FILE)?)
        .apply()?;
    Ok(())
}
