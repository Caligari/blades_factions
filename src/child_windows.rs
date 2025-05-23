use std::sync::Arc;

use eframe::egui::{mutex::RwLock, Align, CentralPanel, Context, Layout, RichText, ScrollArea, ViewportBuilder, ViewportId};
use log::debug;

use crate::{app::{CHANGE_NOTES, FONT_NOTES, HELP_TEXT, UI_PADDING}, APP_NAME};


#[derive(Default)]
pub struct ChildWindows {
    show_about: Arc<RwLock<bool>>,
}

impl ChildWindows {
    pub fn toggle_about ( &mut self ) {
        let current_value = *self.show_about.read();
        *self.show_about.write() = !current_value;
    }

    pub fn show_windows ( &mut self, ctx: &Context ) {
        // Note: this needs to update its own show/hide, if appropriate

        let current_value = *self.show_about.read();
        if current_value {
            self.about(ctx);
        }
    }

    fn about ( &self, ctx: &Context ) {
        const TITLE: &str = "About";
        let title = format!("{} - {}", APP_NAME, TITLE);  // can we make this a const?
        let v_id = ViewportId::from_hash_of(&title);
        let show_about = self.show_about.clone();
        ctx.show_viewport_deferred(
            v_id,
            ViewportBuilder::default()
                .with_title(&title)
                ,
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
                        ui.label(format!("version {}", env!("CARGO_PKG_VERSION")));
                        ui.add_space(UI_PADDING);
                        ui.label(HELP_TEXT.join("\n"));
                        ui.add_space(UI_PADDING * 1.15);
                        ui.label(RichText::new("Change Notes").underline());
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

