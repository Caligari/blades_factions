use std::{
    cell::RefCell,
    collections::BTreeMap,
    ffi::OsStr,
    fmt::Display,
    fs::{self, File, OpenOptions, create_dir_all},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    sync::Arc,
};

use anyhow::anyhow;
use bytes_cast::{BytesCast, unaligned};
use directories_next::ProjectDirs;
use eframe::egui::FontFamily::Proportional;
use eframe::egui::FontId;
use eframe::egui::TextStyle::*;
use eframe::{
    CreationContext, Frame,
    egui::{
        Align, Button, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily,
        Label, Layout, Margin, MenuBar, RichText, Sense, Separator, Stroke, Theme, TopBottomPanel,
        Ui, ViewportCommand,
    },
};
use egui_extras::{Column, TableBuilder};
use enum_iterator::{Sequence, all, cardinality};
use log::{debug, error, info};
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    action::{Action, ActionNode},
    app_data::AppData,
    app_display::{ShowEdit, ShowEditInfo},
    app_settings::AppSettings,
    child_windows::{ChildWindows, FileDialogType, FileTarget},
    district::District,
    faction::Faction,
    localize::fl,
    managed_list::{DistrictRef, FactionRef, Named, PersonRef},
    person::Person,
    todo::TodoUndo,
};

const ZOOM: f32 = 1.0;
pub const UI_PADDING: f32 = 8.0;
const ERROR_SPACE: f32 = 16.0;

const ERROR_BACKGROUND: Color32 = Color32::from_rgb(255, 190, 190);
const ERROR_FOREGROUND: Color32 = Color32::DARK_RED;

// todo: localize this
// probably could be one phrase?
pub const HELP_TEXT: &[&str] = &[
    "created by Liam Routt",
    "for use with Blades in the Dark",
    "",
    "This utility allows you to track the various factions in your game.",
    "",
];

// ? Can these be localized?
pub const CHANGE_NOTES: &[&str] = &["0.1.0 - initial version"];

// todo: localize this
pub const FONT_NOTES: &[&str] = &[
    "Using the Dihjauti font, by T. Christopher White, covered under the SIL Open Font License (http://scripts.sil.org/OFL)",
];

