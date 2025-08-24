use std::slice::Iter;

use eframe::egui::{
    Color32, ComboBox, Frame, Key, Label, Margin, Modifiers, RichText, Sense, Stroke, Ui,
};
use egui_extras::Column;
use log::{debug, info};

use crate::{
    app::EditResult,
    app_data::{AppData, DataIndex},
    localize::fl,
    managed_list::{GenericRef, GenericRefList, ManagedList, Named, StringList},
    sorting::Sorting,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct DisplayLine {
    fields: Vec<String>,
    id: DataIndex,
}

#[allow(dead_code)]
impl DisplayLine {
    pub fn id(&self) -> &String {
        assert!(!self.fields.is_empty()); // this should be impossible
        &self.fields[0]
    }

    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    pub fn field(&self, number: usize) -> &str {
        self.fields
            .get(number)
            .expect("unable to find promised index in display line")
            .as_ref()
    }

    pub fn field_iter(&self) -> Iter<'_, String> {
        self.fields.iter()
    }
}

fn displayline_from_item_ref<T: Named + Clone>(item: &T, index: &GenericRef<T>) -> DisplayLine {
    DisplayLine {
        fields: item.display_fields(),
        id: index.data_index(),
    }
}

// -------------------
#[derive(Clone)]
pub struct DisplayTable {
    lines: Vec<DisplayLine>,
    columns: Vec<ColumnInfo>,
    sorting: Sorting,
}

#[allow(dead_code)]
impl DisplayTable {
    pub fn lines_iter(&self) -> impl Iterator<Item = &DisplayLine> {
        self.lines.iter()
    }

    pub fn line(&self, index: usize) -> &DisplayLine {
        assert!(index < self.lines.len());
        &self.lines[index]
    }

    pub fn lines_len(&self) -> usize {
        self.lines.len()
    }

    pub fn column_definitions_iter(&self) -> impl Iterator<Item = Column> {
        self.columns.iter().map(|i| {
            use ColumnWidth::*;
            match i.width {
                Extended => Column::remainder(),
                Standard(width) => Column::auto().at_least(width),
            }
        })
    }

    pub fn headings_iter(&self) -> impl Iterator<Item = RichText> {
        self.columns.iter().enumerate().map(|(i, c)| {
            let heading_text = c.heading.clone();
            if i == self.sorting.sort_field() {
                heading_text.underline()
            } else {
                heading_text
            }
        })
    }

    pub fn number_columns(&self) -> usize {
        self.columns.len()
    }

    pub fn sorting(&self) -> &Sorting {
        &self.sorting
    }
}

const MIN_COL_WIDTH: f32 = 40.0;
const NAME_COL_WIDTH: f32 = 120.0;

impl<T: Named + Clone> From<&ManagedList<T>> for DisplayTable {
    fn from(list: &ManagedList<T>) -> Self {
        let item_list = list.item_ref_list();
        let mut lines: Vec<DisplayLine> = item_list
            .iter()
            .map(|(index, item)| displayline_from_item_ref(*item, index))
            .collect();
        let sorting = list.get_sorting();
        let sort_fn = |a: &DisplayLine, b: &DisplayLine| {
            a.fields[sorting.sort_field()].cmp(&b.fields[sorting.sort_field()])
        };
        lines.sort_by(sort_fn);
        if sorting.sort_reversed() {
            lines.reverse();
        }
        let headings = T::display_headings();
        assert!(!headings.is_empty());
        let last_heading = headings.len() - 1;
        let columns = headings
            .into_iter()
            .enumerate()
            .map(|(i, h)| {
                use ColumnWidth::*;
                ColumnInfo {
                    width: if i == last_heading {
                        Extended
                    } else if i == 0 {
                        Standard(NAME_COL_WIDTH)
                    } else {
                        Standard(MIN_COL_WIDTH)
                    },
                    heading: h,
                }
            })
            .collect();
        DisplayTable {
            lines,
            columns,
            sorting,
        }
    }
}

// -------------------
// ItemDisplayParams

#[derive(Clone, Copy)]
pub struct ShowEditInfo<'a> {
    name_collision: bool,
    differs_from: bool,
    create_new: bool,
    // could have reference lists??
    app_data: &'a AppData,
}

#[allow(dead_code)]
impl<'a> ShowEditInfo<'a> {
    pub fn new(
        name_collision: bool,
        differs_from: bool,
        create_new: bool,
        app_data: &'a AppData,
    ) -> Self {
        ShowEditInfo {
            name_collision,
            differs_from,
            create_new,
            app_data,
        }
    }

    pub fn name_collision(&self) -> bool {
        self.name_collision
    }

