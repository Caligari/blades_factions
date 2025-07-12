use std::slice::Iter;

use eframe::egui::{Color32, ComboBox, Label, Margin, RichText, Sense, Stroke, Ui};
use log::{debug, info};

use crate::{app::EditResult, app_data::{AppData, DataIndex}, localize::fl, managed_list::{GenericRef, GenericRefList, ManagedList, Named}, sorting::Sorting};


#[allow(dead_code)]
#[derive(Clone)]
pub struct DisplayLine {
    fields: Vec<String>,
    id: DataIndex,
}

#[allow(dead_code)]
impl DisplayLine {
    pub fn id ( &self ) -> &String {
        assert!(!self.fields.is_empty());  // this should be impossible
        &self.fields[0]
    }

    pub fn num_fields ( &self ) -> usize {
        self.fields.len()
    }

    pub fn field ( &self, number: usize) -> &str {
        self.fields.get(number).expect("unable to find promised index in display line").as_ref()
    }

    pub fn field_iter ( &self ) -> Iter<'_, String> {
        self.fields.iter()
    }
}

fn displayline_from_item_ref<T: Named + Clone> ( item: &T, index: &GenericRef<T> ) -> DisplayLine {
    DisplayLine {
        fields: item.display_fields(),
        id: index.data_index(),
    }
}


// -------------------
#[derive(Clone)]
pub struct DisplayTable {
    lines: Vec<DisplayLine>,
    headings: Vec<RichText>,
    sorting: Sorting,
}

#[allow(dead_code)]
impl DisplayTable {
    pub fn lines_iter ( &self ) -> impl Iterator<Item=&DisplayLine> {
        self.lines.iter()
    }

    pub fn line ( &self, index: usize ) -> &DisplayLine {
        assert!(index < self.lines.len());
        &self.lines[index]
    }

    pub fn lines_len ( &self ) -> usize {
        self.lines.len()
    }

    pub fn headings_iter ( &self ) -> impl Iterator<Item=RichText> {
        self.headings.iter().enumerate().map(|(i, h)| {
            let heading_text = h.clone();
            if i == self.sorting.sort_field() {
                heading_text.underline()
            } else { heading_text }
        })
    }

    pub fn number_columns ( &self ) -> usize {
        self.headings.len()
    }

    pub fn sorting ( &self ) -> &Sorting {
        &self.sorting
    }
}

impl<T: Named + Clone> From<&ManagedList<T>> for DisplayTable {
    fn from ( list:&ManagedList<T> ) -> Self {
        let item_list = list.item_ref_list();
        let mut lines: Vec<DisplayLine> = item_list.iter().map(|(index, item)| {
                displayline_from_item_ref(*item, index)
            }).collect();
        let sorting = list.get_sorting();
        let sort_fn = |a: &DisplayLine, b: &DisplayLine| a.fields[sorting.sort_field()].cmp(&b.fields[sorting.sort_field()]);
        lines.sort_by(sort_fn);
        if sorting.sort_reversed() { lines.reverse(); }
        DisplayTable {
            lines,
            headings: T::display_headings(),
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
impl <'a> ShowEditInfo<'a> {
    pub fn new ( name_collision: bool, differs_from: bool, create_new: bool, app_data: &'a AppData ) -> Self {
        ShowEditInfo {
            name_collision,
            differs_from,
            create_new,
            app_data,
        }
    }

    pub fn name_collision ( &self ) -> bool {
        self.name_collision
    }

    pub fn differs_from ( &self ) -> bool {
        self.differs_from
    }

    pub fn create_new ( &self ) -> bool {
        self.create_new
    }

    pub fn show_save ( &self ) -> bool {
        self.differs_from && !self.name_collision
    }

    pub fn app_data ( &self ) -> &AppData {
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
pub const NOTES_ROWS: usize = 6;


pub trait ShowEdit {
    fn show_edit ( &mut self, ui: &mut Ui, item_info: ShowEditInfo ) -> Option<EditResult>;
}

pub fn show_edit_frame ( ui: &mut Ui, item_name: String, debug_name: &str, item_info: ShowEditInfo, contents: impl FnOnce(&mut Ui) ) -> Option<EditResult> {
    let mut result = None;

    ui.horizontal(|ui| {
        if ui.add(Label::new(RichText::new("<").heading()).sense(Sense::click())).clicked() {
            debug!("return from edit {debug_name}");
            result = Some(EditResult::Ignore);  // should this be return?
        }
        ui.add_space(EDGE_SPACER);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(item_name).heading().strong().underline());
                if item_info.show_save() {
                    ui.add_space(60.0);
                    if ui.button(fl!("edit_save")).clicked() {
                        info!("save edited {debug_name} requested");
                        result = Some(EditResult::Submit);
                    }
                }
            });
            ui.add_space(HEAD_SPACE);

            eframe::egui::Frame::default()
                .stroke(Stroke::new(STROKE_WIDTH, STROKE_COLOR))
                .inner_margin(INNER_MARGIN)
                .show(ui, contents);
        });
    });

