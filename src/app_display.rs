use std::slice::Iter;

use eframe::egui::RichText;

use crate::{app_data::DataIndex, managed_list::{GenericRef, ManagedList, Named}, sorting::Sorting};


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