#[allow(dead_code)]
pub struct App {
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
    pub fn new(
        settings: AppSettings,
        project_directories: ProjectDirs,
        cc: &CreationContext<'_>,
    ) -> Self {
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

    fn reset(&mut self) {
        // do not reset: settings, project_directtories, status
        self.status = AppStatus::default();
        self.main_view = MainView::default();
        self.data = AppData::default();
        self.message = None;
        self.child_windows = ChildWindows::default(); // is this sufficient?
        self.todo_undo = TodoUndo::default();
    }

    fn show_top(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top("top").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    // when can we save/load?
                    let load_enabled = matches!(self.status, AppStatus::Ready(_)); // only save from main list
                    let save_as_enabled = load_enabled && !self.data.is_empty();

                    ui.menu_button(fl!("menu"), |ui| {
                        if ui.button(fl!("menu_restart")).clicked() {
                            self.status = AppStatus::Starting;
                            self.message = None;
                            info!("Requested Restart");
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui
                            .add_enabled(load_enabled, Button::new(fl!("menu_load")))
                            .clicked()
                        {
                            info!("Requested Load");
                            self.child_windows.start_file_dialog(
                                FileDialogType::Load,
                                FileTarget::Internal,
                                self.project_directories.data_dir().to_path_buf(),
                            );
                            self.status = AppStatus::Load;
                        }
                        if ui
                            .add_enabled(save_as_enabled, Button::new(fl!("menu_save")))
                            .clicked()
                        {
                            // TODO: save data
                            info!("Requested Save");
                            if self.data.get_loaded_from().is_some() {
                                info!("Doing SaveTo");
                                self.status = AppStatus::SaveTo;
                            } else {
                                info!("Forced SaveAs -> no loaded file data present");
                                // todo: provide default name?
                                self.child_windows.start_file_dialog(
                                    FileDialogType::Save,
                                    FileTarget::Internal,
                                    self.project_directories.data_dir().to_path_buf(),
                                );
                                self.status = AppStatus::SaveAs;
                            }
                        }
                        if ui
                            .add_enabled(save_as_enabled, Button::new(fl!("menu_save_as")))
                            .clicked()
                        {
                            // TODO: save as data - provide default name?
                            info!("Requested Save As");
                            self.child_windows.start_file_dialog(
                                FileDialogType::Save,
                                FileTarget::Internal,
                                self.project_directories.data_dir().to_path_buf(),
                            );
                            self.status = AppStatus::SaveAs;
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui
                            .add_enabled(load_enabled, Button::new(fl!("menu_import")))
                            .clicked()
                        {
                            // TODO: import data
                            info!("Requested Import");
                            self.child_windows.start_file_dialog(
                                FileDialogType::Load,
                                FileTarget::Export,
                                self.project_directories.data_dir().to_path_buf(),
                            );
                            self.status = AppStatus::Import;
                        }
                        if ui
                            .add_enabled(save_as_enabled, Button::new(fl!("menu_export")))
                            .clicked()
                        {
                            // TODO: export data
                            info!("Requested Export");
                            self.child_windows.start_file_dialog(
                                FileDialogType::Save,
                                FileTarget::Export,
                                self.project_directories.data_dir().to_path_buf(),
                            );
                            self.status = AppStatus::Export;
                        }
                        ui.add(Separator::default().spacing(2.));
                        if ui
                            .add_enabled(false, Button::new(fl!("menu_settings")))
                            .clicked()
                        {
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

    fn show_footer(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(5.);
            ui.horizontal(|ui| {
                ui.label(self.status.to_string());
                if let Some(file) = self.data.get_loaded_from()
                    && let Some(name) = file.file_stem()
                {
                    ui.label(RichText::new(format!("({})", name.to_string_lossy())).italics());
                }
                if let Some(message) = &self.message {
                    let error_message = RichText::new(fl!("app_error_err", err = message))
                        .strong()
                        .background_color(ERROR_BACKGROUND)
                        .color(ERROR_FOREGROUND); // todo: check colors with theme
                    ui.label(error_message);
                    // todo: we need an "OK" button to clear the message?
                    ui.add_space(ERROR_SPACE);
                    if ui
                        .button(RichText::new(fl!("acknowledge_error")).color(ERROR_FOREGROUND))
                        .clicked()
                    {
                        self.message = None;
                    }
                }
            });
        });
    }
}

// ---------------------------
#[allow(clippy::match_single_binding)]
#[allow(unused_imports)]
impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        use AppStatus::*;

        ctx.set_visuals(self.settings.theme().default_visuals());

        self.show_top(ctx, frame);
        self.show_footer(ctx);

        // todo: can we run the todo list here? Or will that lead to slowdown?

        if let Some(new_status) = CentralPanel::default().show(ctx,  |ui: &mut Ui| {
            match &self.status {
                Starting => {
                    // if not already starting, kick things off
                    // info!("Starting")
                    // if completed
                    info!("resetting local data");
                    self.reset();

                    info!("Starting => Ready");
                    Some(Ready(RefCell::new(None)))
                    // otherwise, keep doing start
                    // None
                }

                Ready ( hovered_line ) => {
                    // what are we looking at?
                    // select between views
                    let view_request = self.show_select_views(ui);
                    if matches!(view_request, ViewRequest::NewItem) {
                        info!("Ready -> Create New {}", self.main_view.item_name());
                        match self.main_view {
                            MainView::Districts => Some(ShowEditDistrict(None, RefCell::new(District::default()))),
                            MainView::Persons => Some(ShowEditPerson(None, RefCell::new(Person::default()))),
                            MainView::Factions => Some(ShowEditFaction(None, RefCell::new(Faction::default()))),
                        }
                    } else {
                        // not asking for a new item
                        if let ViewRequest::NewView(new_view) = view_request {
                            self.main_view = new_view;
                        }

                        // find or build display data table
                        let display_table = match &self.main_view {
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
                        const STROKE_COLOR: Color32 = Color32::GRAY;
                        const INNER_MARGIN: Margin = Margin::same(6);
                        const HEADER_HEIGHT: f32 = 25.0;
                        const ROW_HEIGHT: f32 = 18.0;
                        const TINY_SPACE: f32 = 4.0;

                        let mut new_sort = None;
                        let mut new_selected = None;
                        let mut new_hovered_line = None;

                        ui.horizontal_top(|ui| {
                            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                ui.add_space(TINY_SPACE);
                                eframe::egui::Frame::default()
                                    .stroke(Stroke::new(STROKE_WIDTH, STROKE_COLOR))
                                    .inner_margin(INNER_MARGIN)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.style_mut().wrap_mode = Some(eframe::egui::TextWrapMode::Extend);
                                            let mut table = TableBuilder::new(ui)
                                                .striped(true)
                                                .sense(Sense::click())
                                                .resizable(true)
                                                .auto_shrink([false, true]);

                                            // todo: move this into DisplayTable - return column definitions
                                            for f in 0..display_table.number_columns() {
                                                const MIN_COL_WIDTH: f32 = 40.0;
                                                const NAME_COL_WIDTH: f32 = 120.0;

                                                let col = if f == 0 {  // first column
                                                    Column::auto().at_least(NAME_COL_WIDTH)
                                                } else {
                                                    Column::auto().at_least(MIN_COL_WIDTH)
                                                };
                                                // for use with extended field, if used
                                                // Column::remainder(),

                                                table = table.column(col);
                                            }

                                            table.header(HEADER_HEIGHT, |mut header| {
                                                for (i, heading) in display_table.headings_iter().enumerate() {
                                                    header.col(|ui| {
                                                        if ui.add(Label::new(heading).sense(Sense::click())).clicked() {
                                                            new_sort = Some(i);
                                                        }
                                                    });
                                                }
                                            })
                                            .body(|body| {
                                                body.rows(ROW_HEIGHT, display_table.lines_len(), | mut row | {
                                                    let i = row.index();
                                                    let display_line = display_table.line(i);
                                                    let mut field_click = false;
                                                    let mut field_hover = false;

                                                    if let Some(h_line) = *hovered_line.borrow() {
                                                        let hovering = i == h_line;
                                                        row.set_hovered(hovering);
                                                    }

                                                    for f in display_line.field_iter() {
                                                        row.col(|ui| {
                                                            let col_resp = ui.add(
                                                                Label::new(f)
                                                                .sense(Sense::click())
                                                            );

                                                            field_click |= col_resp.clicked();

                                                            if col_resp.hovered() {
                                                                field_hover = true;
                                                            }
                                                        });
                                                    }

                                                    let row_resp = row.response();

                                                    if field_click || row_resp.clicked() {
                                                        debug!("row {} ({}) clicked", i, display_line.id());
                                                        new_selected = Some(display_line.id());
                                                    } else if field_hover || row_resp.hovered() {
                                                        let already_hovered = {
                                                            if let Some(hover) = *hovered_line.borrow() {
                                                                hover == i
                                                            } else { false }
                                                        };
                                                        if !already_hovered {
                                                            debug!("row {i} hovered");
                                                        }
                                                        new_hovered_line = Some(i);  // set this regardless
                                                    }
                                                });
                                            });
                                        });
                                    });
                            });
                        });

                        *hovered_line.borrow_mut() = new_hovered_line;

                        match &self.main_view {
                            MainView::Districts => {
                                if let Some(sort_index) = new_sort {
                                    debug!("setting district col {sort_index} to sort");
                                    self.data.set_districts_sort(sort_index);
                                    None
                                } else if let Some(id) = new_selected {
                                    debug!("selected distirct {id}");
                                    if let Some(show) = self.data.find_district(id) {
                                        if let Some(district) = self.data.clone_district(&show) {
                                            info!("Ready -> Show District ({id})");
                                            Some(ShowEditDistrict(Some(show), RefCell::new(district)))
                                        } else { unreachable!("unable to clone district {id} using reference"); }
                                    } else { unreachable!("selected district '{id}' which is not in list"); }
                                } else { None }
                            }

                            MainView::Persons => {
                                if let Some(sort_index) = new_sort {
                                    debug!("setting persons col {sort_index} to sort");
                                    self.data.set_persons_sort(sort_index);
                                    None
                                } else if let Some(id) = new_selected {
                                    debug!("selected person {id}");
                                    if let Some(show) = self.data.find_person(id) {
                                        if let Some(person) = self.data.clone_person(&show) {
                                            info!("Ready -> Show Person ({id})");
                                            Some(ShowEditPerson(Some(show), RefCell::new(person)))
                                        } else { unreachable!("unable to clone person {id} using reference"); }
                                    } else { unreachable!("selected person '{id}' which is not in list"); }
                                } else { None }
                            }

                            MainView::Factions => {
                                if let Some(sort_index) = new_sort {
                                    debug!("setting factions col {sort_index} to sort");
                                    self.data.set_factions_sort(sort_index);
                                    None
                                } else if let Some(id) = new_selected {
                                    debug!("selected faction {id}");
                                    if let Some(show) = self.data.find_faction(id) {
                                        if let Some(faction) = self.data.clone_faction(&show) {
                                            info!("Ready -> Show Faction ({id})");
                                            Some(ShowEditFaction(Some(show), RefCell::new(faction)))
                                        } else { unreachable!("unable to clone faction {id} using reference"); }
                                    } else { unreachable!("selected faction '{id}' which is not in list"); }
                                } else { None }
                            }
                        }
                    }
                }

                ShowEditDistrict( index_ref, district, ) => {
                    let mut district = district.borrow_mut();
                    let (name_collision, differs_from) = if let Some(index_ref) = index_ref {
                        let old_name = index_ref.name().map_or("<none>".to_string(), |n| n);
                        if let Some(old_district) = self.data.clone_district(index_ref) {
                            if old_name != district.name() {
                                (self.data.find_district(district.name()).is_some(), true)
                            } else { (false, old_district != *district) }
                        } else {
                            error!("unable to find district '{old_name}' when index ref exists, during replace");
                            (false, true)
                        }
                    } else { (self.data.find_district(district.name()).is_some(), District::default() != *district) };

                    let item_info = ShowEditInfo::new(name_collision, differs_from, index_ref.is_none(), &self.data);

                    if let Some(edit_result) = district.show_edit(ui, item_info) {
                        use EditResult::*;
                        match edit_result {
                            Submit => {
                                info!("submit edited district");
                                if let Some(index_ref) = index_ref {
                                    // fetch indexed item
                                    if let Some(old_district) = self.data.clone_district(index_ref) {
                                        if old_district != *district {
                                            info!("replacing existing district");
                                            self.todo_undo.add_todo(ActionNode::from(Action::DistrictReplace(index_ref.clone(), district.clone())));
                                        }  else {
                                            debug!("new district matches existing district - no action taken");
                                        }
                                    } else {
                                        let old_name = index_ref.name().map_or("<none>".to_string(), |n| n);
                                        error!("unable to find existing district {old_name} in data, on replace attempt");
                                    }
                                } else {
                                    // no index, thus this is an Add
                                    info!("adding new district");
                                    self.todo_undo.add_todo(ActionNode::from(Action::DistrictAdd(district.clone())));
                                }
                                Some(Ready(RefCell::new(None)))
                            },
                            Ignore => {
                                info!("ignore edited district");
                                Some(Ready(RefCell::new(None)))
                            },
                        }
                    } else { None }
                }

                ShowEditPerson( index_ref, person, ) => {
                    let mut person = person.borrow_mut();
                    let (name_collision, differs_from) = if let Some(index_ref) = index_ref {
                        let old_name = index_ref.name().map_or("<none>".to_string(), |n| n);
                        if let Some(old_person) = self.data.clone_person(index_ref) {
                            if old_name != person.name() {
                                (self.data.find_person(person.name()).is_some(), true)
                            } else { (false, old_person != *person) }
                        } else {
                            error!("unable to find person '{old_name}' when index ref exists, during replace");
                            (false, true)
                        }
                    } else { (self.data.find_person(person.name()).is_some(), Person::default() != *person) };

                    let item_info = ShowEditInfo::new(name_collision, differs_from, index_ref.is_none(), &self.data);

                    if let Some(edit_result) = person.show_edit(ui, item_info) {
                        use EditResult::*;
                        match edit_result {
                            Submit => {
                                info!("submit edited person");
                                if let Some(index_ref) = index_ref {
                                    // fetch indexed item
                                    if let Some(old_person) = self.data.clone_person(index_ref) {
                                        if old_person != *person {
                                            info!("replacing existing person");
                                            self.todo_undo.add_todo(ActionNode::from(Action::PersonReplace(index_ref.clone(), person.clone())));
                                        }  else {
                                            debug!("new person matches existing person - no action taken");
                                        }
                                    } else {
                                        let old_name = index_ref.name().map_or("<none>".to_string(), |n| n);
                                        error!("unable to find existing person {old_name} in data, on replace attempt");
                                    }
                                } else {
                                    // no index, thus this is an Add
                                    info!("adding new person");
                                    self.todo_undo.add_todo(ActionNode::from(Action::PersonAdd(person.clone())));
                                }
                                Some(Ready(RefCell::new(None)))
                            },
                            Ignore => {
                                info!("ignore edited person");
                                Some(Ready(RefCell::new(None)))
                            },
                        }
                    } else { None }
                }

                ShowEditFaction( index_ref, faction, ) => {
                    let mut faction = faction.borrow_mut();
                    let (name_collision, differs_from) = if let Some(index_ref) = index_ref {
                        let old_name = index_ref.name().map_or("<none>".to_string(), |n| n);
                        if let Some(old_faction) = self.data.clone_faction(index_ref) {
                            if old_name != faction.name() {
                                (self.data.find_faction(faction.name()).is_some(), true)
                            } else { (false, old_faction != *faction) }
                        } else {
                            error!("unable to find faction '{old_name}' when index ref exists, during replace");
                            (false, true)
                        }
                    } else { (self.data.find_faction(faction.name()).is_some(), Faction::default() != *faction) };

                    let item_info = ShowEditInfo::new(name_collision, differs_from, index_ref.is_none(), &self.data);

                    if let Some(edit_result) = faction.show_edit(ui, item_info) {
                        use EditResult::*;
                        match edit_result {
                            Submit => {
                                info!("submit edited faction");
                                if let Some(index_ref) = index_ref {
                                    // fetch indexed item
                                    if let Some(old_faction) = self.data.clone_faction(index_ref) {
                                        if old_faction != *faction {
                                            info!("replacing existing faction");
                                            self.todo_undo.add_todo(ActionNode::from(Action::FactionReplace(index_ref.clone(), faction.clone())));
                                        }  else {
                                            debug!("new faction matches existing faction - no action taken");
                                        }
                                    } else {
                                        let old_name = index_ref.name().map_or("<none>".to_string(), |n| n);
                                        error!("unable to find existing faction {old_name} in data, on replace attempt");
                                    }
                                } else {
                                    // no index, thus this is an Add
                                    info!("adding new faction");
                                    self.todo_undo.add_todo(ActionNode::from(Action::FactionAdd(faction.clone())));
                                }
                                Some(Ready(RefCell::new(None)))
                            },
                            Ignore => {
                                info!("ignore edited faction");
                                Some(Ready(RefCell::new(None)))
                            },
                        }
                    } else { None }
                }

                Load => {
                    if let Some(selected) = self.child_windows.selected_file() {
                        if !selected.as_os_str().is_empty() {  // checks for blank file selected, indicating cancel
                            // process file
                            info!("selected file: {}", selected.to_string_lossy());

                            match AppData::load_from_file(selected.as_path()) {
                                Ok(data) => {
                                    self.data = data;
                                    self.main_view = MainView::default();
                                    info!("loaded data from {}", selected.to_string_lossy());
                                }

                                Err(e) => {
                                    let file = selected.file_name().map_or(OsStr::new("<no file>").to_string_lossy(), |f| f.to_string_lossy());
                                    // let message = format!("{}, when loading file [{file}]", e);
                                    let message = format!("Unable to load save file [{file}]");
                                    self.message = Some(message);
                                    error!("Error on file load for [{}]: {}", selected.to_string_lossy(), e);
                                }
                            }
                        } else { info!("no load file selected - ignoring"); }
                        info!("Loading => Ready");
                        Some(Ready(RefCell::new(None)))
                    } else { None }
                }

                SaveAs => {
                    if let Some(selected) = self.child_windows.selected_file() {
                        if !selected.as_os_str().is_empty() {  // checks for blank file selected, indicating cancel
                            // process file
                            info!("selected file: {}", selected.to_string_lossy());

                            match self.data.save_to_file(selected.as_path()) {
                                Ok(()) => {
                                    info!("saved data to {}", selected.to_string_lossy());
                                    self.data.set_loaded_from(Some(selected));
                                }

                                Err(e) => {
                                    let file = selected.file_name().map_or(OsStr::new("<no file>").to_string_lossy(), |f| f.to_string_lossy());
                                    // let message = format!("{}, when loading file [{file}]", e);
                                    let message = format!("Unable to save data to file [{file}]");
                                    self.message = Some(message);
                                    error!("Error on save file for [{}]: {}", selected.to_string_lossy(), e);
                                }
                            }
                        } else { info!("no save file selected - ignoring"); }
                        info!("SaveAs => Ready");
                        Some(Ready(RefCell::new(None)))
                    } else { None }
                }

                SaveTo => {
                    if let Some(selected) = self.data.get_loaded_from() {
                        if !selected.as_os_str().is_empty() {  // checks for blank file selected, indicating cancel
                            // process file
                            info!("saving to file: {}", selected.to_string_lossy());

                            match self.data.save_to_file(selected.as_path()) {
                                Ok(()) => {
                                    info!("saved data to {}", selected.to_string_lossy());
                                }

                                Err(e) => {
                                    let file = selected.file_name().map_or(OsStr::new("<no file>").to_string_lossy(), |f| f.to_string_lossy());
                                    // let message = format!("{}, when loading file [{file}]", e);
                                    let message = format!("Unable to save data to file [{file}]");
                                    self.message = Some(message);
                                    error!("Error on save file for [{}]: {}", selected.to_string_lossy(), e);
                                }
                            }
                        } else { error!("data has no save file - unable to save"); }
                        info!("SaveTo => Ready");
                        Some(Ready(RefCell::new(None)))
                    } else { None }
                }

                Import => {
                    if let Some(selected) = self.child_windows.selected_file() {
                        if !selected.as_os_str().is_empty() {  // checks for blank file selected, indicating cancel
                            // process file
                            info!("selected file: {}", selected.to_string_lossy());

                            match self.data.import_from_file(selected.as_path()) {
                                Ok(()) => {
                                    self.main_view = MainView::default();
                                    info!("imported data from {}", selected.to_string_lossy());
                                }

                                Err(e) => {
                                    let file = selected.file_name().map_or(OsStr::new("<no file>").to_string_lossy(), |f| f.to_string_lossy());
                                    // let message = format!("{}, when loading file [{file}]", e);
                                    let message = format!("Unable to import data from [{file}]");
                                    self.message = Some(message);
                                    error!("Error on data import from [{}]: {}", selected.to_string_lossy(), e);
                                }
                            }
                        } else { info!("no import file selected - ignoring"); }
                        info!("Import => Ready");
                        Some(Ready(RefCell::new(None)))
                    } else { None }
                }

                Export => {
                    if let Some(selected) = self.child_windows.selected_file() {
                        if !selected.as_os_str().is_empty() {  // checks for blank file selected, indicating cancel
                            // process file
                            info!("selected file: {}", selected.to_string_lossy());

                            match self.data.export_to_file(selected.as_path()) {
                                Ok(()) => {
                                    info!("exported data to {}", selected.to_string_lossy());
                                }

                                Err(e) => {
                                    let file = selected.file_name().map_or(OsStr::new("<no file>").to_string_lossy(), |f| f.to_string_lossy());
                                    // let message = format!("{}, when loading file [{file}]", e);
                                    let message = format!("Unable to export data to file [{file}]");
                                    self.message = Some(message);
                                    error!("Error on data export to [{}]: {}", selected.to_string_lossy(), e);
                                }
                            }
                        } else { info!("no export file selected - ignoring"); }
                        info!("Export => Ready");
                        Some(Ready(RefCell::new(None)))
                    } else { None }
                }


            }
        }).inner {
            self.status = new_status;
        }

        self.run_todo();

        self.child_windows.show_windows(ctx);
    }
}

