use std::{fmt::Display, fs::{self, create_dir_all, OpenOptions}, io::{BufReader, BufWriter, Write}, path::Path, sync::Arc};

use directories_next::ProjectDirs;
use eframe::{egui::{menu, Align, Button, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, Label, Layout, Margin, RichText, Sense, Separator, Stroke, Theme, TopBottomPanel, Ui, ViewportCommand}, CreationContext, Frame};
use egui_extras::TableBuilder;
use enum_iterator::{all, cardinality, Sequence};
use log::info;
use serde::{de::DeserializeOwned, Serialize};

use crate::{app_data::AppData, app_settings::AppSettings, child_windows::ChildWindows, localize::fl, todo::TodoUndo};




const ZOOM: f32 = 1.0;
pub const UI_PADDING: f32 = 8.0;


// todo: localize this
// probably could be one phrase?
pub const HELP_TEXT: &[&str] = &[
    "created by Liam Routt",
    "for use with Blades in the Dark",
    "",
    "This utility allows you to track the various factions in your game.", "",
];

// ? Can these be localized?
pub const CHANGE_NOTES: &[&str] = &[
    "0.1.0 - initial version",
];

// todo: localize this
pub const FONT_NOTES: &[&str] = &[
    "Using the Manrope font, by Michael Sharanda, covered under the SIL Open Font License (http://scripts.sil.org/OFL)"
];



#[allow(dead_code)]
pub struct  App {
    status: AppStatus,
    main_view: MainView,
    settings: AppSettings,
    project_directories: ProjectDirs,
    data: AppData,
    message: Option<String>,
    child_windows: ChildWindows,
    todo_undo: TodoUndo,
}

impl App {
    pub fn new ( settings: AppSettings, project_directories: ProjectDirs, cc: &CreationContext<'_> ) -> Self {
        configure_fonts(cc, ZOOM);
        App {
            settings,
            project_directories,
            status: AppStatus::default(),
            main_view: MainView::default(),
            data: AppData::default(),
            message: None,
            child_windows: ChildWindows::default(),
            todo_undo: TodoUndo::default(),
        }
    }

    fn show_top ( &mut self, ctx: &Context, _frame: &mut Frame ) {
        TopBottomPanel::top("top").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.menu_button(fl!("menu"), |ui| {
                        if ui.button(fl!("menu_restart")).clicked() {
                            self.status = AppStatus::Starting;
                            self.message = None;
                            info!("Requested Restart");
                            ui.close_menu();
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui.add_enabled(false, Button::new(fl!("menu_settings"))).clicked() {
                            // TODO: show/change settings
                            // should that be window or full panel?
                            info!("Requested Settings");
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui.button(fl!("menu_exit")).clicked() {
                            info!("Requested Exit");
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }

                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(fl!("about")).clicked() {
                            self.child_windows.toggle_about();
                            info!("Requested About");
                        }
                    });
                });
            });
        });
    }

    fn show_footer ( &self, ctx: &Context ) {
        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.);
            ui.horizontal(|ui| {
                ui.label(self.status.to_string());
                if let Some(message) = &self.message {
                    let error_message = RichText::new(fl!("app_error_err", err = message))
                        .background_color(Color32::RED)
                        .color(Color32::WHITE);
                    ui.label(error_message);
                    // do we need an "OK" button to clear the message?
                }
            });
        });
    }
}