    pub fn differs_from(&self) -> bool {
        self.differs_from
    }

    pub fn create_new(&self) -> bool {
        self.create_new
    }

    pub fn show_save(&self) -> bool {
        self.differs_from && !self.name_collision
    }

    pub fn app_data(&self) -> &AppData {
        self.app_data
    }
}

// ----------------------
// Item layout constants
pub const EDGE_SPACER: f32 = 6.0;
pub const HEAD_SPACE: f32 = 6.0;
pub const STROKE_WIDTH: f32 = 1.;
pub const STROKE_COLOR: Color32 = Color32::GRAY;
pub const INNER_MARGIN: Margin = Margin::same(6);
pub const FIELD_VERTICAL_SPACE: f32 = 10.0;
pub const FIELD_HORIZONTAL_SPACE: f32 = 20.0;
pub const DESCRIPTION_ROWS: usize = 2;
pub const NOTES_ROWS: usize = 4;

pub trait ShowEdit {
    fn show_edit(&mut self, ui: &mut Ui, item_info: ShowEditInfo) -> Option<EditResult>;
}

pub fn show_edit_frame(
    ui: &mut Ui,
    item_name: String,
    debug_name: &str,
    item_info: ShowEditInfo,
    contents: impl FnOnce(&mut Ui),
) -> Option<EditResult> {
    let mut result = None;

    ui.horizontal(|ui| {
        if ui
            .add(Label::new(RichText::new("<").heading()).sense(Sense::click()))
            .clicked()
        {
            debug!("return from edit {debug_name}");
            result = Some(EditResult::Ignore); // should this be return?
        }
        ui.add_space(EDGE_SPACER);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(item_name.clone())
                        .heading()
                        .strong()
                        .underline(),
                );
                if item_info.show_save() {
                    ui.add_space(60.0);
                    if ui.button(fl!("edit_save")).clicked() {
                        info!("save edited {debug_name} requested");
                        result = Some(EditResult::Submit);
                    }
                }
            });
            ui.add_space(HEAD_SPACE);

            // todo: why does this show a small sized window?
            // ScrollArea::vertical()
            //     .id_salt(item_name)
            //     .auto_shrink([false, false])
            //     // .max_height(height)
            //     // .max_width(ui.available_width())
            //     // .show(ui, contents);
            //     .show(ui, |ui| {
            Frame::default()
                .stroke(Stroke::new(STROKE_WIDTH, STROKE_COLOR))
                .inner_margin(INNER_MARGIN)
                .show(ui, contents);
            // });
        });
    });

    result
}

// ------------------------
// Show / Edit GenericRefs

const EMPTY_NAME: &str = "    ";

pub fn show_edit_item<T: Named + Clone>(
    name: &str,
    item: &mut Option<GenericRef<T>>,
    master_list: &ManagedList<T>,
    ui: &mut Ui,
) {
    let item_list = {
        let mut list = master_list.names_sorted();
        list.insert(0, EMPTY_NAME.to_string());
        list
    };
    let mut selected_item = if let Some(is_item) = item {
        let name = is_item.name().unwrap_or_else(|| EMPTY_NAME.to_string());
        item_list
            .iter()
            .position(|n| *n == name)
            .unwrap_or_default()
    } else {
        0
    };
    let was_selected = selected_item;

    ComboBox::from_id_salt(name).show_index(ui, &mut selected_item, item_list.len(), |i| {
        item_list[i].to_string()
    });

    if selected_item != was_selected {
        *item = if selected_item == 0 {
            info!("clearing {name}");
            None
        } else {
            let new_name = &item_list[selected_item];
            info!("setting {name} to {new_name}");
            master_list.find(new_name)
        }
    }
}