impl App {
    fn show_select_views(&self, ui: &mut Ui) -> ViewRequest {
        let select_color = if self.settings.theme() == Theme::Dark {
            Color32::LIGHT_GREEN
        } else {
            Color32::DARK_GREEN
        };
        let mut new_request = ViewRequest::None;
        ui.horizontal(|ui| {
            for (num, view) in all::<MainView>().enumerate() {
                let mut v_text =
                    RichText::new(format!("{} ({})", view, self.data.view_size(view))).heading();
                if self.main_view == view {
                    v_text = v_text.color(select_color).underline();
                }

                if ui.add(Label::new(v_text).sense(Sense::click())).clicked() {
                    info!("selected {view}");
                    new_request = ViewRequest::NewView(view);
                }

                if num != cardinality::<MainView>() {
                    ui.heading(" | ");
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                let item = self.main_view.item_name();
                if ui
                    .add(Button::new(fl!("app_add_itm", itm = item.clone())).sense(Sense::click()))
                    .clicked()
                {
                    info!("New {item} requested");
                    new_request = ViewRequest::NewItem;
                }
            });
        });

        new_request
    }

    fn run_todo(&mut self) {
        if let Some(todo) = self.todo_undo.todo() {
            info!("carrying out todo");
            let result = self.data.do_action(&todo);
            match result {
                Ok(undo) => {
                    info!("todo complete");
                    self.todo_undo.add_undo(undo);
                    // self.todo_undo.add_done(todo);  // todo: not sure this is what we need; is done for undo actions?
                }

                Err(err) => {
                    error!("unable to complete todo action: {err}");
                    // do we add this back to the todo?
                }
            }
        }
    }
}

