use std::slice::Iter;

use crate::{app_data::DataIndex, managed_list::{GenericRef, ManagedList, Named}};


#[allow(dead_code)]
#[derive(Clone)]
pub struct DisplayLine {
    fields: Vec<String>,
    id: DataIndex,
}

#[allow(dead_code)]
impl DisplayLine {
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
    headings: Vec<String>,
}

#[allow(dead_code)]
impl DisplayTable {
    pub fn lines_iter ( &self ) -> Iter<'_, DisplayLine> {
        self.lines.iter()
    }

    pub fn headings_iter ( &self ) -> Iter<'_, String> {
        self.headings.iter()
    }
}

impl<T: Named + Clone> From<&ManagedList<T>> for DisplayTable {
    fn from(list: &ManagedList<T>) -> Self {
        let item_list = list.item_ref_list();
        let mut lines: Vec<DisplayLine> = item_list.iter().map(|(index, item)| {
                displayline_from_item_ref(*item, index)
            }).collect();
        // todo: better sorting?
        lines.sort_by(|a, b| a.fields[0].cmp(&b.fields[0]));
        DisplayTable {
            lines,
            headings: T::display_headings(),
        }
    }
}