    result
}

// ------------------------
// Show / Edit GenericRefs

const EMPTY_NAME: &str = "    ";

pub fn show_edit_item<T: Named +Clone> ( name: &str, district: &mut Option<GenericRef<T>>, master_list: &ManagedList<T>, ui: &mut Ui ) {
    let district_list = {
        let mut list = master_list.names_sorted();
        list.insert(0, EMPTY_NAME.to_string());
        list
    };
    let mut selected_district = if let Some(is_district) = district {
        let name = is_district.name().unwrap_or_else(|| EMPTY_NAME.to_string());
        district_list.iter().position(|n| *n == name).unwrap_or_default()
    } else { 0 };
    let was_selected = selected_district;

    ComboBox::from_id_salt(name)
        .show_index(ui, &mut selected_district, district_list.len(), |i| district_list[i].to_string());

    if selected_district != was_selected {
        *district = if selected_district == 0 {
            info!("clearing {name}");
            None
        } else {
            let new_name = &district_list[selected_district];
            info!("setting {name} to {new_name}");
            master_list.find(new_name)
        }
    }
}

pub fn show_edit_list<T: Named + Clone> ( name: &str, districts: &mut GenericRefList<T>, master_list: &ManagedList<T>, ui: &mut Ui ) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let mut first = true;
        let was_hovered = districts.hovered_name().map(|h| h.to_string());
        let mut hovered = None;
        for dist in districts.list().clone() {
            // this should not be able to point to nothing, but it might?
            if let Some(district_name) = dist.name() {
                if !first {
                    ui.label(", ");
                }

                let district_label = {
                    let mut district_label = RichText::new(district_name.clone());
                    if let Some(hovered_name) = districts.hovered_name() {
                        if hovered_name == district_name {
                            district_label = district_label.color(Color32::DARK_RED).strikethrough();
                        }
                    }
                    district_label
                };
                let resp = ui.add(Label::new(district_label).sense(Sense::click()));
                if resp.clicked() {
                    info!("deleting {district_name} from list {name}");
                    districts.swap_remove(&district_name);
                } else if resp.hovered() {
                    hovered = Some(district_name.clone());
                }
                first = false;
            } else { unreachable!(); }
        }

        if hovered != was_hovered {
            districts.set_hovered(hovered);
        }

        // now perhaps add an entry
        // show + -> click to create empty
        if !first {
            ui.label(", ");
        }

        if let Some(new_name) = districts.new_name() {
            // show combo box
            let district_list = {
                let mut list = master_list.names_sorted();
                list.insert(0, EMPTY_NAME.to_string());
                list
            };
            let mut selected_district = district_list.iter().position(|n| *n == new_name).unwrap_or_default();
            let was_selected = selected_district;

            ComboBox::from_id_salt(name)
                .show_index(ui, &mut selected_district, district_list.len(), |i| district_list[i].to_string());

            if selected_district != was_selected {
                if selected_district == 0 {
                    districts.set_new(Some(EMPTY_NAME));
                } else {
                    let new_name = district_list[selected_district].as_str();
                    // find district ref
                    if let Some(d) = master_list.find(new_name) {
                        info!("adding {new_name} to list {name}");
                        // push to districts
                        districts.push(d);
                        // reset new
                        districts.set_new(None);
                    } else { districts.set_new(Some(new_name)); }
                }
            }
        } else {  // no new name
            if ui.add(Label::new(RichText::new("+").strong()).sense(Sense::click())).clicked() {
                info!("requested add item for {name}");
                districts.set_new(Some(EMPTY_NAME));
            }
        }
    });
}