// ===========================
// ViewRequest

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum ViewRequest {
    #[default]
    None,
    NewView(MainView),
    NewItem, // uses current view
}

// ===========================
// EditResult

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditResult {
    Submit,
    Ignore,
}

// ===========================
// AppStatus

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Default)]
enum AppStatus {
    #[default]
    Starting,
    Ready(RefCell<Option<usize>>),
    ShowEditDistrict(Option<DistrictRef>, RefCell<District>),
    ShowEditPerson(Option<PersonRef>, RefCell<Person>),
    ShowEditFaction(Option<FactionRef>, RefCell<Faction>),
    Load,
    SaveTo, // No file dialog, use existing save file name
    SaveAs, // use file dialog to get file name
    Import,
    Export,
}

impl Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use AppStatus::*;

        write!(
            f,
            "{}",
            match self {
                Starting => fl!("app_starting"),
                Ready(..) => fl!("app_ready"),
                ShowEditDistrict(ind, ..) => {
                    let item = fl!("main_item_district");
                    if ind.is_none() {
                        fl!("app_create_itm", itm = item)
                    } else {
                        fl!("app_edit_itm", itm = item)
                    }
                }
                ShowEditPerson(ind, ..) => {
                    let item = fl!("main_item_person");
                    if ind.is_none() {
                        fl!("app_create_itm", itm = item)
                    } else {
                        fl!("app_edit_itm", itm = item)
                    }
                }
                ShowEditFaction(ind, ..) => {
                    let item = fl!("main_item_faction");
                    if ind.is_none() {
                        fl!("app_create_itm", itm = item)
                    } else {
                        fl!("app_edit_itm", itm = item)
                    }
                }
                Load => fl!("app_loading"),
                SaveAs => fl!("app_saving"),
                SaveTo => fl!("app_saving"),
                Import => fl!("app_importing"),
                Export => fl!("app_exporting"),
            }
        )
    }
}