pub fn show_edit_list<T: Named + Clone>(
    name: &str,
    this_list: &mut GenericRefList<T>,
    master_list: &ManagedList<T>,
    ui: &mut Ui,
) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let mut first = true;
        let was_hovered = this_list.hovered_name().map(|h| h.to_string());
        let mut hovered = None;
        for item in this_list.list().clone() {
            // this should not be able to point to nothing, but it might?
            if let Some(item_name) = item.display_name() {
                if !first {
                    ui.label(", ");
                }

                let item_label = {
                    let mut item_label = RichText::new(item_name.clone());
                    if let Some(hovered_name) = this_list.hovered_name()
                        && hovered_name == item_name
                    {
                        item_label = item_label.color(Color32::DARK_RED).strikethrough();
                    }
                    item_label
                };
                let resp = ui.add(Label::new(item_label).sense(Sense::click()));
                if resp.clicked() {
                    let item_id = item.name().unwrap_or_default();
                    info!("deleting {item_id} from list {name}");
                    this_list.swap_remove(&item_id);
                } else if resp.hovered() {
                    hovered = Some(item_name.clone());
                }
                first = false;
            } else {
                unreachable!();
            }
        }

        if hovered != was_hovered {
            this_list.set_hovered(hovered);
        }

        // now perhaps add an entry
        // show + -> click to create empty
        if !first {
            ui.label(", ");
        }

        if let Some(new_name) = this_list.new_name() {
            // show combo box
            let items_list = {
                let mut list = master_list.names_sorted();
                list.insert(0, EMPTY_NAME.to_string());
                list
            };
            let mut selected_item = items_list
                .iter()
                .position(|n| *n == new_name)
                .unwrap_or_default();
            let was_selected = selected_item;

            ComboBox::from_id_salt(name).show_index(
                ui,
                &mut selected_item,
                items_list.len(),
                |i| items_list[i].to_string(),
            );

            if selected_item != was_selected {
                if selected_item == 0 {
                    this_list.set_new(Some(EMPTY_NAME));
                } else {
                    let new_name = items_list[selected_item].as_str();
                    // find ref
                    if let Some(d) = master_list.find(new_name) {
                        info!("adding {new_name} to list {name}");
                        // push to list
                        this_list.push(d);
                        // reset new
                        this_list.set_new(None);
                    } else {
                        this_list.set_new(Some(new_name));
                    }
                }
            }
        } else {
            // no new name
            if ui
                .add(Label::new(RichText::new("+").strong()).sense(Sense::click()))
                .clicked()
            {
                info!("requested add item for {name}");
                this_list.set_new(Some(EMPTY_NAME));
            }
        }
    });
}

pub fn show_edit_stringlist_italics(name: &str, this_list: &mut StringList, ui: &mut Ui) {
    // todo: italics as option
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let mut first = true;
        let was_hovered = this_list.hovered_name().map(|h| h.to_string());
        let mut hovered = None;
        for item in this_list.list().clone() {
            if !first {
                ui.label(RichText::new(", ").italics());
            }

            let item_label = {
                let mut item_label = RichText::new(item.clone()).italics();
                if let Some(hovered_name) = this_list.hovered_name()
                    && hovered_name == item
                {
                    item_label = item_label.color(Color32::DARK_RED).strikethrough();
                }
                item_label
            };
            let resp = ui.add(Label::new(item_label).sense(Sense::click()));
            if resp.clicked() {
                info!("deleting {item} from list {name}");
                this_list.swap_remove(&item);
            } else if resp.hovered() {
                hovered = Some(item.clone());
            }
            first = false;
        }

        if hovered != was_hovered {
            this_list.set_hovered(hovered);
        }

        // now perhaps add an entry
        // show + -> click to create empty
        if !first {
            ui.label(RichText::new(", ").italics());
        }

        match this_list.new_name() {
            NewStringStatus::Requested => {
                let mut new_item = String::new();
                let resp = ui.text_edit_singleline(&mut new_item);
                resp.request_focus();
                this_list.set_new(NewStringStatus::Showing(new_item));
            }

            NewStringStatus::Showing(item) => {
                let resp = ui.text_edit_singleline(item);
                let new_name = item.clone();
                let done = ui.input(|i| i.key_pressed(Key::Enter)); // can we consume these?
                let exit = ui.input(|i| i.key_pressed(Key::Escape));
                if resp.lost_focus() {
                    if new_name.is_empty() || exit {
                        info!("lost focus, exiting item creation");
                        this_list.set_new(NewStringStatus::NoItem);
                        ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
                    } else if done {
                        info!("leaving, adding {new_name} to list {name}");
                        this_list.push(new_name.to_string());
                        this_list.set_new(NewStringStatus::Requested); // add another one?
                        ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
                    }
                }
            }

            NewStringStatus::NoItem => {
                // no new name
                if ui
                    .add(Label::new(RichText::new("+").strong()).sense(Sense::click()))
                    .clicked()
                {
                    info!("requested add item for {name}");
                    this_list.set_new(NewStringStatus::Requested);
                }
            }
        }
    });
}

// ----------------------
// NewItemStatus
#[derive(Debug, Clone, Default, PartialEq)]
pub enum NewStringStatus {
    #[default]
    NoItem,
    Requested, // could include a timer here of some sort
    Showing(String),
}

// -----------------------
// Column Width
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnWidth {
    Extended,
    Standard(f32),
}

// -----------------------
// Column Info
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    width: ColumnWidth,
    heading: RichText,
}
