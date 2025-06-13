use std::{env::current_exe, fs};

use app_settings::AppSettings;
use directories_next::ProjectDirs;
use eframe::{egui::{Vec2, ViewportBuilder}, run_native, NativeOptions};
use log::{error, info, warn, LevelFilter};

use app::App;

mod app;
mod app_data;
mod localize;
mod app_settings;
mod child_windows;
mod faction;
mod person;
mod district;
mod clock;
mod tier;
mod managed_list;
mod action;
mod todo;
mod app_display;

const APP_NAME: &str = "Blades Factions";
const COMPANY_DOMAIN: &str = "org";
const COMPANY_NAME: &str = "Darcsyde";


fn main() {
    setup_logger().expect("log did not start");
    info!("Starting");

    let Ok(exe_path) = current_exe()
    else { panic!("Unable to find exe path"); };

    let Some(exe_name) = exe_path.file_stem()
    else { panic!("Unable to find exe name in {}", exe_path.display()); };

    let base_name = exe_name.to_string_lossy();

    let Some(base_dir) = ProjectDirs::from(COMPANY_DOMAIN, COMPANY_NAME, &base_name)
    else { panic!("Unable to find project directory for {}, {}, {}", COMPANY_DOMAIN, COMPANY_NAME, base_name); };

    let settings = match AppSettings::load_from_file(base_dir.config_dir()) {
        Ok(settings) => settings,
        Err(err) => {
            warn!("Unable to load settings from {}: {}", base_dir.config_dir().display(), err);
            let settings = AppSettings::default();
            if let Err(err) = settings.save_to_file(base_dir.config_dir()) {
                error!("Unable to save settings to {}: {}", base_dir.config_dir().display(), err);
            }
            settings
        }
    };

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

    let _res = run_native(&app_name,
        win_option,
        Box::new(|cc| Ok(Box::new(App::new(settings, base_dir, cc))))
    );
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