// ===========================
// MainView

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, Sequence, PartialEq, Eq)]
pub enum MainView {
    #[default]
    Factions,
    Persons,
    Districts,
}

impl MainView {
    pub fn item_name(&self) -> String {
        use MainView::*;

        match self {
            Factions => fl!("main_item_faction"),
            Persons => fl!("main_item_person"),
            Districts => fl!("main_item_district"),
        }
    }
}

impl Display for MainView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MainView::*;

        write!(
            f,
            "{}",
            match self {
                Factions => fl!("main_factions"),
                Persons => fl!("main_persons"),
                Districts => fl!("main_districts"),
            }
        )
    }
}

// ===========================
// Additional functions

fn configure_fonts(ctx: &CreationContext, zoom: f32) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "dihjauti".to_string(),
        Arc::new(FontData::from_static(include_bytes!(
            "../Dihjauti-Regular.otf"
        ))),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "dihjauti".to_owned());
    ctx.egui_ctx.set_fonts(fonts);

    // Redefine text_styles
    let text_styles: BTreeMap<_, _> = [
        (Heading, FontId::new(30.0, Proportional)),
        // (Name("Heading2".into()), FontId::new(25.0, Proportional)),
        // (Name("Context".into()), FontId::new(23.0, Proportional)),
        (Body, FontId::new(16.0, Proportional)),
        (Monospace, FontId::new(14.0, Proportional)),
        (Button, FontId::new(14.0, Proportional)),
        (Small, FontId::new(12.0, Proportional)),
    ]
    .into();

    // Mutate global styles with new text styles
    ctx.egui_ctx
        .all_styles_mut(move |style| style.text_styles = text_styles.clone());

    ctx.egui_ctx.set_zoom_factor(zoom);
}

