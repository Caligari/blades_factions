use std::slice::Iter;

use eframe::egui::{Color32, Label, Margin, RichText, Sense, Stroke, Ui};
use log::debug;

use crate::{app::EditResult, app_data::DataIndex, localize::fl, managed_list::{GenericRef, ManagedList, Named}, sorting::Sorting};


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

#[derive(Debug, Clone, Copy, Default)]
pub struct ShowEditInfo {
    name_collision: bool,
    differs_from: bool,
    create_new: bool,
    // could have reference lists??
}

#[allow(dead_code)]
impl ShowEditInfo {
    pub fn new ( name_collision: bool, differs_from: bool, create_new: bool ) -> Self {
        ShowEditInfo {
            name_collision,
            differs_from,
            create_new,
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
                        debug!("save edited {debug_name} requested");
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