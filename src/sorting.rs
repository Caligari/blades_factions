

#[derive(Debug, Default, Clone, Copy)]
pub struct Sorting {
    sort_field: usize,
    sort_reverse: bool,
}

#[allow(dead_code)]
impl Sorting {
    pub fn set_field ( &mut self, index: usize ) {
        if self.sort_field != index {
            self.sort_field = index;
            self.sort_reverse = false;
        } else {
            self.sort_reverse = !self.sort_reverse;
        }
    }

    pub fn sort_field ( &self ) -> usize {
        self.sort_field
    }

    pub fn sort_reversed ( &self ) -> bool {
        self.sort_reverse
    }
}