// ---------------------------
// Load and Save to POT files
const SAVE_FILE_ID: &[u8] = &[0x2b, 0x4a]; // magic number - is this endian neutral?

// used in Settings
pub fn save_to_pot<T>(file_path: &Path, data: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    use anyhow::Context;

    if let Some(dir_path) = file_path.parent() {
        create_dir_all(dir_path)?
    }

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }

    let file_handler = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)
        .with_context(|| {
            format!(
                "Failed to create save file [{}]",
                file_path.to_string_lossy()
            )
        })?;

    let mut buf = pot::to_vec(data)?;
    buf.splice(0..0, SAVE_FILE_ID.iter().cloned());
    let mut buf_writer = BufWriter::new(&file_handler);
    buf_writer.write_all(buf.as_slice())?;
    buf_writer.flush()?;
    Ok(())
}

// used in Settings
pub fn load_from_pot<T>(file_path: &Path) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    use anyhow::Context;

    let file_handler = OpenOptions::new()
        .read(true)
        .open(file_path)
        .with_context(|| format!("Failed to open save file [{}]", file_path.to_string_lossy()))?;
    let mut buf_reader = BufReader::new(&file_handler);
    let mut check_id = [0u8, 0u8];
    buf_reader.read_exact(&mut check_id)?;
    if check_id != SAVE_FILE_ID {
        return Err(anyhow!(
            "File [{}] does not have expected SAVE_FILE_ID",
            file_path.to_string_lossy()
        ));
    }
    let data: T = pot::from_reader(buf_reader)?;
    Ok(data)
}