// ---------------------------
#[allow(clippy::match_single_binding)]
#[allow(unused_imports)]
impl eframe::App for App {
    fn update ( &mut self, ctx: &Context, frame: &mut Frame ) {
        use AppStatus::*;

        ctx.set_visuals(self.settings.theme().default_visuals());

        self.show_top(ctx, frame);
        self.show_footer(ctx);

        if let Some(new_status) = CentralPanel::default().show(ctx,  |ui: &mut Ui| {
            match self.status {
                Starting => {
                    // if not already starting, kick things off
                    // info!("Starting")
                    // if completed
                    info!("Starting => Ready");
                    Some(Ready)
                    // otherwise, keep doing start
                    // None
                }

                Ready => {
                    // what are we looking at?
                    // select between views
                    if let Some(new_view) = self.show_select_views(ui) {
                        self.main_view = new_view;
                        // anything else to do?
                    }

                    // find or build display data table
                    let _display_table = match &self.main_view {
                        MainView::Districts => {
                            self.data.districts_display_table()
                        }

                        MainView::Persons => {
                            self.data.persons_display_table()
                        }

                        MainView::Factions => {
                            self.data.factions_display_table()
                        }
                    };

                    // show table with display data
                    const STROKE_WIDTH: f32 = 1.;
                    // todo: colors will need to change with theme?
                    const STROKE_COLOR: Color32 = Color32::GRAY;
                    const INNER_MARGIN: Margin = Margin::same(6);
                    const OUTER_MARGIN: Margin = Margin::same(1);

                    ui.horizontal_top(|ui| {
                        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                        eframe::egui::Frame::default()
                            .stroke(Stroke::new(STROKE_WIDTH, STROKE_COLOR))
                            .inner_margin(INNER_MARGIN)
                            .outer_margin(OUTER_MARGIN)
                            .show(ui, |ui| {
                                ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                                let mut table = TableBuilder::new(ui)
                                    .striped(true)
                                    .sense(Sense::click())
                                    .auto_shrink([false, false]);
                                // now show display_table headers and data
                            });
                        });
                    });
                    // ?
                    None
                }
                // _ => { None }
            }
        }).inner {
            self.status = new_status;
        }

        self.child_windows.show_windows(ctx);
    }
}

impl App {

    fn show_select_views ( &self, ui: &mut Ui ) -> Option<MainView> {
        let select_color = if self.settings.theme() == Theme::Dark { Color32::LIGHT_GREEN } else { Color32::DARK_GREEN };
        let mut new_view = None;
        ui.horizontal(|ui| {
            for (num, view) in all::<MainView>().enumerate() {
                let mut v_text = RichText::new(view.to_string()).heading();
                if self.main_view == view {
                    v_text = v_text.color(select_color).underline();
                }

                if ui.add(Label::new(v_text).sense(Sense::click())).clicked() {
                    info!("selected {}", view);
                    new_view = Some(view);
                }

                if num != cardinality::<MainView>() {
                    ui.heading(" | ");
                }
            }

        });
        new_view
    }
}

// ===========================
// AppStatus

#[allow(dead_code)]
#[derive(Debug, Default)]
enum AppStatus {
    #[default]
    Starting,
    Ready,
}

impl Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppStatus::*;

        write!(f, "{}", match self {
            Starting => fl!("app_starting"),
            Ready => fl!("app_ready"),
        })
    }
}

// ===========================
// MainView

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, Sequence, PartialEq)]
enum MainView {
    #[default]
    Factions,
    Persons,
    Districts,
}

impl Display for MainView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MainView::*;

        write!(f, "{}", match self {
            Factions => fl!("main_factions"),
            Persons => fl!("main_persons"),
            Districts => fl!("main_districts"),
        })
    }
}

// ===========================
// Additional functions

fn configure_fonts ( ctx: &CreationContext, zoom: f32 ) {
	let mut fonts = FontDefinitions::default();
	fonts.font_data.insert("manrope".to_string(), Arc::new(FontData::from_static(include_bytes!("../manrope_regular.otf"))));
	fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "manrope".to_owned());
	ctx.egui_ctx.set_fonts(fonts);
    ctx.egui_ctx.set_zoom_factor(zoom);
}


// ---------------------------
// Load and Save to POT files

#[allow(dead_code)]
pub fn save_to_pot<T> ( file_path: &Path, data: &T ) -> anyhow::Result<()>
    where T: Serialize {

    if let Some(dir_path) = file_path.parent() {
        create_dir_all(dir_path)?
    }

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }

    let file_handler = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)?;

    let buf= pot::to_vec(data)?;
    let mut buf_writer = BufWriter::new(&file_handler);
    buf_writer.write_all(buf.as_slice())?;
    Ok(())
}

#[allow(dead_code)]
pub fn load_from_pot<T> ( file_path: &Path ) -> anyhow::Result<T>
    where T: DeserializeOwned {
        let file_handler = OpenOptions::new()
        .read(true)
        .open(file_path)?;
    let buf_reader = BufReader::new(&file_handler);
    let data : T = pot::from_reader(buf_reader)?;
    Ok(data)
}
