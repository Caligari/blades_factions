use std::{fs::create_dir_all, path::PathBuf, sync::Arc};

use eframe::egui::{
    Align, CentralPanel, Context, Layout, RichText, ScrollArea, ViewportBuilder, ViewportId,
    mutex::RwLock,
};
use egui_file_dialog::{DialogState, FileDialog};
use log::{debug, error, info};

use crate::{
    APP_NAME,
    app::{CHANGE_NOTES, FONT_NOTES, HELP_TEXT, UI_PADDING},
    localize::fl,
};

// TODO: add settings panel?

#[derive(Default)]
pub struct ChildWindows {
    show_about: Arc<RwLock<bool>>,
    show_file: Arc<RwLock<bool>>,
    file_dialog: FileDialog,
    selected_file: Arc<RwLock<Option<PathBuf>>>,
}

impl ChildWindows {
    pub fn toggle_about(&mut self) {
        let current_value = *self.show_about.read();
        *self.show_about.write() = !current_value;
    }

    /// show the file dialog
    pub fn start_file_dialog(
        &mut self,
        dialog_type: FileDialogType,
        target_type: FileTarget,
        initial_directory: PathBuf,
    ) {
        // start dialog
        // todo: changes for type of file to select?
        if let Err(e) = create_dir_all(initial_directory.clone()) {
            // can fail; do we need Result?
            error!(
                "unable to create initial directory for file save [{}]: {}",
                initial_directory.to_string_lossy(),
                e
            );
        }

        let config = self.file_dialog.config_mut();
        config.initial_directory = initial_directory;

        info!("starting file dialog");

        match dialog_type {
            FileDialogType::Load => self.file_dialog.pick_file(),
            FileDialogType::Save => self.file_dialog.save_file(),
        }

        *self.show_file.write() = true;
        *self.selected_file.write() = None;
    }

    pub fn selected_file(&self) -> Option<PathBuf> {
        self.selected_file.read().clone()
    }

    pub fn show_windows(&mut self, ctx: &Context) {
        // Note: this needs to update its own show/hide, if appropriate

        let about_value = *self.show_about.read();
        if about_value {
            self.about(ctx);
        }

        let file_value = *self.show_file.read();
        if file_value {
            self.file_select(ctx);
        }
    }

    fn file_select(&mut self, ctx: &Context) {
        self.file_dialog.update(ctx);

        if let Some(path) = self.file_dialog.take_picked() {
            *self.selected_file.write() = Some(path);
            *self.show_file.write() = false;
            info!("selected file, closing file dialog");
        } else {
            use DialogState::*;
            if self.file_dialog.state() == Cancelled {
                *self.selected_file.write() = Some(PathBuf::new());
                *self.show_file.write() = false;
            }
        }
    }

    fn about(&self, ctx: &Context) {
        let window_title = fl!("about");
        let title = format!("{APP_NAME} - {window_title}"); // can we make this a const?
        let v_id = ViewportId::from_hash_of(&title);
        let show_about = self.show_about.clone();
        ctx.show_viewport_deferred(
            v_id,
            ViewportBuilder::default().with_title(&title),
            move |ctx, _class| {
                CentralPanel::default().show(ctx, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        const HELP_WIDTH: f32 = 0.75;
                        const SCROLL_HEIGHT: f32 = 160.0;
                        let all_width = ui.available_width();
                        ui.set_max_width(all_width * HELP_WIDTH);

                        ui.add_space(UI_PADDING * 3.5);
                        let app_name = RichText::new(APP_NAME).heading();
                        ui.label(app_name);
                        ui.label(fl!("version_ver", ver = env!("CARGO_PKG_VERSION")));
                        ui.add_space(UI_PADDING);
                        ui.label(HELP_TEXT.join("\n"));
                        ui.add_space(UI_PADDING * 1.15);
                        ui.label(RichText::new(fl!("changenotes")).underline());
                        ScrollArea::vertical()
                            .max_height(SCROLL_HEIGHT)
                            .show(ui, |ui| {
                                ui.label(CHANGE_NOTES.join("\n"));
                            });
                        // ui.label(CHANGE_NOTES.join("\n"));
                        ui.add_space(UI_PADDING * 1.15);
                        ui.separator();
                        ui.label(FONT_NOTES.join("\n"));
                        ui.add_space(UI_PADDING * 2.);
                    })
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    debug!("{} requested close", &title);
                    let mut show_me = show_about.write();
                    *show_me = false;
                }
            },
        );
    }
}

// ---------------------
// FileDialogType

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogType {
    Load,
    Save,
}

// ---------------------
// FileDialogType

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTarget {
    Internal,
    Export,
}