pub fn save_to_save(
    file_path: &Path,
    save_version: u16,
    mut buffer: Vec<u8>,
) -> anyhow::Result<()> {
    use anyhow::Context;

    if let Some(dir_path) = file_path.parent() {
        create_dir_all(dir_path)?
    }

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }

    let file_handler = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)
        .with_context(|| {
            format!(
                "Failed to create save file [{}]",
                file_path.to_string_lossy()
            )
        })?;
    let Ok((file_id, excess)) = unaligned::U16Le::from_bytes(SAVE_FILE_ID) else {
        return Err(anyhow!("unable to convert header bytes"));
    };
    assert!(excess.is_empty());
    let save_header = SaveFileHeader {
        file_id: *file_id,
        save_version: save_version.into(),
    };
    buffer.splice(0..0, save_header.as_bytes().iter().cloned());
    let mut buf_writer = BufWriter::new(&file_handler);
    buf_writer.write_all(buffer.as_slice())?;
    buf_writer.flush()?;

    Ok(())
}

pub fn load_from_save(file_path: &Path) -> anyhow::Result<(u16, BufReader<File>)> {
    use anyhow::Context;

    let file_handle = OpenOptions::new()
        .read(true)
        .open(file_path)
        .with_context(|| format!("Failed to open save file [{}]", file_path.to_string_lossy()))?;

    let mut buf_reader = BufReader::new(file_handle);
    let mut check_header = [0u8, 0u8, 0u8, 0u8];
    buf_reader.read_exact(&mut check_header)?;
    let Ok((header, excess)) = SaveFileHeader::from_bytes(&check_header) else {
        return Err(anyhow!("unable to convert header bytes"));
    };
    if header.file_id.as_bytes() != SAVE_FILE_ID {
        return Err(anyhow!("File does not have expected SAVE_FILE_ID")); // file name? pass up?
    }
    assert!(excess.is_empty());
    let save_version = header.save_version.into();
    Ok((save_version, buf_reader))
}

// ------
#[derive(Debug, BytesCast)]
#[repr(C)]
struct SaveFileHeader {
    file_id: unaligned::U16Le,
    save_version: unaligned::U16Le,
}

// ---------------------------
// Load and Save to JSON files

#[allow(dead_code)]
pub fn save_to_json<T>(file_path: &Path, data: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    use anyhow::Context;

    if let Some(dir_path) = file_path.parent() {
        create_dir_all(dir_path)?
    }

    if file_path.exists() {
        fs::remove_file(file_path)?;
    }

    let file_handler = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)
        .with_context(|| {
            format!(
                "Failed to create export file [{}]",
                file_path.to_string_lossy()
            )
        })?;

    let mut buf_writer = BufWriter::new(&file_handler);
    serde_json::to_writer(&mut buf_writer, data)?;
    buf_writer.flush()?;
    Ok(())
}

#[allow(dead_code)]
pub fn load_from_json<T>(file_path: &Path) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    use anyhow::Context;

    let file_handler = OpenOptions::new()
        .read(true)
        .open(file_path)
        .with_context(|| {
            format!(
                "Failed to open import file [{}]",
                file_path.to_string_lossy()
            )
        })?;
    let buf_reader = BufReader::new(&file_handler);
    let data: T = serde_json::from_reader(buf_reader)?;
    Ok(data)
}
